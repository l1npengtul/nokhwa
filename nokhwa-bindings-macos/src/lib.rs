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

// <some change so we can call this 0.10.4>

#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[macro_use]
extern crate objc;

#[cfg(any(target_os = "macos", target_os = "ios"))]
use std::ffi::c_float;

#[cfg(any(target_os = "macos", target_os = "ios"))]
const UTF8_ENCODING: usize = 4;

#[cfg(any(target_os = "macos", target_os = "ios"))]
type CGFloat = c_float;

/// Shared boilerplate macro for ObjC wrapper structs.
#[cfg(any(target_os = "macos", target_os = "ios"))]
macro_rules! create_boilerplate_impl {
    {
        $( [$class_vis:vis $class_name:ident : $( {$field_vis:vis $field_name:ident : $field_type:ty} ),*] ),+
    } => {
        $(
            $class_vis struct $class_name {
                inner: *mut objc::runtime::Object,
                $(
                    $field_vis $field_name : $field_type
                )*
            }

            impl $class_name {
                pub fn inner(&self) -> *mut objc::runtime::Object {
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
                pub(crate) inner: *mut objc::runtime::Object,
            }

            impl $class_name {
                pub fn inner(&self) -> *mut objc::runtime::Object {
                    self.inner
                }
            }

            impl From<*mut objc::runtime::Object> for $class_name {
                fn from(obj: *mut objc::runtime::Object) -> Self {
                    $class_name {
                        inner: obj,
                    }
                }
            }
        )+
    };
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[allow(non_snake_case)]
pub mod ffi;

/// Backward-compatible alias for the `ffi` module (previously named `core_media`).
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use ffi as core_media;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) mod format;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) mod callback;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod device;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod session;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use callback::AVCaptureVideoCallback;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use device::{
    AVAuthorizationStatus, AVCaptureDevice, AVCaptureDeviceDiscoverySession,
    AVCaptureDeviceFormat, AVCaptureDevicePosition, AVCaptureDeviceType, AVFrameRateRange,
    AVMediaType, CompressionData, DataPipe, current_authorization_status, get_raw_device_info,
    query_avfoundation, request_permission_with_callback,
};

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use session::{
    AVCaptureDeviceInput, AVCaptureSession, AVCaptureVideoDataOutput,
};
