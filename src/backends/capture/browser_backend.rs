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

use crate::js_camera::query_js_cameras;
use image::{ImageBuffer, Rgb};
use nokhwa_core::types::ControlValueSetter;
use nokhwa_core::{
    error::NokhwaError,
    traits::CaptureBackendTrait,
    types::{
        ApiBackend, CameraControl, CameraFormat, CameraIndex, CameraInfo, FrameFormat,
        KnownCameraControl, Resolution,
    },
};
use std::{any::Any, borrow::Cow, collections::HashMap};

/// Captures using the Browser API. This internally wraps [`JSCamera`].
///
/// # Quirks
/// - `FourCC` setting is ignored
/// - Cannot get compatible resolution(s).
/// - CameraControl(s) are not supported.
/// - All frame capture is done by creating (then destorying) a canvas on the DOM.
/// - Many methods are blocking on user input.
pub struct BrowserCaptureDevice {
    camera: JSCamera,
    info: CameraInfo,
}

impl BrowserCaptureDevice {
    // WARN: blocking on pass integer for index
    /// Creates a new camera from an [`CameraIndex`]. It can take [`CameraIndex::Index`] or [`CameraIndex::String`] (NOTE: blocks on [`CameraIndex::Index`])
    ///
    /// # Errors
    /// If the device is not found, browser not supported, or camera is over-constrained this will error.
    pub fn new(index: &CameraIndex, cam_fmt: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        let (group_id, device_id) = match &index {
            CameraIndex::Index(i) => {
                let query_devices =
                    wasm_rs_async_executor::single_threaded::block_on(query_js_cameras())?;
                match query_devices.into_iter().nth(*i as usize) {
                    Some(info) => {
                        let ids = info
                            .to_string()
                            .split(' ')
                            .map(ToString::to_string)
                            .collect::<Vec<String>>();
                        match (ids.get(0), ids.get(1)) {
                            (Some(group_id), Some(device_id)) => {
                                (group_id.clone(), device_id.clone())
                            }
                            (_, _) => {
                                return Err(NokhwaError::OpenDeviceError(
                                    "Invalid Index".to_string(),
                                    index.to_string(),
                                ))
                            }
                        }
                    }
                    None => {
                        return Err(NokhwaError::OpenDeviceError(
                            "Device not found".to_string(),
                            index.to_string(),
                        ))
                    }
                }
            }
            CameraIndex::String(id) => {
                let ids = id
                    .to_string()
                    .split(' ')
                    .map(ToString::to_string)
                    .collect::<Vec<String>>();
                match (ids.get(0), ids.get(1)) {
                    (Some(group_id), Some(device_id)) => (group_id.clone(), device_id.clone()),
                    (_, _) => {
                        return Err(NokhwaError::OpenDeviceError(
                            "Invalid Index".to_string(),
                            index.to_string(),
                        ))
                    }
                }
            }
        };

        let camera_format = cam_fmt.unwrap_or_default();

        let constraints = JSCameraConstraintsBuilder::new()
            .frame_rate(camera_format.frame_rate())
            .resolution(camera_format.resolution())
            .aspect_ratio(f64::from(camera_format.width()) / f64::from(camera_format.height()))
            .group_id(&group_id)
            .group_id_exact(true)
            .device_id(&device_id)
            .device_id_exact(true)
            .resize_mode(JSCameraResizeMode::Any)
            .build();

        let camera = wasm_rs_async_executor::single_threaded::block_on(JSCamera::new(constraints))?;

        let info = (|| {
            let cameras = wasm_rs_async_executor::single_threaded::block_on(query_js_cameras())?;
            let giddid = format!("{} {}", group_id, device_id);
            for cam in cameras {
                if cam.misc() == giddid {
                    return Ok(cam);
                }
            }
            Ok(CameraInfo::new("", "videoinput", &giddid, index.clone()))
        })()?;
        Ok(BrowserCaptureDevice { camera, info })
    }

    /// Creates a new camera from an [`CameraIndex`] and raw parts. It can take [`CameraIndex::Index`] or [`CameraIndex::String`] (NOTE: blocks on [`CameraIndex::Index`])
    ///
    /// # Errors
    /// If the device is not found, browser not supported, or camera is over-constrained this will error.
    pub fn new_with(
        index: &CameraIndex,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
    ) -> Result<Self, NokhwaError> {
        Self::new(
            index,
            Some(CameraFormat::new(Resolution::new(width, height))),
        )
    }
}

impl CaptureBackendTrait for BrowserCaptureDevice {
    fn backend(&self) -> ApiBackend {
        ApiBackend::Browser
    }

    fn camera_info(&self) -> &CameraInfo {
        &self.info
    }

    fn refresh_camera_format(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    fn camera_format(&self) -> CameraFormat {
        CameraFormat::new(self.camera.resolution())
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        let current_constraints = self.camera.constraints();

        let new_constraints = JSCameraConstraintsBuilder::new()
            .resolution(new_fmt.resolution())
            .aspect_ratio(f64::from(new_fmt.width()) / f64::from(new_fmt.height()))
            .frame_rate(new_fmt.frame_rate())
            .group_id(&current_constraints.group_id())
            .device_id(&current_constraints.device_id())
            .resize_mode(JSCameraResizeMode::Any)
            .build();

        let _constraint_err = self.camera.set_constraints(new_constraints);
        match self.camera.apply_constraints() {
            Ok(_) => Ok(()),
            Err(why) => {
                let _returnerr = self.camera.set_constraints(current_constraints); // swallow errors - revert
                Err(why)
            }
        }
    }

    fn compatible_list_by_resolution(
        &mut self,
        _: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        Err(NokhwaError::NotImplementedError(
            "Not Implemented".to_string(),
        ))
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        Ok(vec![FrameFormat::MJPEG, FrameFormat::YUYV])
    }

    fn resolution(&self) -> Resolution {
        self.camera.resolution()
    }

    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        let mut current_format = self.camera_format();
        current_format.set_resolution(new_res);
        self.set_camera_format(current_format)
    }

    fn frame_rate(&self) -> u32 {
        self.camera.constraints().frame_rate()
    }

    fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        let mut current_format = self.camera_format();
        current_format.set_frame_rate(new_fps);
        self.set_camera_format(current_format)
    }

    fn frame_format(&self) -> FrameFormat {
        FrameFormat::MJPEG
    }

    fn set_frame_format(&mut self, _: FrameFormat) -> Result<(), NokhwaError> {
        Ok(())
    }

    fn camera_control(&self, control: KnownCameraControl) -> Result<CameraControl, NokhwaError> {
        todo!()
    }

    fn camera_controls(&self) -> Result<Vec<CameraControl>, NokhwaError> {
        todo!()
    }

    fn set_camera_control(
        &mut self,
        id: KnownCameraControl,
        value: ControlValueSetter,
    ) -> Result<(), NokhwaError> {
        todo!()
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        Ok(())
    }

    fn is_stream_open(&self) -> bool {
        self.camera.is_open()
    }

    fn frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        self.camera.frame()
    }

    fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        self.camera.frame_raw()
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        self.camera.stop_all()
    }
}
