//! AVCaptureDevice and related types.

use block::ConcreteBlock;
use cocoa_foundation::base::Nil;
use cocoa_foundation::foundation::{NSArray, NSInteger, NSUInteger};
use core_media_sys::{
    CMFormatDescriptionGetMediaSubType, CMFormatDescriptionRef, CMTime, CMVideoDimensions,
};
use flume::{Receiver, Sender};
use nokhwa_core::{
    error::NokhwaError,
    types::{
        ApiBackend, CameraControl, CameraFormat, CameraIndex, CameraInfo,
        ControlValueDescription, ControlValueSetter, FrameFormat, KnownCameraControl,
        KnownCameraControlFlag, Resolution,
    },
};
use objc::runtime::{Object, BOOL, NO, YES};
use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::BTreeMap,
    ffi::{c_float, c_void},
};

use crate::ffi::{
    AVCaptureExposureDurationCurrent, AVCaptureExposureTargetBiasCurrent, AVCaptureISOCurrent,
    AVCaptureWhiteBalanceGains, AVMediaTypeVideo, CGPoint, CMVideoFormatDescriptionGetDimensions,
    OSType,
};
use crate::format::{
    compare_ns_string, ns_arr_to_vec, nsstr_to_str, raw_fcc_to_frameformat, str_to_nsstr,
    try_ns_arr_to_vec, vec_to_ns_arr,
};
use crate::CGFloat;

pub type CompressionData<'a> = (Cow<'a, [u8]>, FrameFormat);
pub type DataPipe<'a> = (Sender<CompressionData<'a>>, Receiver<CompressionData<'a>>);

pub fn request_permission_with_callback(callback: impl Fn(bool) + Send + Sync + 'static) {
    let cls = class!(AVCaptureDevice);

    let wrapper = move |bool: BOOL| {
        callback(bool == YES);
    };

    let objc_fn_block: ConcreteBlock<(BOOL,), (), _> = ConcreteBlock::new(wrapper);
    let objc_fn_pass = objc_fn_block.copy();

    unsafe {
        let _: () = msg_send![cls, requestAccessForMediaType:(AVMediaTypeVideo.clone()) completionHandler:objc_fn_pass];
    }
}

pub fn current_authorization_status() -> AVAuthorizationStatus {
    let cls = class!(AVCaptureDevice);
    let status: AVAuthorizationStatus = unsafe {
        msg_send![cls, authorizationStatusForMediaType:AVMediaType::Video.into_ns_str()]
    };
    status
}

// fuck it, use deprecated APIs
pub fn query_avfoundation() -> Result<Vec<CameraInfo>, NokhwaError> {
    Ok(AVCaptureDeviceDiscoverySession::new(vec![
        AVCaptureDeviceType::UltraWide,
        AVCaptureDeviceType::WideAngle,
        AVCaptureDeviceType::Telephoto,
        AVCaptureDeviceType::TrueDepth,
        AVCaptureDeviceType::External,
    ])?
    .devices())
}

pub fn get_raw_device_info(index: CameraIndex, device: *mut Object) -> CameraInfo {
    let name = nsstr_to_str(unsafe { msg_send![device, localizedName] });
    let manufacturer = nsstr_to_str(unsafe { msg_send![device, manufacturer] });
    let position: AVCaptureDevicePosition = unsafe { msg_send![device, position] };
    let lens_aperture: f64 = unsafe { msg_send![device, lensAperture] };
    let device_type = nsstr_to_str(unsafe { msg_send![device, deviceType] });
    let model_id = nsstr_to_str(unsafe { msg_send![device, modelID] });
    let description = format!(
        "{}: {} - {}, {:?} f{}",
        manufacturer, model_id, device_type, position, lens_aperture
    );
    let misc = nsstr_to_str(unsafe { msg_send![device, uniqueID] });

    CameraInfo::new(name.as_ref(), &description, misc.as_ref(), index)
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum AVCaptureDeviceType {
    Dual,
    DualWide,
    Triple,
    WideAngle,
    UltraWide,
    Telephoto,
    TrueDepth,
    External,
}

impl From<AVCaptureDeviceType> for *mut Object {
    fn from(device_type: AVCaptureDeviceType) -> Self {
        match device_type {
            AVCaptureDeviceType::Dual => str_to_nsstr("AVCaptureDeviceTypeBuiltInDualCamera"),
            AVCaptureDeviceType::DualWide => {
                str_to_nsstr("AVCaptureDeviceTypeBuiltInDualWideCamera")
            }
            AVCaptureDeviceType::Triple => {
                str_to_nsstr("AVCaptureDeviceTypeBuiltInTripleCamera")
            }
            AVCaptureDeviceType::WideAngle => {
                str_to_nsstr("AVCaptureDeviceTypeBuiltInWideAngleCamera")
            }
            AVCaptureDeviceType::UltraWide => {
                str_to_nsstr("AVCaptureDeviceTypeBuiltInUltraWideCamera")
            }
            AVCaptureDeviceType::Telephoto => {
                str_to_nsstr("AVCaptureDeviceTypeBuiltInTelephotoCamera")
            }
            AVCaptureDeviceType::TrueDepth => {
                str_to_nsstr("AVCaptureDeviceTypeBuiltInTrueDepthCamera")
            }
            AVCaptureDeviceType::External => str_to_nsstr("AVCaptureDeviceTypeExternal"),
        }
    }
}

impl AVCaptureDeviceType {
    pub fn into_ns_str(self) -> *mut Object {
        <*mut Object>::from(self)
    }
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum AVMediaType {
    Audio,
    ClosedCaption,
    DepthData,
    Metadata,
    MetadataObject,
    Muxed,
    Subtitle,
    Text,
    Timecode,
    Video,
}

impl From<AVMediaType> for *mut Object {
    fn from(media_type: AVMediaType) -> Self {
        match media_type {
            AVMediaType::Audio => unsafe { crate::ffi::AVMediaTypeAudio.0 },
            AVMediaType::ClosedCaption => unsafe { crate::ffi::AVMediaTypeClosedCaption.0 },
            AVMediaType::DepthData => unsafe { crate::ffi::AVMediaTypeDepthData.0 },
            AVMediaType::Metadata => unsafe { crate::ffi::AVMediaTypeMetadata.0 },
            AVMediaType::MetadataObject => unsafe { crate::ffi::AVMediaTypeMetadataObject.0 },
            AVMediaType::Muxed => unsafe { crate::ffi::AVMediaTypeMuxed.0 },
            AVMediaType::Subtitle => unsafe { crate::ffi::AVMediaTypeSubtitle.0 },
            AVMediaType::Text => unsafe { crate::ffi::AVMediaTypeText.0 },
            AVMediaType::Timecode => unsafe { crate::ffi::AVMediaTypeTimecode.0 },
            AVMediaType::Video => unsafe { crate::ffi::AVMediaTypeVideo.0 },
        }
    }
}

impl TryFrom<*mut Object> for AVMediaType {
    type Error = NokhwaError;

    fn try_from(value: *mut Object) -> Result<Self, Self::Error> {
        unsafe {
            if compare_ns_string(value, (crate::ffi::AVMediaTypeAudio).clone()) {
                Ok(AVMediaType::Audio)
            } else if compare_ns_string(value, (crate::ffi::AVMediaTypeClosedCaption).clone()) {
                Ok(AVMediaType::ClosedCaption)
            } else if compare_ns_string(value, (crate::ffi::AVMediaTypeDepthData).clone()) {
                Ok(AVMediaType::DepthData)
            } else if compare_ns_string(value, (crate::ffi::AVMediaTypeMetadata).clone()) {
                Ok(AVMediaType::Metadata)
            } else if compare_ns_string(value, (crate::ffi::AVMediaTypeMetadataObject).clone()) {
                Ok(AVMediaType::MetadataObject)
            } else if compare_ns_string(value, (crate::ffi::AVMediaTypeMuxed).clone()) {
                Ok(AVMediaType::Muxed)
            } else if compare_ns_string(value, (crate::ffi::AVMediaTypeSubtitle).clone()) {
                Ok(AVMediaType::Subtitle)
            } else if compare_ns_string(value, (crate::ffi::AVMediaTypeText).clone()) {
                Ok(AVMediaType::Text)
            } else if compare_ns_string(value, (crate::ffi::AVMediaTypeTimecode).clone()) {
                Ok(AVMediaType::Timecode)
            } else if compare_ns_string(value, (crate::ffi::AVMediaTypeVideo).clone()) {
                Ok(AVMediaType::Video)
            } else {
                let name = nsstr_to_str(value);
                Err(NokhwaError::GetPropertyError {
                    property: "AVMediaType".to_string(),
                    error: format!("Invalid AVMediaType {name}"),
                })
            }
        }
    }
}

impl AVMediaType {
    pub fn into_ns_str(self) -> *mut Object {
        <*mut Object>::from(self)
    }
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[repr(isize)]
pub enum AVCaptureDevicePosition {
    Unspecified = 0,
    Back = 1,
    Front = 2,
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[repr(isize)]
pub enum AVAuthorizationStatus {
    NotDetermined = 0,
    Restricted = 1,
    Denied = 2,
    Authorized = 3,
}

create_boilerplate_impl! {
    [pub AVFrameRateRange],
    [pub AVCaptureDeviceDiscoverySession]
}

impl AVFrameRateRange {
    pub fn max(&self) -> f64 {
        unsafe { msg_send![self.inner, maxFrameRate] }
    }

    pub fn min(&self) -> f64 {
        unsafe { msg_send![self.inner, minFrameRate] }
    }
}

#[derive(Debug)]
pub struct AVCaptureDeviceFormat {
    pub(crate) internal: *mut Object,
    pub resolution: CMVideoDimensions,
    pub fps_list: Vec<f64>,
    pub fourcc: FrameFormat,
}

impl TryFrom<*mut Object> for AVCaptureDeviceFormat {
    type Error = NokhwaError;

    fn try_from(value: *mut Object) -> Result<Self, Self::Error> {
        let media_type_raw: *mut Object = unsafe { msg_send![value, mediaType] };
        let media_type = AVMediaType::try_from(media_type_raw)?;
        if media_type != AVMediaType::Video {
            return Err(NokhwaError::StructureError {
                structure: "AVMediaType".to_string(),
                error: "Not Video".to_string(),
            });
        }
        let mut fps_list = ns_arr_to_vec::<AVFrameRateRange>(unsafe {
            msg_send![value, videoSupportedFrameRateRanges]
        })
        .into_iter()
        .flat_map(|v| {
            if v.min() != 0_f64 && v.min() != 1_f64 {
                vec![v.min(), v.max()]
            } else {
                vec![v.max()] // this gets deduped!
            }
        })
        .collect::<Vec<f64>>();
        fps_list.sort_by(|n, m| n.partial_cmp(m).unwrap_or(Ordering::Equal));
        fps_list.dedup();
        let description_obj: *mut Object = unsafe { msg_send![value, formatDescription] };
        let resolution =
            unsafe { CMVideoFormatDescriptionGetDimensions(description_obj as *mut c_void) };
        let fcc_raw =
            unsafe { CMFormatDescriptionGetMediaSubType(description_obj as *mut c_void) };
        #[allow(non_upper_case_globals)]
        let fourcc = match raw_fcc_to_frameformat(fcc_raw) {
            Some(fcc) => fcc,
            None => {
                return Err(NokhwaError::StructureError {
                    structure: "FourCharCode".to_string(),
                    error: format!("Unknown FourCharCode {fcc_raw:?}"),
                })
            }
        };

        Ok(AVCaptureDeviceFormat {
            internal: value,
            resolution,
            fps_list,
            fourcc,
        })
    }
}

impl AVCaptureDeviceDiscoverySession {
    pub fn new(device_types: Vec<AVCaptureDeviceType>) -> Result<Self, NokhwaError> {
        let device_types = vec_to_ns_arr(device_types);
        let position = 0 as NSInteger;

        let media_type_video = unsafe { AVMediaTypeVideo.clone() }.0;

        let discovery_session_cls = class!(AVCaptureDeviceDiscoverySession);
        let discovery_session: *mut Object = unsafe {
            msg_send![discovery_session_cls, discoverySessionWithDeviceTypes:device_types mediaType:media_type_video position:position]
        };

        Ok(AVCaptureDeviceDiscoverySession {
            inner: discovery_session,
        })
    }

    pub fn default() -> Result<Self, NokhwaError> {
        AVCaptureDeviceDiscoverySession::new(vec![
            AVCaptureDeviceType::UltraWide,
            AVCaptureDeviceType::Telephoto,
            AVCaptureDeviceType::External,
            AVCaptureDeviceType::Dual,
            AVCaptureDeviceType::DualWide,
            AVCaptureDeviceType::Triple,
        ])
    }

    pub fn devices(&self) -> Vec<CameraInfo> {
        let device_ns_array: *mut Object = unsafe { msg_send![self.inner, devices] };
        let objects_len: NSUInteger = unsafe { NSArray::count(device_ns_array) };
        let mut devices = Vec::with_capacity(objects_len as usize);
        for index in 0..objects_len {
            let device = unsafe { device_ns_array.objectAtIndex(index) };
            devices.push(get_raw_device_info(
                CameraIndex::Index(index as u32),
                device,
            ));
        }

        devices
    }
}

pub struct AVCaptureDevice {
    inner: *mut Object,
    device: CameraInfo,
    locked: bool,
}

impl AVCaptureDevice {
    pub fn inner(&self) -> *mut Object {
        self.inner
    }
}

impl AVCaptureDevice {
    pub fn new(index: &CameraIndex) -> Result<Self, NokhwaError> {
        match &index {
            CameraIndex::Index(idx) => {
                let devices = query_avfoundation()?;

                match devices.get(*idx as usize) {
                    Some(device) => Ok(AVCaptureDevice::from_id(
                        &device.misc(),
                        Some(index.clone()),
                    )?),
                    None => Err(NokhwaError::OpenDeviceError(
                        idx.to_string(),
                        "Not Found".to_string(),
                    )),
                }
            }
            CameraIndex::String(id) => Ok(AVCaptureDevice::from_id(id, None)?),
        }
    }

    pub fn from_id(id: &str, index_hint: Option<CameraIndex>) -> Result<Self, NokhwaError> {
        let nsstr_id = str_to_nsstr(id);
        let avfoundation_capture_cls = class!(AVCaptureDevice);
        let capture: *mut Object =
            unsafe { msg_send![avfoundation_capture_cls, deviceWithUniqueID: nsstr_id] };
        if capture.is_null() {
            return Err(NokhwaError::OpenDeviceError(
                id.to_string(),
                "Device is null".to_string(),
            ));
        }
        let camera_info = get_raw_device_info(
            index_hint.unwrap_or_else(|| CameraIndex::String(id.to_string())),
            capture,
        );

        Ok(AVCaptureDevice {
            inner: capture,
            device: camera_info,
            locked: false,
        })
    }

    pub fn info(&self) -> &CameraInfo {
        &self.device
    }

    pub fn supported_formats_raw(&self) -> Result<Vec<AVCaptureDeviceFormat>, NokhwaError> {
        try_ns_arr_to_vec::<AVCaptureDeviceFormat, NokhwaError>(unsafe {
            msg_send![self.inner, formats]
        })
    }

    pub fn supported_formats(&self) -> Result<Vec<CameraFormat>, NokhwaError> {
        Ok(self
            .supported_formats_raw()?
            .iter()
            .flat_map(|av_fmt| {
                let resolution = av_fmt.resolution;
                av_fmt.fps_list.iter().map(move |fps_f64| {
                    let fps = *fps_f64 as u32;

                    let resolution =
                        Resolution::new(resolution.width as u32, resolution.height as u32); // FIXME: what the fuck?
                    CameraFormat::new(resolution, av_fmt.fourcc, fps)
                })
            })
            .filter(|x| x.frame_rate() != 0)
            .collect())
    }

    pub fn already_in_use(&self) -> bool {
        unsafe {
            let result: BOOL = msg_send![self.inner(), isInUseByAnotherApplication];
            result == YES
        }
    }

    pub fn is_suspended(&self) -> bool {
        unsafe {
            let result: BOOL = msg_send![self.inner, isSuspended];
            result == YES
        }
    }

    pub fn lock(&self) -> Result<(), NokhwaError> {
        if self.locked {
            return Ok(());
        }
        if self.already_in_use() {
            return Err(NokhwaError::InitializeError {
                backend: ApiBackend::AVFoundation,
                error: "Already in use".to_string(),
            });
        }
        let err_ptr: *mut c_void = std::ptr::null_mut();
        let accepted: BOOL = unsafe { msg_send![self.inner, lockForConfiguration: err_ptr] };
        if !err_ptr.is_null() {
            return Err(NokhwaError::SetPropertyError {
                property: "lockForConfiguration".to_string(),
                value: "Locked".to_string(),
                error: "Cannot lock for configuration".to_string(),
            });
        }
        // Space these out for debug purposes
        if !accepted == YES {
            return Err(NokhwaError::SetPropertyError {
                property: "lockForConfiguration".to_string(),
                value: "Locked".to_string(),
                error: "Lock Rejected".to_string(),
            });
        }
        Ok(())
    }

    pub fn unlock(&mut self) {
        if self.locked {
            self.locked = false;
            unsafe { msg_send![self.inner, unlockForConfiguration] }
        }
    }

    // thank you ffmpeg
    pub fn set_all(&mut self, descriptor: CameraFormat) -> Result<(), NokhwaError> {
        self.lock()?;
        let format_list = try_ns_arr_to_vec::<AVCaptureDeviceFormat, NokhwaError>(unsafe {
            msg_send![self.inner, formats]
        })?;
        let format_description_sel = sel!(formatDescription);

        let mut selected_format: *mut Object = std::ptr::null_mut();
        let mut selected_range: *mut Object = std::ptr::null_mut();

        for format in format_list {
            let format_desc_ref: CMFormatDescriptionRef =
                unsafe { msg_send![format.internal, performSelector: format_description_sel] };
            let dimensions = unsafe { CMVideoFormatDescriptionGetDimensions(format_desc_ref) };

            if dimensions.height == descriptor.resolution().height() as i32
                && dimensions.width == descriptor.resolution().width() as i32
            {
                selected_format = format.internal;

                for range in ns_arr_to_vec::<AVFrameRateRange>(unsafe {
                    msg_send![format.internal, videoSupportedFrameRateRanges]
                }) {
                    let max_fps: f64 = unsafe { msg_send![range.inner, maxFrameRate] };
                    // Older Apple cameras (i.e. iMac 2013) return 29.97000002997 as FPS.
                    if (f64::from(descriptor.frame_rate()) - max_fps).abs() < 0.999 {
                        selected_range = range.inner;
                        break;
                    }
                }
            }
        }
        if selected_range.is_null() || selected_format.is_null() {
            return Err(NokhwaError::SetPropertyError {
                property: "CameraFormat".to_string(),
                value: descriptor.to_string(),
                error: "Not Found/Rejected/Unsupported".to_string(),
            });
        }

        let activefmtkey = str_to_nsstr("activeFormat");
        let min_frame_duration = str_to_nsstr("minFrameDuration");
        let active_video_min_frame_duration = str_to_nsstr("activeVideoMinFrameDuration");
        let active_video_max_frame_duration = str_to_nsstr("activeVideoMaxFrameDuration");
        let _: () =
            unsafe { msg_send![self.inner, setValue:selected_format forKey:activefmtkey] };
        let min_frame_duration: *mut Object =
            unsafe { msg_send![selected_range, valueForKey: min_frame_duration] };
        let _: () = unsafe {
            msg_send![self.inner, setValue:min_frame_duration forKey:active_video_min_frame_duration]
        };
        let _: () = unsafe {
            msg_send![self.inner, setValue:min_frame_duration forKey:active_video_max_frame_duration]
        };
        self.unlock();
        Ok(())
    }

    // 0 => Focus POI
    // 1 => Focus Manual Setting
    // 2 => Exposure POI
    // 3 => Exposure Face Driven
    // 4 => Exposure Target Bias
    // 5 => Exposure ISO
    // 6 => Exposure Duration
    pub fn get_controls(&self) -> Result<Vec<CameraControl>, NokhwaError> {
        let active_format: *mut Object = unsafe { msg_send![self.inner, activeFormat] };

        let mut controls = vec![];
        // get focus modes

        let focus_current: NSInteger = unsafe { msg_send![self.inner, focusMode] };
        let focus_locked: BOOL =
            unsafe { msg_send![self.inner, isFocusModeSupported:NSInteger::from(0)] };
        let focus_auto: BOOL =
            unsafe { msg_send![self.inner, isFocusModeSupported:NSInteger::from(1)] };
        let focus_continuous: BOOL =
            unsafe { msg_send![self.inner, isFocusModeSupported:NSInteger::from(2)] };

        {
            let mut supported_focus_values = vec![];

            if focus_locked == YES {
                supported_focus_values.push(0)
            }
            if focus_auto == YES {
                supported_focus_values.push(1)
            }
            if focus_continuous == YES {
                supported_focus_values.push(2)
            }

            controls.push(CameraControl::new(
                KnownCameraControl::Focus,
                "FocusMode".to_string(),
                ControlValueDescription::Enum {
                    value: focus_current,
                    possible: supported_focus_values,
                    default: focus_current,
                },
                vec![],
                true,
            ));
        }

        let focus_poi_supported: BOOL =
            unsafe { msg_send![self.inner, isFocusPointOfInterestSupported] };
        let focus_poi: CGPoint = unsafe { msg_send![self.inner, focusPointOfInterest] };

        controls.push(CameraControl::new(
            KnownCameraControl::Other(0),
            "FocusPointOfInterest".to_string(),
            ControlValueDescription::Point {
                value: (focus_poi.x as f64, focus_poi.y as f64),
                default: (0.5, 0.5),
            },
            if focus_poi_supported == NO {
                vec![
                    KnownCameraControlFlag::Disabled,
                    KnownCameraControlFlag::ReadOnly,
                ]
            } else {
                vec![]
            },
            focus_auto == YES || focus_continuous == YES,
        ));

        let focus_manual: BOOL =
            unsafe { msg_send![self.inner, isLockingFocusWithCustomLensPositionSupported] };
        let focus_lenspos: f32 = unsafe { msg_send![self.inner, lensPosition] };

        controls.push(CameraControl::new(
            KnownCameraControl::Other(1),
            "FocusManualLensPosition".to_string(),
            ControlValueDescription::FloatRange {
                min: 0.0,
                max: 1.0,
                value: focus_lenspos as f64,
                step: f64::MIN_POSITIVE,
                default: 1.0,
            },
            if focus_manual == YES {
                vec![]
            } else {
                vec![
                    KnownCameraControlFlag::Disabled,
                    KnownCameraControlFlag::ReadOnly,
                ]
            },
            focus_manual == YES,
        ));

        // get exposures
        let exposure_current: NSInteger = unsafe { msg_send![self.inner, exposureMode] };
        let exposure_locked: BOOL =
            unsafe { msg_send![self.inner, isExposureModeSupported:NSInteger::from(0)] };
        let exposure_auto: BOOL =
            unsafe { msg_send![self.inner, isExposureModeSupported:NSInteger::from(1)] };
        let exposure_continuous: BOOL =
            unsafe { msg_send![self.inner, isExposureModeSupported:NSInteger::from(2)] };
        let exposure_custom: BOOL =
            unsafe { msg_send![self.inner, isExposureModeSupported:NSInteger::from(3)] };

        {
            let mut supported_exposure_values = vec![];

            if exposure_locked == YES {
                supported_exposure_values.push(0);
            }
            if exposure_auto == YES {
                supported_exposure_values.push(1);
            }
            if exposure_continuous == YES {
                supported_exposure_values.push(2);
            }
            if exposure_custom == YES {
                supported_exposure_values.push(3);
            }

            controls.push(CameraControl::new(
                KnownCameraControl::Exposure,
                "ExposureMode".to_string(),
                ControlValueDescription::Enum {
                    value: exposure_current,
                    possible: supported_exposure_values,
                    default: exposure_current,
                },
                vec![],
                true,
            ));
        }

        let exposure_poi_supported: BOOL =
            unsafe { msg_send![self.inner, isExposurePointOfInterestSupported] };
        let exposure_poi: CGPoint = unsafe { msg_send![self.inner, exposurePointOfInterest] };

        controls.push(CameraControl::new(
            KnownCameraControl::Other(2),
            "ExposurePointOfInterest".to_string(),
            ControlValueDescription::Point {
                value: (exposure_poi.x as f64, exposure_poi.y as f64),
                default: (0.5, 0.5),
            },
            if exposure_poi_supported == NO {
                vec![
                    KnownCameraControlFlag::Disabled,
                    KnownCameraControlFlag::ReadOnly,
                ]
            } else {
                vec![]
            },
            focus_auto == YES || focus_continuous == YES,
        ));

        let expposure_face_driven_supported: BOOL =
            unsafe { msg_send![self.inner, isFaceDrivenAutoExposureEnabled] };
        let exposure_face_driven: BOOL = unsafe {
            msg_send![
                self.inner,
                automaticallyAdjustsFaceDrivenAutoExposureEnabled
            ]
        };

        controls.push(CameraControl::new(
            KnownCameraControl::Other(3),
            "ExposureFaceDriven".to_string(),
            ControlValueDescription::Boolean {
                value: exposure_face_driven == YES,
                default: false,
            },
            if expposure_face_driven_supported == NO {
                vec![
                    KnownCameraControlFlag::Disabled,
                    KnownCameraControlFlag::ReadOnly,
                ]
            } else {
                vec![]
            },
            exposure_poi_supported == YES,
        ));

        let exposure_bias: f32 = unsafe { msg_send![self.inner, exposureTargetBias] };
        let exposure_bias_min: f32 = unsafe { msg_send![self.inner, minExposureTargetBias] };
        let exposure_bias_max: f32 = unsafe { msg_send![self.inner, maxExposureTargetBias] };

        controls.push(CameraControl::new(
            KnownCameraControl::Other(4),
            "ExposureBiasTarget".to_string(),
            ControlValueDescription::FloatRange {
                min: exposure_bias_min as f64,
                max: exposure_bias_max as f64,
                value: exposure_bias as f64,
                step: f32::MIN_POSITIVE as f64,
                default: unsafe { AVCaptureExposureTargetBiasCurrent } as f64,
            },
            vec![],
            true,
        ));

        let exposure_duration: CMTime = unsafe { msg_send![self.inner, exposureDuration] };
        let exposure_duration_min: CMTime =
            unsafe { msg_send![active_format, minExposureDuration] };
        let exposure_duration_max: CMTime =
            unsafe { msg_send![active_format, maxExposureDuration] };

        controls.push(CameraControl::new(
            KnownCameraControl::Gamma,
            "ExposureDuration".to_string(),
            ControlValueDescription::IntegerRange {
                min: exposure_duration_min.value,
                max: exposure_duration_max.value,
                value: exposure_duration.value,
                step: 1,
                default: unsafe { AVCaptureExposureDurationCurrent.value },
            },
            if exposure_custom == YES {
                vec![
                    KnownCameraControlFlag::ReadOnly,
                    KnownCameraControlFlag::Volatile,
                ]
            } else {
                vec![KnownCameraControlFlag::Volatile]
            },
            exposure_custom == YES,
        ));

        let exposure_iso: f32 = unsafe { msg_send![self.inner, ISO] };
        let exposure_iso_min: f32 = unsafe { msg_send![active_format, minISO] };
        let exposure_iso_max: f32 = unsafe { msg_send![active_format, maxISO] };

        controls.push(CameraControl::new(
            KnownCameraControl::Brightness,
            "ExposureISO".to_string(),
            ControlValueDescription::FloatRange {
                min: exposure_iso_min as f64,
                max: exposure_iso_max as f64,
                value: exposure_iso as f64,
                step: f32::MIN_POSITIVE as f64,
                default: unsafe { AVCaptureISOCurrent } as f64,
            },
            if exposure_custom == YES {
                vec![
                    KnownCameraControlFlag::ReadOnly,
                    KnownCameraControlFlag::Volatile,
                ]
            } else {
                vec![KnownCameraControlFlag::Volatile]
            },
            exposure_custom == YES,
        ));

        let lens_aperture: f32 = unsafe { msg_send![self.inner, lensAperture] };

        controls.push(CameraControl::new(
            KnownCameraControl::Iris,
            "LensAperture".to_string(),
            ControlValueDescription::Float {
                value: lens_aperture as f64,
                default: lens_aperture as f64,
                step: lens_aperture as f64,
            },
            vec![KnownCameraControlFlag::ReadOnly],
            false,
        ));

        // get whiteblaance
        let white_balance_current: NSInteger =
            unsafe { msg_send![self.inner, whiteBalanceMode] };
        let white_balance_manual: BOOL =
            unsafe { msg_send![self.inner, isWhiteBalanceModeSupported:NSInteger::from(0)] };
        let white_balance_auto: BOOL =
            unsafe { msg_send![self.inner, isWhiteBalanceModeSupported:NSInteger::from(1)] };
        let white_balance_continuous: BOOL =
            unsafe { msg_send![self.inner, isWhiteBalanceModeSupported:NSInteger::from(2)] };

        {
            let mut possible = vec![];

            if white_balance_manual == YES {
                possible.push(0);
            }
            if white_balance_auto == YES {
                possible.push(1);
            }
            if white_balance_continuous == YES {
                possible.push(2);
            }

            controls.push(CameraControl::new(
                KnownCameraControl::WhiteBalance,
                "WhiteBalanceMode".to_string(),
                ControlValueDescription::Enum {
                    value: white_balance_current as i64,
                    possible,
                    default: 0,
                },
                vec![],
                true,
            ));
        }

        let white_balance_gains: AVCaptureWhiteBalanceGains =
            unsafe { msg_send![self.inner, deviceWhiteBalanceGains] };
        let white_balance_default: AVCaptureWhiteBalanceGains =
            unsafe { msg_send![self.inner, grayWorldDeviceWhiteBalanceGains] };
        let white_balancne_max: AVCaptureWhiteBalanceGains =
            unsafe { msg_send![self.inner, maxWhiteBalanceGain] };
        let white_balance_gain_supported: BOOL = unsafe {
            msg_send![
                self.inner,
                isLockingWhiteBalanceWithCustomDeviceGainsSupported
            ]
        };

        controls.push(CameraControl::new(
            KnownCameraControl::Gain,
            "WhiteBalanceGain".to_string(),
            ControlValueDescription::RGB {
                value: (
                    white_balance_gains.redGain as f64,
                    white_balance_gains.greenGain as f64,
                    white_balance_gains.blueGain as f64,
                ),
                max: (
                    white_balancne_max.redGain as f64,
                    white_balancne_max.greenGain as f64,
                    white_balancne_max.blueGain as f64,
                ),
                default: (
                    white_balance_default.redGain as f64,
                    white_balance_default.greenGain as f64,
                    white_balance_default.blueGain as f64,
                ),
            },
            if white_balance_gain_supported == YES {
                vec![
                    KnownCameraControlFlag::Disabled,
                    KnownCameraControlFlag::ReadOnly,
                ]
            } else {
                vec![]
            },
            white_balance_gain_supported == YES,
        ));

        // get flash
        let has_torch: BOOL = unsafe { msg_send![self.inner, isTorchAvailable] };
        let torch_active: BOOL = unsafe { msg_send![self.inner, isTorchActive] };
        let torch_off: BOOL =
            unsafe { msg_send![self.inner, isTorchModeSupported:NSInteger::from(0)] };
        let torch_on: BOOL =
            unsafe { msg_send![self.inner, isTorchModeSupported:NSInteger::from(1)] };
        let torch_auto: BOOL =
            unsafe { msg_send![self.inner, isTorchModeSupported:NSInteger::from(2)] };

        {
            let mut possible = vec![];

            if torch_off == YES {
                possible.push(0);
            }
            if torch_on == YES {
                possible.push(1);
            }
            if torch_auto == YES {
                possible.push(2);
            }

            controls.push(CameraControl::new(
                KnownCameraControl::Other(5),
                "TorchMode".to_string(),
                ControlValueDescription::Enum {
                    value: (torch_active == YES) as i64,
                    possible,
                    default: 0,
                },
                if has_torch == YES {
                    vec![
                        KnownCameraControlFlag::Disabled,
                        KnownCameraControlFlag::ReadOnly,
                    ]
                } else {
                    vec![]
                },
                has_torch == YES,
            ));
        }

        // get low light boost
        let has_llb: BOOL = unsafe { msg_send![self.inner, isLowLightBoostSupported] };
        let llb_enabled: BOOL = unsafe { msg_send![self.inner, isLowLightBoostEnabled] };

        {
            controls.push(CameraControl::new(
                KnownCameraControl::BacklightComp,
                "LowLightCompensation".to_string(),
                ControlValueDescription::Boolean {
                    value: llb_enabled == YES,
                    default: false,
                },
                if has_llb == NO {
                    vec![
                        KnownCameraControlFlag::Disabled,
                        KnownCameraControlFlag::ReadOnly,
                    ]
                } else {
                    vec![]
                },
                has_llb == YES,
            ));
        }

        // get zoom factor
        let zoom_current: CGFloat = unsafe { msg_send![self.inner, videoZoomFactor] };
        let zoom_min: CGFloat = unsafe { msg_send![self.inner, minAvailableVideoZoomFactor] };
        let zoom_max: CGFloat = unsafe { msg_send![self.inner, maxAvailableVideoZoomFactor] };

        controls.push(CameraControl::new(
            KnownCameraControl::Zoom,
            "Zoom".to_string(),
            ControlValueDescription::FloatRange {
                min: zoom_min as f64,
                max: zoom_max as f64,
                value: zoom_current as f64,
                step: f32::MIN_POSITIVE as f64,
                default: 1.0,
            },
            vec![],
            true,
        ));

        // zoom distortion correction
        let distortion_correction_supported: BOOL =
            unsafe { msg_send![self.inner, isGeometricDistortionCorrectionSupported] };
        let distortion_correction_current_value: BOOL =
            unsafe { msg_send![self.inner, isGeometricDistortionCorrectionEnabled] };

        controls.push(CameraControl::new(
            KnownCameraControl::Other(6),
            "DistortionCorrection".to_string(),
            ControlValueDescription::Boolean {
                value: distortion_correction_current_value == YES,
                default: false,
            },
            if distortion_correction_supported == YES {
                vec![
                    KnownCameraControlFlag::ReadOnly,
                    KnownCameraControlFlag::Disabled,
                ]
            } else {
                vec![]
            },
            distortion_correction_supported == YES,
        ));

        Ok(controls)
    }

    pub fn set_control(
        &mut self,
        id: KnownCameraControl,
        value: ControlValueSetter,
    ) -> Result<(), NokhwaError> {
        let rc = self.get_controls()?;
        let controls = rc
            .iter()
            .map(|cc| (cc.control(), cc))
            .collect::<BTreeMap<_, _>>();

        match id {
            KnownCameraControl::Brightness => {
                let isoctrl = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Control does not exist".to_string(),
                })?;

                if isoctrl.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error:
                            "Exposure is in improper state to set ISO (Please set to `custom`!)"
                                .to_string(),
                    });
                }

                if isoctrl.flag().contains(&KnownCameraControlFlag::Disabled) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Disabled".to_string(),
                    });
                }

                let current_duration = unsafe { AVCaptureExposureDurationCurrent };
                let new_iso = *value.as_float().ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Expected float".to_string(),
                })? as f32;

                if !isoctrl.description().verify_setter(&value) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Failed to verify value".to_string(),
                    });
                }

                let _: () = unsafe {
                    msg_send![self.inner, setExposureModeCustomWithDuration:current_duration ISO:new_iso completionHandler:Nil]
                };

                Ok(())
            }
            KnownCameraControl::Gamma => {
                let duration_ctrl = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Control does not exist".to_string(),
                })?;

                if duration_ctrl
                    .flag()
                    .contains(&KnownCameraControlFlag::ReadOnly)
                {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Exposure is in improper state to set Duration (Please set to `custom`!)"
                            .to_string(),
                    });
                }

                if duration_ctrl
                    .flag()
                    .contains(&KnownCameraControlFlag::Disabled)
                {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Disabled".to_string(),
                    });
                }
                let current_duration: CMTime =
                    unsafe { msg_send![self.inner, exposureDuration] };

                let current_iso = unsafe { AVCaptureISOCurrent };
                let new_duration = CMTime {
                    value: *value.as_integer().ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Expected i64".to_string(),
                    })?,
                    timescale: current_duration.timescale,
                    flags: current_duration.flags,
                    epoch: current_duration.epoch,
                };

                if !duration_ctrl.description().verify_setter(&value) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Failed to verify value".to_string(),
                    });
                }

                let _: () = unsafe {
                    msg_send![self.inner, setExposureModeCustomWithDuration:new_duration ISO:current_iso completionHandler:Nil]
                };

                Ok(())
            }
            KnownCameraControl::WhiteBalance => {
                let wb_enum_value = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Control does not exist".to_string(),
                })?;

                if wb_enum_value
                    .flag()
                    .contains(&KnownCameraControlFlag::ReadOnly)
                {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Read Only".to_string(),
                    });
                }

                if wb_enum_value
                    .flag()
                    .contains(&KnownCameraControlFlag::Disabled)
                {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Disabled".to_string(),
                    });
                }
                let setter =
                    NSInteger::from(*value.as_enum().ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Expected Enum".to_string(),
                    })? as i32);

                if !wb_enum_value.description().verify_setter(&value) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Failed to verify value".to_string(),
                    });
                }

                let _: () = unsafe { msg_send![self.inner, whiteBalanceMode: setter] };

                Ok(())
            }
            KnownCameraControl::BacklightComp => {
                let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Control does not exist".to_string(),
                })?;

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Read Only".to_string(),
                    });
                }

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Disabled".to_string(),
                    });
                }

                let setter =
                    NSInteger::from(*value.as_enum().ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Expected Enum".to_string(),
                    })? as i32);

                if !ctrlvalue.description().verify_setter(&value) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Failed to verify value".to_string(),
                    });
                }

                let _: () = unsafe { msg_send![self.inner, whiteBalanceMode: setter] };

                Ok(())
            }
            KnownCameraControl::Gain => {
                let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Control does not exist".to_string(),
                })?;

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Read Only".to_string(),
                    });
                }

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Disabled".to_string(),
                    });
                }

                let setter = NSInteger::from(*value.as_boolean().ok_or(
                    NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Expected Boolean".to_string(),
                    },
                )? as i32);

                if !ctrlvalue.description().verify_setter(&value) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Failed to verify value".to_string(),
                    });
                }

                let _: () = unsafe { msg_send![self.inner, whiteBalanceMode: setter] };

                Ok(())
            }
            KnownCameraControl::Zoom => {
                let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Control does not exist".to_string(),
                })?;

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Read Only".to_string(),
                    });
                }

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Disabled".to_string(),
                    });
                }

                let setter = *value.as_float().ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Expected float".to_string(),
                })? as c_float;

                if !ctrlvalue.description().verify_setter(&value) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Failed to verify value".to_string(),
                    });
                }

                let _: () = unsafe {
                    msg_send![self.inner, rampToVideoZoomFactor: setter withRate: 1.0_f32]
                };

                Ok(())
            }
            KnownCameraControl::Exposure => {
                let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Control does not exist".to_string(),
                })?;

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Read Only".to_string(),
                    });
                }

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Disabled".to_string(),
                    });
                }

                let setter =
                    NSInteger::from(*value.as_enum().ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Expected Enum".to_string(),
                    })? as i32);

                if !ctrlvalue.description().verify_setter(&value) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Failed to verify value".to_string(),
                    });
                }

                let _: () = unsafe { msg_send![self.inner, exposureMode: setter] };

                Ok(())
            }
            KnownCameraControl::Iris => Err(NokhwaError::SetPropertyError {
                property: id.to_string(),
                value: value.to_string(),
                error: "Read Only".to_string(),
            }),
            KnownCameraControl::Focus => {
                let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Control does not exist".to_string(),
                })?;

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Read Only".to_string(),
                    });
                }

                if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Disabled".to_string(),
                    });
                }

                let setter =
                    NSInteger::from(*value.as_enum().ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Expected Enum".to_string(),
                    })? as i32);

                if !ctrlvalue.description().verify_setter(&value) {
                    return Err(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Failed to verify value".to_string(),
                    });
                }

                let _: () = unsafe { msg_send![self.inner, focusMode: setter] };

                Ok(())
            }
            KnownCameraControl::Other(i) => match i {
                0 => {
                    let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Control does not exist".to_string(),
                    })?;

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Read Only".to_string(),
                        });
                    }

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Disabled".to_string(),
                        });
                    }

                    let setter = value
                        .as_point()
                        .ok_or(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Expected Point".to_string(),
                        })
                        .map(|(x, y)| CGPoint {
                            x: *x as f32,
                            y: *y as f32,
                        })?;

                    if !ctrlvalue.description().verify_setter(&value) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Failed to verify value".to_string(),
                        });
                    }

                    let _: () = unsafe { msg_send![self.inner, focusPointOfInterest: setter] };

                    Ok(())
                }
                1 => {
                    let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Control does not exist".to_string(),
                    })?;

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Read Only".to_string(),
                        });
                    }

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Disabled".to_string(),
                        });
                    }

                    let setter = *value.as_float().ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Expected float".to_string(),
                    })? as c_float;

                    if !ctrlvalue.description().verify_setter(&value) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Failed to verify value".to_string(),
                        });
                    }

                    let _: () = unsafe {
                        msg_send![self.inner, setFocusModeLockedWithLensPosition: setter handler: Nil]
                    };

                    Ok(())
                }
                2 => {
                    let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Control does not exist".to_string(),
                    })?;

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Read Only".to_string(),
                        });
                    }

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Disabled".to_string(),
                        });
                    }

                    let setter = value
                        .as_point()
                        .ok_or(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Expected Point".to_string(),
                        })
                        .map(|(x, y)| CGPoint {
                            x: *x as f32,
                            y: *y as f32,
                        })?;

                    if !ctrlvalue.description().verify_setter(&value) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Failed to verify value".to_string(),
                        });
                    }

                    let _: () =
                        unsafe { msg_send![self.inner, exposurePointOfInterest: setter] };

                    Ok(())
                }
                3 => {
                    let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Control does not exist".to_string(),
                    })?;

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Read Only".to_string(),
                        });
                    }

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Disabled".to_string(),
                        });
                    }

                    let setter =
                        if *value.as_boolean().ok_or(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Expected Boolean".to_string(),
                        })? {
                            YES
                        } else {
                            NO
                        };

                    if !ctrlvalue.description().verify_setter(&value) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Failed to verify value".to_string(),
                        });
                    }

                    let _: () = unsafe {
                        msg_send![
                            self.inner,
                            automaticallyAdjustsFaceDrivenAutoExposureEnabled: setter
                        ]
                    };

                    Ok(())
                }
                4 => {
                    let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Control does not exist".to_string(),
                    })?;

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Read Only".to_string(),
                        });
                    }

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Disabled".to_string(),
                        });
                    }

                    let setter = *value.as_float().ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Expected Float".to_string(),
                    })? as f32;

                    if !ctrlvalue.description().verify_setter(&value) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Failed to verify value".to_string(),
                        });
                    }

                    let _: () = unsafe {
                        msg_send![self.inner, setExposureTargetBias: setter handler: Nil]
                    };

                    Ok(())
                }
                5 => {
                    let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Control does not exist".to_string(),
                    })?;

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Read Only".to_string(),
                        });
                    }

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Disabled".to_string(),
                        });
                    }

                    let setter = NSInteger::from(*value.as_enum().ok_or(
                        NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Expected Enum".to_string(),
                        },
                    )? as i32);

                    if !ctrlvalue.description().verify_setter(&value) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Failed to verify value".to_string(),
                        });
                    }

                    let _: () = unsafe { msg_send![self.inner, torchMode: setter] };

                    Ok(())
                }
                6 => {
                    let ctrlvalue = controls.get(&id).ok_or(NokhwaError::SetPropertyError {
                        property: id.to_string(),
                        value: value.to_string(),
                        error: "Control does not exist".to_string(),
                    })?;

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::ReadOnly) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Read Only".to_string(),
                        });
                    }

                    if ctrlvalue.flag().contains(&KnownCameraControlFlag::Disabled) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Disabled".to_string(),
                        });
                    }

                    let setter =
                        if *value.as_boolean().ok_or(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Expected Boolean".to_string(),
                        })? {
                            YES
                        } else {
                            NO
                        };

                    if !ctrlvalue.description().verify_setter(&value) {
                        return Err(NokhwaError::SetPropertyError {
                            property: id.to_string(),
                            value: value.to_string(),
                            error: "Failed to verify value".to_string(),
                        });
                    }

                    let _: () = unsafe {
                        msg_send![self.inner, geometricDistortionCorrectionEnabled: setter]
                    };

                    Ok(())
                }
                _ => Err(NokhwaError::SetPropertyError {
                    property: id.to_string(),
                    value: value.to_string(),
                    error: "Unknown Control".to_string(),
                }),
            },
            _ => Err(NokhwaError::SetPropertyError {
                property: id.to_string(),
                value: value.to_string(),
                error: "Unknown Control".to_string(),
            }),
        }
    }

    pub fn active_format(&self) -> Result<CameraFormat, NokhwaError> {
        let af: *mut Object = unsafe { msg_send![self.inner, activeFormat] };
        let avf_format = AVCaptureDeviceFormat::try_from(af)?;
        let resolution = avf_format.resolution;
        let fourcc = avf_format.fourcc;
        let mut a = avf_format
            .fps_list
            .into_iter()
            .map(move |fps_f64| {
                let fps = fps_f64 as u32;

                let resolution =
                    Resolution::new(resolution.width as u32, resolution.height as u32); // FIXME: what the fuck?
                CameraFormat::new(resolution, fourcc, fps)
            })
            .collect::<Vec<_>>();
        a.sort_by(|a, b| a.frame_rate().cmp(&b.frame_rate()));

        if a.len() != 0 {
            Ok(a[a.len() - 1])
        } else {
            Err(NokhwaError::GetPropertyError {
                property: "activeFormat".to_string(),
                error: "None??".to_string(),
            })
        }
    }
}
