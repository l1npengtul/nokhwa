// FFI declarations (extern C blocks, framework links, type aliases, constants)

use core_media_sys::{
    CMBlockBufferRef, CMFormatDescriptionRef, CMSampleBufferRef, CMTime, CMVideoDimensions,
    FourCharCode,
};
use objc::{runtime::Object, Message};
use std::ops::Deref;

use crate::CGFloat;

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

    pub fn CVPixelBufferGetDataSize(pixelBuffer: CVPixelBufferRef)
        -> std::os::raw::c_ulong;

    pub fn CVPixelBufferGetBaseAddress(
        pixelBuffer: CVPixelBufferRef,
    ) -> *mut std::os::raw::c_void;

    pub fn CVPixelBufferGetPixelFormatType(pixelBuffer: CVPixelBufferRef) -> OSType;
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CGPoint {
    pub x: CGFloat,
    pub y: CGFloat,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __CVBuffer {
    _unused: [u8; 0],
}

#[allow(non_snake_case)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
#[repr(C)]
pub struct AVCaptureWhiteBalanceGains {
    pub blueGain: f32,
    pub greenGain: f32,
    pub redGain: f32,
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

    pub static AVCaptureLensPositionCurrent: f32;
    pub static AVCaptureExposureTargetBiasCurrent: f32;
    pub static AVCaptureExposureDurationCurrent: CMTime;
    pub static AVCaptureISOCurrent: f32;
}
