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
    Camera, CameraControl, CameraFormat, CameraInfo, CaptureAPIBackend, CaptureBackendTrait,
    FrameFormat, KnownCameraControls, NokhwaError, Resolution, Buffer
};
use image::{ImageBuffer, Rgb};
use parking_lot::Mutex;
use std::{
    any::Any,
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

type AtomicLock<T> = Arc<Mutex<T>>;
pub type CallbackFn = fn(
    _camera: &Arc<Mutex<Camera>>,
    _frame_callback: &Arc<
        Mutex<Option<Box<dyn FnMut(Buffer) + Send + 'static>>>,
    >,
    _last_frame_captured: &Arc<Mutex<Buffer>>,
    _die_bool: &Arc<AtomicBool>,
);
type HeldCallbackType =
    Arc<Mutex<Option<Box<dyn FnMut(Buffer) + Send + 'static>>>>;

/// Creates a camera that runs in a different thread that you can use a callback to access the frames of.
/// It uses a `Arc` and a `Mutex` to ensure that this feels like a normal camera, but callback based.
/// See [`Camera`] for more details on the camera itself.
///
/// Your function is called every time there is a new frame. In order to avoid frame loss, it should
/// complete before a new frame is available. If you need to do heavy image processing, it may be
/// beneficial to directly pipe the data to a new thread to process it there.
///
/// Note that this does not have `WGPU` capabilities. However, it should be easy to implement.
/// # SAFETY
/// The `Mutex` guarantees exclusive access to the underlying camera struct. They should be safe to
/// impl `Send` on.
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "output-threaded")))]
pub struct CallbackCamera {
    camera: AtomicLock<Camera>,
    frame_callback: HeldCallbackType,
    last_frame_captured: AtomicLock<Buffer>,
    die_bool: Arc<AtomicBool>,
}

impl CallbackCamera {
    /// Create a new `ThreadedCamera` from an `index` and `format`. `format` can be `None`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn new(index: usize, format: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        CallbackCamera::with_backend(index, format, CaptureAPIBackend::Auto)
    }

    /// Create a new camera from an `index`, `format`, and `backend`. `format` can be `None`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn with_backend(
        index: usize,
        format: Option<CameraFormat>,
        backend: CaptureAPIBackend,
    ) -> Result<Self, NokhwaError> {
        Self::customized_all(index, format, backend, None)
    }

    /// Create a new `ThreadedCamera` from raw values.
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
        CallbackCamera::with_backend(index, Some(camera_format), backend)
    }

    /// Create a new `ThreadedCamera` from raw values, including the raw capture function.
    ///
    /// **This is meant for advanced users only.**
    ///
    /// An example capture function can be found by clicking `[src]` and scrolling down to the bottom to function `camera_frame_thread_loop()`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn customized_all(
        index: usize,
        format: Option<CameraFormat>,
        backend: CaptureAPIBackend,
        func: Option<CallbackFn>,
    ) -> Result<Self, NokhwaError> {
        let format = match format {
            Some(fmt) => fmt,
            None => CameraFormat::default(),
        };
        let camera = Arc::new(Mutex::new(Camera::with_backend(
            index,
            Some(format),
            backend,
        )?));
        let frame_callback = Arc::new(Mutex::new(None));
        let last_frame_captured = Arc::new(Mutex::new(Buffer::default()));
        let die_bool = Arc::new(AtomicBool::new(false));

        let camera_clone = camera.clone();
        let frame_callback_clone = frame_callback.clone();
        let last_frame_captured_clone = last_frame_captured.clone();
        let die_bool_clone = die_bool.clone();

        let thread_callback = match func {
            Some(cb) => cb,
            None => camera_frame_thread_loop,
        };

        match std::thread::Builder::new()
            .name(format!("CaptureProcessThreadofCamera {}", index))
            .spawn(move || {
                thread_callback(
                    &camera_clone,
                    &frame_callback_clone,
                    &last_frame_captured_clone,
                    &die_bool_clone,
                );
            }) {
            Ok(handle) => handle,
            Err(why) => {
                return Err(NokhwaError::OpenDeviceError(
                    index.to_string(),
                    format!("ThreadError: {}", why),
                ))
            }
        };

        Ok(CallbackCamera {
            camera,
            frame_callback,
            last_frame_captured,
            die_bool,
        })
    }

    /// Gets the current Camera's index.
    #[must_use]
    pub fn index(&self) -> usize {
        self.camera.lock().index().clone()
    }

    /// Sets the current Camera's index. Note that this re-initializes the camera.
    /// # Errors
    /// The Backend may fail to initialize.
    pub fn set_index(&mut self, new_idx: usize) -> Result<(), NokhwaError> {
        self.camera.lock().set_index(new_idx)
    }

    /// Gets the current Camera's backend
    #[must_use]
    pub fn backend(&self) -> CaptureAPIBackend {
        self.camera.lock().backend()
    }

    /// Sets the current Camera's backend. Note that this re-initializes the camera.
    /// # Errors
    /// The new backend may not exist or may fail to initialize the new camera.
    pub fn set_backend(&mut self, new_backend: CaptureAPIBackend) -> Result<(), NokhwaError> {
        self.camera.lock().set_backend(new_backend)
    }

    /// Gets the camera information such as Name and Index as a [`CameraInfo`].
    #[must_use]
    pub fn info(&self) -> CameraInfo {
        self.camera.lock().info().clone()
    }

    /// Gets the current [`CameraFormat`].
    #[must_use]
    pub fn camera_format(&self) -> Result<CameraFormat, NokhwaError> {
        self.camera.lock().camera_format()
    }

    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    pub fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        *self.last_frame_captured.lock() = Buffer::new(new_res, Vec::default(), self.camera_format()?.format());
        self.camera.lock().set_camera_format(new_fmt)
    }

    /// A hashmap of [`Resolution`]s mapped to framerates
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a [`UnsupportedOperationError`](crate::NokhwaError::UnsupportedOperationError).
    pub fn compatible_list_by_resolution(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        self.camera.lock().compatible_list_by_resolution(fourcc)
    }

    /// A Vector of compatible [`FrameFormat`]s.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a [`UnsupportedOperationError`](crate::NokhwaError::UnsupportedOperationError).
    pub fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        self.camera.lock().compatible_fourcc()
    }

    /// Gets the current camera resolution (See: [`Resolution`], [`CameraFormat`]).
    #[must_use]
    pub fn resolution(&self) -> Result<Resolution, NokhwaError> {
        self.camera.lock().resolution()
    }

    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    pub fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        *self.last_frame_captured.lock() = Buffer::new(new_res, Vec::default(), self.camera_format()?.format());
        self.camera.lock().set_resolution(new_res)
    }

    /// Gets the current camera framerate (See: [`CameraFormat`]).
    #[must_use]
    pub fn frame_rate(&self) -> Result<u32, NokhwaError> {
        self.camera.lock().frame_rate()
    }

    /// Will set the current framerate
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new framerate, this will return an error.
    pub fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        self.camera.lock().set_frame_rate(new_fps)
    }

    /// Gets the current camera's frame format (See: [`FrameFormat`], [`CameraFormat`]).
    #[must_use]
    pub fn frame_format(&self) -> Result<FrameFormat, NokhwaError> {
        self.camera.lock().frame_format()
    }

    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame format, this will return an error.
    pub fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        self.camera.lock().set_frame_format(fourcc)
    }

    /// Gets the current supported list of [`KnownCameraControls`]
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn supported_camera_controls(&self) -> Result<Vec<KnownCameraControls>, NokhwaError> {
        self.camera.lock().supported_camera_controls()
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

        for (kc, cc) in maybe_camera_controls {
            control_map.insert(kc, cc);
        }

        Ok(control_map)
    }

    /// Gets the current supported list of [`CameraControl`]s keyed by its name as a `String`.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn camera_controls_known_camera_controls(
        &self,
    ) -> Result<HashMap<KnownCameraControls, CameraControl>, NokhwaError> {
        let known_controls = self.supported_camera_controls()?;
        let maybe_camera_controls = known_controls
            .iter()
            .map(|x| (*x, self.camera_control(*x)))
            .filter(|(_, x)| x.is_ok())
            .map(|(c, x)| (c, Result::unwrap(x)))
            .collect::<Vec<(KnownCameraControls, CameraControl)>>();
        let mut control_map = HashMap::with_capacity(maybe_camera_controls.len());

        for (kc, cc) in maybe_camera_controls {
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
        control: KnownCameraControls,
    ) -> Result<CameraControl, NokhwaError> {
        self.camera.lock().camera_control(control)
    }

    /// Sets the control to `control` in the camera.
    /// Usually, the pipeline is calling [`camera_control()`](crate::CaptureBackendTrait::camera_control()), getting a camera control that way
    /// then calling one of the methods to set the value: [`set_value()`](CameraControl::set_value()) or [`with_value()`](CameraControl::with_value()).
    /// # Errors
    /// If the `control` is not supported, the value is invalid (less than min, greater than max, not in step), or there was an error setting the control,
    /// this will error.
    pub fn set_camera_control(&mut self, control: CameraControl) -> Result<(), NokhwaError> {
        self.camera.lock().set_camera_control(control)
    }

    /// Gets the current supported list of Controls as an `Any` from the backend.
    /// The `Any`'s type is defined by the backend itself, please check each of the backend's documentation.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn raw_supported_camera_controls(&self) -> Result<Vec<Box<dyn Any>>, NokhwaError> {
        self.camera.lock().raw_supported_camera_controls()
    }

    /// Sets the control to `control` in the camera.
    /// The control's type is defined the backend itself. It may be a string, or more likely its a integer ID.
    /// The backend itself has documentation of the proper input/return values, please check each of the backend's documentation.
    /// # Errors
    /// If the `control` is not supported or there is an error while getting the camera control values (e.g. unexpected value, too high, wrong Any type)
    /// this will error.
    pub fn raw_camera_control(&self, control: &dyn Any) -> Result<Box<dyn Any>, NokhwaError> {
        self.camera.lock().raw_camera_control(control)
    }

    /// Sets the control to `control` in the camera.
    /// The `control`/`value`'s type is defined the backend itself. It may be a string, or more likely its a integer ID/Value.
    /// Usually, the pipeline is calling [`camera_control()`](crate::CaptureBackendTrait::camera_control()), getting a camera control that way
    /// then calling one of the methods to set the value: [`set_value()`](CameraControl::set_value()) or [`with_value()`](CameraControl::with_value()).
    /// # Errors
    /// If the `control` is not supported, the value is invalid (wrong Any type, backend refusal), or there was an error setting the control,
    /// this will error.
    pub fn set_raw_camera_control(
        &mut self,
        control: &dyn Any,
        value: &dyn Any,
    ) -> Result<(), NokhwaError> {
        self.camera.lock().set_raw_camera_control(control, value)
    }

    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`frame()`](crate::Camera::frame()) before you call [`open_stream()`](crate::Camera::open_stream()).
    /// The callback will be called every frame.
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    pub fn open_stream<F>(&mut self, mut callback: F) -> Result<(), NokhwaError>
    where
        F: (FnMut(Buffer)) + Send + 'static,
    {
        *self.frame_callback.lock() =
            Some(Box::new(move |image: Buffer| {
                callback(image)
            }));
        self.camera.lock().open_stream()
    }

    /// Sets the frame callback to the new specified function. This function will be called instead of the previous one(s).
    pub fn set_callback<F>(&mut self, mut callback: F)
    where
        F: (FnMut(Buffer)) + Send + 'static,
    {
        *self.frame_callback.lock() =
            Some(Box::new(move |image: Buffer| {
                callback(image)
            }));
    }

    /// Polls the camera for a frame, analogous to [`Camera::frame`](crate::Camera::frame)
    /// # Errors
    /// This will error if the camera fails to capture a frame.
    pub fn poll_frame(&mut self) -> Result<Buffer, NokhwaError> {
        let frame = self.camera.lock().frame()?;
        *self.last_frame_captured.lock() = frame.clone();
        Ok(frame)
    }

    /// Gets the last frame captured by the camera.
    #[must_use]
    pub fn last_frame(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        self.last_frame_captured.lock().clone()
    }

    /// Checks if stream if open. If it is, it will return true.
    #[must_use]
    pub fn is_stream_open(&self) -> bool {
        self.camera.lock().is_stream_open()
    }

    /// Will drop the stream.
    /// # Errors
    /// Please check the `Quirks` section of each backend.
    pub fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        self.camera.lock().stop_stream()
    }
}

impl<C> Drop for CallbackCamera<C>
where
    C: CaptureBackendTrait,
{
    fn drop(&mut self) {
        let _stop_stream_err = self.stop_stream();
        self.die_bool.store(true, Ordering::SeqCst);
    }
}

fn camera_frame_thread_loop<C: CaptureBackendTrait>(
    camera: &AtomicLock<Camera<C>>,
    frame_callback: &HeldCallbackType,
    last_frame_captured: &AtomicLock<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    die_bool: &Arc<AtomicBool>,
) {
    loop {
        if let Ok(img) = camera.lock().fr {
            *last_frame_captured.lock() = img.clone();
            if let Some(cb) = (*frame_callback.lock()).as_mut() {
                cb(img);
            }
        }
        if die_bool.load(Ordering::SeqCst) {
            break;
        }
    }
}
