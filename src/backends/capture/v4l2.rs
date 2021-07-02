use crate::{
    error::NokhwaError,
    mjpeg_to_rgb888,
    utils::{CameraFormat, CameraInfo},
    yuyv422_to_rgb888, CaptureBackendTrait, FrameFormat, Resolution,
};
use image::{ImageBuffer, Rgb};
use std::collections::HashMap;
use v4l::{
    buffer::Type,
    frameinterval::FrameIntervalEnum,
    framesize::FrameSizeEnum,
    io::traits::CaptureStream,
    prelude::*,
    video::{capture::Parameters, Capture},
    Format, FourCC,
};

#[cfg(feature = "input-v4l")]
impl From<CameraFormat> for Format {
    fn from(cam_fmt: CameraFormat) -> Self {
        let pxfmt = match cam_fmt.format() {
            FrameFormat::MJPEG => FourCC::new(b"MJPG"),
            FrameFormat::YUYV => FourCC::new(b"YUYV"),
        };

        Format::new(cam_fmt.width(), cam_fmt.height(), pxfmt)
    }
}

/// The backend struct that interfaces with V4L2.
/// To see what this does, please see [`CaptureBackendTrait`].
/// # Quirks
/// - Calling [`set_resolution()`](CaptureBackendTrait::set_resolution), [`set_frame_rate()`](CaptureBackendTrait::set_frame_rate), or [`set_frame_format()`](CaptureBackendTrait::set_frame_format) each internally calls [`set_camera_format()`](CaptureBackendTrait::set_camera_format).
pub struct V4LCaptureDevice<'a> {
    camera_format: CameraFormat,
    camera_info: CameraInfo,
    device: Device,
    stream_handle: Option<MmapStream<'a>>,
}

impl<'a> V4LCaptureDevice<'a> {
    /// Creates a new capture device using the V4L2 backend. Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default.
    /// # Errors
    /// This function will error if the camera is currently busy or if V4L2 can't read device information.
    pub fn new(index: usize, cam_fmt: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        let device = match Device::new(index) {
            Ok(dev) => dev,
            Err(why) => {
                return Err(NokhwaError::CouldntOpenDevice(format!(
                    "V4L2 Error: {}",
                    why.to_string()
                )))
            }
        };

        let camera_info = match device.query_caps() {
            Ok(caps) => CameraInfo::new(caps.card, "".to_string(), caps.driver, index),
            Err(why) => {
                return Err(NokhwaError::CouldntQueryDevice {
                    property: "Capabilities".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let camera_format = match cam_fmt {
            Some(cfmt) => cfmt,
            None => CameraFormat::default(),
        };

        let fourcc = match camera_format.format() {
            FrameFormat::MJPEG => FourCC::new(b"MJPG"),
            FrameFormat::YUYV => FourCC::new(b"YUYV"),
        };

        let new_param = Parameters::with_fps(camera_format.framerate());
        let new_v4l_fmt = Format::new(camera_format.width(), camera_format.height(), fourcc);

        match Capture::set_format(&device, &new_v4l_fmt) {
            Ok(v4l_fmt) => {
                if v4l_fmt.height != new_v4l_fmt.height
                    && v4l_fmt.width != new_v4l_fmt.width
                    && v4l_fmt.fourcc != new_v4l_fmt.fourcc
                {
                    return Err(NokhwaError::CouldntSetProperty {
                        property: "Format(V4L Resolution, FourCC)".to_string(),
                        value: camera_format.to_string(),
                        error: "Rejected".to_string(),
                    });
                }
            }
            Err(why) => {
                return Err(NokhwaError::CouldntSetProperty {
                    property: "Format(V4L Resolution, FourCC)".to_string(),
                    value: camera_format.to_string(),
                    error: why.to_string(),
                })
            }
        }

        match Capture::set_params(&device, &new_param) {
            Ok(param) => {
                if new_param.interval.denominator != param.interval.denominator {
                    return Err(NokhwaError::CouldntSetProperty {
                        property: "Parameter(V4L FPS)".to_string(),
                        value: camera_format.framerate().to_string(),
                        error: "Rejected".to_string(),
                    });
                }
            }
            Err(why) => {
                return Err(NokhwaError::CouldntSetProperty {
                    property: "Parameter(V4L FPS)".to_string(),
                    value: camera_format.framerate().to_string(),
                    error: why.to_string(),
                })
            }
        }

        Ok(V4LCaptureDevice {
            camera_format,
            camera_info,
            device,
            stream_handle: None,
        })
    }

    /// Create a new V4L Camera with desired settings.
    /// # Errors
    /// This function will error if the camera is currently busy or if V4L2 can't read device information.
    pub fn new_with(
        index: usize,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
    ) -> Result<Self, NokhwaError> {
        let camera_format = Some(CameraFormat::new_from(width, height, fourcc, fps));
        V4LCaptureDevice::new(index, camera_format)
    }

    fn get_resolution_list(&self, fourcc: FrameFormat) -> Result<Vec<Resolution>, NokhwaError> {
        let format = match fourcc {
            FrameFormat::MJPEG => FourCC::new(b"MJPG"),
            FrameFormat::YUYV => FourCC::new(b"YUYV"),
        };

        match v4l::video::Capture::enum_framesizes(&self.device, format) {
            Ok(framesizes) => {
                let mut resolutions = vec![];
                for framesize in framesizes {
                    match framesize.size {
                        FrameSizeEnum::Discrete(dis) => {
                            resolutions.push(Resolution::new(dis.width, dis.height))
                        }
                        FrameSizeEnum::Stepwise(step) => {
                            resolutions.push(Resolution::new(step.min_width, step.min_height));
                            resolutions.push(Resolution::new(step.max_width, step.max_height));
                            // TODO: Respect step size
                        }
                    }
                }
                Ok(resolutions)
            }
            Err(why) => Err(NokhwaError::CouldntQueryDevice {
                property: "Resolutions".to_string(),
                error: why.to_string(),
            }),
        }
    }

    /// Get the inner device (immutable) for e.g. Controls
    #[allow(clippy::must_use_candidate)]
    pub fn inner_device(&self) -> &Device {
        &self.device
    }

    /// Get the inner device (mutable) for e.g. Controls
    pub fn inner_device_mut(&mut self) -> &mut Device {
        &mut self.device
    }
}

impl<'a> CaptureBackendTrait for V4LCaptureDevice<'a> {
    fn camera_info(&self) -> CameraInfo {
        self.camera_info.clone()
    }

    fn camera_format(&self) -> CameraFormat {
        self.camera_format
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        let prev_format = match Capture::format(&self.device) {
            Ok(fmt) => fmt,
            Err(why) => {
                return Err(NokhwaError::CouldntQueryDevice {
                    property: "Resolution, FrameFormat".to_string(),
                    error: why.to_string(),
                })
            }
        };
        let prev_fps = match Capture::params(&self.device) {
            Ok(fps) => fps,
            Err(why) => {
                return Err(NokhwaError::CouldntQueryDevice {
                    property: "Framerate".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let format: Format = new_fmt.into();
        let framerate = Parameters::with_fps(new_fmt.framerate());

        if let Err(why) = Capture::set_format(&self.device, &format) {
            return Err(NokhwaError::CouldntSetProperty {
                property: "Resolution, FrameFormat".to_string(),
                value: format.to_string(),
                error: why.to_string(),
            });
        }
        if let Err(why) = Capture::set_params(&self.device, &framerate) {
            return Err(NokhwaError::CouldntSetProperty {
                property: "Framerate".to_string(),
                value: framerate.to_string(),
                error: why.to_string(),
            });
        }

        if self.stream_handle.is_some() {
            return match self.open_stream() {
                Ok(_) => Ok(()),
                Err(why) => {
                    // undo
                    if let Err(why) = Capture::set_format(&self.device, &prev_format) {
                        return Err(NokhwaError::CouldntSetProperty {
                            property: format!("Attempt undo due to stream acquisition failure with error {}. Resolution, FrameFormat", why.to_string()),
                            value: prev_format.to_string(),
                            error: why.to_string(),
                        });
                    }
                    if let Err(why) = Capture::set_params(&self.device, &prev_fps) {
                        return Err(NokhwaError::CouldntSetProperty {
                            property:
                            format!("Attempt undo due to stream acquisition failure with error {}. Framerate", why.to_string()),
                            value: prev_fps.to_string(),
                            error: why.to_string(),
                        });
                    }
                    Err(why)
                }
            };
        }
        self.camera_format = new_fmt;
        Ok(())
    }

    fn compatible_list_by_resolution(
        &self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        let resolutions = self.get_resolution_list(fourcc)?;
        let format = match fourcc {
            FrameFormat::MJPEG => FourCC::new(b"MJPG"),
            FrameFormat::YUYV => FourCC::new(b"YUYV"),
        };
        let mut resmap = HashMap::new();
        for res in resolutions {
            let mut compatible_fps = vec![];
            match Capture::enum_frameintervals(&self.device, format, res.width(), res.height()) {
                Ok(intervals) => {
                    for interval in intervals {
                        match interval.interval {
                            FrameIntervalEnum::Discrete(dis) => {
                                compatible_fps.push(dis.denominator);
                            }
                            FrameIntervalEnum::Stepwise(step) => {
                                compatible_fps.push(step.min.denominator);
                                compatible_fps.push(step.max.denominator);
                            }
                        }
                    }
                }
                Err(why) => {
                    return Err(NokhwaError::CouldntQueryDevice {
                        property: "Framerate".to_string(),
                        error: why.to_string(),
                    })
                }
            }
            resmap.insert(res, compatible_fps);
        }
        Ok(resmap)
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        match Capture::enum_formats(&self.device) {
            Ok(formats) => {
                let mut frameformat_vec = vec![];
                for format in formats {
                    let format_as_string = match format.fourcc.str() {
                        Ok(s) => s,
                        Err(why) => {
                            return Err(NokhwaError::CouldntQueryDevice {
                                property: "FrameFormat".to_string(),
                                error: why.to_string(),
                            })
                        }
                    };
                    match format_as_string {
                        "YUYV" => frameformat_vec.push(FrameFormat::YUYV),
                        "MJPG" => frameformat_vec.push(FrameFormat::MJPEG),
                        _ => {}
                    }
                }
                frameformat_vec.sort();
                frameformat_vec.dedup();
                Ok(frameformat_vec)
            }
            Err(why) => Err(NokhwaError::CouldntQueryDevice {
                property: "FrameFormat".to_string(),
                error: why.to_string(),
            }),
        }
    }

    fn resolution(&self) -> Resolution {
        self.camera_format.resolution()
    }

    #[allow(clippy::option_if_let_else)]
    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        let mut new_fmt = self.camera_format;
        new_fmt.set_resolution(new_res);
        self.set_camera_format(new_fmt)
    }

    fn frame_rate(&self) -> u32 {
        self.camera_format.framerate()
    }

    #[allow(clippy::option_if_let_else)]
    fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        let mut new_fmt = self.camera_format;
        new_fmt.set_framerate(new_fps);
        self.set_camera_format(new_fmt)
    }

    fn frame_format(&self) -> FrameFormat {
        self.camera_format.format()
    }

    #[allow(clippy::option_if_let_else)]
    fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        let mut new_fmt = self.camera_format;
        new_fmt.set_format(fourcc);
        self.set_camera_format(new_fmt)
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        let stream = match MmapStream::new(&self.device, Type::VideoCapture) {
            Ok(s) => s,
            Err(why) => return Err(NokhwaError::CouldntOpenStream(why.to_string())),
        };
        self.stream_handle = Some(stream);
        Ok(())
    }

    fn is_stream_open(&self) -> bool {
        self.stream_handle.is_some()
    }

    fn frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        let raw_frame = self.frame_raw()?;
        let cam_fmt = self.camera_format;
        let conv = match cam_fmt.format() {
            FrameFormat::MJPEG => mjpeg_to_rgb888(&raw_frame)?,
            FrameFormat::YUYV => yuyv422_to_rgb888(&raw_frame)?,
        };
        let imagebuf =
            match ImageBuffer::from_vec(cam_fmt.width(), cam_fmt.height(), conv) {
                Some(buf) => {
                    let rgbbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = buf;
                    rgbbuf
                }
                None => return Err(NokhwaError::CouldntCaptureFrame(
                    "Imagebuffer is not large enough! This is probably a bug, please report it!"
                        .to_string(),
                )),
            };
        Ok(imagebuf)
    }

    fn frame_raw(&mut self) -> Result<Vec<u8>, NokhwaError> {
        match &mut self.stream_handle {
            Some(streamh) => match streamh.next() {
                Ok((data, _)) => Ok(data.to_vec()),
                Err(why) => Err(NokhwaError::CouldntCaptureFrame(why.to_string())),
            },
            None => Err(NokhwaError::CouldntCaptureFrame(
                "Stream not initialized! Please call \"open_stream()\" first!".to_string(),
            )),
        }
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        if self.stream_handle.is_some() {
            self.stream_handle = None;
        }
        Ok(())
    }
}
