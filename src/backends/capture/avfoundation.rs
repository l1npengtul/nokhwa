/*
 * Copyright 2022 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
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
#[cfg(target_os = "macos")]
use nokhwa_bindings_macos::{
    AVCaptureDevice, AVCaptureDeviceInput, AVCaptureSession, AVCaptureVideoCallback,
    AVCaptureVideoDataOutput,
};
use nokhwa_core::pixel_format::RgbFormat;
use nokhwa_core::types::RequestedFormatType;
use nokhwa_core::{
    buffer::Buffer,
    error::NokhwaError,
    traits::CaptureBackendTrait,
    types::{
        ApiBackend, CameraControl, CameraFormat, CameraIndex, CameraInfo, ControlValueSetter,
        FrameFormat, KnownCameraControl, RequestedFormat, Resolution,
    },
};
use std::{borrow::Cow, collections::HashMap};
#[cfg(target_os = "macos")]
use std::{
    ffi::CString,
    sync::{Arc, Mutex},
};

/// The backend struct that interfaces with V4L2.
/// To see what this does, please see [`CaptureBackendTrait`].
/// # Quirks
/// - While working with `iOS` is allowed, it is not officially supported and may not work.
/// - You **must** call [`nokhwa_initialize`](crate::nokhwa_initialize) **before** doing anything with `AVFoundation`.
/// - This only works on 64 bit platforms.
/// - FPS adjustment does not work.
/// - If permission has not been granted and you call `init()` it will error.
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-avfoundation")))]
#[cfg(target_os = "macos")]
pub struct AVFoundationCaptureDevice {
    device: AVCaptureDevice,
    dev_input: Option<AVCaptureDeviceInput>,
    session: Option<AVCaptureSession>,
    data_out: Option<AVCaptureVideoDataOutput>,
    data_collect: Option<AVCaptureVideoCallback>,
    info: CameraInfo,
    buffer_name: CString,
    format: CameraFormat,
    frame_buffer_lock: Arc<Mutex<(Vec<u8>, FrameFormat)>>,
}

#[cfg(target_os = "macos")]
impl AVFoundationCaptureDevice {
    /// Creates a new capture device using the `AVFoundation` backend. Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default.
    /// # Errors
    /// This function will error if the camera is currently busy or if `AVFoundation` can't read device information, or permission was not given by the user.
    pub fn new(index: &CameraIndex, req_fmt: RequestedFormat) -> Result<Self, NokhwaError> {
        let mut device = AVCaptureDevice::new(index)?;
        device.lock()?;
        let formats = device.supported_formats()?;
        let camera_fmt = req_fmt.fulfill(&formats).ok_or_else(|| {
            NokhwaError::OpenDeviceError("Cannot fulfill request".to_string(), req_fmt.to_string())
        })?;
        device.set_all(camera_fmt)?;
        let device_descriptor = device.info().clone();
        let buffername =
            CString::new(format!("{}_INDEX{}_", device_descriptor, index)).map_err(|why| {
                NokhwaError::StructureError {
                    structure: "CString Buffername".to_string(),
                    error: why.to_string(),
                }
            })?;

        Ok(AVFoundationCaptureDevice {
            device,
            dev_input: None,
            session: None,
            data_out: None,
            data_collect: None,
            info: device_descriptor,
            buffer_name: buffername,
            format: camera_fmt,
            frame_buffer_lock: Arc::new(Mutex::new((vec![], FrameFormat::MJPEG))),
        })
    }

    /// Creates a new capture device using the `AVFoundation` backend with desired settings.
    ///
    /// # Errors
    /// This function will error if the camera is currently busy or if `AVFoundation` can't read device information, or permission was not given by the user.
    #[deprecated(since = "0.10.0", note = "please use `new` instead.")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new_with(
        index: usize,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
    ) -> Result<Self, NokhwaError> {
        let camera_format = CameraFormat::new_from(width, height, fourcc, fps);
        AVFoundationCaptureDevice::new(
            &CameraIndex::Index(index as u32),
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format)),
        )
    }
}

#[cfg(target_os = "macos")]
impl CaptureBackendTrait for AVFoundationCaptureDevice {
    fn backend(&self) -> ApiBackend {
        ApiBackend::AVFoundation
    }

    fn camera_info(&self) -> &CameraInfo {
        &self.info
    }

    fn refresh_camera_format(&mut self) -> Result<(), NokhwaError> {
        Ok(())
    }

    fn camera_format(&self) -> CameraFormat {
        self.format
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        self.device.set_all(new_fmt)?;
        self.format = new_fmt;
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn compatible_list_by_resolution(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        let supported_cfmt = self
            .device
            .supported_formats()?
            .into_iter()
            .filter(|x| x.format() != fourcc);
        let mut res_list = HashMap::new();
        for format in supported_cfmt {
            match res_list.get_mut(&format.resolution()) {
                Some(fpses) => Vec::push(fpses, format.frame_rate()),
                None => {
                    res_list.insert(format.resolution(), vec![format.frame_rate()]);
                }
            }
        }
        Ok(res_list)
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        let mut formats = self
            .device
            .supported_formats()?
            .into_iter()
            .map(|fmt| fmt.format())
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

    fn camera_control(&self, control: KnownCameraControl) -> Result<CameraControl, NokhwaError> {
        for ctrl in self.device.get_controls()? {
            if ctrl.control() == control {
                return Ok(ctrl);
            }
        }

        Err(NokhwaError::GetPropertyError {
            property: control.to_string(),
            error: "Not Found".to_string(),
        })
    }

    fn camera_controls(&self) -> Result<Vec<CameraControl>, NokhwaError> {
        self.device.get_controls()
    }

    fn set_camera_control(
        &mut self,
        id: KnownCameraControl,
        value: ControlValueSetter,
    ) -> Result<(), NokhwaError> {
        self.device.lock()?;
        let res = self.device.set_control(id, value);
        self.device.unlock();
        res
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        let input = AVCaptureDeviceInput::new(&self.device)?;
        let session = AVCaptureSession::new();
        session.begin_configuration();
        session.add_input(&input)?;

        let frame_mutex = self.frame_buffer_lock.clone();
        let bufname = &self.buffer_name;
        let videocallback = AVCaptureVideoCallback::new(bufname, frame_mutex)?;
        let output = AVCaptureVideoDataOutput::new();
        output.add_delegate(&videocallback)?;
        session.add_output(&output)?;
        session.commit_configuration();
        session.start()?;

        self.dev_input = Some(input);
        self.session = Some(session);
        self.data_collect = Some(videocallback);
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

    fn frame(&mut self) -> Result<Buffer, NokhwaError> {
        self.refresh_camera_format()?;
        let cfmt = self.camera_format();
        let buffer = Buffer::new(cfmt.resolution(), &self.frame_raw()?, cfmt.format());
        Ok(buffer)
    }

    fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        let mut framebuffer_empty = match self.frame_buffer_lock.lock() {
            Ok(f) => f.0.is_empty(),
            Err(why) => return Err(NokhwaError::ReadFrameError(why.to_string())),
        };

        loop {
            if framebuffer_empty {
                match self.frame_buffer_lock.lock() {
                    Ok(f) => framebuffer_empty = f.0.is_empty(),
                    Err(why) => return Err(NokhwaError::ReadFrameError(why.to_string())),
                }
            } else {
                break;
            }
        }

        match self.frame_buffer_lock.lock() {
            Ok(mut f) => {
                let mut new_frame = vec![];
                std::mem::swap(&mut new_frame, &mut f.0);
                Ok(Cow::from(new_frame))
            }
            Err(why) => Err(NokhwaError::ReadFrameError(why.to_string())),
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

#[cfg(target_os = "macos")]
impl Drop for AVFoundationCaptureDevice {
    fn drop(&mut self) {
        if self.stop_stream().is_err() {}
        self.device.unlock();
    }
}

/// The backend struct that interfaces with V4L2.
/// To see what this does, please see [`CaptureBackendTrait`].
/// # Quirks
/// - While working with `iOS` is allowed, it is not officially supported and may not work.
/// - You **must** call [`nokhwa_initialize`](crate::nokhwa_initialize) **before** doing anything with `AVFoundation`.
/// - This only works on 64 bit platforms.
/// - FPS adjustment does not work.
/// - If permission has not been granted and you call `init()` it will error.
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-avfoundation")))]
#[cfg(not(target_os = "macos"))]
pub struct AVFoundationCaptureDevice {}

#[cfg(not(target_os = "macos"))]
#[allow(unused_variables)]
#[allow(unreachable_code)]
impl AVFoundationCaptureDevice {
    /// Creates a new capture device using the `AVFoundation` backend. Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default.
    /// # Errors
    /// This function will error if the camera is currently busy or if `AVFoundation` can't read device information, or permission was not given by the user.
    pub fn new(index: &CameraIndex, req_fmt: RequestedFormat) -> Result<Self, NokhwaError> {
        todo!()
    }

    /// Creates a new capture device using the `AVFoundation` backend with desired settings.
    ///
    /// # Errors
    /// This function will error if the camera is currently busy or if `AVFoundation` can't read device information, or permission was not given by the user.
    #[deprecated(since = "0.10.0", note = "please use `new` instead.")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new_with(
        index: usize,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
    ) -> Result<Self, NokhwaError> {
        todo!()
    }
}

#[cfg(not(target_os = "macos"))]
#[allow(unreachable_code)]
impl CaptureBackendTrait for AVFoundationCaptureDevice {
    fn backend(&self) -> ApiBackend {
        todo!()
    }

    fn camera_info(&self) -> &CameraInfo {
        todo!()
    }

    fn refresh_camera_format(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    fn camera_format(&self) -> CameraFormat {
        todo!()
    }

    fn set_camera_format(&mut self, _: CameraFormat) -> Result<(), NokhwaError> {
        todo!()
    }

    fn compatible_list_by_resolution(
        &mut self,
        _: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        todo!()
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        todo!()
    }

    fn resolution(&self) -> Resolution {
        todo!()
    }

    fn set_resolution(&mut self, _: Resolution) -> Result<(), NokhwaError> {
        todo!()
    }

    fn frame_rate(&self) -> u32 {
        todo!()
    }

    fn set_frame_rate(&mut self, _: u32) -> Result<(), NokhwaError> {
        todo!()
    }

    fn frame_format(&self) -> FrameFormat {
        todo!()
    }

    fn set_frame_format(&mut self, _: FrameFormat) -> Result<(), NokhwaError> {
        todo!()
    }

    fn camera_control(&self, _: KnownCameraControl) -> Result<CameraControl, NokhwaError> {
        todo!()
    }

    fn camera_controls(&self) -> Result<Vec<CameraControl>, NokhwaError> {
        todo!()
    }

    fn set_camera_control(
        &mut self,
        _: KnownCameraControl,
        _: ControlValueSetter,
    ) -> Result<(), NokhwaError> {
        todo!()
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    fn is_stream_open(&self) -> bool {
        todo!()
    }

    fn frame(&mut self) -> Result<Buffer, NokhwaError> {
        todo!()
    }

    fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        todo!()
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }
}

#[cfg(not(target_os = "macos"))]
#[allow(unreachable_code)]
impl Drop for AVFoundationCaptureDevice {
    fn drop(&mut self) {
        todo!()
    }
}
