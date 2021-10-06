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
    Camera, CameraFormat, CameraInfo, CaptureAPIBackend, FrameFormat, NokhwaError, Resolution,
};
use image::{ImageBuffer, Rgb};
use parking_lot::FairMutex;
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

/// Creates a camera that runs in a different thread that you can use a callback to access the frames of.
/// It uses a `Arc` and a `FairMutex` to ensure that this feels like a normal camera, but callback based.
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
#[derive(Clone)]
pub struct ThreadedCamera {
    camera: Arc<FairMutex<Camera>>,
    frame_callback: Arc<FairMutex<Option<fn(ImageBuffer<Rgb<u8>, Vec<u8>>)>>>,
    die_bool: Arc<AtomicBool>,
}

impl ThreadedCamera {
    /// Create a new camera from an `index` and `format`. `format` can be `None`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn new(index: usize, format: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        ThreadedCamera::with_backend(index, format, CaptureAPIBackend::Auto)
    }

    /// Create a new camera from an `index`, `format`, and `backend`. `format` can be `None`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn with_backend(
        index: usize,
        format: Option<CameraFormat>,
        backend: CaptureAPIBackend,
    ) -> Result<Self, NokhwaError> {
        let camera = Arc::new(FairMutex::new(Camera::with_backend(
            index, format, backend,
        )?));
        let frame_callback = Arc::new(FairMutex::new(None));
        let die_bool = Arc::new(AtomicBool::new(false));

        let die_clone = die_bool.clone();
        let camera_clone = camera.clone();
        let callback_clone = frame_callback.clone();
        std::thread::spawn(move || {
            camera_frame_thread_loop(camera_clone, callback_clone, die_clone)
        });

        Ok(ThreadedCamera {
            camera,
            frame_callback,
            die_bool,
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
        ThreadedCamera::with_backend(index, Some(camera_format), backend)
    }

    /// Gets the current Camera's index.
    #[must_use]
    pub fn index(&self) -> usize {
        self.camera.lock().index()
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
    pub fn camera_format(&self) -> CameraFormat {
        self.camera.lock().camera_format()
    }

    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    pub fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
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
    pub fn resolution(&self) -> Resolution {
        self.camera.lock().resolution()
    }

    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    pub fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        self.camera.lock().set_resolution(new_res)
    }

    /// Gets the current camera framerate (See: [`CameraFormat`]).
    #[must_use]
    pub fn frame_rate(&self) -> u32 {
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
    pub fn frame_format(&self) -> FrameFormat {
        self.camera.lock().frame_format()
    }

    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame format, this will return an error.
    pub fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        self.camera.lock().set_frame_format(fourcc)
    }

    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`frame()`](crate::Camera::frame()) before you call [`open_stream()`](crate::Camera::open_stream()).
    /// The callback will be called every frame.
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    pub fn open_stream(
        &mut self,
        callback: fn(ImageBuffer<Rgb<u8>, Vec<u8>>),
    ) -> Result<(), NokhwaError> {
        *self.frame_callback.lock() = Some(callback);
        self.camera.lock().open_stream()
    }

    pub fn set_callback(&mut self, callback: fn(ImageBuffer<Rgb<u8>, Vec<u8>>)) {
        *self.frame_callback.lock() = Some(callback);
    }

    /// Checks if stream if open. If it is, it will return true.
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

impl Drop for ThreadedCamera {
    fn drop(&mut self) {
        let _ = self.stop_stream();
        self.die_bool.store(true, Ordering::SeqCst);
    }
}

fn camera_frame_thread_loop(
    camera: Arc<FairMutex<Camera>>,
    callback: Arc<FairMutex<Option<fn(ImageBuffer<Rgb<u8>, Vec<u8>>)>>>,
    die_bool: Arc<AtomicBool>,
) {
    loop {
        if let Ok(img) = camera.lock().frame() {
            if let Some(cb) = callback.lock().deref() {
                cb(img)
            }
        }
        if die_bool.load(Ordering::SeqCst) {
            break;
        }
    }
}
