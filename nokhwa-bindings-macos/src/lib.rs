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

// hello, future peng here
// whatever is written here will induce horrors uncomprehendable.
// save yourselves. write apple code in swift and bind it to rust.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[macro_use]
extern crate objc;
#[cfg(any(target_os = "macos", target_os = "ios"))]
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
    #[error("Cannot add input to session: Rejected")]
    RejectedInput,
    #[error("Cannot add output to session: Rejected")]
    RejectedOutput,
    #[error("Failed to open stream: {0}")]
    StreamOpen(String),
    #[error("Failed to read frame: {0}")]
    ReadFrame(String),
    #[error("Unsupported")]
    NotSupported,
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[allow(non_snake_case)]
pub mod core_media {
    // all of this is stolen from bindgen
    // steal it idc
    use core_media_sys::{
        CMBlockBufferRef, CMFormatDescriptionRef, CMSampleBufferRef, CMTime, CMVideoDimensions,
        FourCharCode,
    };
    use objc::{runtime::Object, Message};
    use std::ops::Deref;

    pub type Id = *mut Object;

    #[repr(transparent)]
    #[derive(Clone)]
    pub struct NSObject(pub Id);
    impl Deref for NSObject {
        type Target = Object;
        fn deref(&self) -> &Self::Target {
            unsafe { &*self.0 }
        }
    }
    unsafe impl Message for NSObject {}
    impl NSObject {
        pub fn alloc() -> Self {
            Self(unsafe { msg_send!(objc::class!(NSObject), alloc) })
        }
    }

    #[repr(transparent)]
    #[derive(Clone)]
    pub struct NSString(pub Id);
    impl Deref for NSString {
        type Target = Object;
        fn deref(&self) -> &Self::Target {
            unsafe { &*self.0 }
        }
    }
    unsafe impl Message for NSString {}
    impl NSString {
        pub fn alloc() -> Self {
            Self(unsafe { msg_send!(objc::class!(NSString), alloc) })
        }
    }

    pub type AVMediaType = NSString;

    #[allow(non_snake_case)]
    #[link(name = "CoreMedia", kind = "framework")]
    extern "C" {
        pub fn CMVideoFormatDescriptionGetDimensions(
            videoDesc: CMFormatDescriptionRef,
        ) -> CMVideoDimensions;

        pub fn CMTimeMake(value: i64, scale: i32) -> CMTime;

        pub fn CMBlockBufferGetDataLength(theBuffer: CMBlockBufferRef) -> std::os::raw::c_int;

        pub fn CMBlockBufferCopyDataBytes(
            theSourceBuffer: CMBlockBufferRef,
            offsetToData: usize,
            dataLength: usize,
            destination: *mut std::os::raw::c_void,
        ) -> std::os::raw::c_int;

        pub fn CMSampleBufferGetDataBuffer(sbuf: CMSampleBufferRef) -> CMBlockBufferRef;

        pub fn dispatch_queue_create(
            label: *const std::os::raw::c_char,
            attr: NSObject,
        ) -> NSObject;

        pub fn dispatch_release(object: NSObject);

        pub fn CMSampleBufferGetImageBuffer(sbuf: CMSampleBufferRef) -> CVImageBufferRef;

        pub fn CVPixelBufferLockBaseAddress(
            pixelBuffer: CVPixelBufferRef,
            lockFlags: CVPixelBufferLockFlags,
        ) -> CVReturn;

        pub fn CVPixelBufferUnlockBaseAddress(
            pixelBuffer: CVPixelBufferRef,
            unlockFlags: CVPixelBufferLockFlags,
        ) -> CVReturn;

        pub fn CVPixelBufferGetDataSize(pixelBuffer: CVPixelBufferRef) -> std::os::raw::c_ulong;

        pub fn CVPixelBufferGetBaseAddress(
            pixelBuffer: CVPixelBufferRef,
        ) -> *mut std::os::raw::c_void;

        pub fn CVPixelBufferGetPixelFormatType(pixelBuffer: CVPixelBufferRef) -> OSType;
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct __CVBuffer {
        _unused: [u8; 0],
    }
    pub type CVBufferRef = *mut __CVBuffer;

    pub type CVImageBufferRef = CVBufferRef;
    pub type CVPixelBufferRef = CVImageBufferRef;
    pub type CVPixelBufferLockFlags = u64;
    pub type CVReturn = i32;

    pub type OSType = FourCharCode;
    pub type AVVideoCodecType = NSString;

    #[link(name = "AVFoundation", kind = "framework")]
    extern "C" {
        pub static AVVideoCodecKey: NSString;
        pub static AVVideoCodecTypeHEVC: AVVideoCodecType;
        pub static AVVideoCodecTypeH264: AVVideoCodecType;
        pub static AVVideoCodecTypeJPEG: AVVideoCodecType;
        pub static AVVideoCodecTypeAppleProRes4444: AVVideoCodecType;
        pub static AVVideoCodecTypeAppleProRes422: AVVideoCodecType;
        pub static AVVideoCodecTypeAppleProRes422HQ: AVVideoCodecType;
        pub static AVVideoCodecTypeAppleProRes422LT: AVVideoCodecType;
        pub static AVVideoCodecTypeAppleProRes422Proxy: AVVideoCodecType;
        pub static AVVideoCodecTypeHEVCWithAlpha: AVVideoCodecType;
        pub static AVVideoCodecHEVC: NSString;
        pub static AVVideoCodecH264: NSString;
        pub static AVVideoCodecJPEG: NSString;
        pub static AVVideoCodecAppleProRes4444: NSString;
        pub static AVVideoCodecAppleProRes422: NSString;
        pub static AVVideoWidthKey: NSString;
        pub static AVVideoHeightKey: NSString;
        pub static AVVideoExpectedSourceFrameRateKey: NSString;

        pub static AVMediaTypeVideo: AVMediaType;
        pub static AVMediaTypeAudio: AVMediaType;
        pub static AVMediaTypeText: AVMediaType;
        pub static AVMediaTypeClosedCaption: AVMediaType;
        pub static AVMediaTypeSubtitle: AVMediaType;
        pub static AVMediaTypeTimecode: AVMediaType;
        pub static AVMediaTypeMetadata: AVMediaType;
        pub static AVMediaTypeMuxed: AVMediaType;
        pub static AVMediaTypeMetadataObject: AVMediaType;
        pub static AVMediaTypeDepthData: AVMediaType;
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod avfoundation {
    use crate::core_media::{
        AVMediaTypeAudio, AVMediaTypeClosedCaption, AVMediaTypeDepthData, AVMediaTypeMetadata,
        AVMediaTypeMetadataObject, AVMediaTypeMuxed, AVMediaTypeSubtitle, AVMediaTypeText,
        AVMediaTypeTimecode, CVPixelBufferGetPixelFormatType,
    };
    use crate::{
        core_media::{
            dispatch_queue_create, AVMediaTypeVideo, CMSampleBufferGetImageBuffer,
            CMVideoFormatDescriptionGetDimensions, CVImageBufferRef, CVPixelBufferGetBaseAddress,
            CVPixelBufferGetDataSize, CVPixelBufferLockBaseAddress, CVPixelBufferUnlockBaseAddress,
            NSObject,
        },
        AVFError,
    };
    use block::ConcreteBlock;
    use cocoa_foundation::foundation::{NSArray, NSInteger, NSString, NSUInteger};
    use core_media_sys::{
        kCMPixelFormat_422YpCbCr8_yuvs, kCMPixelFormat_8IndexedGray_WhiteIsZero,
        kCMVideoCodecType_422YpCbCr8, kCMVideoCodecType_JPEG, kCMVideoCodecType_JPEG_OpenDML,
        CMFormatDescriptionGetMediaSubType, CMFormatDescriptionRef, CMSampleBufferRef,
        CMVideoDimensions,
    };
    use dashmap::DashMap;
    use flume::{Receiver, Sender};
    use objc::{
        declare::ClassDecl,
        runtime::{Class, Object, Protocol, Sel, BOOL, YES},
    };
    use std::{
        borrow::Cow,
        cmp::Ordering,
        convert::TryFrom,
        error::Error,
        ffi::{c_void, CStr, CString},
        sync::{
            atomic::{AtomicBool, Ordering as MemOrdering},
            Arc, Mutex, TryLockError,
        },
    };

    const UTF8_ENCODING: usize = 4;

    macro_rules! create_boilerplate_impl {
        {
            $( [$class_vis:vis $class_name:ident : $( {$field_vis:vis $field_name:ident : $field_type:ty} ),*] ),+
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
            let _: *mut c_void = msg_send![obj, autorelease];
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
        let _: *mut c_void = unsafe { msg_send![mutable_array, autorelease] };
        let _: *mut c_void = unsafe { msg_send![immutable_array, autorelease] };
        immutable_array
    }

    fn ns_arr_to_vec<T: From<*mut Object>>(data: *mut Object) -> Vec<T> {
        let length = unsafe { NSArray::count(data) };

        let mut out_vec: Vec<T> = Vec::with_capacity(length as usize);
        for index in 0..length {
            let item = unsafe { NSArray::objectAtIndex(data, index) };
            out_vec.push(T::from(item));
        }
        let _: *mut c_void = unsafe { msg_send![data, autorelease] };
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
        let _: *mut c_void = unsafe { msg_send![data, autorelease] };
        Ok(out_vec)
    }

    fn compare_ns_string(this: *mut Object, other: crate::core_media::NSString) -> bool {
        unsafe {
            let equal: BOOL = msg_send![this, isEqualToString: other];
            equal == YES
        }
    }

    fn default_callback(_: bool) {}

    pub type CompressionData<'a> = (Cow<'a, [u8]>, AVFourCC);
    pub type DataPipe<'a> = (Sender<CompressionData<'a>>, Receiver<CompressionData<'a>>);

    lazy_static! {
        static ref CAMERA_AUTHORIZED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        static ref USER_CALLBACK_FN: Arc<Mutex<fn(bool)>> = Arc::new(Mutex::new(default_callback));
        static ref PIPE_MAP: Arc<DashMap<usize, DataPipe<'static>>> = Arc::new(DashMap::new());
        static ref CALLBACK_CLASS: &'static Class = {
            let mut decl = ClassDecl::new("MyCaptureCallback", class!(NSObject)).unwrap();

            // frame stack
            decl.add_ivar::<usize>("_index");

            extern "C" fn my_callback_get_index(this: &Object, _: Sel) -> usize {
                unsafe {
                    *this.get_ivar("_index")
                }
            }

            extern "C" fn my_callback_set_index(this: &mut Object, _: Sel, new_index: usize) {
                unsafe {
                    this.set_ivar("_index", new_index);
                }
            }

            // Delegate compliance method
            // SAFETY: This assumes that the buffer byte size is a u8. Any other size will cause unsafety.
            #[allow(non_snake_case)]
            #[allow(non_upper_case_globals)]
            extern fn capture_out_callback(this: &mut Object, _: Sel, _: *mut Object, didOutputSampleBuffer: CMSampleBufferRef, _: *mut Object) {
                let image_buffer: CVImageBufferRef = unsafe { CMSampleBufferGetImageBuffer(didOutputSampleBuffer) };
                unsafe { CVPixelBufferLockBaseAddress(image_buffer, 0); };

                let buffer_codec = unsafe { CVPixelBufferGetPixelFormatType(image_buffer) };

                let fourcc = match buffer_codec {
                    kCMVideoCodecType_422YpCbCr8 | kCMPixelFormat_422YpCbCr8_yuvs => AVFourCC::YUV2,
                    kCMVideoCodecType_JPEG | kCMVideoCodecType_JPEG_OpenDML => AVFourCC::MJPEG,
                    _ => {
                        return;
                    }
                };

                let buffer_length = unsafe { CVPixelBufferGetDataSize(image_buffer) };
                let buffer_ptr = unsafe { CVPixelBufferGetBaseAddress(image_buffer) };
                let buffer_as_vec = unsafe { std::slice::from_raw_parts_mut(buffer_ptr as *mut u8, buffer_length as usize).to_vec() };

                unsafe { CVPixelBufferUnlockBaseAddress(image_buffer, 0) };
                let index: usize = unsafe { msg_send![this, index] };
                let pipes = &PIPE_MAP.get(&index);
                if let Some(pipe) = pipes {
                    let _ = pipe.value().0.send((Cow::from(buffer_as_vec), fourcc));
                }
            }

            #[allow(non_snake_case)]
            extern fn capture_drop_callback(_: &mut Object, _: Sel, _: *mut Object, _: *mut Object, _: *mut Object) {
            }

            unsafe {
                decl.add_method(
                    sel!(index), my_callback_get_index as extern "C" fn(&Object, Sel) -> usize
                );
                decl.add_method(
                    sel!(setIndex:), my_callback_set_index as extern "C" fn(&mut Object, Sel, usize)
                );
                decl.add_method(
                    sel!(captureOutput:didOutputSampleBuffer:fromConnection:), capture_out_callback as extern "C" fn(&mut Object, Sel, *mut Object, CMSampleBufferRef, *mut Object)
                );
                decl.add_method(
                    sel!(captureOutput:didDropSampleBuffer:fromConnection:), capture_drop_callback as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object, *mut Object)
                );

                decl.add_protocol(Protocol::get("AVCaptureVideoDataOutputSampleBufferDelegate").unwrap());
            }

            decl.register()
        };
    }

    fn objc_authorization_event_callback_fn(result: BOOL) {
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

        let objc_fn_block: ConcreteBlock<(BOOL,), (), fn(BOOL)> =
            ConcreteBlock::new(objc_authorization_event_callback_fn);
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
        match status {
            AVAuthorizationStatus::Authorized => CAMERA_AUTHORIZED.store(true, MemOrdering::SeqCst),
            _ => CAMERA_AUTHORIZED.store(false, MemOrdering::SeqCst),
        }
        status
    }

    // fuck it, use deprecated APIs
    pub fn query_avfoundation() -> Result<Vec<AVCaptureDeviceDescriptor>, AVFError> {
        Ok(AVCaptureDevice::devices_with_type(AVMediaType::Video)
            .into_iter()
            .enumerate()
            .map(|(idx, dev)| AVCaptureDeviceDescriptor::from_capture_device(dev, idx))
            .collect::<Vec<AVCaptureDeviceDescriptor>>())
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
                AVMediaType::Audio => unsafe { AVMediaTypeAudio.0 },
                AVMediaType::ClosedCaption => unsafe { AVMediaTypeClosedCaption.0 },
                AVMediaType::DepthData => unsafe { AVMediaTypeDepthData.0 },
                AVMediaType::Metadata => unsafe { AVMediaTypeMetadata.0 },
                AVMediaType::MetadataObject => unsafe { AVMediaTypeMetadataObject.0 },
                AVMediaType::Muxed => unsafe { AVMediaTypeMuxed.0 },
                AVMediaType::Subtitle => unsafe { AVMediaTypeSubtitle.0 },
                AVMediaType::Text => unsafe { AVMediaTypeText.0 },
                AVMediaType::Timecode => unsafe { AVMediaTypeTimecode.0 },
                AVMediaType::Video => unsafe { AVMediaTypeVideo.0 },
            }
        }
    }

    impl TryFrom<*mut Object> for AVMediaType {
        type Error = AVFError;

        fn try_from(value: *mut Object) -> Result<Self, Self::Error> {
            unsafe {
                if compare_ns_string(value, (AVMediaTypeAudio).clone()) {
                    Ok(AVMediaType::Audio)
                } else if compare_ns_string(value, (AVMediaTypeClosedCaption).clone()) {
                    Ok(AVMediaType::ClosedCaption)
                } else if compare_ns_string(value, (AVMediaTypeDepthData).clone()) {
                    Ok(AVMediaType::DepthData)
                } else if compare_ns_string(value, (AVMediaTypeMetadata).clone()) {
                    Ok(AVMediaType::Metadata)
                } else if compare_ns_string(value, (AVMediaTypeMetadataObject).clone()) {
                    Ok(AVMediaType::MetadataObject)
                } else if compare_ns_string(value, (AVMediaTypeMuxed).clone()) {
                    Ok(AVMediaType::Muxed)
                } else if compare_ns_string(value, (AVMediaTypeSubtitle).clone()) {
                    Ok(AVMediaType::Subtitle)
                } else if compare_ns_string(value, (AVMediaTypeText).clone()) {
                    Ok(AVMediaType::Text)
                } else if compare_ns_string(value, (AVMediaTypeTimecode).clone()) {
                    Ok(AVMediaType::Timecode)
                } else if compare_ns_string(value, (AVMediaTypeVideo).clone()) {
                    Ok(AVMediaType::Video)
                } else {
                    let name = nsstr_to_str(value);
                    Err(AVFError::InvalidValue {
                        found: name.to_string(),
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

    #[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
    #[repr(u32)]
    pub enum AVFourCC {
        YUV2,
        MJPEG,
        GRAY8,
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

    impl AVCaptureDeviceDescriptor {
        pub fn from_capture_device(
            device: AVCaptureDevice,
            index: usize,
        ) -> AVCaptureDeviceDescriptor {
            let device = device.inner();
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
            AVCaptureDeviceDescriptor {
                name,
                description,
                misc,
                index: index as u64,
            }
        }
    }

    pub struct AVCaptureVideoCallback {
        index: usize,
        delegate: *mut Object,
    }

    impl AVCaptureVideoCallback {
        pub fn new(index: usize) -> Self {
            let cls = &CALLBACK_CLASS as &Class;
            let delegate: *mut Object = unsafe { msg_send![cls, alloc] };
            let delegate: *mut Object = unsafe { msg_send![delegate, init] };

            let data_pipe: DataPipe = flume::unbounded();
            let _ = &PIPE_MAP.insert(index, data_pipe);

            AVCaptureVideoCallback { index, delegate }
        }

        pub fn index(&self) -> usize {
            self.index
        }

        pub fn data_len(&self) -> usize {
            unsafe { msg_send![self.delegate, dataLength] }
        }

        pub fn frame_to_slice<'a>(&self) -> Result<CompressionData<'a>, AVFError> {
            let pipe_map = &PIPE_MAP.get(&self.index); // why rust
            let pipe_recv = match pipe_map {
                Some(pipe) => &pipe.value().1,
                None => return Err(AVFError::ReadFrame("Data Pipe None".to_string())),
            };
            let data = match pipe_recv.drain().last() {
                Some(frame) => frame,
                None => match pipe_recv.recv() {
                    Ok(f) => f,
                    Err(why) => {
                        return Err(AVFError::ReadFrame(format!(
                            "Failed to read frame from pipe: {}",
                            why
                        )))
                    }
                },
            };
            Ok(data)
        }

        pub fn frame_to_slice_no_block<'a>(&self) -> Result<CompressionData<'a>, AVFError> {
            let pipe_map = &PIPE_MAP.get(&self.index); // why rust
            let pipe_recv = match pipe_map {
                Some(pipe) => &pipe.value().1,
                None => return Err(AVFError::ReadFrame("Data Pipe None".to_string())),
            };
            let data = match pipe_recv.drain().last() {
                Some(frame) => frame,
                None => {
                    return Err(AVFError::ReadFrame(
                        "Failed to read frame from pipe: None".to_string(),
                    ))
                }
            };
            Ok(data)
        }

        pub fn inner(&self) -> *mut Object {
            self.delegate
        }
    }

    impl Drop for AVCaptureVideoCallback {
        fn drop(&mut self) {
            unsafe {
                let _: () = msg_send![self.delegate, autorelease];
            }
        }
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

    pub type AVVideoResolution = CMVideoDimensions;

    #[derive(Copy, Clone, Debug)]
    pub struct CaptureDeviceFormatDescriptor {
        pub resolution: AVVideoResolution,
        pub fps: u32,
        pub fourcc: AVFourCC,
    }

    impl CaptureDeviceFormatDescriptor {
        pub fn compatible_with_capture_format(&self, other: &AVCaptureDeviceFormat) -> bool {
            for fps in &other.fps_list {
                if self.resolution.height == other.resolution.height
                    && self.resolution.width == other.resolution.width
                    && self.fourcc == other.fourcc
                    && (*fps as u32) == self.fps
                {
                    return true;
                }
            }
            false
        }
    }

    #[derive(Debug)]
    pub struct AVCaptureDeviceFormat {
        pub(crate) internal: *mut Object,
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
            .flat_map(|v| [v.min(), v.max()])
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
                kCMVideoCodecType_422YpCbCr8 | kCMPixelFormat_422YpCbCr8_yuvs => AVFourCC::YUV2,
                kCMVideoCodecType_JPEG | kCMVideoCodecType_JPEG_OpenDML => AVFourCC::MJPEG,
                kCMPixelFormat_8IndexedGray_WhiteIsZero => AVFourCC::GRAY8,
                _ => {
                    return Err(AVFError::InvalidValue {
                        found: fcc_raw.to_string(),
                    })
                }
            };
            let _: *mut c_void = unsafe { msg_send![description_obj, autorelease] };
            Ok(AVCaptureDeviceFormat {
                internal: value,
                resolution,
                fps_list,
                fourcc,
            })
        }
    }

    impl Drop for AVCaptureDeviceFormat {
        fn drop(&mut self) {
            unsafe { msg_send![self.internal, autorelease] }
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
                msg_send![discovery_session_cls, discoverySessionWithDeviceTypes:device_types mediaType:media_type position:position]
            };
            Ok(AVCaptureDeviceDiscoverySession {
                inner: discovery_session,
            })
        }

        pub fn default() -> Result<Self, AVFError> {
            AVCaptureDeviceDiscoverySession::new(
                vec![
                    AVCaptureDeviceType::UltraWide,
                    AVCaptureDeviceType::Telephoto,
                    AVCaptureDeviceType::ExternalUnknown,
                    AVCaptureDeviceType::Dual,
                    AVCaptureDeviceType::DualWide,
                    AVCaptureDeviceType::Triple,
                ],
                AVMediaType::Video,
                AVCaptureDevicePosition::Unspecified,
            )
        }

        pub fn devices(&self) -> Vec<AVCaptureDeviceDescriptor> {
            let device_ns_array: *mut Object = unsafe { msg_send![self.inner, devices] };
            let objects_len: NSUInteger = unsafe { NSArray::count(device_ns_array) };
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
            let _: *mut c_void = unsafe { msg_send![device_ns_array, release] };
            devices
        }
    }

    impl Drop for AVCaptureDeviceDiscoverySession {
        fn drop(&mut self) {
            unsafe { msg_send![self.inner, autorelease] }
        }
    }

    impl AVCaptureDevice {
        pub fn devices_with_type(_: AVMediaType) -> Vec<AVCaptureDevice> {
            let cls = class!(AVCaptureDevice);
            let video_type = unsafe { AVMediaTypeVideo.clone() };
            let devices: *mut Object = unsafe { msg_send![cls, devicesWithMediaType: video_type] };
            ns_arr_to_vec(devices)
        }

        pub fn new(index: usize) -> Result<Self, AVFError> {
            let devices = AVCaptureDeviceDiscoverySession::new(
                vec![
                    AVCaptureDeviceType::UltraWide,
                    AVCaptureDeviceType::Telephoto,
                    AVCaptureDeviceType::ExternalUnknown,
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
                unsafe { msg_send![avfoundation_capture_cls, deviceWithUniqueID: nsstr_id] };
            Ok(AVCaptureDevice { inner: capture })
        }

        pub fn supported_formats(&self) -> Result<Vec<AVCaptureDeviceFormat>, AVFError> {
            try_ns_arr_to_vec::<AVCaptureDeviceFormat, AVFError>(unsafe {
                msg_send![self.inner, formats]
            })
        }

        pub fn already_in_use(&self) -> bool {
            unsafe {
                let result: BOOL = msg_send![self.inner(), isInUseByAnotherApplication];
                result == YES
            }
        }

        pub fn is_suspended(&self) -> bool {
            unsafe { msg_send![self.inner, isSuspended] }
        }

        pub fn lock(&self) -> Result<(), AVFError> {
            if self.already_in_use() {
                return Err(AVFError::AlreadyBusy("In Use".to_string()));
            }
            let err_ptr: *mut c_void = std::ptr::null_mut();
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

        pub fn unlock(&self) {
            unsafe { msg_send![self.inner, unlockForConfiguration] }
        }

        // thank you ffmpeg
        pub fn set_all(
            &mut self,
            descriptor: CaptureDeviceFormatDescriptor,
        ) -> Result<(), AVFError> {
            let format_list = self.supported_formats()?;
            let format_description_sel = sel!(formatDescription);

            let mut selected_format: *mut Object = std::ptr::null_mut();
            let mut selected_range: *mut Object = std::ptr::null_mut();

            for format in format_list {
                let format_desc_ref: CMFormatDescriptionRef =
                    unsafe { msg_send![format.internal, performSelector: format_description_sel] };
                let dimensions = unsafe { CMVideoFormatDescriptionGetDimensions(format_desc_ref) };

                if dimensions.height == descriptor.resolution.height
                    && dimensions.width == descriptor.resolution.width
                {
                    selected_format = format.internal;

                    for range in ns_arr_to_vec::<AVFrameRateRange>(unsafe {
                        msg_send![format.internal, valueForKey:"videoSupportedFrameRateRanges"]
                    }) {
                        let max_fps: f64 =
                            unsafe { msg_send![range.inner, valueForKey:"maxFrameRate"] };

                        if (f64::from(descriptor.fps) - max_fps).abs() < 0.01 {
                            selected_range = range.inner;
                            break;
                        }
                    }
                }
            }

            if selected_range.is_null() || selected_format.is_null() {
                return Err(AVFError::ConfigNotAccepted);
            }

            self.lock()?;
            let _: () =
                unsafe { msg_send![self.inner, setValue:selected_format forKey:"activeFormat"] };
            let min_frame_duration: *mut Object =
                unsafe { msg_send![selected_range, valueForKey:"minFrameDuration"] };
            let _: () = unsafe {
                msg_send![self.inner, setValue:min_frame_duration forKey:"activeVideoMinFrameDuration"]
            };
            let _: () = unsafe {
                msg_send![self.inner, setValue:min_frame_duration forKey:"activeVideoMaxFrameDuration"]
            };
            Ok(())
        }
    }

    impl Drop for AVCaptureDevice {
        fn drop(&mut self) {
            unsafe {
                let _: () = msg_send![self.inner, release];
            }
        }
    }

    impl AVCaptureDeviceInput {
        pub fn new(capture_device: &AVCaptureDevice) -> Result<Self, AVFError> {
            let cls = class!(AVCaptureDeviceInput);
            let err_ptr: *mut c_void = std::ptr::null_mut();
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

    impl Drop for AVCaptureDeviceInput {
        fn drop(&mut self) {
            unsafe { msg_send![self.inner, autorelease] }
        }
    }

    pub struct AVCaptureVideoDataOutput {
        inner: *mut Object,
    }

    impl AVCaptureVideoDataOutput {
        pub fn new() -> Self {
            AVCaptureVideoDataOutput::default()
        }

        pub fn add_delegate(&self, delegate: &AVCaptureVideoCallback) -> Result<(), AVFError> {
            unsafe {
                let avf_queue_str = match CString::new("avf_queue") {
                    Ok(avf) => avf.into_raw(),
                    Err(_) => {
                        // should not happen
                        return Err(AVFError::StreamOpen("String contains null? This is a bug, please report it: https://github.com/l1npengtul/nokhwa".to_string()));
                    }
                };
                let queue = dispatch_queue_create(avf_queue_str, NSObject(std::ptr::null_mut()));

                let _: () = msg_send![
                    self.inner,
                    setSampleBufferDelegate: delegate.delegate
                    queue: queue
                ];
            };
            Ok(())
        }
    }

    impl Default for AVCaptureVideoDataOutput {
        fn default() -> Self {
            let cls = class!(AVCaptureVideoDataOutput);
            let inner: *mut Object = unsafe { msg_send![cls, new] };

            AVCaptureVideoDataOutput { inner }
        }
    }

    impl Drop for AVCaptureVideoDataOutput {
        fn drop(&mut self) {
            unsafe { msg_send![self.inner, autorelease] }
        }
    }

    impl AVCaptureSession {
        pub fn new() -> Self {
            AVCaptureSession::default()
        }

        pub fn begin_configuration(&self) {
            unsafe { msg_send![self.inner, beginConfiguration] }
        }

        pub fn commit_configuration(&self) {
            unsafe { msg_send![self.inner, commitConfiguration] }
        }

        pub fn can_add_input(&self, input: &AVCaptureDeviceInput) -> bool {
            let result: BOOL = unsafe { msg_send![self.inner, canAddInput:input.inner] };
            result == YES
        }

        pub fn add_input(&self, input: &AVCaptureDeviceInput) -> Result<(), AVFError> {
            if self.can_add_input(input) {
                let _: () = unsafe { msg_send![self.inner, addInput:input.inner] };
                return Ok(());
            }
            Err(AVFError::RejectedInput)
        }

        pub fn remove_input(&self, input: &AVCaptureDeviceInput) {
            unsafe { msg_send![self.inner, removeInput:input.inner] }
        }

        pub fn can_add_output(&self, output: &AVCaptureVideoDataOutput) -> bool {
            let result: BOOL = unsafe { msg_send![self.inner, canAddOutput:output.inner] };
            result == YES
        }

        pub fn add_output(&self, output: &AVCaptureVideoDataOutput) -> Result<(), AVFError> {
            if self.can_add_output(output) {
                let _: () = unsafe { msg_send![self.inner, addOutput:output.inner] };
                return Ok(());
            }
            Err(AVFError::RejectedInput)
        }

        pub fn remove_output(&self, output: &AVCaptureVideoDataOutput) {
            unsafe { msg_send![self.inner, removeOutput:output.inner] }
        }

        pub fn is_running(&self) -> bool {
            let running: BOOL = unsafe { msg_send![self.inner, isRunning] };
            running == YES
        }

        pub fn start(&self) -> Result<(), AVFError> {
            let start_stream_fn = || {
                let _: () = unsafe { msg_send![self.inner, startRunning] };
            };

            if std::panic::catch_unwind(start_stream_fn).is_err() {
                return Err(AVFError::StreamOpen(
                    "Cannot run AVCaptureSession".to_string(),
                ));
            }
            Ok(())
        }

        pub fn stop(&self) {
            unsafe { msg_send![self.inner, stopRunning] }
        }

        pub fn is_interrupted(&self) -> bool {
            let interrupted: BOOL = unsafe { msg_send![self.inner, isInterrupted] };
            interrupted == YES
        }
    }

    impl Drop for AVCaptureSession {
        fn drop(&mut self) {
            self.stop();
            unsafe { msg_send![self.inner, autorelease] }
        }
    }

    impl Default for AVCaptureSession {
        fn default() -> Self {
            let cls = class!(AVCaptureSession);
            let session: *mut Object = {
                let alloc: *mut Object = unsafe { msg_send![cls, alloc] };
                unsafe { msg_send![alloc, init] }
            };
            AVCaptureSession { inner: session }
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub mod avfoundation {
    use crate::AVFError;
    use flume::{Receiver, Sender};
    use std::borrow::Cow;

    pub type CompressionData<'a> = (Cow<'a, [u8]>, AVFourCC);
    pub type DataPipe<'a> = (Sender<CompressionData<'a>>, Receiver<CompressionData<'a>>);

    pub fn request_permission_with_callback(_: fn(bool)) {}

    pub fn current_authorization_status() -> AVAuthorizationStatus {
        AVAuthorizationStatus::NotDetermined
    }

    // fuck it, use deprecated APIs
    pub fn query_avfoundation() -> Result<Vec<AVCaptureDeviceDescriptor>, AVFError> {
        Err(AVFError::NotSupported)
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
        YUV2,
        MJPEG,
        GRAY8,
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

    pub struct AVCaptureVideoCallback {}

    impl AVCaptureVideoCallback {
        pub fn new(_: usize) -> Self {
            AVCaptureVideoCallback {}
        }

        pub fn index(&self) -> usize {
            0
        }

        pub fn data_len(&self) -> usize {
            0
        }

        pub fn frame_to_slice<'a>(&self) -> Result<CompressionData<'a>, AVFError> {
            Err(AVFError::NotSupported)
        }

        pub fn frame_to_slice_no_block<'a>(&self) -> Result<CompressionData<'a>, AVFError> {
            Err(AVFError::NotSupported)
        }
    }

    pub struct AVFrameRateRange {}
    pub struct AVCaptureDeviceDiscoverySession {}
    pub struct AVCaptureDevice {}
    pub struct AVCaptureDeviceInput {}
    pub struct AVCaptureSession {}

    impl AVFrameRateRange {
        pub fn max(&self) -> f64 {
            0_f64
        }

        pub fn min(&self) -> f64 {
            0_f64
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct CMVideoDimensions {
        pub width: i32,
        pub height: i32,
    }

    pub type AVVideoResolution = CMVideoDimensions;

    #[derive(Copy, Clone, Debug)]
    pub struct CaptureDeviceFormatDescriptor {
        pub resolution: AVVideoResolution,
        pub fps: u32,
        pub fourcc: AVFourCC,
    }

    impl CaptureDeviceFormatDescriptor {
        pub fn compatible_with_capture_format(&self, other: &AVCaptureDeviceFormat) -> bool {
            for fps in &other.fps_list {
                if self.resolution.height == other.resolution.height
                    && self.resolution.width == other.resolution.width
                    && self.fourcc == other.fourcc
                    && (*fps as u32) == self.fps
                {
                    return true;
                }
            }
            false
        }
    }

    #[derive(Debug)]
    pub struct AVCaptureDeviceFormat {
        pub resolution: CMVideoDimensions,
        pub fps_list: Vec<f64>,
        pub fourcc: AVFourCC,
    }

    impl AVCaptureDeviceDiscoverySession {
        pub fn new(
            _: Vec<AVCaptureDeviceType>,
            _: AVMediaType,
            _: AVCaptureDevicePosition,
        ) -> Result<Self, AVFError> {
            Err(AVFError::NotSupported)
        }

        pub fn default() -> Result<Self, AVFError> {
            AVCaptureDeviceDiscoverySession::new(
                vec![
                    AVCaptureDeviceType::UltraWide,
                    AVCaptureDeviceType::Telephoto,
                    AVCaptureDeviceType::ExternalUnknown,
                    AVCaptureDeviceType::Dual,
                    AVCaptureDeviceType::DualWide,
                    AVCaptureDeviceType::Triple,
                ],
                AVMediaType::Video,
                AVCaptureDevicePosition::Unspecified,
            )
        }

        pub fn devices(&self) -> Vec<AVCaptureDeviceDescriptor> {
            vec![]
        }
    }

    impl AVCaptureDevice {
        pub fn devices_with_type(_: AVMediaType) -> Vec<AVCaptureDevice> {
            vec![]
        }

        pub fn new(_: usize) -> Result<Self, AVFError> {
            Err(AVFError::NotSupported)
        }

        pub fn from_id(_: &str) -> Result<Self, AVFError> {
            Err(AVFError::NotSupported)
        }

        pub fn supported_formats(&self) -> Result<Vec<AVCaptureDeviceFormat>, AVFError> {
            Err(AVFError::NotSupported)
        }

        pub fn already_in_use(&self) -> bool {
            false
        }

        pub fn is_suspended(&self) -> bool {
            false
        }

        pub fn lock(&self) -> Result<(), AVFError> {
            Err(AVFError::NotSupported)
        }

        pub fn unlock(&self) {}

        pub fn set_frame_rate(&mut self, _: u32) {}

        pub fn set_all(&mut self, _: CaptureDeviceFormatDescriptor) -> Result<(), AVFError> {
            Err(AVFError::NotSupported)
        }
    }

    impl AVCaptureDeviceInput {
        pub fn new(_: &AVCaptureDevice) -> Result<Self, AVFError> {
            Err(AVFError::NotSupported)
        }
    }

    pub struct AVCaptureVideoDataOutput {}

    impl AVCaptureVideoDataOutput {
        pub fn new() -> Self {
            AVCaptureVideoDataOutput::default()
        }

        pub fn add_delegate(&self, _: &AVCaptureVideoCallback) -> Result<(), AVFError> {
            Err(AVFError::NotSupported)
        }
    }

    impl Default for AVCaptureVideoDataOutput {
        fn default() -> Self {
            AVCaptureVideoDataOutput {}
        }
    }

    impl AVCaptureSession {
        pub fn new() -> Self {
            AVCaptureSession::default()
        }

        pub fn begin_configuration(&self) {}

        pub fn commit_configuration(&self) {}

        pub fn can_add_input(&self, _: &AVCaptureDeviceInput) -> bool {
            false
        }

        pub fn add_input(&self, _: &AVCaptureDeviceInput) -> Result<(), AVFError> {
            Err(AVFError::NotSupported)
        }

        pub fn remove_input(&self, _: &AVCaptureDeviceInput) {}

        pub fn can_add_output(&self, _: &AVCaptureVideoDataOutput) -> bool {
            false
        }

        pub fn add_output(&self, _: &AVCaptureVideoDataOutput) -> Result<(), AVFError> {
            Err(AVFError::NotSupported)
        }

        pub fn remove_output(&self, _: &AVCaptureVideoDataOutput) {}

        pub fn is_running(&self) -> bool {
            false
        }

        pub fn start(&self) -> Result<(), AVFError> {
            Err(AVFError::NotSupported)
        }

        pub fn stop(&self) {}

        pub fn is_interrupted(&self) -> bool {
            false
        }
    }

    impl Default for AVCaptureSession {
        fn default() -> Self {
            AVCaptureSession {}
        }
    }
}
