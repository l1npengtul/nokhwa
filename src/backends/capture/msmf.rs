/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
    all_known_camera_controls, mjpeg_to_rgb888, yuyv422_to_rgb888, CameraControl, CameraFormat,
    CameraInfo, CaptureAPIBackend, CaptureBackendTrait, FrameFormat, KnownCameraControls,
    NokhwaError, Resolution,
};
use image::{ImageBuffer, Rgb};
use nokhwa_bindings_windows::{wmf::MediaFoundationDevice, MFControl, MediaFoundationControls};
use std::{any::Any, borrow::Cow, cell::RefCell, collections::HashMap};

/// The backend that deals with Media Foundation on Windows.
/// To see what this does, please see [`CaptureBackendTrait`].
///
/// Note: This requires Windows 7 or newer to work.
/// # Quirks
/// - This does build on non-windows platforms, however when you do the backend will be empty and will return an error for any given operation.
/// - Please check [`nokhwa-bindings-windows`](https://github.com/l1npengtul/nokhwa/tree/senpai/nokhwa-bindings-windows) source code to see the internal raw interface.
/// - [`raw_supported_camera_controls()`](CaptureBackendTrait::raw_supported_camera_controls), [`raw_camera_control()`](CaptureBackendTrait::raw_camera_control), [`set_raw_camera_control()`](CaptureBackendTrait::set_raw_camera_control) is **not** supported.
/// - The symbolic link for the device is listed in the `misc` attribute of the [`CameraInfo`].
/// - The names may contain invalid characters since they were converted from UTF16.
pub struct MediaFoundationCaptureDevice {
    inner: RefCell<MediaFoundationDevice>,
}

impl MediaFoundationCaptureDevice {
    /// Creates a new capture device using the Media Foundation backend. Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default.
    /// # Errors
    /// This function will error if Media Foundation fails to get the device.
    pub fn new(index: usize, camera_fmt: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        let mut mf_device = MediaFoundationDevice::new(index)?;
        if let Some(fmt) = camera_fmt {
            mf_device.set_format(fmt.into())?;
        }
        Ok(MediaFoundationCaptureDevice {
            inner: RefCell::new(mf_device),
        })
    }

    /// Create a new Media Foundation Device with desired settings.
    /// # Errors
    /// This function will error if Media Foundation fails to get the device.
    pub fn new_with(
        index: usize,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
    ) -> Result<Self, NokhwaError> {
        let camera_format = Some(CameraFormat::new_from(width, height, fourcc, fps));
        MediaFoundationCaptureDevice::new(index, camera_format)
    }
}

impl CaptureBackendTrait for MediaFoundationCaptureDevice {
    fn backend(&self) -> CaptureAPIBackend {
        CaptureAPIBackend::MediaFoundation
    }

    fn camera_info(&self) -> CameraInfo {
        let inner_borrow = self.inner.borrow();
        CameraInfo::new(
            inner_borrow.name(),
            "".to_string(),
            inner_borrow.symlink(),
            inner_borrow.index(),
        )
    }

    fn camera_format(&self) -> CameraFormat {
        self.inner.format().into()
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        if let Err(why) = self.inner.borrow_mut().set_format(new_fmt.into()) {
            Err(why.into())
        }
        Ok(())
    }

    fn compatible_list_by_resolution(
        &self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        let inner_borrow = match self.inner.try_borrow_mut() {
            Ok(mut brw) => (&mut brw),
            Err(why) => {
                return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let mf_camera_format_list = inner_borrow.compatible_format_list()?;
        let mut resolution_map = HashMap::new();

        for mf_camera_format in mf_camera_format_list {
            let camera_format: CameraFormat = mf_camera_format.into();

            // check fcc
            if camera_format.format() != fourcc {
                continue;
            }

            match resolution_map.get_mut(&camera_format.resolution()) {
                Some(fps_list) => {
                    fps_list.append(camera_format.framerate());
                }
                None => {
                    if let Some(mut wtf_why_we_here_list) = resolution_map
                        .insert(camera_format.resolution(), vec![camera_format.framerate()])
                    {
                        wtf_why_we_here_list.push(camera_format.framerate());
                        resolution_map.insert(camera_format.resolution(), wtf_why_we_here_list);
                    }
                }
            }
        }
        Ok(resolution_map)
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        let inner_borrow = match self.inner.try_borrow_mut() {
            Ok(mut brw) => (&mut brw),
            Err(why) => {
                return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let mf_camera_format_list = inner_borrow.compatible_format_list()?;
        let mut frame_format_list = vec![];

        for mf_camera_format in mf_camera_format_list {
            let camera_format: CameraFormat = mf_camera_format.into();

            if !frame_format_list.contains(&camera_format.format()) {
                frame_format_list.push(camera_format.format())
            }

            // TODO: Update as we get more frame formats!
            if frame_format_list.len() == 2 {
                break;
            }
        }
        Ok(frame_format_list)
    }

    fn resolution(&self) -> Resolution {
        self.camera_format().resolution()
    }

    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        let mut new_format = self.camera_format();
        new_format.set_resolution(new_res);
        self.set_camera_format(new_format)
    }

    fn frame_rate(&self) -> u32 {
        self.camera_format().framerate()
    }

    fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        let mut new_format = self.camera_format();
        new_format.set_framerate(new_fps);
        self.set_camera_format(new_format)
    }

    fn frame_format(&self) -> FrameFormat {
        self.camera_format().format()
    }

    fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        let mut new_format = self.camera_format();
        new_format.set_format(fourcc);
        self.set_camera_format(new_format)
    }

    fn supported_camera_controls(&self) -> Result<Vec<KnownCameraControls>, NokhwaError> {
        let inner_borrow = match self.inner.try_borrow() {
            Ok(brw) => (&*brw),
            Err(why) => {
                return return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let mut supported_camera_controls: Vec<KnownCameraControls> = vec![];

        for camera_control in all_known_camera_controls() {
            let msmf_camera_control: MediaFoundationControls = camera_control.into();

            if let Ok(supported) = inner_borrow.control(msmf_camera_control) {
                supported_camera_controls.push(supported.control().into());
            }
        }

        Ok(supported_camera_controls)
    }

    fn camera_control(&self, control: KnownCameraControls) -> Result<CameraControl, NokhwaError> {
        let inner_borrow = match self.inner.try_borrow() {
            Ok(brw) => (&*brw),
            Err(why) => {
                return return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let msmf_camera_control: MediaFoundationControls = control.into();

        match inner_borrow.control(msmf_camera_control) {
            Ok(ctrl) => Ok(ctrl.into()),
            Err(why) => Err(why.into()),
        }
    }

    fn set_camera_control(&mut self, control: CameraControl) -> Result<(), NokhwaError> {
        let mut inner_borrow = match self.inner.try_borrow_mut() {
            Ok(brw) => (&mut *brw),
            Err(why) => {
                return return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let msmf_camera_control: MFControl = control.into();

        if let Err(why) = inner_borrow.set_control(msmf_camera_control) {
            return Err(why.into());
        }
        Ok(())
    }

    fn raw_supported_camera_controls(&self) -> Result<Vec<Box<dyn Any>>, NokhwaError> {
        Err(NokhwaError::UnsupportedOperationError(
            CaptureAPIBackend::MediaFoundation,
        ))
    }

    fn raw_camera_control(&self, _control: &dyn Any) -> Result<Box<dyn Any>, NokhwaError> {
        Err(NokhwaError::UnsupportedOperationError(
            CaptureAPIBackend::MediaFoundation,
        ))
    }

    fn set_raw_camera_control(
        &mut self,
        _control: &dyn Any,
        _value: &dyn Any,
    ) -> Result<(), NokhwaError> {
        Err(NokhwaError::UnsupportedOperationError(
            CaptureAPIBackend::MediaFoundation,
        ))
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        let mut inner_borrow = match self.inner.try_borrow_mut() {
            Ok(brw) => (&mut *brw),
            Err(why) => {
                return return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        if let Err(why) = inner_borrow.start_stream() {
            return Err(why.into());
        }

        Ok(())
    }

    fn is_stream_open(&self) -> bool {
        self.inner.borrow().is_stream_open()
    }

    fn frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        let raw_data = self.frame_raw()?;
        let camera_format = self.camera_format();
        let conv = match cam_fmt.format() {
            FrameFormat::MJPEG => mjpeg_to_rgb888(&raw_data)?,
            FrameFormat::YUYV => yuyv422_to_rgb888(&raw_data)?,
        };

        let imagebuf =
            match ImageBuffer::from_vec(camera_format.width(), camera_format.height(), conv) {
                Some(buf) => {
                    let rgbbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = buf;
                    rgbbuf
                }
                None => return Err(NokhwaError::ReadFrameError(
                    "Imagebuffer is not large enough! This is probably a bug, please report it!"
                        .to_string(),
                )),
            };

        Ok(imagebuf)
    }

    fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        let mut inner_borrow = match self.inner.try_borrow_mut() {
            Ok(brw) => (&mut *brw),
            Err(why) => {
                return return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        match inner_borrow.raw_bytes() {
            Ok(data) => Ok(data),
            Err(why) => Err(why.into()),
        }
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        let mut inner_borrow = match self.inner.try_borrow_mut() {
            Ok(brw) => (&mut *brw),
            Err(why) => {
                return return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        Ok(inner_borrow.stop_stream())
    }
}
