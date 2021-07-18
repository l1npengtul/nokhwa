#![deny(clippy::pedantic)]
#![warn(clippy::all)]
#![allow(clippy::must_use_candidate)]

//! # nokhwa-bindings-windows
//! This crate is the `MediaFoundation` bindings for the `nokhwa` crate.
//!
//! It is not meant for general consumption. If you are looking for a Windows camera capture crate, consider using `nokhwa` with feature `input-msmf`.
//!
//! No support or API stability will be given. Subject to change at any time.

use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    slice::from_raw_parts,
};
use thiserror::Error;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::pub_enum_variant_names)]
#[derive(Error, Debug, Clone)]
pub enum BindingError {
    #[error("Failed to initialize Media Foundation: {0}")]
    InitializeError(String),
    #[error("Failed to de-initialize Media Foundation: {0}")]
    DeInitializeError(String),
    #[error("Failed to set GUID {0} to {1}: {2}")]
    GUIDSetError(String, String, String),
    #[error("Failed to Read GUID {0}: {1}")]
    GUIDReadError(String, String),
    #[error("Attribute Error: {0}")]
    AttributeError(String),
    #[error("Failed to enumerate: {0}")]
    EnumerateError(String),
    #[error("Failed to open device {0}: {1}")]
    DeviceOpenFailError(usize, String),
    #[error("Failed to read frame: {0}")]
    ReadFrameError(String),
    #[error("Not Implemented!")]
    NotImplementedError,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MFResolution {
    pub width_x: u32,
    pub height_y: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Hash, PartialOrd, Ord, Eq)]
pub enum MFFrameFormat {
    MJPEG,
    YUYV,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub struct MFCameraFormat {
    resolution: MFResolution,
    format: MFFrameFormat,
    frame_rate: u32,
}

impl Default for MFCameraFormat {
    fn default() -> Self {
        MFCameraFormat {
            resolution: MFResolution {
                width_x: 640,
                height_y: 480,
            },
            format: MFFrameFormat::MJPEG,
            frame_rate: 15,
        }
    }
}

impl MFCameraFormat {
    pub fn new(resolution: MFResolution, format: MFFrameFormat, frame_rate: u32) -> Self {
        MFCameraFormat {
            resolution,
            format,
            frame_rate: frame_rate,
        }
    }

    pub fn new_from(res_x: u32, res_y: u32, format: MFFrameFormat, fps: u32) -> Self {
        MFCameraFormat {
            resolution: MFResolution {
                width_x: res_x,
                height_y: res_y,
            },
            format,
            frame_rate: fps,
        }
    }

    pub fn resolution(&self) -> MFResolution {
        self.resolution
    }

    pub fn width(&self) -> u32 {
        self.resolution.width_x
    }

    pub fn height(&self) -> u32 {
        self.resolution.height_y
    }

    pub fn set_resolution(&mut self, resolution: MFResolution) {
        self.resolution = resolution;
    }

    pub fn framerate(&self) -> u32 {
        self.frame_rate
    }

    pub fn set_framerate(&mut self, framerate: u32) {
        self.frame_rate = framerate;
    }

    pub fn format(&self) -> MFFrameFormat {
        self.format
    }

    pub fn set_format(&mut self, format: MFFrameFormat) {
        self.format = format;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaFoundationDeviceDescriptor<'a> {
    index: usize,
    name: Cow<'a, [u16]>,
    symlink: Cow<'a, [u16]>,
}

impl<'a> MediaFoundationDeviceDescriptor<'a> {
    /// # Errors
    /// If name or symlink is a nullptr, this will error.
    /// # Safety
    /// name and symlink must not be null
    pub unsafe fn new(
        index: usize,
        name: *mut u16,
        symlink: *mut u16,
        name_len: u32,
        symlink_len: u32,
    ) -> Result<Self, BindingError> {
        let name = if name.is_null() {
            return Err(BindingError::AttributeError("name nullptr".to_string()));
        } else {
            Cow::from(from_raw_parts(name, name_len as usize))
        };

        let symlink = if symlink.is_null() {
            return Err(BindingError::AttributeError("symlink nullptr".to_string()));
        } else {
            Cow::from(from_raw_parts(symlink, symlink_len as usize))
        };

        Ok(MediaFoundationDeviceDescriptor {
            index,
            name,
            symlink,
        })
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> &Cow<[u16]> {
        &self.name
    }

    pub fn symlink(&self) -> &Cow<[u16]> {
        &self.symlink
    }

    pub fn name_as_string(&self) -> String {
        String::from_utf16_lossy(self.name.borrow())
    }

    pub fn link_as_string(&self) -> String {
        String::from_utf16_lossy(self.symlink.borrow())
    }
}

impl<'a> PartialOrd for MediaFoundationDeviceDescriptor<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for MediaFoundationDeviceDescriptor<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum MediaFoundationControls {
    Brightness,
    Contrast,
    Hue,
    Saturation,
    Sharpness,
    Gamma,
    ColorEnable,
    WhiteBalance,
    BacklightComp,
    Gain,
    Pan,
    Tilt,
    Roll,
    Zoom,
    Exposure,
    Iris,
    Focus,
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct MFControl {
    control: MediaFoundationControls,
    min: i32,
    max: i32,
    step: i32,
    current: i32,
    default: i32,
    manual: bool,
    active: bool,
}

impl MFControl {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        control: MediaFoundationControls,
        min: i32,
        max: i32,
        step: i32,
        current: i32,
        default: i32,
        manual: bool,
        active: bool,
    ) -> Self {
        MFControl {
            control,
            min,
            max,
            step,
            current,
            default,
            manual,
            active,
        }
    }

    pub fn control(&self) -> MediaFoundationControls {
        self.control
    }

    pub fn set_control(&mut self, control: MediaFoundationControls) {
        self.control = control;
    }

    pub fn min(&self) -> i32 {
        self.min
    }

    pub fn set_min(&mut self, min: i32) {
        self.min = min;
    }

    pub fn max(&self) -> i32 {
        self.max
    }

    pub fn set_max(&mut self, max: i32) {
        self.max = max;
    }

    pub fn step(&self) -> i32 {
        self.step
    }

    pub fn set_step(&mut self, step: i32) {
        self.step = step;
    }

    pub fn current(&self) -> i32 {
        self.current
    }

    pub fn set_current(&mut self, current: i32) {
        self.current = current;
    }
    pub fn default(&self) -> i32 {
        self.default
    }

    pub fn set_default(&mut self, default: i32) {
        self.default = default;
    }

    pub fn manual(&self) -> bool {
        self.manual
    }

    pub fn set_manual(&mut self, manual: bool) {
        self.manual = manual;
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

#[cfg(all(windows, not(feature = "docs-only")))]
windows::include_bindings!();

#[cfg(all(windows, not(feature = "docs-only")))]
pub mod wmf {
    use crate::{
        BindingError, MFCameraFormat, MFControl, MFFrameFormat, MFResolution,
        MediaFoundationControls, MediaFoundationDeviceDescriptor,
        Windows::Win32::{
            Foundation::PWSTR,
            Graphics::DirectShow::{
                CameraControl_Exposure, CameraControl_Focus, CameraControl_Iris, CameraControl_Pan,
                CameraControl_Roll, CameraControl_Tilt, CameraControl_Zoom, IAMCameraControl,
                IAMVideoProcAmp, VideoProcAmp_BacklightCompensation, VideoProcAmp_Brightness,
                VideoProcAmp_ColorEnable, VideoProcAmp_Contrast, VideoProcAmp_Gain,
                VideoProcAmp_Gamma, VideoProcAmp_Hue, VideoProcAmp_Saturation,
                VideoProcAmp_Sharpness, VideoProcAmp_WhiteBalance,
            },
            Media::MediaFoundation::{
                IMFActivate, IMFAttributes, IMFMediaSource, IMFSample, IMFSourceReader,
                MFCreateAttributes, MFCreateDeviceSource, MFCreateMediaType,
                MFCreateSourceReaderFromMediaSource, MFEnumDeviceSources, MFMediaType_Video,
                MFShutdown, MFStartup, MFSTARTUP_NOSOCKET, MF_API_VERSION,
                MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK, MF_MT_FRAME_RATE,
                MF_MT_FRAME_RATE_RANGE_MAX, MF_MT_FRAME_RATE_RANGE_MIN, MF_MT_FRAME_SIZE,
                MF_MT_MAJOR_TYPE, MF_MT_SUBTYPE, MF_SOURCE_READER_FIRST_VIDEO_STREAM,
            },
            System::Com::{CoInitializeEx, CoUninitialize, COINIT},
        },
    };
    use std::{borrow::Cow, cell::Cell, ffi::c_void, mem::MaybeUninit, slice::from_raw_parts};
    use windows::{Guid, Interface};

    static mut INITIALIZED: bool = false;

    // See: https://stackoverflow.com/questions/80160/what-does-coinit-speed-over-memory-do
    const CO_INIT_APARTMENT_THREADED: COINIT = COINIT(0x2);
    const CO_INIT_DISABLE_OLE1DDE: COINIT = COINIT(0x4);

    // See: https://gix.github.io/media-types/#major-types
    const MF_VIDEO_FORMAT_YUY2: Guid = Guid::from_values(
        0x3259_5559,
        0x0000,
        0x0010,
        [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
    );
    const MF_VIDEO_FORMAT_MJPEG: Guid = Guid::from_values(
        0x4750_4A4D,
        0x0000,
        0x0010,
        [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
    );

    const CAM_CTRL_AUTO: i32 = 0x0001;
    const CAM_CTRL_MANUAL: i32 = 0x0002;

    pub fn initialize_mf() -> Result<(), BindingError> {
        if !(unsafe { INITIALIZED }) {
            let a = Box::new(0);
            let c_void = { Box::into_raw(a).cast::<c_void>() };
            if let Err(why) = unsafe {
                CoInitializeEx(c_void, CO_INIT_APARTMENT_THREADED | CO_INIT_DISABLE_OLE1DDE)
            } {
                return Err(BindingError::InitializeError(why.to_string()));
            }

            if let Err(why) = unsafe { MFStartup(MF_API_VERSION, MFSTARTUP_NOSOCKET) } {
                unsafe {
                    CoUninitialize();
                }
                return Err(BindingError::InitializeError(why.to_string()));
            }

            unsafe {
                INITIALIZED = true;
            }

            // reconstruct the box and free the RAM
            let _b = unsafe { Box::from_raw(c_void) }; // automatic cleanup
        }
        Ok(())
    }

    pub fn deinitialize_mf() -> Result<(), BindingError> {
        if unsafe { INITIALIZED } {
            unsafe {
                if let Err(why) = MFShutdown() {
                    return Err(BindingError::DeInitializeError(why.to_string()));
                }
                CoUninitialize();
                INITIALIZED = false;
            }
        }
        Ok(())
    }

    pub fn query_msmf() -> Result<Vec<MediaFoundationDeviceDescriptor<'static>>, BindingError> {
        initialize_mf()?;

        let mut attributes: Option<IMFAttributes> = None;
        if let Err(why) = unsafe { MFCreateAttributes(&mut attributes, 1) } {
            return Err(BindingError::AttributeError(why.to_string()));
        }

        let attributes = match attributes {
            Some(attr) => {
                if let Err(why) = unsafe {
                    attr.SetGUID(
                        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                    )
                } {
                    return Err(BindingError::AttributeError(why.to_string()));
                }
                attr
            }
            None => {
                return Err(BindingError::AttributeError(
                    "Attributes is null!".to_string(),
                ));
            }
        };

        let mut count: u32 = 0;
        let mut mf_device_enum: MaybeUninit<*mut Option<IMFActivate>> = MaybeUninit::uninit();

        if let Err(why) =
            unsafe { MFEnumDeviceSources(attributes, mf_device_enum.as_mut_ptr(), &mut count) }
        {
            return Err(BindingError::EnumerateError(why.to_string()));
        }

        let mut device_list = vec![];

        for (index, mf_activate) in
            unsafe { from_raw_parts(mf_device_enum.assume_init(), count as usize) }
                .into_iter()
                .enumerate()
        {
            match mf_activate {
                Some(activate_ptr) => {
                    let mut name: PWSTR = PWSTR(&mut 0_u16);
                    let mut len_name = 0;
                    let mut symlink: PWSTR = PWSTR(&mut 0_u16);
                    let mut len_symlink = 0;

                    if let Err(why) = unsafe {
                        activate_ptr.GetAllocatedString(
                            &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                            &mut name,
                            &mut len_name,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) = unsafe {
                        activate_ptr.GetAllocatedString(
                            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                            &mut symlink,
                            &mut len_symlink,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK".to_string(),
                            why.to_string(),
                        ));
                    }

                    let device_descriptor = unsafe {
                        MediaFoundationDeviceDescriptor::new(
                            index,
                            name.0,
                            symlink.0,
                            len_name,
                            len_symlink,
                        )
                    }?;

                    device_list.push(device_descriptor);
                }
                None => {
                    continue; // swallow errors
                }
            }
        }
        Ok(device_list)
    }

    pub struct MediaFoundationDevice {
        is_open: Cell<bool>,
        device_specifier: MediaFoundationDeviceDescriptor<'static>,
        device_format: MFCameraFormat,
        media_source: IMFMediaSource,
        source_reader: IMFSourceReader,
    }

    impl MediaFoundationDevice {
        pub fn new(index: usize) -> Result<Self, BindingError> {
            let device_descriptor = {
                match query_msmf()?.into_iter().nth(index) {
                    Some(dd) => dd,
                    None => {
                        return Err(BindingError::DeviceOpenFailError(
                            index,
                            "No Device".to_string(),
                        ))
                    }
                }
            };

            let attributes = {
                let cow_vec = device_descriptor.symlink.to_vec().as_mut_ptr();
                let symlink_pwstr = PWSTR(cow_vec);

                let mut attr: Option<IMFAttributes> = None;

                if let Err(why) = unsafe { MFCreateAttributes(&mut attr, 2) } {
                    return Err(BindingError::AttributeError(why.to_string()));
                }

                let attr = match attr {
                    Some(at) => {
                        if let Err(why) = unsafe {
                            at.SetGUID(
                                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                            )
                        } {
                            return Err(BindingError::AttributeError(why.to_string()));
                        }

                        if let Err(why) = unsafe {
                            at.SetString(
                                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                                &symlink_pwstr,
                            )
                        } {
                            return Err(BindingError::AttributeError(why.to_string()));
                        }
                        at
                    }
                    None => {
                        return Err(BindingError::DeviceOpenFailError(
                            index,
                            "Attribute Error".to_string(),
                        ))
                    }
                };

                attr
            };

            let media_source = match unsafe { MFCreateDeviceSource(&attributes) } {
                Ok(src) => src,
                Err(why) => return Err(BindingError::DeviceOpenFailError(index, why.to_string())),
            };

            let source_reader = match unsafe {
                MFCreateSourceReaderFromMediaSource(&media_source, None)
            } {
                Ok(sr) => sr,
                Err(why) => return Err(BindingError::DeviceOpenFailError(index, why.to_string())),
            };

            Ok(MediaFoundationDevice {
                is_open: Cell::new(false),
                device_specifier: device_descriptor,
                device_format: MFCameraFormat::default(),
                media_source,
                source_reader,
            })
        }

        pub fn index(&self) -> usize {
            self.device_specifier.index
        }

        pub fn name(&self) -> String {
            self.device_specifier.name_as_string()
        }

        pub fn symlink(&self) -> String {
            self.device_specifier.link_as_string()
        }

        pub fn compatible_format_list(&mut self) -> Result<Vec<MFCameraFormat>, BindingError> {
            let mut camera_format_list = vec![];
            let mut index = 0;

            loop {
                let media_type = match unsafe {
                    self.source_reader
                        .GetNativeMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, index)
                } {
                    Ok(mt) => mt,
                    Err(why) => {
                        if why.code().0 == 14009 {
                            break;
                        }
                        return Err(BindingError::GUIDReadError(
                            "MF_SOURCE_READER_FIRST_VIDEO_STREAM".to_string(),
                            why.to_string(),
                        ));
                    }
                };

                let fourcc = match unsafe { media_type.GetGUID(&MF_MT_SUBTYPE) } {
                    Ok(fcc) => fcc,
                    Err(why) => {
                        return Err(BindingError::GUIDReadError(
                            "MF_MT_SUBTYPE".to_string(),
                            why.to_string(),
                        ))
                    }
                };

                let (width, height) = match unsafe { media_type.GetUINT64(&MF_MT_FRAME_SIZE) } {
                    Ok(res_u64) => {
                        let width = (res_u64 >> 32) as u32;
                        let height = res_u64 as u32; // the cast will truncate the upper bits
                        (width, height)
                    }
                    Err(why) => {
                        return Err(BindingError::GUIDReadError(
                            "MF_MT_FRAME_SIZE".to_string(),
                            why.to_string(),
                        ))
                    }
                };

                // MFRatio is represented as 2 u32s in memory. This means we cann convert it to 2
                let frame_rate_max =
                    match unsafe { media_type.GetUINT64(&MF_MT_FRAME_RATE_RANGE_MAX) } {
                        Ok(fraction_u64) => {
                            let mut numerator = (fraction_u64 >> 32) as u32;
                            let denominator = fraction_u64 as u32;
                            if denominator != 0 {
                                numerator = 0;
                            }
                            numerator
                        }
                        Err(why) => {
                            return Err(BindingError::GUIDReadError(
                                "MF_MT_FRAME_RATE_RANGE_MAX".to_string(),
                                why.to_string(),
                            ))
                        }
                    };

                let frame_rate = match unsafe { media_type.GetUINT64(&MF_MT_FRAME_RATE) } {
                    Ok(fraction_u64) => {
                        let mut numerator = (fraction_u64 >> 32) as u32;
                        let denominator = fraction_u64 as u32;
                        if denominator != 0 {
                            numerator = 0;
                        }
                        numerator
                    }
                    Err(why) => {
                        return Err(BindingError::GUIDReadError(
                            "MF_MT_FRAME_RATE".to_string(),
                            why.to_string(),
                        ))
                    }
                };

                let frame_rate_min =
                    match unsafe { media_type.GetUINT64(&MF_MT_FRAME_RATE_RANGE_MIN) } {
                        Ok(fraction_u64) => {
                            let mut numerator = (fraction_u64 >> 32) as u32;
                            let denominator = fraction_u64 as u32;
                            if denominator != 0 {
                                numerator = 0;
                            }
                            numerator
                        }
                        Err(why) => {
                            return Err(BindingError::GUIDReadError(
                                "MF_MT_FRAME_RATE_RANGE_MIN".to_string(),
                                why.to_string(),
                            ))
                        }
                    };

                if fourcc == MF_VIDEO_FORMAT_MJPEG {
                    if frame_rate_min != 0 {
                        camera_format_list.push(MFCameraFormat {
                            resolution: MFResolution {
                                width_x: width,
                                height_y: height,
                            },
                            format: MFFrameFormat::MJPEG,
                            frame_rate: frame_rate_min,
                        });
                    }

                    if frame_rate != 0 && frame_rate_min != frame_rate {
                        camera_format_list.push(MFCameraFormat {
                            resolution: MFResolution {
                                width_x: width,
                                height_y: height,
                            },
                            format: MFFrameFormat::MJPEG,
                            frame_rate,
                        });
                    }

                    if frame_rate_max != 0 && frame_rate != frame_rate_max {
                        camera_format_list.push(MFCameraFormat {
                            resolution: MFResolution {
                                width_x: width,
                                height_y: height,
                            },
                            format: MFFrameFormat::MJPEG,
                            frame_rate: frame_rate_max,
                        });
                    }
                } else if fourcc == MF_VIDEO_FORMAT_YUY2 {
                    if frame_rate_min != 0 {
                        camera_format_list.push(MFCameraFormat {
                            resolution: MFResolution {
                                width_x: width,
                                height_y: height,
                            },
                            format: MFFrameFormat::YUYV,
                            frame_rate: frame_rate_min,
                        });
                    }

                    if frame_rate != 0 && frame_rate_min != frame_rate {
                        camera_format_list.push(MFCameraFormat {
                            resolution: MFResolution {
                                width_x: width,
                                height_y: height,
                            },
                            format: MFFrameFormat::YUYV,
                            frame_rate: frame_rate,
                        });
                    }

                    if frame_rate_max != 0 && frame_rate != frame_rate_max {
                        camera_format_list.push(MFCameraFormat {
                            resolution: MFResolution {
                                width_x: width,
                                height_y: height,
                            },
                            format: MFFrameFormat::YUYV,
                            frame_rate: frame_rate_max,
                        });
                    }
                }

                index = index + 1;
            }

            Ok(camera_format_list)
        }

        pub fn control(&self, control: MediaFoundationControls) -> Result<MFControl, BindingError> {
            let camera_control = match self.media_source.cast::<IAMCameraControl>() {
                Ok(cc) => cc,
                Err(why) => {
                    return Err(BindingError::GUIDReadError(
                        "IAMCameraControl".to_string(),
                        why.to_string(),
                    ))
                }
            };

            let video_proc_amp = match self.media_source.cast::<IAMVideoProcAmp>() {
                Ok(vpa) => vpa,
                Err(why) => {
                    return Err(BindingError::GUIDReadError(
                        "IAMVideoProcAmp".to_string(),
                        why.to_string(),
                    ))
                }
            };

            let mut min = 0;
            let mut max = 0;
            let mut step = 0;
            let mut default = 0;
            let mut value = 0;
            let mut flag = 0;

            match control {
                MediaFoundationControls::Brightness => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_Brightness.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Brightness-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) = unsafe {
                        video_proc_amp.Get(VideoProcAmp_Brightness.0, &mut value, &mut flag)
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Brightness-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Contrast => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_Contrast.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Contrast-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) = unsafe {
                        video_proc_amp.Get(VideoProcAmp_Contrast.0, &mut value, &mut flag)
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Contrast-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Hue => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_Hue.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Hue-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) =
                        unsafe { video_proc_amp.Get(VideoProcAmp_Hue.0, &mut value, &mut flag) }
                    {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Hue-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Saturation => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_Saturation.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Saturation-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) = unsafe {
                        video_proc_amp.Get(VideoProcAmp_Saturation.0, &mut value, &mut flag)
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Saturation-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Sharpness => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_Sharpness.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Sharpness-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) = unsafe {
                        video_proc_amp.Get(VideoProcAmp_Sharpness.0, &mut value, &mut flag)
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Sharpness-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Gamma => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_Gamma.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Gamma-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) =
                        unsafe { video_proc_amp.Get(VideoProcAmp_Gamma.0, &mut value, &mut flag) }
                    {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Gamma-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::ColorEnable => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_ColorEnable.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_ColorEnable-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) = unsafe {
                        video_proc_amp.Get(VideoProcAmp_ColorEnable.0, &mut value, &mut flag)
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_ColorEnable-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::WhiteBalance => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_WhiteBalance.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_WhiteBalance-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) = unsafe {
                        video_proc_amp.Get(VideoProcAmp_WhiteBalance.0, &mut value, &mut flag)
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_WhiteBalance-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::BacklightComp => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_BacklightCompensation.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_BacklightCompensation-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) = unsafe {
                        video_proc_amp.Get(
                            VideoProcAmp_BacklightCompensation.0,
                            &mut value,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_BacklightCompensation-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Gain => {
                    if let Err(why) = unsafe {
                        video_proc_amp.GetRange(
                            VideoProcAmp_Gain.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Gain-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) =
                        unsafe { video_proc_amp.Get(VideoProcAmp_Gain.0, &mut value, &mut flag) }
                    {
                        return Err(BindingError::GUIDReadError(
                            "VideoProcAmp_Gain-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Pan => {
                    if let Err(why) = unsafe {
                        camera_control.GetRange(
                            CameraControl_Pan.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Pan-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) =
                        unsafe { camera_control.Get(CameraControl_Pan.0, &mut value, &mut flag) }
                    {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Pan-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Tilt => {
                    if let Err(why) = unsafe {
                        camera_control.GetRange(
                            CameraControl_Tilt.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Tilt-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) =
                        unsafe { camera_control.Get(CameraControl_Tilt.0, &mut value, &mut flag) }
                    {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Tilt-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Roll => {
                    if let Err(why) = unsafe {
                        camera_control.GetRange(
                            CameraControl_Roll.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Roll-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) =
                        unsafe { camera_control.Get(CameraControl_Roll.0, &mut value, &mut flag) }
                    {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Roll-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Zoom => {
                    if let Err(why) = unsafe {
                        camera_control.GetRange(
                            CameraControl_Zoom.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Zoom-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) =
                        unsafe { camera_control.Get(CameraControl_Zoom.0, &mut value, &mut flag) }
                    {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Zoom-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Exposure => {
                    if let Err(why) = unsafe {
                        camera_control.GetRange(
                            CameraControl_Exposure.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Exposure-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) = unsafe {
                        camera_control.Get(CameraControl_Exposure.0, &mut value, &mut flag)
                    } {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Exposure-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Iris => {
                    if let Err(why) = unsafe {
                        camera_control.GetRange(
                            CameraControl_Iris.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Iris-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) =
                        unsafe { camera_control.Get(CameraControl_Iris.0, &mut value, &mut flag) }
                    {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Iris-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Focus => {
                    if let Err(why) = unsafe {
                        camera_control.GetRange(
                            CameraControl_Focus.0,
                            &mut min,
                            &mut max,
                            &mut step,
                            &mut default,
                            &mut flag,
                        )
                    } {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Focus-Range".to_string(),
                            why.to_string(),
                        ));
                    }

                    if let Err(why) =
                        unsafe { camera_control.Get(CameraControl_Focus.0, &mut value, &mut flag) }
                    {
                        return Err(BindingError::GUIDReadError(
                            "CameraControl_Focus-Value".to_string(),
                            why.to_string(),
                        ));
                    }
                }
            }

            let is_manual = match flag {
                CAM_CTRL_MANUAL => true,
                _ => false,
            };

            Ok(MFControl::new(
                control, min, max, step, value, default, is_manual, true,
            ))
        }

        pub fn set_control(&mut self, control: MFControl) -> Result<(), BindingError> {
            let camera_control = match self.media_source.cast::<IAMCameraControl>() {
                Ok(cc) => cc,
                Err(why) => {
                    return Err(BindingError::GUIDReadError(
                        "IAMCameraControl".to_string(),
                        why.to_string(),
                    ))
                }
            };

            let video_proc_amp = match self.media_source.cast::<IAMVideoProcAmp>() {
                Ok(vpa) => vpa,
                Err(why) => {
                    return Err(BindingError::GUIDReadError(
                        "IAMVideoProcAmp".to_string(),
                        why.to_string(),
                    ))
                }
            };

            let value = control.current;
            let flags = match control.manual {
                true => CAM_CTRL_MANUAL,
                false => CAM_CTRL_AUTO,
            };
            let flag_str = match control.manual {
                true => "CAM_CTRL_MANUAL",
                false => "CAM_CTRL_AUTO",
            };

            match control.control {
                MediaFoundationControls::Brightness => {
                    if let Err(why) =
                        unsafe { video_proc_amp.Set(VideoProcAmp_Brightness.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_Brightness".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Contrast => {
                    if let Err(why) =
                        unsafe { video_proc_amp.Set(VideoProcAmp_Contrast.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_Contrast".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Hue => {
                    if let Err(why) =
                        unsafe { video_proc_amp.Set(VideoProcAmp_Hue.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_Hue".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Saturation => {
                    if let Err(why) =
                        unsafe { video_proc_amp.Set(VideoProcAmp_Saturation.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_Saturation".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Sharpness => {
                    if let Err(why) =
                        unsafe { video_proc_amp.Set(VideoProcAmp_Sharpness.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_Sharpness".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Gamma => {
                    if let Err(why) =
                        unsafe { video_proc_amp.Set(VideoProcAmp_Gamma.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_Gamma".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::ColorEnable => {
                    if let Err(why) =
                        unsafe { video_proc_amp.Set(VideoProcAmp_ColorEnable.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_ColorEnable".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::WhiteBalance => {
                    if let Err(why) =
                        unsafe { video_proc_amp.Set(VideoProcAmp_WhiteBalance.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_WhiteBalance".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::BacklightComp => {
                    if let Err(why) = unsafe {
                        video_proc_amp.Set(VideoProcAmp_BacklightCompensation.0, value, flags)
                    } {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_BacklightCompensation".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Gain => {
                    if let Err(why) =
                        unsafe { video_proc_amp.Set(VideoProcAmp_Gain.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "VideoProcAmp_Gain".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Pan => {
                    if let Err(why) =
                        unsafe { camera_control.Set(CameraControl_Pan.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "CameraControl_Pan".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Tilt => {
                    if let Err(why) =
                        unsafe { camera_control.Set(CameraControl_Tilt.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "CameraControl_Tilt".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Roll => {
                    if let Err(why) =
                        unsafe { camera_control.Set(CameraControl_Roll.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "CameraControl_Roll".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Zoom => {
                    if let Err(why) =
                        unsafe { camera_control.Set(CameraControl_Zoom.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "CameraControl_Zoom".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Exposure => {
                    if let Err(why) =
                        unsafe { camera_control.Set(CameraControl_Exposure.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "CameraControl_Exposure".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Iris => {
                    if let Err(why) =
                        unsafe { camera_control.Set(CameraControl_Iris.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "CameraControl_Iris".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
                MediaFoundationControls::Focus => {
                    if let Err(why) =
                        unsafe { camera_control.Set(CameraControl_Focus.0, value, flags) }
                    {
                        return Err(BindingError::GUIDSetError(
                            "CameraControl_Focus".to_string(),
                            format!("{} {}", value, flag_str),
                            why.to_string(),
                        ));
                    }
                }
            }

            Ok(())
        }

        pub fn format(&self) -> MFCameraFormat {
            self.device_format
        }

        pub fn set_format(&mut self, format: MFCameraFormat) -> Result<(), BindingError> {
            // convert to media_type
            let media_type = match unsafe { MFCreateMediaType() } {
                Ok(mt) => mt,
                Err(why) => return Err(BindingError::AttributeError(why.to_string())),
            };

            // set relevant things

            let resolution = ((format.resolution.width_x as u64) << 32_u64)
                + (format.resolution.height_y as u64);
            let fps = ((format.frame_rate as u64) << 32) + 1_u64;
            let fourcc = match format.format {
                MFFrameFormat::MJPEG => MF_VIDEO_FORMAT_MJPEG,
                MFFrameFormat::YUYV => MF_VIDEO_FORMAT_YUY2,
            };

            // setting to the new media_type
            if let Err(why) = unsafe { media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video) } {
                return Err(BindingError::GUIDSetError(
                    "MF_MT_MAJOR_TYPE".to_string(),
                    "MFMediaType_Video".to_string(),
                    why.to_string(),
                ));
            }
            if let Err(why) = unsafe { media_type.SetGUID(&MF_MT_SUBTYPE, &fourcc) } {
                return Err(BindingError::GUIDSetError(
                    "MF_MT_SUBTYPE".to_string(),
                    format!("{:?}", fourcc),
                    why.to_string(),
                ));
            }
            if let Err(why) = unsafe { media_type.SetUINT64(&MF_MT_FRAME_SIZE, resolution) } {
                return Err(BindingError::GUIDSetError(
                    "MF_MT_FRAME_SIZE".to_string(),
                    resolution.to_string(),
                    why.to_string(),
                ));
            }
            if let Err(why) = unsafe { media_type.SetUINT64(&MF_MT_FRAME_RATE, fps) } {
                return Err(BindingError::GUIDSetError(
                    "MF_MT_FRAME_RATE".to_string(),
                    fps.to_string(),
                    why.to_string(),
                ));
            }
            if let Err(why) = unsafe { media_type.SetUINT64(&MF_MT_FRAME_RATE_RANGE_MIN, fps) } {
                return Err(BindingError::GUIDSetError(
                    "MF_MT_FRAME_RATE_RANGE_MIN".to_string(),
                    fps.to_string(),
                    why.to_string(),
                ));
            }
            if let Err(why) = unsafe { media_type.SetUINT64(&MF_MT_FRAME_RATE_RANGE_MAX, fps) } {
                return Err(BindingError::GUIDSetError(
                    "MF_MT_FRAME_RATE_RANGE_MAX".to_string(),
                    fps.to_string(),
                    why.to_string(),
                ));
            }

            let mut reserved = 0_u32;

            if let Err(why) = unsafe {
                self.source_reader.SetCurrentMediaType(
                    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                    &mut reserved,
                    media_type.clone(),
                )
            } {
                return Err(BindingError::GUIDSetError(
                    "MF_SOURCE_READER_FIRST_VIDEO_STREAM".to_string(),
                    format!("{:?}", media_type),
                    why.to_string(),
                ));
            }

            self.device_format = format;
            Ok(())
        }

        pub fn is_stream_open(&self) -> bool {
            self.is_open.get()
        }

        pub fn start_stream(&mut self) -> Result<(), BindingError> {
            self.is_open.set(true);

            Ok(())
        }

        pub fn raw_bytes(&mut self) -> Result<Cow<[u8]>, BindingError> {
            let mut imf_sample: Option<IMFSample> = None;
            let mut stream_index = 0_u32;
            let mut stream_flags = 0_u32;
            let mut time_stamp = 0_i64;

            if let Err(why) = unsafe {
                self.source_reader.ReadSample(
                    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                    0,
                    &mut stream_index,
                    &mut stream_flags,
                    &mut time_stamp,
                    &mut imf_sample,
                )
            } {
                return Err(BindingError::ReadFrameError(why.to_string()));
            }

            let imf_sample = match imf_sample {
                Some(sample) => sample,
                None => return Err(BindingError::ReadFrameError("Sample Not Init".to_string())),
            };

            let imf_buffer = match unsafe { imf_sample.ConvertToContiguousBuffer() } {
                Ok(buf) => buf,
                Err(why) => return Err(BindingError::ReadFrameError(why.to_string())),
            };

            let mut buffer_length = match unsafe { imf_buffer.GetCurrentLength() } {
                Ok(len) => len,
                Err(why) => return Err(BindingError::ReadFrameError(why.to_string())),
            };

            let mut raw_data_vec: Vec<u8> = vec![];
            raw_data_vec.resize(buffer_length as usize, 0_u8);

            let mut max_buffer_len = usize::MAX as u32;

            if let Err(why) = unsafe {
                imf_buffer.Lock(
                    &mut raw_data_vec.as_mut_ptr(),
                    &mut max_buffer_len,
                    &mut buffer_length,
                )
            } {
                return Err(BindingError::ReadFrameError(why.to_string()));
            }

            if let Err(why) = unsafe { imf_buffer.Unlock() } {
                return Err(BindingError::ReadFrameError(why.to_string()));
            }

            Ok(Cow::from(raw_data_vec))
        }

        pub fn stop_stream(&mut self) {
            self.is_open.set(false);
        }
    }

    impl Drop for MediaFoundationDevice {
        fn drop(&mut self) {
            // swallow errors
            unsafe {
                if let Err(_) = self
                    .source_reader
                    .Flush(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
                {}
                if let Err(_) = self.media_source.Shutdown() {}
            }
        }
    }
}

#[cfg(any(not(windows), feature = "docs-only"))]
#[allow(clippy::missing_errors_doc)]
#[allow(clippy::unused_self)]
pub mod wmf {
    use crate::{
        BindingError, MFCameraFormat, MFControl, MediaFoundationControls,
        MediaFoundationDeviceDescriptor,
    };
    use std::{borrow::Cow, cell::Cell};

    pub fn initialize_mf() -> Result<(), BindingError> {
        Err(BindingError::NotImplementedError)
    }

    pub fn deinitialize_mf() -> Result<(), BindingError> {
        Err(BindingError::NotImplementedError)
    }

    pub fn query_msmf() -> Result<Vec<MediaFoundationDeviceDescriptor<'static>>, BindingError> {
        Err(BindingError::NotImplementedError)
    }

    pub struct MediaFoundationDevice {
        op: Cell<bool>,
    }

    impl MediaFoundationDevice {
        pub fn new(_index: usize) -> Result<Self, BindingError> {
            Ok(MediaFoundationDevice {
                op: Cell::new(false),
            })
        }

        pub fn index(&self) -> usize {
            usize::MAX
        }

        pub fn name(&self) -> String {
            "".to_string()
        }

        pub fn symlink(&self) -> String {
            "".to_string()
        }

        pub fn compatible_format_list(&mut self) -> Result<Vec<MFCameraFormat>, BindingError> {
            Err(BindingError::NotImplementedError)
        }

        pub fn control(
            &self,
            _control: MediaFoundationControls,
        ) -> Result<MFControl, BindingError> {
            Err(BindingError::NotImplementedError)
        }

        pub fn set_control(&mut self, _control: MFControl) -> Result<(), BindingError> {
            Err(BindingError::NotImplementedError)
        }

        pub fn format(&self) -> MFCameraFormat {
            MFCameraFormat::default()
        }

        pub fn set_format(&mut self, _format: MFCameraFormat) -> Result<(), BindingError> {
            Err(BindingError::NotImplementedError)
        }

        pub fn is_stream_open(&self) -> bool {
            false
        }

        pub fn start_stream(&mut self) -> Result<(), BindingError> {
            Err(BindingError::NotImplementedError)
        }

        pub fn raw_bytes(&mut self) -> Result<Cow<[u8]>, BindingError> {
            Err(BindingError::NotImplementedError)
        }

        pub fn stop_stream(&mut self) {
            self.op.set(true);
        }
    }

    impl Drop for MediaFoundationDevice {
        fn drop(&mut self) {}
    }
}
