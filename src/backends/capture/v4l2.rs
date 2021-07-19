/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
    error::NokhwaError,
    mjpeg_to_rgb888,
    utils::{CameraFormat, CameraInfo},
    yuyv422_to_rgb888, CameraControl, CaptureAPIBackend, CaptureBackendTrait, FrameFormat,
    KnownCameraControlFlag, KnownCameraControls, Resolution,
};
use image::{ImageBuffer, Rgb};
use std::{borrow::Cow, collections::HashMap};
use v4l::{
    buffer::Type,
    frameinterval::FrameIntervalEnum,
    framesize::FrameSizeEnum,
    io::traits::CaptureStream,
    prelude::*,
    video::{capture::Parameters, Capture},
    Format, FourCC,
};

use std::any::Any;
pub use v4l::control::{Control, Description, Flags};

/// Generates a camera control from a device and a description of control
/// # Error
/// If the control is not supported, the value is invalid or string, or the control is write only/the control cannot be read from,
/// this will error.
pub fn to_camera_control(
    device: &Device,
    value: &Description,
) -> Result<CameraControl, NokhwaError> {
    // make sure flags is valid
    if value.flags.contains(Flags::from(0x0040)) {
        return Err(NokhwaError::NotImplementedError(
            "Control Write Only!".to_string(),
        ));
    }

    let control: KnownCameraControls = match try_id_to_known_camera_control(value.id) {
        Some(kcc) => kcc,
        None => {
            return Err(NokhwaError::StructureError {
                structure: "KnownCameraControl".to_string(),
                error: "Unsupported V4L2 ID".to_string(),
            })
        }
    };

    // value
    let current_value = match device.control(value.id) {
        Ok(val) => match val {
            Control::Value(v) => v,
            Control::Value64(v64) => {
                if v64 <= i32::MAX as i64 {
                    v64 as i32
                } else {
                    return Err(NokhwaError::GetPropertyError {
                        property: format!("Control V4L2ID: {}", value.id),
                        error: "Too large!".to_string(),
                    });
                }
            }
            Control::String(_) => {
                return Err(NokhwaError::GetPropertyError {
                    property: format!("Control V4L2ID: {}", value.id),
                    error: "Unsupported Type String".to_string(),
                })
            }
        },
        Err(why) => {
            return Err(NokhwaError::GetPropertyError {
                property: format!("Control V4L2ID: {}", value.id),
                error: why.to_string(),
            })
        }
    };

    let active =
        if value.flags.contains(Flags::from(0x0001)) || value.flags.contains(Flags::from(0x0010)) {
            true
        } else {
            false
        };

    Ok(CameraControl::new(
        control,
        value.minimum,
        value.maximum,
        current_value,
        value.step,
        value.default,
        KnownCameraControlFlag::Manual,
        active,
    )?)
}

/// Attempts to convert a [`KnownCameraControls`] into a V4L2 Control ID.
/// If the associated control is not found, this will return `None` (ColorEnable, Roll)
pub fn try_known_camera_control_to_id(ctrl: KnownCameraControls) -> Option<u32> {
    match ctrl {
        KnownCameraControls::Brightness => Some(9963776),
        KnownCameraControls::Contrast => Some(9963777),
        KnownCameraControls::Hue => Some(9963779),
        KnownCameraControls::Saturation => Some(9963778),
        KnownCameraControls::Sharpness => Some(9963803),
        KnownCameraControls::Gamma => Some(9963792),
        KnownCameraControls::WhiteBalance => Some(9963802),
        KnownCameraControls::BacklightComp => Some(9963804),
        KnownCameraControls::Gain => Some(9963795),
        KnownCameraControls::Pan => Some(10094852),
        KnownCameraControls::Tilt => Some(100948530),
        KnownCameraControls::Zoom => Some(10094862),
        KnownCameraControls::Exposure => Some(99637930),
        KnownCameraControls::Iris => Some(10094866),
        KnownCameraControls::Focus => Some(10094859),
        _ => None,
    }
}

/// Attempts to convert a [`u32`] V4L2 Control ID into a [`KnownCameraControls`]
/// If the associated control is not found, this will return `None` (ColorEnable, Roll)
pub fn try_id_to_known_camera_control(id: u32) -> Option<KnownCameraControls> {
    match id {
        9963776 => Some(KnownCameraControls::Brightness),
        9963777 => Some(KnownCameraControls::Contrast),
        9963779 => Some(KnownCameraControls::Hue),
        9963778 => Some(KnownCameraControls::Saturation),
        9963803 => Some(KnownCameraControls::Sharpness),
        9963792 => Some(KnownCameraControls::Gamma),
        9963802 => Some(KnownCameraControls::WhiteBalance),
        9963804 => Some(KnownCameraControls::BacklightComp),
        9963795 => Some(KnownCameraControls::Gain),
        10094852 => Some(KnownCameraControls::Pan),
        100948530 => Some(KnownCameraControls::Tilt),
        10094862 => Some(KnownCameraControls::Zoom),
        99637930 => Some(KnownCameraControls::Exposure),
        10094866 => Some(KnownCameraControls::Iris),
        10094859 => Some(KnownCameraControls::Focus),
        _ => None,
    }
}

fn clone_control(ctrl: &Control) -> Control {
    match ctrl {
        Control::Value(v) => Control::Value(*v),
        Control::Value64(v) => Control::Value64(*v),
        Control::String(v) => Control::String(v.clone()),
    }
}

/// The backend struct that interfaces with V4L2.
/// To see what this does, please see [`CaptureBackendTrait`].
/// # Quirks
/// - Calling [`set_resolution()`](CaptureBackendTrait::set_resolution), [`set_frame_rate()`](CaptureBackendTrait::set_frame_rate), or [`set_frame_format()`](CaptureBackendTrait::set_frame_format) each internally calls [`set_camera_format()`](CaptureBackendTrait::set_camera_format).
/// - The `Any` return type for [`raw_supported_camera_controls()`](CaptureBackendTrait::raw_supported_camera_controls) is [`Description`]
/// - The `Any` type for [`raw_camera_control()`](CaptureBackendTrait::raw_camera_control) is [`u32`], and its return `Any` is a [`Control`]
/// - The `Any` type for `control` for [`set_raw_camera_control()`](CaptureBackendTrait::set_raw_camera_control) is [`u32`] and [`Control`]
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
                return Err(NokhwaError::OpenDeviceError(
                    index.to_string(),
                    format!("V4L2 Error: {}", why.to_string()),
                ))
            }
        };

        let camera_info = match device.query_caps() {
            Ok(caps) => CameraInfo::new(caps.card, "".to_string(), caps.driver, index),
            Err(why) => {
                return Err(NokhwaError::GetPropertyError {
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
                    return Err(NokhwaError::SetPropertyError {
                        property: "Format(V4L Resolution, FourCC)".to_string(),
                        value: camera_format.to_string(),
                        error: "Rejected".to_string(),
                    });
                }
            }
            Err(why) => {
                return Err(NokhwaError::SetPropertyError {
                    property: "Format(V4L Resolution, FourCC)".to_string(),
                    value: camera_format.to_string(),
                    error: why.to_string(),
                })
            }
        }

        match Capture::set_params(&device, &new_param) {
            Ok(param) => {
                if new_param.interval.denominator != param.interval.denominator {
                    return Err(NokhwaError::SetPropertyError {
                        property: "Parameter(V4L FPS)".to_string(),
                        value: camera_format.framerate().to_string(),
                        error: "Rejected".to_string(),
                    });
                }
            }
            Err(why) => {
                return Err(NokhwaError::SetPropertyError {
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
            Err(why) => Err(NokhwaError::GetPropertyError {
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
    fn backend(&self) -> CaptureAPIBackend {
        CaptureAPIBackend::Video4Linux
    }

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
                return Err(NokhwaError::GetPropertyError {
                    property: "Resolution, FrameFormat".to_string(),
                    error: why.to_string(),
                })
            }
        };
        let prev_fps = match Capture::params(&self.device) {
            Ok(fps) => fps,
            Err(why) => {
                return Err(NokhwaError::GetPropertyError {
                    property: "Framerate".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let format: Format = new_fmt.into();
        let framerate = Parameters::with_fps(new_fmt.framerate());

        if let Err(why) = Capture::set_format(&self.device, &format) {
            return Err(NokhwaError::SetPropertyError {
                property: "Resolution, FrameFormat".to_string(),
                value: format.to_string(),
                error: why.to_string(),
            });
        }
        if let Err(why) = Capture::set_params(&self.device, &framerate) {
            return Err(NokhwaError::SetPropertyError {
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
                        return Err(NokhwaError::SetPropertyError {
                            property: format!("Attempt undo due to stream acquisition failure with error {}. Resolution, FrameFormat", why.to_string()),
                            value: prev_format.to_string(),
                            error: why.to_string(),
                        });
                    }
                    if let Err(why) = Capture::set_params(&self.device, &prev_fps) {
                        return Err(NokhwaError::SetPropertyError {
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
                    return Err(NokhwaError::GetPropertyError {
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
                            return Err(NokhwaError::GetPropertyError {
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
            Err(why) => Err(NokhwaError::GetPropertyError {
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

    fn supported_camera_controls(&self) -> Result<Vec<KnownCameraControls>, NokhwaError> {
        let v4l2_ctrls = match self.device.query_controls() {
            Ok(ctrls) => ctrls,
            Err(why) => {
                return Err(NokhwaError::GetPropertyError {
                    property: "Controls".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let mut camera_controls = vec![];
        for ctrl in v4l2_ctrls {
            if let Ok(cam_control) = to_camera_control(&self.device, &ctrl) {
                camera_controls.push(cam_control.control())
            }
        }
        Ok(camera_controls)
    }

    fn camera_control(&self, control: KnownCameraControls) -> Result<CameraControl, NokhwaError> {
        let id = match try_known_camera_control_to_id(control) {
            Some(id) => id,
            None => {
                return Err(NokhwaError::GetPropertyError {
                    property: "KnownCameraControls V4L2ID".to_string(),
                    error: "Invalid".to_string(),
                })
            }
        };

        match self.device.control(id) {
            Ok(_) => {
                let v4l2_ctrls = match self.device.query_controls() {
                    Ok(ctrls) => ctrls,
                    Err(why) => {
                        return Err(NokhwaError::GetPropertyError {
                            property: "Controls".to_string(),
                            error: why.to_string(),
                        })
                    }
                };

                for ctrl in v4l2_ctrls {
                    if let Ok(cam_control) = to_camera_control(&self.device, &ctrl) {
                        if Some(ctrl.id) == try_known_camera_control_to_id(cam_control.control()) {
                            return Ok(cam_control);
                        }
                    }
                }
            }
            Err(why) => {
                return Err(NokhwaError::GetPropertyError {
                    property: "Camera Control".to_string(),
                    error: why.to_string(),
                })
            }
        }
        Err(NokhwaError::GetPropertyError {
            property: "Camera Control".to_string(),
            error: "Not Found".to_string(),
        })
    }

    fn set_camera_control(&mut self, control: CameraControl) -> Result<(), NokhwaError> {
        let id = match try_known_camera_control_to_id(control.control()) {
            Some(id) => id,
            None => {
                return Err(NokhwaError::GetPropertyError {
                    property: "KnownCameraControls V4L2ID".to_string(),
                    error: "Invalid".to_string(),
                })
            }
        };

        if let Err(why) = self.device.set_control(id, Control::Value(control.value())) {
            return Err(NokhwaError::SetPropertyError {
                property: format!("{} V4L2ID {}", control.control(), id),
                value: control.value().to_string(),
                error: why.to_string(),
            });
        }
        Ok(())
    }

    fn raw_supported_camera_controls(&self) -> Result<Vec<Box<dyn Any>>, NokhwaError> {
        let v4l2_ctrls = match self.device.query_controls() {
            Ok(ctrls) => ctrls,
            Err(why) => {
                return Err(NokhwaError::GetPropertyError {
                    property: "Controls".to_string(),
                    error: why.to_string(),
                })
            }
        };

        Ok(v4l2_ctrls
            .into_iter()
            .map(|c| {
                let a: Box<dyn Any> = Box::new(c);
                a
            })
            .collect())
    }

    fn raw_camera_control(&self, control: &dyn Any) -> Result<Box<dyn Any>, NokhwaError> {
        let id = match control.downcast_ref::<u32>() {
            Some(id) => *id,
            None => {
                return Err(NokhwaError::StructureError {
                    structure: "V4L2 ID".to_string(),
                    error: "Failed Any Cast".to_string(),
                })
            }
        };

        match self.device.control(id) {
            Ok(v) => Ok(Box::new(v)),
            Err(why) => Err(NokhwaError::GetPropertyError {
                property: "Control V4L2".to_string(),
                error: why.to_string(),
            }),
        }
    }

    fn set_raw_camera_control(
        &mut self,
        control: &dyn Any,
        value: &dyn Any,
    ) -> Result<(), NokhwaError> {
        let id = match control.downcast_ref::<u32>() {
            Some(id) => *id,
            None => {
                return Err(NokhwaError::StructureError {
                    structure: "V4L2 ID".to_string(),
                    error: "Failed Any Cast".to_string(),
                })
            }
        };

        let value = match value.downcast_ref::<Control>() {
            Some(v) => match v {
                Control::Value(v) => Control::Value(*v),
                Control::Value64(v64) => Control::Value64(*v64),
                Control::String(s) => Control::String(s.clone()),
            },
            None => {
                return Err(NokhwaError::StructureError {
                    structure: "V4L2 Control Value".to_string(),
                    error: "Failed Any Cast".to_string(),
                })
            }
        };

        if let Err(why) = self.device.set_control(id, clone_control(&value)) {
            return Err(NokhwaError::SetPropertyError {
                property: format!("V4L2 Control ID {}", id),
                value: format!("{:?}", value),
                error: why.to_string(),
            });
        }

        Ok(())
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        let stream = match MmapStream::new(&self.device, Type::VideoCapture) {
            Ok(s) => s,
            Err(why) => return Err(NokhwaError::OpenStreamError(why.to_string())),
        };
        self.stream_handle = Some(stream);
        Ok(())
    }

    fn is_stream_open(&self) -> bool {
        self.stream_handle.is_some()
    }

    fn frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        let cam_fmt = self.camera_format;
        let raw_frame = self.frame_raw()?;
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
                None => return Err(NokhwaError::ReadFrameError(
                    "Imagebuffer is not large enough! This is probably a bug, please report it!"
                        .to_string(),
                )),
            };
        Ok(imagebuf)
    }

    fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        match &mut self.stream_handle {
            Some(streamh) => match streamh.next() {
                Ok((data, _)) => Ok(Cow::from(data)),
                Err(why) => Err(NokhwaError::ReadFrameError(why.to_string())),
            },
            None => Err(NokhwaError::ReadFrameError(
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
