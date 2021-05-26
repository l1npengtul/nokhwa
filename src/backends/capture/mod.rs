#[cfg(feature = "input_v4l")]
mod v4l2;
#[cfg(feature = "input_v4l")]
pub use v4l2::V4LCaptureDevice;
#[cfg(feature = "input_uvc")]
mod uvc_backend;
#[cfg(feature = "input_uvc")]
pub use uvc_backend::UVCCaptureDevice;
