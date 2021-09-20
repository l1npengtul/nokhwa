/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[cfg(all(feature = "input-v4l", target_os = "linux"))]
mod v4l2;
#[cfg(all(feature = "input-v4l", target_os = "linux"))]
pub use v4l2::V4LCaptureDevice;
#[cfg(all(feature = "input-msmf", target_os = "windows"))]
mod msmf;
#[cfg(all(feature = "input-msmf", target_os = "windows"))]
pub use msmf::MediaFoundationCaptureDevice;
#[cfg(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
))]
mod avfoundation;
#[cfg(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
))]
pub use avfoundation::AVFoundationCaptureDevice;
#[cfg(feature = "input-uvc")]
mod uvc_backend;
#[cfg(feature = "input-uvc")]
pub use uvc_backend::UVCCaptureDevice;
#[cfg(feature = "input-gst")]
mod gst_backend;
#[cfg(feature = "input-gst")]
pub use gst_backend::GStreamerCaptureDevice;
#[cfg(feature = "input-opencv")]
mod opencv_backend;
#[cfg(feature = "input-opencv")]
pub use opencv_backend::OpenCvCaptureDevice;
