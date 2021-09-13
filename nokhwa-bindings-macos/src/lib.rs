#![cfg_attr(
    any(target_os = "macos", target_os = "ios"),
    link(name = "AVFoundation", kind = "Framework")
)]

use objc::runtime::{Class, Object};
use std::ffi::c_void;

#[macro_use]
extern crate objc;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BindingError {}

pub mod avfoundation {
    use crate::BindingError;
    use cocoa_foundation::foundation::{NSArray, NSInteger, NSString, NSUInteger};
    use objc::runtime::{method_setImplementation, Object};
    use std::borrow::Cow;
    use std::ffi::{c_void, CStr};
    use std::os::raw::c_long;

    fn str_to_nsstr(string: &str) -> *mut Object {
        const UTF8_ENCODING: usize = 4;

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
            let _: *mut c_void = msg_send![obj, autorelease];
            obj
        }
    }

    fn nsstr_to_str<'a>(nsstr: *mut Object) -> Cow<'a, str> {
        const UTF8_ENCODING: usize = 4;

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
        let _: *mut c_void = unsafe { msg_send![mutable_array, autorelease] };
        immutable_array
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

    // Localized Name
    //
    #[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
    pub struct AVCaptureDeviceDescriptor {
        pub name: String,
        pub description: String,
        pub misc: String,
        pub index: u64,
    }

    pub struct AVCaptureDeviceDiscoverySession {
        inner: *mut Object,
    }

    impl AVCaptureDeviceDiscoverySession {
        pub fn new(
            device_types: Vec<AVCaptureDeviceType>,
            media_type: AVMediaType,
            position: AVCaptureDevicePosition,
        ) -> Result<Self, BindingError> {
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
            devices
        }
    }

    pub struct AVCaptureDevice {
        inner: *mut Object,
    }

    impl AVCaptureDevice {
        pub fn new(id: &str) -> Result<Self, BindingError> {
            let nsstr_id = str_to_nsstr(id);
            let avfoundation_capture_cls = class!(AVCaptureDevice);
            let capture: *mut Object =
                unsafe { msg_send![avfoundation_capture_cls, deviceUniqueID: nsstr_id] };
            Ok(AVCaptureDevice { inner: capture })
        }
    }
}
