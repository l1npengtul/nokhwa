//! Format conversion helpers.

use cocoa_foundation::foundation::{NSArray, NSString};
use core_media_sys::{
    kCMPixelFormat_24RGB, kCMPixelFormat_422YpCbCr8_yuvs,
    kCMPixelFormat_8IndexedGray_WhiteIsZero, kCMVideoCodecType_422YpCbCr8,
    kCMVideoCodecType_JPEG, kCMVideoCodecType_JPEG_OpenDML,
};
use core_video_sys::{
    kCVPixelFormatType_420YpCbCr10BiPlanarVideoRange,
    kCVPixelFormatType_420YpCbCr8BiPlanarFullRange,
    kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
};
use nokhwa_core::types::FrameFormat;
use objc::runtime::objc_getClass;
use objc::runtime::{Object, BOOL, YES};
use std::{
    borrow::Cow,
    convert::TryFrom,
    error::Error,
    ffi::{c_void, CStr, CString},
};

use crate::ffi::{self, OSType};
use crate::UTF8_ENCODING;

pub(crate) fn str_to_nsstr(string: &str) -> *mut Object {
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
        obj
    }
}

pub(crate) fn nsstr_to_str<'a>(nsstr: *mut Object) -> Cow<'a, str> {
    let data = unsafe { CStr::from_ptr(nsstr.UTF8String()) };
    data.to_string_lossy()
}

pub(crate) fn vec_to_ns_arr<T: Into<*mut Object>>(data: Vec<T>) -> *mut Object {
    let cstr = CString::new("NSMutableArray").unwrap();
    let ns_arr_cls = unsafe { objc_getClass(cstr.as_ptr()) };
    let mutable_array: *mut Object = unsafe { msg_send![ns_arr_cls, array] };
    data.into_iter().for_each(|item| {
        let item_obj: *mut Object = item.into();
        let _: () = unsafe { msg_send![mutable_array, addObject: item_obj] };
    });
    mutable_array
}

pub(crate) fn ns_arr_to_vec<T: From<*mut Object>>(data: *mut Object) -> Vec<T> {
    let length = unsafe { NSArray::count(data) };

    let mut out_vec: Vec<T> = Vec::with_capacity(length as usize);
    for index in 0..length {
        let item = unsafe { NSArray::objectAtIndex(data, index) };
        out_vec.push(T::from(item));
    }
    out_vec
}

pub(crate) fn try_ns_arr_to_vec<T, TE>(data: *mut Object) -> Result<Vec<T>, TE>
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
    Ok(out_vec)
}

pub(crate) fn compare_ns_string(this: *mut Object, other: ffi::NSString) -> bool {
    unsafe {
        let equal: BOOL = msg_send![this, isEqualToString: other];
        equal == YES
    }
}

#[allow(non_upper_case_globals)]
pub(crate) fn raw_fcc_to_frameformat(raw: OSType) -> Option<FrameFormat> {
    match raw {
        kCMVideoCodecType_422YpCbCr8 | kCMPixelFormat_422YpCbCr8_yuvs => {
            Some(FrameFormat::YUYV)
        }
        kCMVideoCodecType_JPEG | kCMVideoCodecType_JPEG_OpenDML => Some(FrameFormat::MJPEG),
        kCMPixelFormat_8IndexedGray_WhiteIsZero => Some(FrameFormat::GRAY),
        kCVPixelFormatType_420YpCbCr10BiPlanarVideoRange
        | kCVPixelFormatType_420YpCbCr8BiPlanarFullRange
        | kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange => Some(FrameFormat::YUYV),
        kCMPixelFormat_24RGB => Some(FrameFormat::RAWRGB),
        _ => None,
    }
}
