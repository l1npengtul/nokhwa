use std::collections::HashMap;

use crate::{
    error::NokhwaError,
    mjpeg_to_rgb888,
    utils::{CameraFormat, CameraInfo},
    yuyv422_to_rgb888, CaptureBackendTrait, FrameFormat, Resolution,
};
use image::{ImageBuffer, Rgb};
use v4l::{
    buffer::Type,
    io::traits::CaptureStream,
    video::{capture::Parameters, Capture},
    Format, FourCC,
};
use v4l::{frameinterval::FrameIntervalEnum, framesize::FrameSizeEnum, prelude::*};

#[cfg(feature = "input_v4l")]
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
/// To see what this does, please see [`CaptureBackendTrait`]
/// # Quirks
/// Calling [`set_resolution()`](CaptureBackendTrait::set_resolution), [`set_framerate()`](CaptureBackendTrait::set_framerate), or [`set_frameformat()`](CaptureBackendTrait::set_frameformat)
/// each internally calls [`set_camera_format()`](CaptureBackendTrait::set_camera_format).
pub struct V4LCaptureDevice<'a> {
    camera_format: Option<CameraFormat>,
    camera_info: CameraInfo,
    device: Device,
    stream_handle: Option<MmapStream<'a>>,
}

impl<'a> V4LCaptureDevice<'a> {
    /// Creates a new capture device using the V4L2 backend. Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    /// # Errors
    /// This function will error if the camera is currently busy or if V4L2 can't read device information.
    pub fn new(index: usize, camera_format: Option<CameraFormat>) -> Result<Self, NokhwaError> {
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
}

impl<'a> CaptureBackendTrait for V4LCaptureDevice<'a> {
    fn get_info(&self) -> CameraInfo {
        self.camera_info.clone()
    }

    #[allow(clippy::option_if_let_else)]
    fn init_camera_format_default(&mut self, overwrite: bool) -> Result<(), NokhwaError> {
        match self.camera_format {
            Some(_) => {
                if overwrite {
                    return self.set_camera_format(CameraFormat::default());
                }
                Ok(())
            }
            None => self.set_camera_format(CameraFormat::default()),
        }
    }

    fn get_camera_format(&self) -> Option<CameraFormat> {
        self.camera_format
    }

    fn get_compatible_list_by_resolution(
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
            match self
                .device
                .enum_frameintervals(format, res.width(), res.height())
            {
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

    fn get_resolution_list(&self, fourcc: FrameFormat) -> Result<Vec<Resolution>, NokhwaError> {
        let format = match fourcc {
            FrameFormat::MJPEG => FourCC::new(b"MJPG"),
            FrameFormat::YUYV => FourCC::new(b"YUYV"),
        };

        match self.device.enum_framesizes(format) {
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

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        let prev_format = match self.device.format() {
            Ok(fmt) => fmt,
            Err(why) => {
                return Err(NokhwaError::CouldntQueryDevice {
                    property: "Resolution, FrameFormat".to_string(),
                    error: why.to_string(),
                })
            }
        };
        let prev_fps = match self.device.params() {
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

        if let Err(why) = self.device.set_format(&format) {
            return Err(NokhwaError::CouldntSetProperty {
                property: "Resolution, FrameFormat".to_string(),
                value: format.to_string(),
                error: why.to_string(),
            });
        }
        if let Err(why) = self.device.set_params(&framerate) {
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
                    if let Err(why) = self.device.set_format(&prev_format) {
                        return Err(NokhwaError::CouldntSetProperty {
                            property: format!("Attempt undo due to stream acquisition failure with error {}. Resolution, FrameFormat", why.to_string()),
                            value: prev_format.to_string(),
                            error: why.to_string(),
                        });
                    }
                    if let Err(why) = self.device.set_params(&prev_fps) {
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
        self.camera_format = Some(new_fmt);
        Ok(())
    }

    fn get_resolution(&self) -> Option<Resolution> {
        self.camera_format.map(|fmt| fmt.resoltuion())
    }

    #[allow(clippy::option_if_let_else)]
    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        if let Some(fmt) = self.camera_format {
            let mut new_fmt = fmt;
            new_fmt.set_resolution(new_res);
            self.set_camera_format(new_fmt)
        } else {
            self.camera_format = Some(CameraFormat::new(new_res, FrameFormat::MJPEG, 0));
            Ok(())
        }
    }

    fn get_framerate(&self) -> Option<u32> {
        self.camera_format.map(|fmt| fmt.framerate())
    }

    #[allow(clippy::option_if_let_else)]
    fn set_framerate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        if let Some(fmt) = self.camera_format {
            let mut new_fmt = fmt;
            new_fmt.set_framerate(new_fps);
            self.set_camera_format(new_fmt)
        } else {
            self.camera_format = Some(CameraFormat::new(
                Resolution::new(0, 0),
                FrameFormat::MJPEG,
                new_fps,
            ));
            Ok(())
        }
    }

    fn get_frameformat(&self) -> Option<FrameFormat> {
        self.camera_format.map(|fmt| fmt.format())
    }

    #[allow(clippy::option_if_let_else)]
    fn set_frameformat(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        if let Some(fmt) = self.camera_format {
            let mut new_fmt = fmt;
            new_fmt.set_format(fourcc);
            self.set_camera_format(new_fmt)
        } else {
            self.camera_format = Some(CameraFormat::new(Resolution::new(0, 0), fourcc, 0));
            Ok(())
        }
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

    fn get_frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        let raw_frame = self.get_frame_raw()?;
        let cam_fmt = match self.camera_format {
            Some(fmt) => fmt,
            None => {
                return Err(NokhwaError::CouldntCaptureFrame(
                    "CameraFormat isn't initialized! This is probably a bug, please report it"
                        .to_string(),
                ));
            }
        };
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

    fn get_frame_raw(&mut self) -> Result<Vec<u8>, NokhwaError> {
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
