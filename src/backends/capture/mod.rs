/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[cfg(feature = "input-v4l")]
mod v4l2;
#[cfg(feature = "input-v4l")]
pub use v4l2::V4LCaptureDevice;
#[cfg(feature = "input-uvc")]
mod uvc_backend;
#[cfg(feature = "input-uvc")]
pub use uvc_backend::UVCCaptureDevice;
#[cfg(feature = "input-msmf")]
mod msmf;
#[cfg(feature = "input-msmf")]
pub use msmf::MediaFoundationCaptureDevice;
#[cfg(feature = "input-gst")]
mod gst_backend;
#[cfg(feature = "input-gst")]
pub use gst_backend::GStreamerCaptureDevice;
#[cfg(feature = "input-opencv")]
mod opencv_backend;
#[cfg(feature = "input-opencv")]
pub use opencv_backend::OpenCvCaptureDevice;
