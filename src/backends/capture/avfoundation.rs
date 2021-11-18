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
    mjpeg_to_rgb888, yuyv422_to_rgb888, CameraControl, CameraFormat, CameraInfo, CaptureAPIBackend,
    CaptureBackendTrait, FrameFormat, KnownCameraControls, NokhwaError, Resolution,
};
use image::{ImageBuffer, Rgb};
use nokhwa_bindings_macos::avfoundation::{
    query_avfoundation, AVCaptureDevice, AVCaptureDeviceInput, AVCaptureSession,
    AVCaptureVideoCallback, AVCaptureVideoDataOutput, AVFourCC,
};
use std::{any::Any, borrow::Cow, collections::HashMap};

/// The backend struct that interfaces with V4L2.
/// To see what this does, please see [`CaptureBackendTrait`].
/// # Quirks
/// - While working with `iOS` is allowed, it is not officially supported and may not work.
/// - You **must** call [`nokhwa_initialize`](crate::nokhwa_initialize) **before** doing anything with `AVFoundation`.
/// - This only works on 64 bit platforms.
/// - FPS adjustment does not work.
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-avfoundation")))]
pub struct AVFoundationCaptureDevice {
    device: AVCaptureDevice,
    dev_input: Option<AVCaptureDeviceInput>,
    session: Option<AVCaptureSession>,
    data_out: Option<AVCaptureVideoDataOutput>,
    data_collect: Option<AVCaptureVideoCallback>,
    info: CameraInfo,
    format: CameraFormat,
}

impl AVFoundationCaptureDevice {
    /// Creates a new capture device using the `AVFoundation` backend. Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default.
    /// # Errors
    /// This function will error if the camera is currently busy or if `AVFoundation` can't read device information, or permission was not given by the user.
    pub fn new(index: usize, camera_format: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        let camera_format = match camera_format {
            Some(fmt) => fmt,
            None => CameraFormat::default(),
        };

        let device_descriptor: CameraInfo = match query_avfoundation()?.into_iter().nth(index) {
            Some(descriptor) => descriptor.into(),
            None => {
                return Err(NokhwaError::OpenDeviceError(
                    index.to_string(),
                    "No Device".to_string(),
                ))
            }
        };

        let mut device = AVCaptureDevice::from_id(&device_descriptor.misc())?;
        device.lock()?;
        device.set_all(camera_format.into())?;

        Ok(AVFoundationCaptureDevice {
            device,
            dev_input: None,
            session: None,
            data_out: None,
            data_collect: None,
            info: device_descriptor,
            format: camera_format,
        })
    }

    /// Creates a new capture device using the `AVFoundation` backend with desired settings.
    ///
    /// # Errors
    /// This function will error if the camera is currently busy or if `AVFoundation` can't read device information, or permission was not given by the user.
    pub fn new_with(
        index: usize,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
    ) -> Result<Self, NokhwaError> {
        let camera_format = Some(CameraFormat::new_from(width, height, fourcc, fps));
        AVFoundationCaptureDevice::new(index, camera_format)
    }
}

impl CaptureBackendTrait for AVFoundationCaptureDevice {
    fn backend(&self) -> CaptureAPIBackend {
        CaptureAPIBackend::AVFoundation
    }

    fn camera_info(&self) -> &CameraInfo {
        &self.info
    }

    fn camera_format(&self) -> CameraFormat {
        self.format
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        self.device.set_all(new_fmt.into())?;
        self.format = new_fmt;
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn compatible_list_by_resolution(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        Ok(self
            .device
            .supported_formats()?
            .into_iter()
            .map(|fmt| {
                (
                    FrameFormat::from(fmt.fourcc),
                    Resolution::from(fmt.resolution),
                    (&fmt.fps_list)
                        .iter()
                        .map(|f| *f as u32)
                        .collect::<Vec<u32>>(),
                )
            })
            .filter(|x| (*x).0 == fourcc)
            .map(|fmt| (fmt.1, fmt.2))
            .collect::<HashMap<Resolution, Vec<u32>>>())
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        let mut formats = self
            .device
            .supported_formats()?
            .into_iter()
            .map(|fmt| FrameFormat::from(fmt.fourcc))
            .collect::<Vec<FrameFormat>>();
        formats.sort();
        formats.dedup();
        Ok(formats)
    }

    fn resolution(&self) -> Resolution {
        self.camera_format().resolution()
    }

    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        let mut format = self.camera_format();
        format.set_resolution(new_res);
        self.set_camera_format(format)
    }

    fn frame_rate(&self) -> u32 {
        self.camera_format().frame_rate()
    }

    fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        let mut format = self.camera_format();
        format.set_frame_rate(new_fps);
        self.set_camera_format(format)
    }

    fn frame_format(&self) -> FrameFormat {
        self.camera_format().format()
    }

    fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        let mut format = self.camera_format();
        format.set_format(fourcc);
        self.set_camera_format(format)
    }

    fn supported_camera_controls(&self) -> Result<Vec<KnownCameraControls>, NokhwaError> {
        Err(NokhwaError::NotImplementedError(
            "Not Implemented".to_string(),
        ))
    }

    fn camera_control(&self, _: KnownCameraControls) -> Result<CameraControl, NokhwaError> {
        Err(NokhwaError::NotImplementedError(
            "Not Implemented".to_string(),
        ))
    }

    fn set_camera_control(&mut self, _: CameraControl) -> Result<(), NokhwaError> {
        Err(NokhwaError::NotImplementedError(
            "Not Implemented".to_string(),
        ))
    }

    fn raw_supported_camera_controls(&self) -> Result<Vec<Box<dyn Any>>, NokhwaError> {
        Err(NokhwaError::NotImplementedError(
            "Not Implemented".to_string(),
        ))
    }

    fn raw_camera_control(&self, _: &dyn Any) -> Result<Box<dyn Any>, NokhwaError> {
        Err(NokhwaError::NotImplementedError(
            "Not Implemented".to_string(),
        ))
    }

    fn set_raw_camera_control(&mut self, _: &dyn Any, _: &dyn Any) -> Result<(), NokhwaError> {
        Err(NokhwaError::NotImplementedError(
            "Not Implemented".to_string(),
        ))
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        let input = AVCaptureDeviceInput::new(&self.device)?;
        let session = AVCaptureSession::new();
        session.begin_configuration();
        session.add_input(&input)?;
        let callback = AVCaptureVideoCallback::new(self.info.index());
        let output = AVCaptureVideoDataOutput::new();
        output.add_delegate(&callback)?;
        session.add_output(&output)?;
        session.commit_configuration();
        session.start()?;

        self.dev_input = Some(input);
        self.session = Some(session);
        self.data_collect = Some(callback);
        self.data_out = Some(output);
        Ok(())
    }

    fn is_stream_open(&self) -> bool {
        if self.session.is_some()
            && self.data_out.is_some()
            && self.data_collect.is_some()
            && self.dev_input.is_some()
        {
            return true;
        }
        match &self.session {
            Some(session) => (!session.is_interrupted()) && session.is_running(),
            None => false,
        }
    }

    fn frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        let cam_fmt = self.camera_format();
        let conv = self.frame_raw()?.to_vec();
        let image_buf =
            match ImageBuffer::from_vec(cam_fmt.width(), cam_fmt.height(), conv) {
                Some(buf) => {
                    let rgb_buf: ImageBuffer<Rgb<u8>, Vec<u8>> = buf;
                    rgb_buf
                }
                None => return Err(NokhwaError::ReadFrameError(
                    "ImageBuffer is not large enough! This is probably a bug, please report it!"
                        .to_string(),
                )),
            };
        Ok(image_buf)
    }

    fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        match &self.session {
            Some(session) => {
                if !session.is_running() {
                    return Err(NokhwaError::ReadFrameError(
                        "Stream Not Started".to_string(),
                    ));
                }
                if session.is_interrupted() {
                    return Err(NokhwaError::ReadFrameError(
                        "Stream Interrupted".to_string(),
                    ));
                }
            }
            None => {
                return Err(NokhwaError::ReadFrameError(
                    "Stream Not Started".to_string(),
                ))
            }
        }

        match &self.data_collect {
            Some(collector) => {
                let data = collector.frame_to_slice()?;
                let data = match data.1 {
                    AVFourCC::YUV2 => Cow::from(yuyv422_to_rgb888(&data.0)?),
                    AVFourCC::MJPEG => Cow::from(mjpeg_to_rgb888(&data.0)?),
                };
                Ok(data)
            }
            None => Err(NokhwaError::ReadFrameError(
                "Stream Not Started".to_string(),
            )),
        }
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        if !self.is_stream_open() {
            return Ok(());
        }

        let session = match &self.session {
            Some(session) => session,
            None => return Ok(()),
        };

        let output = match &self.data_out {
            Some(output) => output,
            None => return Ok(()),
        };

        let input = match &self.dev_input {
            Some(input) => input,
            None => return Ok(()),
        };

        session.remove_output(output);
        session.remove_input(input);
        session.stop();
        Ok(())
    }
}

impl Drop for AVFoundationCaptureDevice {
    fn drop(&mut self) {
        if self.stop_stream().is_err() {}
        self.device.unlock();
    }
}
