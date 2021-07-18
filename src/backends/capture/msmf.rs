use crate::CameraFormat;
use nokhwa_bindings_windows::wmf::MediaFoundationDevice;

pub struct MediaFoundationCaptureDevice {
    inner: MediaFoundationDevice,
}
