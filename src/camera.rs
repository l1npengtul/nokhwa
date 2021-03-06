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

use crate::{
    buffer::Buffer, pixel_format::PixelFormat, BackendsEnum, CameraControl, CameraFormat,
    CameraInfo, CaptureAPIBackend, CaptureBackendTrait, FrameFormat, KnownCameraControl,
    NokhwaError, Resolution,
};
use std::{any::Any, borrow::Cow, collections::HashMap};
#[cfg(feature = "output-wgpu")]
use wgpu::{Device as WgpuDevice, Queue as WgpuQueue, Texture as WgpuTexture};

/// The main `Camera` struct. This is the struct that abstracts over all the backends, providing a simplified interface for use.
pub struct Camera {
    idx: usize,
    backend: BackendsEnum,
    backend_api: CaptureAPIBackend,
}

impl Camera {
    /// Create a new camera from an `index` and `format`
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn new(index: usize, format: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        Camera::with_backend(index, format, CaptureAPIBackend::Auto)
    }

    /// Create a new camera from an `index`, `format`, and `backend`. `format` can be `None`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn with_backend(
        index: usize,
        format: Option<CameraFormat>,
        backend: CaptureAPIBackend,
    ) -> Result<Self, NokhwaError> {
        let camera_backend = init_camera(index, format, backend)?;

        Ok(Camera {
            idx: index,
            backend: camera_backend,
            backend_api: backend,
        })
    }

    /// Create a new `Camera` from raw values.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn new_with(
        index: usize,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
        backend: CaptureAPIBackend,
    ) -> Result<Self, NokhwaError> {
        let camera_format = CameraFormat::new_from(width, height, fourcc, fps);
        Camera::with_backend(index, Some(camera_format), backend)
    }

    /// Gets the current Camera's index.
    #[must_use]
    pub fn index(&self) -> usize {
        self.idx
    }

    /// Sets the current Camera's index. Note that this re-initializes the camera.
    /// # Errors
    /// The Backend may fail to initialize.
    pub fn set_index(&mut self, new_idx: usize) -> Result<(), NokhwaError> {
        {
            self.backend.stop_stream()?;
        }
        let new_camera_format = self.backend.camera_format()?;
        let new_camera = init_camera(new_idx, Some(new_camera_format), self.backend_api)?;
        self.backend = new_camera;
        Ok(())
    }

    /// Gets the current Camera's backend
    #[must_use]
    pub fn backend(&self) -> CaptureAPIBackend {
        self.backend_api
    }

    /// Sets the current Camera's backend. Note that this re-initializes the camera.
    /// # Errors
    /// The new backend may not exist or may fail to initialize the new camera.
    pub fn set_backend(&mut self, new_backend: CaptureAPIBackend) -> Result<(), NokhwaError> {
        {
            self.backend.stop_stream()?;
        }
        let new_camera_format = self.backend.camera_format()?;
        let new_camera = init_camera(self.idx, Some(new_camera_format), new_backend)?;
        self.backend = new_camera;
        Ok(())
    }

    /// Gets the camera information such as Name and Index as a [`CameraInfo`].
    #[must_use]
    pub fn info(&self) -> &CameraInfo {
        self.backend.camera_info()
    }

    /// Gets the current [`CameraFormat`].
    pub fn cached_camera_format(&self) -> CameraFormat {
        self.backend.cached_camera_format()
    }

    /// Gets the current [`CameraFormat`]. This will force refresh to the current latest if it has changed.
    pub fn camera_format(&self) -> Result<CameraFormat, NokhwaError> {
        self.backend.camera_format()
    }

    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    pub fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        self.backend.set_camera_format(new_fmt)
    }

    /// A hashmap of [`Resolution`]s mapped to framerates
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a [`UnsupportedOperationError`](crate::NokhwaError::UnsupportedOperationError).
    pub fn compatible_list_by_resolution(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        self.backend.compatible_list_by_resolution(fourcc)
    }

    /// A Vector of compatible [`FrameFormat`]s.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a [`UnsupportedOperationError`](crate::NokhwaError::UnsupportedOperationError).
    pub fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        self.backend.compatible_fourcc()
    }

    /// A Vector of available [`CameraFormat`]s.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a [`UnsupportedOperationError`](crate::NokhwaError::UnsupportedOperationError).
    pub fn compatible_camera_formats(&mut self) -> Result<Vec<CameraFormat>, NokhwaError> {
        let mut camera_formats = Vec::with_capacity(64);
        for foramt in self.compatible_fourcc()? {
            let resolution_and_fps: HashMap<Resolution, Vec<u32>> =
                self.compatible_list_by_resolution(foramt)?;
            for (res, rates) in resolution_and_fps {
                for fps in rates {
                    camera_formats.push(CameraFormat::new(res, foramt, fps))
                }
            }
        }
        Ok(camera_formats)
    }

    /// Gets the current camera resolution (See: [`Resolution`], [`CameraFormat`]).
    pub fn cached_resolution(&self) -> Resolution {
        self.backend.cached_resolution()
    }

    /// Gets the current camera resolution (See: [`Resolution`], [`CameraFormat`]). This will force refresh to the current latest if it has changed.
    pub fn resolution(&self) -> Result<Resolution, NokhwaError> {
        self.backend.resolution()
    }

    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    pub fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        self.backend.set_resolution(new_res)
    }

    /// Gets the current camera framerate (See: [`CameraFormat`]).
    pub fn cached_frame_rate(&self) -> u32 {
        self.backend.cached_frame_rate()
    }

    /// Gets the current camera framerate (See: [`CameraFormat`]).
    pub fn frame_rate(&self) -> Result<u32, NokhwaError> {
        self.backend.frame_rate()
    }

    /// Will set the current framerate
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new framerate, this will return an error.
    pub fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        self.backend.set_frame_rate(new_fps)
    }

    /// Gets the current camera's frame format (See: [`FrameFormat`], [`CameraFormat`]).
    pub fn cached_frame_format(&self) -> FrameFormat {
        self.backend.cached_frame_format()
    }

    /// Gets the current camera's frame format (See: [`FrameFormat`], [`CameraFormat`]). This will force refresh to the current latest if it has changed.
    pub fn frame_format(&self) -> Result<FrameFormat, NokhwaError> {
        self.backend.frame_format()
    }

    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    ///
    /// This will also update the cache.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame format, this will return an error.
    pub fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        self.backend.set_frame_format(fourcc)
    }

    /// Gets the current supported list of [`KnownCameraControls`]
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn supported_camera_controls(&self) -> Result<Vec<KnownCameraControl>, NokhwaError> {
        self.backend.supported_camera_controls()
    }

    /// Gets the current supported list of [`CameraControl`]s keyed by its name as a `String`.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn camera_controls(&self) -> Result<Vec<CameraControl>, NokhwaError> {
        let known_controls = self.supported_camera_controls()?;
        let maybe_camera_controls = known_controls
            .iter()
            .map(|x| self.camera_control(*x))
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .collect::<Vec<CameraControl>>();

        Ok(maybe_camera_controls)
    }

    /// Gets the current supported list of [`CameraControl`]s keyed by its name as a `String`.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn camera_controls_string(&self) -> Result<HashMap<String, CameraControl>, NokhwaError> {
        let known_controls = self.supported_camera_controls()?;
        let maybe_camera_controls = known_controls
            .iter()
            .map(|x| (x.to_string(), self.camera_control(*x)))
            .filter(|(_, x)| x.is_ok())
            .map(|(c, x)| (c, Result::unwrap(x)))
            .collect::<Vec<(String, CameraControl)>>();
        let mut control_map = HashMap::with_capacity(maybe_camera_controls.len());

        for (kc, cc) in maybe_camera_controls.into_iter() {
            control_map.insert(kc, cc);
        }

        Ok(control_map)
    }

    /// Gets the current supported list of [`CameraControl`]s keyed by its name as a `String`.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn camera_controls_known_camera_controls(
        &self,
    ) -> Result<HashMap<KnownCameraControl, CameraControl>, NokhwaError> {
        let known_controls = self.supported_camera_controls()?;
        let maybe_camera_controls = known_controls
            .iter()
            .map(|x| (*x, self.camera_control(*x)))
            .filter(|(_, x)| x.is_ok())
            .map(|(c, x)| (c, Result::unwrap(x)))
            .collect::<Vec<(KnownCameraControl, CameraControl)>>();
        let mut control_map = HashMap::with_capacity(maybe_camera_controls.len());

        for (kc, cc) in maybe_camera_controls.into_iter() {
            control_map.insert(kc, cc);
        }

        Ok(control_map)
    }

    /// Gets the value of [`KnownCameraControls`].
    /// # Errors
    /// If the `control` is not supported or there is an error while getting the camera control values (e.g. unexpected value, too high, etc)
    /// this will error.
    pub fn camera_control(
        &self,
        control: KnownCameraControl,
    ) -> Result<CameraControl, NokhwaError> {
        self.backend.camera_control(control)
    }

    /// Sets the control to `control` in the camera.
    /// Usually, the pipeline is calling [`camera_control()`](CaptureBackendTrait::camera_control), getting a camera control that way
    /// then calling one of the methods to set the value: [`set_value()`](CameraControl::set_value()) or [`with_value()`](CameraControl::with_value()).
    /// # Errors
    /// If the `control` is not supported, the value is invalid (less than min, greater than max, not in step), or there was an error setting the control,
    /// this will error.
    pub fn set_camera_control(&mut self, control: CameraControl) -> Result<(), NokhwaError> {
        self.backend.set_camera_control(control)
    }

    /// Gets the current supported list of Controls as an `Any` from the backend.
    /// The `Any`'s type is defined by the backend itself, please check each of the backend's documentation.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn raw_supported_camera_controls(&self) -> Result<Vec<Box<dyn Any>>, NokhwaError> {
        self.backend.raw_supported_camera_controls()
    }

    /// Sets the control to `control` in the camera.
    /// The control's type is defined the backend itself. It may be a string, or more likely its a integer ID.
    /// The backend itself has documentation of the proper input/return values, please check each of the backend's documentation.
    /// # Errors
    /// If the `control` is not supported or there is an error while getting the camera control values (e.g. unexpected value, too high, wrong Any type)
    /// this will error.
    pub fn raw_camera_control(&self, control: &dyn Any) -> Result<Box<dyn Any>, NokhwaError> {
        self.backend.raw_camera_control(control)
    }

    /// Sets the control to `control` in the camera.
    /// The `control`/`value`'s type is defined the backend itself. It may be a string, or more likely its a integer ID/Value.
    /// Usually, the pipeline is calling [`camera_control()`](CaptureBackendTrait::camera_control), getting a camera control that way
    /// then calling one of the methods to set the value: [`set_value()`](CameraControl::set_value()) or [`with_value()`](CameraControl::with_value()).
    /// # Errors
    /// If the `control` is not supported, the value is invalid (wrong Any type, backend refusal), or there was an error setting the control,
    /// this will error.
    pub fn set_raw_camera_control(
        &mut self,
        control: &dyn Any,
        value: &dyn Any,
    ) -> Result<(), NokhwaError> {
        self.backend.set_raw_camera_control(control, value)
    }

    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`frame()`](CaptureBackendTrait::frame()) before you call [`open_stream()`](CaptureBackendTrait::open_stream()).
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    pub fn open_stream(&mut self) -> Result<(), NokhwaError> {
        self.backend.open_stream()
    }

    /// Checks if stream if open. If it is, it will return true.
    #[must_use]
    pub fn is_stream_open(&self) -> bool {
        self.backend.is_stream_open()
    }

    /// Will get a frame from the camera as a Raw RGB image buffer. Depending on the backend, if you have not called [`open_stream()`](CaptureBackendTrait::open_stream()) before you called this,
    /// it will either return an error.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), the decoding fails (e.g. MJPEG -> u8), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet,
    /// this will error.
    pub fn frame(&mut self) -> Result<Buffer, NokhwaError> {
        self.backend.frame()
    }

    /// Will get a frame from the camera **without** any processing applied, meaning you will usually get a frame you need to decode yourself.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet, this will error.
    pub fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        match self.backend.frame_raw() {
            Ok(f) => Ok(f),
            Err(why) => Err(why),
        }
    }

    /// Directly writes the current frame(RGB24) into said `buffer`. If `convert_rgba` is true, the buffer written will be written as an RGBA frame instead of a RGB frame. Returns the amount of bytes written on successful capture.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet, this will error.
    pub fn write_frame_to_buffer(
        &mut self,
        buffer: &mut [u8],
        write_alpha: bool,
    ) -> Result<usize, NokhwaError> {
        self.backend.write_frame_to_buffer(buffer, write_alpha)
    }

    #[cfg(feature = "output-wgpu")]
    #[cfg_attr(feature = "docs-features", doc(cfg(feature = "output-wgpu")))]
    /// Directly copies a frame to a Wgpu texture. This will automatically convert the frame into a RGBA frame.
    /// # Errors
    /// If the frame cannot be captured or the resolution is 0 on any axis, this will error.
    pub fn frame_texture<'a, F: PixelFormat>(
        &mut self,
        device: &WgpuDevice,
        queue: &WgpuQueue,
        label: Option<&'a str>,
    ) -> Result<WgpuTexture, NokhwaError> {
        self.backend.frame_texture(device, queue, label)
    }

    /// Will drop the stream.
    /// # Errors
    /// Please check the `Quirks` section of each backend.
    pub fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        self.backend.stop_stream()
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.stop_stream().unwrap();
    }
}

// TODO: Update as we go
#[allow(clippy::ifs_same_cond)]
fn figure_out_auto() -> Option<CaptureAPIBackend> {
    let platform = std::env::consts::OS;
    let mut cap = CaptureAPIBackend::Auto;
    if cfg!(feature = "input-v4l") && platform == "linux" {
        cap = CaptureAPIBackend::Video4Linux;
    } else if cfg!(feature = "input-msmf") && platform == "windows" {
        cap = CaptureAPIBackend::MediaFoundation;
    } else if cfg!(feature = "input-avfoundation") && (platform == "macos" || platform == "ios") {
        cap = CaptureAPIBackend::AVFoundation;
    } else if cfg!(feature = "input-uvc") {
        cap = CaptureAPIBackend::UniversalVideoClass;
    } else if cfg!(feature = "input-gst") {
        cap = CaptureAPIBackend::GStreamer;
    } else if cfg!(feature = "input-opencv") {
        cap = CaptureAPIBackend::OpenCv;
    }
    if cap == CaptureAPIBackend::Auto {
        return None;
    }
    Some(cap)
}

macro_rules! cap_impl_fn {
    {
        $( ($backend:expr, $init_fn:ident, $cfg:meta, $backend_name:ident) ),+
    } => {
        $(
            paste::paste! {
                #[cfg ($cfg) ]
                fn [< init_ $backend_name>](idx: usize, setting: Option<CameraFormat>) -> Option<Result<BackendsEnum, NokhwaError>> {
                    use crate::backends::capture::$backend;
                    match <$backend>::$init_fn(idx, setting) {
                        Ok(cap) => Some(Ok(cp.into())),
                        Err(why) => Some(Err(why)),
                    }
                }
                #[cfg(not( $cfg ))]
                fn [< init_ $backend_name>](_idx: usize, _setting: Option<CameraFormat>) -> Option<Result<BackendsEnum, NokhwaError>> {
                    None
                }
            }
        )+
    };
}

macro_rules! cap_impl_matches {
    {
        $use_backend: expr, $index:expr, $setting:expr,
        $( ($feature:expr, $backend:ident, $fn:ident) ),+
    } => {
        {
            let i = $index;
            let s = $setting;
            match $use_backend {
                CaptureAPIBackend::Auto => match figure_out_auto() {
                    Some(cap) => match cap {
                        $(
                            CaptureAPIBackend::$backend => {
                                match cfg!(feature = $feature) {
                                    true => {
                                        match $fn(i,s) {
                                            Some(cap) => match cap {
                                                Ok(c) => c,
                                                Err(why) => return Err(why),
                                            }
                                            None => {
                                                return Err(NokhwaError::NotImplementedError(
                                                    "Platform requirements not satisfied (Wrong Platform - Not Implemented).".to_string(),
                                                ));
                                            }
                                        }
                                    }
                                    false => {
                                        return Err(NokhwaError::NotImplementedError(
                                            "Platform requirements not satisfied. (Wrong Platform - Not Selected)".to_string(),
                                        ));
                                    }
                                }
                            }
                        )+
                        _ => {
                            return Err(NokhwaError::NotImplementedError(
                                "Platform requirements not satisfied. (Invalid Backend)".to_string(),
                            ));
                        }
                    }
                    None => {
                        return Err(NokhwaError::NotImplementedError(
                            "Platform requirements not satisfied. (No Selection)".to_string(),
                        ));
                    }
                }
                $(
                    CaptureAPIBackend::$backend => {
                        match cfg!(feature = $feature) {
                            true => {
                                match $fn(i,s) {
                                    Some(cap) => match cap {
                                        Ok(c) => c,
                                        Err(why) => return Err(why),
                                    }
                                    None => {
                                        return Err(NokhwaError::NotImplementedError(
                                            "Platform requirements not satisfied (Wrong Platform - Not Implemented).".to_string(),
                                        ));
                                    }
                                }
                            }
                            false => {
                                return Err(NokhwaError::NotImplementedError(
                                    "Platform requirements not satisfied. (Wrong Platform - Not Selected)".to_string(),
                                ));
                            }
                        }
                    }
                )+

                _ => {
                    return Err(NokhwaError::NotImplementedError(
                        "Platform requirements not satisfied. (Wrong Platform - Not Selected)".to_string(),
                    ));
                }
            }
        }
    }
}

cap_impl_fn! {
    // (GStreamerCaptureDevice, new, feature = "input-gst", gst),
    (OpenCvCaptureDevice, new_autopref, feature = "input-opencv", opencv),
    // (UVCCaptureDevice, create, feature = "input-uvc", uvc),
    (V4LCaptureDevice, new, all(feature = "input-v4l", target_os = "linux"), v4l),
    (MediaFoundationCaptureDevice, new, all(feature = "input-msmf", target_os = "windows"), msmf),
    (AVFoundationCaptureDevice, new, all(feature = "input-avfoundation", any(target_os = "macos", target_os = "ios")), avfoundation)
}

fn init_camera(
    index: usize,
    format: Option<CameraFormat>,
    backend: CaptureAPIBackend,
) -> Result<BackendsEnum, NokhwaError> {
    let camera_backend = cap_impl_matches! {
            backend, index, format,
            ("input-v4l", Video4Linux, init_v4l),
            ("input-msmf", MediaFoundation, init_msmf),
            ("input-avfoundation", AVFoundation, init_avfoundation),
            // ("input-uvc", UniversalVideoClass, init_uvc),
            // ("input-gst", GStreamer, init_gst),
            ("input-opencv", OpenCv, init_opencv)
    };
    Ok(camera_backend)
}

#[cfg(feature = "output-threaded")]
unsafe impl Send for Camera {}
