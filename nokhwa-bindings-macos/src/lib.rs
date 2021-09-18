#![cfg_attr(
    any(target_os = "macos", target_os = "ios"),
    link(name = "AVFoundation", kind = "framework")
)]
#![cfg_attr(
    any(target_os = "macos", target_os = "ios"),
    link(name = "CoreMedia", kind = "framework")
)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[macro_use]
extern crate objc;
#[macro_use]
extern crate lazy_static;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AVFError {
    #[error("Invalid: Expected {expected} Found {found}")]
    InvalidType { expected: String, found: String },
    #[error("Invalid Value: {found}")]
    InvalidValue { found: String },
    #[error("Already Busy: {0}")]
    AlreadyBusy(String),
    #[error("Failed to open device {index}: {why}")]
    FailedToOpenDevice { index: usize, why: String },
    #[error("Config Not Accepted")]
    ConfigNotAccepted,
    #[error("General Error: {0}")]
    General(String),
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod core_media {
    use core_media_sys::{CMFormatDescriptionRef, CMTime, CMVideoDimensions};

    #[allow(non_snake_case)]
    extern "C" {
        pub fn CMVideoFormatDescriptionGetDimensions(
            videoDesc: CMFormatDescriptionRef,
        ) -> CMVideoDimensions;

        pub fn CMTimeMake(value: i64, scale: i32) -> CMTime;
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod avfoundation {
    use crate::core_media::{CMTimeMake, CMVideoFormatDescriptionGetDimensions};
    use crate::AVFError;
    use cocoa_foundation::foundation::{NSArray, NSInteger, NSString, NSUInteger};
    use core_media_sys::{
        kCMVideoCodecType_422YpCbCr8, kCMVideoCodecType_JPEG, CMFormatDescriptionGetMediaSubType,
        CMTime, CMVideoDimensions,
    };
    use objc::runtime::{method_setImplementation, Object, BOOL, YES};
    use std::{
        borrow::{Borrow, Cow},
        cmp::Ordering,
        convert::TryFrom,
        error::Error,
        ffi::{c_void, CStr},
        sync::{
            atomic::{AtomicBool, Ordering as MemOrdering},
            Arc, Mutex, TryLockError, TryLockResult,
        },
    };

    const UTF8_ENCODING: usize = 4;

    macro_rules! create_boilerplate_impl {
        {
            $( [$class_vis:vis $class_name:ident : $( $field_vis:vis $field_name:ident : $field_type:ty ),*] ),+
        } => {
            $(
                $class_vis struct $class_name {
                    inner: *mut Object,
                    $(
                        $field_vis $field_name : $field_type
                    )*
                }

                impl $class_name {
                    pub fn inner(&self) -> *mut Object {
                        self.inner
                    }
                }
            )+
        };

        {
            $( [$class_vis:vis $class_name:ident ] ),+
        } => {
            $(
                $class_vis struct $class_name {
                    inner: *mut Object,
                }

                impl $class_name {
                    pub fn inner(&self) -> *mut Object {
                        self.inner
                    }
                }

                impl From<*mut Object> for $class_name {
                    fn from(obj: *mut Object) -> Self {
                        $class_name {
                            inner: obj,
                        }
                    }
                }


            )+
        };
    }

    fn str_to_nsstr(string: &str) -> *mut Object {
        let cls = class!(NSString);
        let bytes = string.as_ptr() as *const c_void;
        unsafe {
            let obj: *mut Object = msg_send![cls, alloc];
            let obj: *mut Object = msg_send![
                obj,
                initWithBytes:bytes
                length:string.len()
                encoding:UTF8_ENCODING
            ];
            let _: *mut std::ffi::c_void = msg_send![obj, autorelease];
            obj
        }
    }

    fn nsstr_to_str<'a>(nsstr: *mut Object) -> Cow<'a, str> {
        let data = unsafe { CStr::from_ptr(nsstr.UTF8String()) };
        data.to_string_lossy()
    }

    fn vec_to_ns_arr<T: Into<*mut Object>>(data: Vec<T>) -> *mut Object {
        let ns_mut_array_cls = class!(NSMutableArray);
        let ns_array_cls = class!(NSArray);
        let mutable_array: *mut Object = unsafe { msg_send![ns_mut_array_cls, array] };
        data.into_iter().for_each(|item| {
            let item_obj: *mut Object = item.into();
            let _: *mut c_void = unsafe { msg_send![mutable_array, addObject: item_obj] };
        });
        let immutable_array: *mut Object =
            unsafe { msg_send![ns_array_cls, arrayWithArray: mutable_array] };
        let _: *mut std::ffi::c_void = unsafe { msg_send![mutable_array, autorelease] };
        let _: *mut std::ffi::c_void = unsafe { msg_send![immutable_array, autorelease] };
        immutable_array
    }

    fn ns_arr_to_vec<T: From<*mut Object>>(data: *mut Object) -> Vec<T> {
        let length = unsafe { NSArray::count(data) };

        let mut out_vec: Vec<T> = Vec::with_capacity(length as usize);
        for index in 0..length {
            let item = unsafe { NSArray::objectAtIndex(data, index) };
            out_vec.push(T::from(item));
        }
        let _: *mut std::ffi::c_void = unsafe { msg_send![data, autorelease] };
        out_vec
    }

    fn try_ns_arr_to_vec<T, TE>(data: *mut Object) -> Result<Vec<T>, TE>
    where
        TE: Error,
        T: TryFrom<*mut Object, Error = TE>,
    {
        let length = unsafe { NSArray::count(data) };

        let mut out_vec: Vec<T> = Vec::with_capacity(length as usize);
        for index in 0..length {
            let item = unsafe { NSArray::objectAtIndex(data, index) };
            out_vec.push(T::try_from(item)?);
        }
        let _: *mut std::ffi::c_void = unsafe { msg_send![data, autorelease] };
        Ok(out_vec)
    }

    fn default_callback(_: bool) {}

    lazy_static! {
        static ref CAMERA_AUTHORIZED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        static ref USER_CALLBACK_FN: Arc<Mutex<fn(bool)>> = Arc::new(Mutex::new(default_callback));
    }

    extern "C" fn objc_authorization_event_callback_fn(result: BOOL) {
        let result = if result == YES {
            CAMERA_AUTHORIZED.store(true, MemOrdering::SeqCst);
            true
        } else {
            CAMERA_AUTHORIZED.store(false, MemOrdering::SeqCst);
            false
        };

        loop {
            match USER_CALLBACK_FN.try_lock() {
                Ok(callback) => {
                    callback(result);
                    break;
                }
                Err(why) => match why {
                    TryLockError::Poisoned(_) => {
                        break;
                    }
                    TryLockError::WouldBlock => {
                        continue;
                    }
                },
            }
        }
    }

    pub fn request_permission_with_callback(callback: fn(bool)) {
        let cls = class!(AVCaptureDevice);
        loop {
            match USER_CALLBACK_FN.try_lock() {
                Ok(mut cb) => {
                    *cb = callback;
                    break;
                }
                Err(why) => match why {
                    TryLockError::Poisoned(_) => {
                        break;
                    }
                    TryLockError::WouldBlock => {
                        continue;
                    }
                },
            }
        }
        // send in a C function and hope it works
        unsafe {
            msg_send![cls, requestAccessForMediaType:AVMediaType::Video.into_ns_str() completionHandler:objc_authorization_event_callback_fn]
        }
    }

    pub fn current_authorization_status() -> AVAuthorizationStatus {
        let cls = class!(AVCaptureDevice);
        let status: AVAuthorizationStatus = unsafe {
            msg_send![cls, authorizationStatusForMediaType:AVMediaType::Video.into_ns_str()]
        };
        match status {
            AVAuthorizationStatus::Authorized => CAMERA_AUTHORIZED.store(true, MemOrdering::SeqCst),
            _ => CAMERA_AUTHORIZED.store(false, MemOrdering::SeqCst),
        }
        status
    }

    pub fn query_avfoundation() -> Result<Vec<AVCaptureDeviceDescriptor>, AVFError> {
        Ok(AVCaptureDeviceDiscoverySession::new(
            vec![
                AVCaptureDeviceType::UltraWide,
                AVCaptureDeviceType::Telephoto,
            ],
            AVMediaType::Video,
            AVCaptureDevicePosition::Unspecified,
        )?
        .devices())
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
        ExternalUnknown,
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
                AVCaptureDeviceType::ExternalUnknown => {
                    str_to_nsstr("AVCaptureDeviceTypeBuiltInExternalUnknown")
                }
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
                AVMediaType::Audio => str_to_nsstr("AVMediaTypeAudio"),
                AVMediaType::ClosedCaption => str_to_nsstr("AVMediaTypeClosedCaption"),
                AVMediaType::DepthData => str_to_nsstr("AVMediaTypeDepthData"),
                AVMediaType::Metadata => str_to_nsstr("AVMediaTypeMetadata"),
                AVMediaType::MetadataObject => str_to_nsstr("AVMediaTypeMetadataObject"),
                AVMediaType::Muxed => str_to_nsstr("AVMediaTypeMuxed"),
                AVMediaType::Subtitle => str_to_nsstr("AVMediaTypeSubtitle"),
                AVMediaType::Text => str_to_nsstr("AVMediaTypeText"),
                AVMediaType::Timecode => str_to_nsstr("AVMediaTypeTimecode"),
                AVMediaType::Video => str_to_nsstr("AVMediaTypeVideo"),
            }
        }
    }

    impl TryFrom<*mut Object> for AVMediaType {
        type Error = AVFError;

        fn try_from(value: *mut Object) -> Result<Self, Self::Error> {
            let borrow_str = nsstr_to_str(value);
            let value_str: &str = borrow_str.borrow();
            let v = match value_str {
                "AVMediaTypeAudio" => Ok(AVMediaType::Audio),
                "AVMediaTypeClosedCaption" => Ok(AVMediaType::ClosedCaption),
                "AVMediaTypeDepthData" => Ok(AVMediaType::DepthData),
                "AVMediaTypeMetadata" => Ok(AVMediaType::Metadata),
                "AVMediaTypeMetadataObject" => Ok(AVMediaType::MetadataObject),
                "AVMediaTypeMuxed" => Ok(AVMediaType::Muxed),
                "AVMediaTypeSubtitle" => Ok(AVMediaType::Subtitle),
                "AVMediaTypeText" => Ok(AVMediaType::Text),
                "AVMediaTypeTimecode" => Ok(AVMediaType::Timecode),
                "AVMediaTypeVideo" => Ok(AVMediaType::Video),
                _ => {
                    return Err(AVFError::InvalidValue {
                        found: value_str.to_string(),
                    })
                }
            };

            let _: *mut std::ffi::c_void = unsafe { msg_send![value, autorelease] };
            v
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

    #[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
    #[repr(u32)]
    pub enum AVFourCC {
        YUV2 = kCMVideoCodecType_JPEG,
        MJPEG = kCMVideoCodecType_422YpCbCr8,
    }

    // Localized Name
    //
    #[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
    pub struct AVCaptureDeviceDescriptor {
        pub name: String,
        pub description: String,
        pub misc: String,
        pub index: u64,
    }

    create_boilerplate_impl! {
        [pub AVFrameRateRange],
        [pub AVCaptureDeviceDiscoverySession],
        [pub AVCaptureDevice],
        [pub AVCaptureDeviceInput],
        [pub AVCaptureSession]
    }

    impl AVFrameRateRange {
        pub fn max(&self) -> f64 {
            unsafe { msg_send![self.inner, maxFrameRate] }
        }

        pub fn min(&self) -> f64 {
            unsafe { msg_send![self.inner, minFrameRate] }
        }
    }

    #[derive(Clone, Debug)]
    pub struct AVCaptureDeviceFormat {
        pub resolution: CMVideoDimensions,
        pub fps_list: Vec<f64>,
        pub fourcc: AVFourCC,
    }

    impl TryFrom<*mut Object> for AVCaptureDeviceFormat {
        type Error = AVFError;

        fn try_from(value: *mut Object) -> Result<Self, Self::Error> {
            let media_type_raw: *mut Object = unsafe { msg_send![value, mediaType] };
            let media_type = AVMediaType::try_from(media_type_raw)?;
            if media_type != AVMediaType::Video {
                return Err(AVFError::InvalidType {
                    expected: "AVMediaTypeVideo".to_string(),
                    found: format!("{:?}", media_type),
                });
            }
            let mut fps_list = ns_arr_to_vec::<AVFrameRateRange>(unsafe {
                msg_send![value, videoSupportedFrameRateRanges]
            })
            .into_iter()
            .map(|v| [v.min(), v.max()])
            .flatten()
            .collect::<Vec<f64>>();
            fps_list.sort_by(|n, m| n.partial_cmp(m).unwrap_or(Ordering::Equal));
            fps_list.dedup();
            let description_obj: *mut Object = unsafe { msg_send![value, formatDescription] };
            let resolution =
                unsafe { CMVideoFormatDescriptionGetDimensions(description_obj as *mut c_void) };
            let fcc_raw =
                unsafe { CMFormatDescriptionGetMediaSubType(description_obj as *mut c_void) };
            #[allow(non_upper_case_globals)]
            let fourcc = match fcc_raw {
                kCMVideoCodecType_422YpCbCr8 => AVFourCC::YUV2,
                kCMVideoCodecType_JPEG => AVFourCC::MJPEG,
                _ => {
                    return Err(AVFError::InvalidValue {
                        found: fcc_raw.to_string(),
                    })
                }
            };
            let _: *mut std::ffi::c_void = unsafe { msg_send![description_obj, autorelease] };
            let _: *mut std::ffi::c_void = unsafe { msg_send![value, autorelease] };
            Ok(AVCaptureDeviceFormat {
                resolution,
                fps_list,
                fourcc,
            })
        }
    }

    impl AVCaptureDeviceDiscoverySession {
        pub fn new(
            device_types: Vec<AVCaptureDeviceType>,
            media_type: AVMediaType,
            position: AVCaptureDevicePosition,
        ) -> Result<Self, AVFError> {
            let device_types = vec_to_ns_arr(device_types);
            let media_type: *mut Object = media_type.into();
            let position = position as NSInteger;

            let discovery_session_cls = class!(AVCaptureDeviceDiscoverySession);
            let discovery_session: *mut Object = unsafe {
                msg_send![discovery_session_cls, deviceTypes:device_types mediaType:media_type position:position]
            };
            Ok(AVCaptureDeviceDiscoverySession {
                inner: discovery_session,
            })
        }

        pub fn devices(&self) -> Vec<AVCaptureDeviceDescriptor> {
            let device_ns_array: *mut Object = unsafe { msg_send![self.inner, devices] };
            let objects_len: NSUInteger = unsafe { device_ns_array.count() };
            let mut devices = vec![AVCaptureDeviceDescriptor::default(); objects_len as usize];
            for index in 0..objects_len {
                let device = unsafe { device_ns_array.objectAtIndex(index) };
                let name = nsstr_to_str(unsafe { msg_send![device, localizedName] }).to_string();
                let manufacturer = nsstr_to_str(unsafe { msg_send![device, manufacturer] });
                let position: AVCaptureDevicePosition = unsafe { msg_send![device, position] };
                let lens_aperture: f64 = unsafe { msg_send![device, lensAperture] };
                let device_type = nsstr_to_str(unsafe { msg_send![device, deviceType] });
                let model_id = nsstr_to_str(unsafe { msg_send![device, modelID] });
                let description = format!(
                    "{}: {} - {}, {:?} f{}",
                    manufacturer, model_id, device_type, position, lens_aperture
                );
                let misc = nsstr_to_str(unsafe { msg_send![device, uniqueID] }).to_string();

                devices.push(AVCaptureDeviceDescriptor {
                    name,
                    description,
                    misc,
                    index,
                })
            }
            let _: *mut std::ffi::c_void = unsafe { msg_send![device_ns_array, release] };
            devices
        }
    }

    impl AVCaptureDevice {
        pub fn new(index: usize) -> Result<Self, AVFError> {
            let devices = AVCaptureDeviceDiscoverySession::new(
                vec![
                    AVCaptureDeviceType::UltraWide,
                    AVCaptureDeviceType::Telephoto,
                ],
                AVMediaType::Video,
                AVCaptureDevicePosition::Unspecified,
            )?
            .devices();
            match devices.get(index) {
                Some(device) => Ok(AVCaptureDevice::from_id(&device.misc)?),
                None => Err(AVFError::FailedToOpenDevice {
                    index,
                    why: "No device at index".to_string(),
                }),
            }
        }

        pub fn from_id(id: &str) -> Result<Self, AVFError> {
            let nsstr_id = str_to_nsstr(id);
            let avfoundation_capture_cls = class!(AVCaptureDevice);
            let capture: *mut Object =
                unsafe { msg_send![avfoundation_capture_cls, deviceUniqueID: nsstr_id] };
            Ok(AVCaptureDevice { inner: capture })
        }

        pub fn supported_formats(&self) -> Result<Vec<AVCaptureDeviceFormat>, AVFError> {
            try_ns_arr_to_vec::<AVCaptureDeviceFormat, AVFError>(unsafe {
                msg_send![self.inner, formats]
            })
        }

        pub fn already_in_use(&self) -> bool {
            unsafe { msg_send![self.inner, inUseByOtherApplication] }
        }

        pub fn is_suspended(&self) -> bool {
            unsafe { msg_send![self.inner, isSuspended] }
        }

        pub fn lock(&mut self) -> Result<(), AVFError> {
            if self.already_in_use() {
                return Err(AVFError::AlreadyBusy("In Use".to_string()));
            }
            let err_ptr: *mut Object = std::ptr::null_mut();
            let accepted: BOOL = unsafe { msg_send![self.inner, lockForConfiguration: err_ptr] };
            if !err_ptr.is_null() {
                return Err(AVFError::ConfigNotAccepted);
            }
            // Space these out for debug purposes
            if !accepted == YES {
                return Err(AVFError::ConfigNotAccepted);
            }
            Ok(())
        }

        pub fn unlock(&mut self) {
            unsafe { msg_send![self.inner, unlockForConfiguration] }
        }

        pub fn set_frame_rate(&mut self, fps: u32) {
            let mut time = unsafe { CMTimeMake(1, fps as i32) };
            let time_ptr: *mut CMTime = &mut time;
            unsafe {
                let _: () = msg_send![self.inner, activeVideoMinFrameDuration: time_ptr];
                let _: () = msg_send![self.inner, activeVideoMaxFrameDuration: time_ptr];
            }
        }

        // pub fn set_frame_format(&mut self, frame_format: )
    }

    impl Drop for AVCaptureDevice {
        fn drop(&mut self) {
            unsafe {
                let _: () = msg_send![self.inner, release];
            }
        }
    }

    impl AVCaptureDeviceInput {
        pub fn new(capture_device: AVCaptureDevice) -> Result<Self, AVFError> {
            let cls = class!(AVCaptureDeviceInput);
            let err_ptr: *mut Object = std::ptr::null_mut();
            let capture_input: *mut Object = unsafe {
                let allocated: *mut Object = msg_send![cls, alloc];
                msg_send![allocated, initWithDevice:capture_device.inner() error:err_ptr]
            };
            if !err_ptr.is_null() {
                return Err(AVFError::General("Failed to create input".to_string()));
            }

            Ok(AVCaptureDeviceInput {
                inner: capture_input,
            })
        }
    }
}
