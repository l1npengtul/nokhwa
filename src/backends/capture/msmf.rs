use crate::{CameraFormat, NokhwaError};
use nokhwa_bindings_windows::wmf::MediaFoundationDevice;

pub struct MediaFoundationCaptureDevice {
    inner: MediaFoundationDevice,
}

impl MediaFoundationCaptureDevice {
    pub fn new(index: usize, camera_fmt: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        let mf_device = MediaFoundationDevice::new(index)?;
        if let Some(fmt) = camera_fmt {
            mf_device.
        }
        Ok (
            MediaFoundationCaptureDevice { inner: mf_device }
        )
    }


}
