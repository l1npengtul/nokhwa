/*
 * Copyright 2021 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
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

use crate::{
    all_known_camera_controls, mjpeg_to_rgb, yuyv422_to_rgb, CameraControl, CameraFormat,
    CameraIndex, CameraInfo, CaptureAPIBackend, CaptureBackendTrait, FrameFormat,
    KnownCameraControlFlag, KnownCameraControls, NokhwaError, Resolution,
};
use image::{ImageBuffer, Rgb};
use nokhwa_bindings_windows::{wmf::MediaFoundationDevice, MFControl, MediaFoundationControls};
use std::{any::Any, borrow::Cow, collections::HashMap};

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
/// - When you call new or drop the struct, `initialize`/`de_initialize` will automatically be called.
// TODO: Allow CameraIndex to contain a device string.
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-msmf")))]
pub struct MediaFoundationCaptureDevice<'a> {
    inner: MediaFoundationDevice<'a>,
    info: CameraInfo,
}

impl<'a> MediaFoundationCaptureDevice<'a> {
    /// Creates a new capture device using the Media Foundation backend. Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default.
    /// # Errors
    /// This function will error if Media Foundation fails to get the device. This will also error if the index is a [`CameraIndex::String`] that cannot be parsed into a `usize`.
    pub fn new(
        index: &CameraIndex<'a>,
        camera_fmt: Option<CameraFormat>,
    ) -> Result<Self, NokhwaError> {
        let mut mf_device = match &index {
            CameraIndex::Index(idx) => MediaFoundationDevice::new(*idx as usize),
            CameraIndex::String(lnk) => MediaFoundationDevice::with_string(
                &lnk.as_bytes()
                    .into_iter()
                    .map(|x| *x as u16)
                    .collect::<Vec<u16>>(),
            ),
        }?;
        if let Some(fmt) = camera_fmt {
            mf_device.set_format(fmt.into())?;
        }

        let info = CameraInfo::new(
            mf_device.name(),
            "MediaFoundation Camera Device".to_string(),
            mf_device.symlink(),
            index,
        );

        Ok(MediaFoundationCaptureDevice {
            inner: mf_device,
            info,
        })
    }

    /// Create a new Media Foundation Device with desired settings.
    /// # Errors
    /// This function will error if Media Foundation fails to get the device. This will also error if the index is a [`CameraIndex::String`] that cannot be parsed into a `usize`.
    pub fn new_with(
        index: &CameraIndex<'a>,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
    ) -> Result<Self, NokhwaError> {
        let camera_format = Some(CameraFormat::new_from(width, height, fourcc, fps));
        MediaFoundationCaptureDevice::new(index, camera_format)
    }
}

impl<'a> CaptureBackendTrait for MediaFoundationCaptureDevice<'a> {
    fn backend(&self) -> CaptureAPIBackend {
        CaptureAPIBackend::MediaFoundation
    }

    fn camera_info(&self) -> &CameraInfo {
        &self.info
    }

    fn camera_format(&self) -> CameraFormat {
        self.inner.format().into()
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        if let Err(why) = self.inner.set_format(new_fmt.into()) {
            return Err(why.into());
        }
        Ok(())
    }

    fn compatible_list_by_resolution(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        let mf_camera_format_list = self.inner.compatible_format_list()?;
        let mut resolution_map: HashMap<Resolution, Vec<u32>> = HashMap::new();

        for mf_camera_format in mf_camera_format_list {
            let camera_format: CameraFormat = mf_camera_format.into();

            // check fcc
            if camera_format.format() != fourcc {
                continue;
            }

            match resolution_map.get_mut(&camera_format.resolution()) {
                Some(fps_list) => {
                    fps_list.push(camera_format.frame_rate());
                }
                None => {
                    if let Some(mut wtf_why_we_here_list) = resolution_map
                        .insert(camera_format.resolution(), vec![camera_format.frame_rate()])
                    {
                        wtf_why_we_here_list.push(camera_format.frame_rate());
                        resolution_map.insert(camera_format.resolution(), wtf_why_we_here_list);
                    }
                }
            }
        }
        Ok(resolution_map)
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        let mf_camera_format_list = self.inner.compatible_format_list()?;
        let mut frame_format_list = vec![];

        for mf_camera_format in mf_camera_format_list {
            let camera_format: CameraFormat = mf_camera_format.into();

            if !frame_format_list.contains(&camera_format.format()) {
                frame_format_list.push(camera_format.format());
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
        self.camera_format().frame_rate()
    }

    fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        let mut new_format = self.camera_format();
        new_format.set_frame_rate(new_fps);
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
        let mut supported_camera_controls: Vec<KnownCameraControls> = vec![];

        for camera_control in all_known_camera_controls() {
            let msmf_camera_control: MediaFoundationControls = match camera_control {
                KnownCameraControls::Brightness => MediaFoundationControls::Brightness,
                KnownCameraControls::Contrast => MediaFoundationControls::Contrast,
                KnownCameraControls::Hue => MediaFoundationControls::Hue,
                KnownCameraControls::Saturation => MediaFoundationControls::Saturation,
                KnownCameraControls::Sharpness => MediaFoundationControls::Sharpness,
                KnownCameraControls::Gamma => MediaFoundationControls::Gamma,
                KnownCameraControls::ColorEnable => MediaFoundationControls::ColorEnable,
                KnownCameraControls::WhiteBalance => MediaFoundationControls::WhiteBalance,
                KnownCameraControls::BacklightComp => MediaFoundationControls::BacklightComp,
                KnownCameraControls::Gain => MediaFoundationControls::Gain,
                KnownCameraControls::Pan => MediaFoundationControls::Pan,
                KnownCameraControls::Tilt => MediaFoundationControls::Tilt,
                KnownCameraControls::Roll => MediaFoundationControls::Roll,
                KnownCameraControls::Zoom => MediaFoundationControls::Zoom,
                KnownCameraControls::Exposure => MediaFoundationControls::Exposure,
                KnownCameraControls::Iris => MediaFoundationControls::Iris,
                KnownCameraControls::Focus => MediaFoundationControls::Focus,
            };

            if let Ok(supported) = self.inner.control(msmf_camera_control) {
                supported_camera_controls.push(supported.control().into());
            }
        }

        Ok(supported_camera_controls)
    }

    fn camera_control(&self, control: KnownCameraControls) -> Result<CameraControl, NokhwaError> {
        let msmf_camera_control: MediaFoundationControls = match control {
            KnownCameraControls::Brightness => MediaFoundationControls::Brightness,
            KnownCameraControls::Contrast => MediaFoundationControls::Contrast,
            KnownCameraControls::Hue => MediaFoundationControls::Hue,
            KnownCameraControls::Saturation => MediaFoundationControls::Saturation,
            KnownCameraControls::Sharpness => MediaFoundationControls::Sharpness,
            KnownCameraControls::Gamma => MediaFoundationControls::Gamma,
            KnownCameraControls::ColorEnable => MediaFoundationControls::ColorEnable,
            KnownCameraControls::WhiteBalance => MediaFoundationControls::WhiteBalance,
            KnownCameraControls::BacklightComp => MediaFoundationControls::BacklightComp,
            KnownCameraControls::Gain => MediaFoundationControls::Gain,
            KnownCameraControls::Pan => MediaFoundationControls::Pan,
            KnownCameraControls::Tilt => MediaFoundationControls::Tilt,
            KnownCameraControls::Roll => MediaFoundationControls::Roll,
            KnownCameraControls::Zoom => MediaFoundationControls::Zoom,
            KnownCameraControls::Exposure => MediaFoundationControls::Exposure,
            KnownCameraControls::Iris => MediaFoundationControls::Iris,
            KnownCameraControls::Focus => MediaFoundationControls::Focus,
        };

        let ctrl = match self.inner.control(msmf_camera_control) {
            Ok(ctrl) => ctrl,
            Err(why) => return Err(why.into()),
        };

        let flag = if ctrl.manual() {
            KnownCameraControlFlag::Manual
        } else {
            KnownCameraControlFlag::Automatic
        };

        let min = MFControl::min(&ctrl);
        let max = MFControl::max(&ctrl);

        CameraControl::new(
            control,
            min,
            max,
            ctrl.current(),
            ctrl.step(),
            ctrl.default(),
            flag,
            ctrl.active(),
        )
    }

    fn set_camera_control(&mut self, control: CameraControl) -> Result<(), NokhwaError> {
        let ctrl = match control.control() {
            KnownCameraControls::Brightness => MediaFoundationControls::Brightness,
            KnownCameraControls::Contrast => MediaFoundationControls::Contrast,
            KnownCameraControls::Hue => MediaFoundationControls::Hue,
            KnownCameraControls::Saturation => MediaFoundationControls::Saturation,
            KnownCameraControls::Sharpness => MediaFoundationControls::Sharpness,
            KnownCameraControls::Gamma => MediaFoundationControls::Gamma,
            KnownCameraControls::ColorEnable => MediaFoundationControls::ColorEnable,
            KnownCameraControls::WhiteBalance => MediaFoundationControls::WhiteBalance,
            KnownCameraControls::BacklightComp => MediaFoundationControls::BacklightComp,
            KnownCameraControls::Gain => MediaFoundationControls::Gain,
            KnownCameraControls::Pan => MediaFoundationControls::Pan,
            KnownCameraControls::Tilt => MediaFoundationControls::Tilt,
            KnownCameraControls::Roll => MediaFoundationControls::Roll,
            KnownCameraControls::Zoom => MediaFoundationControls::Zoom,
            KnownCameraControls::Exposure => MediaFoundationControls::Exposure,
            KnownCameraControls::Iris => MediaFoundationControls::Iris,
            KnownCameraControls::Focus => MediaFoundationControls::Focus,
        };

        let flag = match control.flag() {
            KnownCameraControlFlag::Automatic => false,
            KnownCameraControlFlag::Manual => true,
        };

        let msmf_camera_control = MFControl::new(
            ctrl,
            control.minimum_value(),
            control.maximum_value(),
            control.step(),
            control.value(),
            control.default(),
            flag,
            control.active(),
        );

        if let Err(why) = self.inner.set_control(msmf_camera_control) {
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
        if let Err(why) = self.inner.start_stream() {
            return Err(why.into());
        }

        Ok(())
    }

    fn is_stream_open(&self) -> bool {
        self.inner.is_stream_open()
    }

    fn frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        let camera_format = self.camera_format();
        let raw_data = self.frame_raw()?;
        let conv = match camera_format.format() {
            FrameFormat::MJPEG => mjpeg_to_rgb(raw_data.as_ref(), false)?,
            FrameFormat::YUYV => yuyv422_to_rgb(raw_data.as_ref(), false)?,
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
        match self.inner.raw_bytes() {
            Ok(data) => Ok(data),
            Err(why) => Err(why.into()),
        }
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        self.inner.stop_stream();
        Ok(())
    }
}
