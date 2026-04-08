//! AVCaptureDeviceInput, AVCaptureVideoDataOutput, and AVCaptureSession.

use cocoa_foundation::{
    base::nil,
    foundation::{NSDictionary, NSInteger},
};
use core_foundation::base::TCFType;
use core_foundation::number::CFNumber;
use core_media_sys::{
    kCMPixelFormat_422YpCbCr8_yuvs, kCMPixelFormat_24RGB,
    kCMPixelFormat_8IndexedGray_WhiteIsZero, kCMVideoCodecType_JPEG,
};
use core_video_sys::{
    kCVPixelBufferPixelFormatTypeKey, kCVPixelFormatType_420YpCbCr10BiPlanarVideoRange,
};
use nokhwa_core::{
    error::NokhwaError,
    types::{ApiBackend, FrameFormat},
};
use objc::runtime::{Object, BOOL, YES};
use std::ffi::c_void;

use crate::callback::AVCaptureVideoCallback;
use crate::device::AVCaptureDevice;

create_boilerplate_impl! {
    [pub AVCaptureDeviceInput],
    [pub AVCaptureSession]
}

impl AVCaptureDeviceInput {
    pub fn new(capture_device: &AVCaptureDevice) -> Result<Self, NokhwaError> {
        let cls = class!(AVCaptureDeviceInput);
        let err_ptr: *mut c_void = std::ptr::null_mut();
        let capture_input: *mut Object = unsafe {
            let allocated: *mut Object = msg_send![cls, alloc];
            msg_send![allocated, initWithDevice:capture_device.inner() error:err_ptr]
        };
        if !err_ptr.is_null() {
            return Err(NokhwaError::InitializeError {
                backend: ApiBackend::AVFoundation,
                error: "Failed to create input".to_string(),
            });
        }

        Ok(AVCaptureDeviceInput {
            inner: capture_input,
        })
    }
}

pub struct AVCaptureVideoDataOutput {
    inner: *mut Object,
}

impl AVCaptureVideoDataOutput {
    pub fn new() -> Self {
        AVCaptureVideoDataOutput::default()
    }

    pub fn add_delegate(&self, delegate: &AVCaptureVideoCallback) -> Result<(), NokhwaError> {
        unsafe {
            let _: () = msg_send![
                self.inner,
                setSampleBufferDelegate: delegate.delegate
                queue: delegate.queue().0
            ];
        };
        Ok(())
    }

    pub fn set_frame_format(&self, format: FrameFormat) -> Result<(), NokhwaError> {
        let cmpixelfmt = match format {
            FrameFormat::YUYV => kCMPixelFormat_422YpCbCr8_yuvs,
            FrameFormat::MJPEG => kCMVideoCodecType_JPEG,
            FrameFormat::GRAY => kCMPixelFormat_8IndexedGray_WhiteIsZero,
            FrameFormat::NV12 => kCVPixelFormatType_420YpCbCr10BiPlanarVideoRange,
            FrameFormat::RAWRGB => kCMPixelFormat_24RGB,
            FrameFormat::RAWBGR => {
                return Err(NokhwaError::SetPropertyError {
                    property: "setVideoSettings".to_string(),
                    value: "set frame format".to_string(),
                    error: "Unsupported frame format BGR".to_string(),
                });
            }
        };
        let obj = CFNumber::from(cmpixelfmt as i32);
        let obj = obj.as_CFTypeRef() as *mut Object;
        let key = unsafe { kCVPixelBufferPixelFormatTypeKey } as *mut Object;
        let dict = unsafe { NSDictionary::dictionaryWithObject_forKey_(nil, obj, key) };
        let _: () = unsafe { msg_send![self.inner, setVideoSettings:dict] };
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

    pub fn add_input(&self, input: &AVCaptureDeviceInput) -> Result<(), NokhwaError> {
        if self.can_add_input(input) {
            let _: () = unsafe { msg_send![self.inner, addInput:input.inner] };
            return Ok(());
        }
        Err(NokhwaError::SetPropertyError {
            property: "AVCaptureDeviceInput".to_string(),
            value: "add new input".to_string(),
            error: "Rejected".to_string(),
        })
    }

    pub fn remove_input(&self, input: &AVCaptureDeviceInput) {
        unsafe { msg_send![self.inner, removeInput:input.inner] }
    }

    pub fn can_add_output(&self, output: &AVCaptureVideoDataOutput) -> bool {
        let result: BOOL = unsafe { msg_send![self.inner, canAddOutput:output.inner] };
        result == YES
    }

    pub fn add_output(&self, output: &AVCaptureVideoDataOutput) -> Result<(), NokhwaError> {
        if self.can_add_output(output) {
            let _: () = unsafe { msg_send![self.inner, addOutput:output.inner] };
            return Ok(());
        }
        Err(NokhwaError::SetPropertyError {
            property: "AVCaptureVideoDataOutput".to_string(),
            value: "add new output".to_string(),
            error: "Rejected".to_string(),
        })
    }

    pub fn remove_output(&self, output: &AVCaptureVideoDataOutput) {
        unsafe { msg_send![self.inner, removeOutput:output.inner] }
    }

    pub fn is_running(&self) -> bool {
        let running: BOOL = unsafe { msg_send![self.inner, isRunning] };
        running == YES
    }

    pub fn start(&self) -> Result<(), NokhwaError> {
        let start_stream_fn = || {
            let _: () = unsafe { msg_send![self.inner, startRunning] };
        };

        if std::panic::catch_unwind(start_stream_fn).is_err() {
            return Err(NokhwaError::OpenStreamError(
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
