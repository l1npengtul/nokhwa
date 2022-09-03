/*
 * Copyright 2022 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::{
    mjpeg_to_rgb,
    utils::{CameraFormat, CameraInfo},
    yuyv422_to_rgb, ApiBackend, CameraControl, CameraIndex, CaptureBackendTrait,
    ControlValueDescription, ControlValueSetter, FrameFormat, KnownCameraControl,
    KnownCameraControlFlag, RequestedFormat, Resolution,
};
use image::ImageBuffer;
use nokhwa_core::buffer::Buffer;
use nokhwa_core::error::NokhwaError;
use nokhwa_core::pixel_format::FormatDecoder;
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::format,
    io::{self, ErrorKind},
    iter::Repeat,
};
use v4l::io::mmap::Stream;
use v4l::{
    buffer::Type,
    control::{Control, Flags, Value},
    frameinterval::FrameIntervalEnum,
    framesize::FrameSizeEnum,
    io::traits::CaptureStream,
    prelude::MmapStream,
    video::{capture::Parameters, Capture},
    Device, Format, FourCC, Fraction,
};

/// Attempts to convert a [`KnownCameraControl`] into a V4L2 Control ID.
/// If the associated control is not found, this will return `None` (`ColorEnable`, `Roll`)
pub fn known_camera_control_to_id(ctrl: KnownCameraControl) -> u32 {
    match ctrl {
        KnownCameraControl::Brightness => 9_963_776,
        KnownCameraControl::Contrast => 9_963_777,
        KnownCameraControl::Hue => 9_963_779,
        KnownCameraControl::Saturation => 9_963_778,
        KnownCameraControl::Sharpness => 9_963_803,
        KnownCameraControl::Gamma => 9_963_792,
        KnownCameraControl::WhiteBalance => 9_963_802,
        KnownCameraControl::BacklightComp => 9_963_804,
        KnownCameraControl::Gain => 9_963_795,
        KnownCameraControl::Pan => 10_094_852,
        KnownCameraControl::Tilt => 100_948_530,
        KnownCameraControl::Zoom => 10_094_862,
        KnownCameraControl::Exposure => 10_094_850,
        KnownCameraControl::Iris => 10_094_866,
        KnownCameraControl::Focus => 10_094_859,
        KnownCameraControl::Other(id) => id as u32,
    }
}

/// Attempts to convert a [`u32`] V4L2 Control ID into a [`KnownCameraControl`]
/// If the associated control is not found, this will return `None` (`ColorEnable`, `Roll`)
pub fn id_to_known_camera_control(id: u32) -> KnownCameraControl {
    match id {
        9_963_776 => KnownCameraControl::Brightness,
        9_963_777 => KnownCameraControl::Contrast,
        9_963_779 => KnownCameraControl::Hue,
        9_963_778 => KnownCameraControl::Saturation,
        9_963_803 => KnownCameraControl::Sharpness,
        9_963_792 => KnownCameraControl::Gamma,
        9_963_802 => KnownCameraControl::WhiteBalance,
        9_963_804 => KnownCameraControl::BacklightComp,
        9_963_795 => KnownCameraControl::Gain,
        10_094_852 => KnownCameraControl::Pan,
        100_948_530 => KnownCameraControl::Tilt,
        10_094_862 => KnownCameraControl::Zoom,
        10_094_850 => KnownCameraControl::Exposure,
        10_094_866 => KnownCameraControl::Iris,
        10_094_859 => KnownCameraControl::Focus,
        id => KnownCameraControl::Other(id as u128),
    }
}

/// The backend struct that interfaces with V4L2.
/// To see what this does, please see [`CaptureBackendTrait`].
/// # Quirks
/// - Calling [`set_resolution()`](CaptureBackendTrait::set_resolution), [`set_frame_rate()`](CaptureBackendTrait::set_frame_rate), or [`set_frame_format()`](CaptureBackendTrait::set_frame_format) each internally calls [`set_camera_format()`](CaptureBackendTrait::set_camera_format).
/// - The `Any` return type for [`raw_supported_camera_controls()`](CaptureBackendTrait::raw_supported_camera_controls) is [`Description`]
/// - The `Any` type for [`raw_camera_control()`](CaptureBackendTrait::raw_camera_control) is [`u32`], and its return `Any` is a [`Control`]
/// - The `Any` type for `control` for [`set_raw_camera_control()`](CaptureBackendTrait::set_raw_camera_control) is [`u32`] and [`Control`]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-v4l")))]
pub struct V4LCaptureDevice<'a> {
    initialized: bool,
    camera_format: CameraFormat,
    camera_info: CameraInfo,
    device: Device,
    stream_handle: Option<MmapStream<'a>>,
}

impl<'a> V4LCaptureDevice<'a> {
    /// Creates a new capture device using the `V4L2` backend. Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// If `camera_format` is `None`, it will be spawned with a random [`CameraFormat`] as determined by [`init()`](crate::CaptureBackendTrait::init).
    ///
    /// If `camera_format` is not `None`, the camera will try to use it when you call [`init()`](crate::CaptureBackendTrait::init).
    /// # Errors
    /// This function will error if the camera is currently busy or if `V4L2` can't read device information.
    pub fn new(index: CameraIndex, cam_fmt: RequestedFormat) -> Result<Self, NokhwaError> {
        let mut device = match Device::new(index.as_index()? as usize) {
            Ok(dev) => dev,
            Err(why) => {
                return Err(NokhwaError::OpenDeviceError(
                    index.to_string(),
                    format!("V4L2 Error: {}", why),
                ))
            }
        };

        // get all formats
        // get all fcc
        let mut camera_formats = vec![];

        let frame_formats = match device.enum_formats() {
            Ok(formats) => {
                let mut frame_format_vec = vec![];
                formats
                    .iter()
                    .for_each(|fmt| frame_format_vec.push(fmt.fourcc));
                frame_format_vec.sort();
                frame_format_vec.dedup();
                Ok(frame_format_vec)
            }
            Err(why) => Err(NokhwaError::GetPropertyError {
                property: "FrameFormat".to_string(),
                error: why.to_string(),
            }),
        }?;

        for ff in frame_formats {
            let framefmt = match fourcc_to_frameformat(ff) {
                Some(s) => s,
                None => continue,
            };
            // i write unmaintainable blobs of code because i am so cute uwu~~
            let mut formats = device
                .enum_framesizes(ff)
                .map_err(|why| NokhwaError::GetPropertyError {
                    property: "ResolutionList".to_string(),
                    error: why.to_string(),
                })?
                .into_iter()
                .flat_map(|x| match x.size {
                    FrameSizeEnum::Discrete(d) => Some(d).into_iter(),
                    // we step over each step, getting a new resolution.
                    FrameSizeEnum::Stepwise(s) => (s.min_width..s.max_width)
                        .step_by(s.step_width as usize)
                        .zip((s.min_height..s.max_height).step_by(s.step_height as usize))
                        .map(|(x, y)| Resolution::new(x, y))
                        .into_iter(),
                })
                .flat_map(|res| {
                    device
                        .enum_frameintervals(ff, res.x(), res.y())?
                        .iter()
                        .flat_map(|x| match &x.interval {
                            FrameIntervalEnum::Discrete(dis) => {
                                if dis.denominator == 1 {
                                    Some(CameraFormat::new(
                                        Resolution::new(x.width, x.height),
                                        framefmt,
                                        dis.numerator,
                                    ))
                                    .into_iter()
                                } else {
                                    None.into_iter()
                                }
                            }
                            FrameIntervalEnum::Stepwise(step) => {
                                let mut intvec = vec![];
                                for fstep in (step.min.numerator..step.max.numerator)
                                    .step_by(step.step.numerator as usize)
                                {
                                    if step.max.denominator != 1 || step.min.denominator != 1 {
                                        intvec.push(CameraFormat::new(
                                            Resolution::new(x.width, x.height),
                                            framefmt,
                                            fstep,
                                        ))
                                    }
                                }
                                intvec.into_iter()
                            }
                        })
                })
                .collect::<Result<Vec<CameraFormat>, NokhwaError>>()?;
            camera_formats.append(&mut formats);
        }

        let format = cam_fmt
            .fufill(&camera_formats)
            .ok_or(NokhwaError::GetPropertyError {
                property: "CameraFormat".to_string(),
                error: "Failed to Fufill".to_string(),
            })?;

        if let Err(why) = device.set_format(&Format::new(
            format.width(),
            format.height(),
            frameformat_to_fourcc(format.format()),
        )) {
            return Err(NokhwaError::SetPropertyError {
                property: "Resolution, FrameFormat".to_string(),
                value: format.to_string(),
                error: why.to_string(),
            });
        }
        if let Err(why) = device.set_params(&Parameters::with_fps(format.frame_rate())) {
            return Err(NokhwaError::SetPropertyError {
                property: "Frame rate".to_string(),
                value: frame_rate.to_string(),
                error: why.to_string(),
            });
        }

        let mut v4l2 = V4LCaptureDevice {
            initialized: false,
            camera_format: format,
            camera_info,
            device,
            stream_handle: None,
        };

        v4l2.force_refresh_camera_format()?;
        if v4l2.camera_format() == format {
            return Err(NokhwaError::SetPropertyError {
                property: "CameraFormat".to_string(),
                value: "".to_string(),
                error: "Not same/Rejected".to_string(),
            });
        }

        Ok(v4l2)
    }

    /// Create a new `V4L2` Camera with desired settings. This may or may not work.
    /// # Errors
    /// This function will error if the camera is currently busy or if `V4L2` can't read device information.
    #[deprecated(since = "0.10.0", note = "please use `new` instead.")]
    pub fn new_with(
        index: CameraIndex,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
    ) -> Result<Self, NokhwaError> {
        let camera_format = CameraFormat::new_from(width, height, fourcc, fps);
        V4LCaptureDevice::new(index, RequestedFormat::Exact(camera_format))
    }

    fn get_resolution_list(&self, fourcc: FrameFormat) -> Result<Vec<Resolution>, NokhwaError> {
        let format = frameformat_to_fourcc(fourcc);

        // match Capture::enum_framesizes(&self.device, format) {
        match self.device.enum_framesizes(format) {
            Ok(frame_sizes) => {
                let mut resolutions = vec![];
                for frame_size in frame_sizes {
                    match frame_size.size {
                        FrameSizeEnum::Discrete(dis) => {
                            resolutions.push(Resolution::new(dis.width, dis.height));
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
    /// apps  bloodtests  contact  css  images  index  index.html  injectionsupplies  transfem  transmasc
    #[allow(clippy::must_use_candidate)]
    pub fn inner_device(&self) -> &Device {
        &self.device
    }

    /// Get the inner device (mutable) for e.g. Controls
    pub fn inner_device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    /// Force refreshes the inner [`CameraFormat`] state.
    pub fn force_refresh_camera_format(&mut self) -> Result<(), NokhwaError> {
        match self.device.format() {
            Ok(format) => {
                let frame_format =
                    fourcc_to_frameformat(format.fourcc).ok_or(NokhwaError::GetPropertyError {
                        property: "FrameFormat".to_string(),
                        error: "unsupported".to_string(),
                    })?;

                let fps = match self.device.params() {
                    Ok(params) => {
                        if params.interval.denominator != 1
                            || params.interval.numerator % params.interval.denominator != 0
                        {
                            return Err(NokhwaError::GetPropertyError {
                                property: "V4L2 FrameRate".to_string(),
                                error: format!(
                                    "Framerate not whole number: {} / {}",
                                    params.interval.numerator, params.interval.denominator
                                ),
                            });
                        }

                        if params.interval.denominator != 1 {
                            params.interval.numerator / params.interval.denominator
                        } else {
                            params.interval.numerator
                        }
                    }
                    Err(why) => {
                        return Err(NokhwaError::GetPropertyError {
                            property: "V4L2 FrameRate".to_string(),
                            error: why.to_string(),
                        })
                    }
                };

                self.camera_format = CameraFormat::new(
                    Resolution::new(format.width, format.height),
                    frame_format,
                    fps,
                );
                Ok(())
            }
            Err(why) => Err(NokhwaError::GetPropertyError {
                property: "parameters".to_string(),
                error: why.to_string(),
            }),
        }
    }
}

impl<'a> CaptureBackendTrait for V4LCaptureDevice<'a> {
    fn backend(&self) -> ApiBackend {
        ApiBackend::Video4Linux
    }

    fn camera_info(&self) -> &CameraInfo {
        &self.camera_info
    }

    fn refresh_camera_format(&mut self) -> Result<(), NokhwaError> {
        self.force_refresh_camera_format()
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
                    property: "Frame rate".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let format: Format = new_fmt.into();
        let frame_rate = Parameters::with_fps(new_fmt.frame_rate());

        if let Err(why) = Capture::set_format(&self.device, &format) {
            return Err(NokhwaError::SetPropertyError {
                property: "Resolution, FrameFormat".to_string(),
                value: format.to_string(),
                error: why.to_string(),
            });
        }
        if let Err(why) = Capture::set_params(&self.device, &frame_rate) {
            return Err(NokhwaError::SetPropertyError {
                property: "Frame rate".to_string(),
                value: frame_rate.to_string(),
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
                            property: format!("Attempt undo due to stream acquisition failure with error {}. Resolution, FrameFormat", why),
                            value: prev_format.to_string(),
                            error: why.to_string(),
                        });
                    }
                    if let Err(why) = Capture::set_params(&self.device, &prev_fps) {
                        return Err(NokhwaError::SetPropertyError {
                            property:
                            format!("Attempt undo due to stream acquisition failure with error {}. Frame rate", why),
                            value: prev_fps.to_string(),
                            error: why.to_string(),
                        });
                    }
                    Err(why)
                }
            };
        }
        self.camera_format = new_fmt;

        self.force_refresh_camera_format()?;
        if self.camera_format != new_fmt {
            return Err(NokhwaError::SetPropertyError {
                property: "CameraFormat".to_string(),
                value: new_fmt.to_string(),
                error: "Rejected".to_string(),
            });
        }

        Ok(())
    }

    fn compatible_list_by_resolution(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        let resolutions = self.get_resolution_list(fourcc)?;
        let format = frameformat_to_fourcc(fourcc);
        let mut res_map = HashMap::new();
        for res in resolutions {
            let mut compatible_fps = vec![];
            match self.device.enum_frameintervals(format) {
                Ok(intervals) => {
                    for interval in intervals {
                        match interval.interval {
                            FrameIntervalEnum::Discrete(dis) => {
                                compatible_fps.push(dis.denominator);
                            }
                            FrameIntervalEnum::Stepwise(step) => {
                                for fstep in (step.min.numerator..step.max.numerator)
                                    .step_by(step.step.numerator as usize)
                                {
                                    if step.max.denominator != 1 || step.min.denominator != 1 {
                                        compatible_fps.push(fstep)
                                    }
                                }
                            }
                        }
                    }
                }
                Err(why) => {
                    return Err(NokhwaError::GetPropertyError {
                        property: "Frame rate".to_string(),
                        error: why.to_string(),
                    })
                }
            }
            res_map.insert(res, compatible_fps);
        }
        Ok(res_map)
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        match self.device.enum_formats() {
            Ok(formats) => {
                let mut frame_format_vec = vec![];
                for format in formats {
                    match fourcc_to_frameformat(format.fourcc) {
                        Some(ff) => frame_format_vec.push(ff),
                        None => continue,
                    }
                }
                frame_format_vec.sort();
                frame_format_vec.dedup();
                Ok(frame_format_vec)
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

    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        let mut new_fmt = self.camera_format;
        new_fmt.set_resolution(new_res);
        self.set_camera_format(new_fmt)
    }

    fn frame_rate(&self) -> u32 {
        self.camera_format.frame_rate()
    }

    fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        let mut new_fmt = self.camera_format;
        new_fmt.set_frame_rate(new_fps);
        self.set_camera_format(new_fmt)
    }

    fn frame_format(&self) -> FrameFormat {
        self.camera_format.format()
    }

    fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        let mut new_fmt = self.camera_format;
        new_fmt.set_format(fourcc);
        self.set_camera_format(new_fmt)
    }

    fn camera_control(&self, control: KnownCameraControl) -> Result<CameraControl, NokhwaError> {
        let controls = self.camera_controls()?;
        for supported_control in controls {
            if supported_control.control() == control {
                return Ok(supported_control);
            }
        }
        Err(NokhwaError::GetPropertyError {
            property: control.to_string(),
            error: "not found/not supported".to_string(),
        })
    }

    fn camera_controls(&self) -> Result<Vec<CameraControl>, NokhwaError> {
        self.device
            .query_controls()
            .map_err(|why| NokhwaError::GetPropertyError {
                property: "V4L2 Controls".to_string(),
                error: why.to_string(),
            })?
            .into_iter()
            .map(|desc| {
                let id_as_kcc = id_to_known_camera_control(desc.id);
                let ctrl_current = self.device.control(desc.id)?.value;

                let ctrl_value_desc = match (desc.typ, ctrl_current) {
                    (
                        Type::Integer
                        | Type::Integer64
                        | Type::Menu
                        | Type::U8
                        | Type::U16
                        | Type::U32
                        | Type::IntegerMenu,
                        Value::Integer(current),
                    ) => ControlValueDescription::IntegerRange {
                        min: desc.minimum as i64,
                        max: desc.maximum,
                        value: current,
                        step: desc.step as i64,
                        default: desc.default,
                    },
                    (Type::Boolean, Value::Boolean(current)) => ControlValueDescription::Boolean {
                        value: current,
                        default: desc.default != 0,
                    },

                    (Type::String, Value::String(current)) => ControlValueDescription::String {
                        value: current,
                        default: None,
                    },
                    _ => {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            "what is this?????? todo: support ig",
                        ))
                    }
                };

                let is_readonly = desc
                    .flags
                    .intersects(Flags::READ_ONLY)
                    .then(|| KnownCameraControlFlag::ReadOnly);
                let is_writeonly = desc
                    .flags
                    .intersects(Flags::WRITE_ONLY)
                    .then(|| KnownCameraControlFlag::WriteOnly);
                let is_disabled = desc
                    .flags
                    .intersects(Flags::DISABLED)
                    .then(|| KnownCameraControlFlag::Disabled);
                let is_volatile = desc
                    .flags
                    .intersects(Flags::VOLATILE)
                    .then(|| KnownCameraControlFlag::Volatile);
                let is_inactive = desc
                    .flags
                    .intersects(Flags::INACTIVE)
                    .then(|| KnownCameraControlFlag::Inactive);
                let flags_vec = vec![
                    is_inactive,
                    is_readonly,
                    is_volatile,
                    is_disabled,
                    is_writeonly,
                ]
                .into_iter()
                .filter(|x| x.is_some())
                .collect::<Option<Vec<KnownCameraControlFlag>>>()
                .unwrap_or_default();

                Ok(CameraControl::new(
                    id_as_kcc,
                    desc.name,
                    ctrl_value_desc,
                    flags_vec,
                    !desc.flags.intersects(Flags::INACTIVE),
                ))
            })
            .filter(|x| x.is_ok())
            .collect::<Result<Vec<CameraControl>, io::Error>>()
            .map_err(|x| NokhwaError::GetPropertyError {
                property: "www".to_string(),
                error: x.to_string(),
            })
    }

    fn set_camera_control(
        &mut self,
        id: KnownCameraControl,
        value: ControlValueSetter,
    ) -> Result<(), NokhwaError> {
        let value = match value {
            ControlValueSetter::None => Value::None,
            ControlValueSetter::Integer(i) => Value::Integer(i),
            ControlValueSetter::Float(f) => {
                return Err(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: f.to_string(),
                    error: "not supported".to_string(),
                })
            }
            ControlValueSetter::Boolean(b) => Value::Boolean(b),
            ControlValueSetter::String(s) => Value::String(s),
            ControlValueSetter::Bytes(b) => Value::CompoundU8(b),
        };
        self.device
            .set_control(Control {
                id: known_camera_control_to_id(id),
                value,
            })
            .map_err(|why| NokhwaError::SetPropertyError {
                property: id.to_string(),
                value: value.to_string(),
                error: why.to_string(),
            })?;
        // verify

        let control = self.camera_control(id)?;
        if control.value().value() == value {
            return Ok(());
        }
        return Err(NokhwaError::SetPropertyError {
            property: id.to_string(),
            value: value.to_string(),
            error: "Rejected".to_string(),
        });
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

    fn frame(&mut self) -> Result<Buffer, NokhwaError> {
        let cam_fmt = self.camera_format;
        let raw_frame = self.frame_raw()?;
        let conv = match cam_fmt.format() {
            FrameFormat::MJPEG => mjpeg_to_rgb(&raw_frame, false)?,
            FrameFormat::YUYV => yuyv422_to_rgb(&raw_frame, false)?,
            FrameFormat::GRAY => raw_frame.to_vec(),
        };
        Ok(Buffer::new_with_vec(
            cam_fmt.resolution(),
            conv,
            cam_fmt.format(),
        ))
    }

    fn frame_raw(&mut self) -> Result<Cow<'a, [u8]>, NokhwaError> {
        match &self.stream_handle {
            Some(mut sh) => match sh.next() {
                Ok((data, _)) => Ok(Cow::Borrowed(data)),
                Err(why) => return Err(NokhwaError::ReadFrameError(why.to_string())),
            },
            None => {}
        }
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        if self.stream_handle.is_some() {
            self.stream_handle = None;
        }
        Ok(())
    }
}

fn fourcc_to_frameformat(fourcc: FourCC) -> Option<FrameFormat> {
    match fourcc.str().ok()? {
        "YUYV" => Some(FrameFormat::YUYV),
        "MJPG" => Some(FrameFormat::MJPEG),
        "GRAY" => Some(FrameFormat::GRAY),
        _ => None,
    }
}

fn frameformat_to_fourcc(fourcc: FrameFormat) -> FourCC {
    match fourcc {
        FrameFormat::MJPEG => FourCC::new(b"MJPG"),
        FrameFormat::YUYV => FourCC::new(b"YUYV"),
        FrameFormat::GRAY => FourCC::new(b"GRAY"),
    }
}
