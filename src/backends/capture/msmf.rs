/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{CameraFormat, NokhwaError};
use nokhwa_bindings_windows::wmf::MediaFoundationDevice;

pub struct MediaFoundationCaptureDevice {
    inner: MediaFoundationDevice,
}

impl MediaFoundationCaptureDevice {
    pub fn new(index: usize, camera_fmt: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        let mut mf_device = MediaFoundationDevice::new(index)?;
        if let Some(fmt) = camera_fmt {
            mf_device.set_format(fmt.into())?;
        }
        Ok(MediaFoundationCaptureDevice { inner: mf_device })
    }
}
