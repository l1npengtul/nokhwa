/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[cfg(all(feature = "input-v4l", target_os = "linux"))]
// I'm too lazy to set up a skeleton facade for V4L so here it will stay
mod v4l2;
#[cfg(all(feature = "input-v4l", target_os = "linux"))]
pub use v4l2::V4LCaptureDevice;
#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
mod msmf;
#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
pub use msmf::MediaFoundationCaptureDevice;
#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
))]
mod avfoundation;
#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
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
