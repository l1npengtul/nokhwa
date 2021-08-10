/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
    CameraFormat, CameraInfo, CaptureAPIBackend, CaptureBackendTrait, FrameFormat, NokhwaError,
    Resolution,
};
use image::{buffer::ConvertBuffer, ImageBuffer, Rgb, RgbaImage};
use std::{cell::RefCell, collections::HashMap};
#[cfg(feature = "output-wgpu")]
use wgpu::{
    Device as WgpuDevice, Extent3d, ImageCopyTexture, ImageDataLayout, Queue as WgpuQueue,
    Texture as WgpuTexture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
};

/// The main `Camera` struct. This is the struct that abstracts over all the backends, providing a simplified interface for use.
pub struct Camera {
    idx: usize,
    backend: RefCell<Box<dyn CaptureBackendTrait>>,
    backend_api: CaptureAPIBackend,
}

#[allow(clippy::nonminimal_bool)]
impl Camera {
    /// Create a new camera from an `index`, `format`, and `backend`. `format` can be `None`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn new(
        index: usize,
        format: Option<CameraFormat>,
        backend: CaptureAPIBackend,
    ) -> Result<Self, NokhwaError> {
        let camera_backend = init_camera(index, format, backend)?;

        Ok(Camera {
            idx: index,
            backend: RefCell::new(camera_backend),
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
        Camera::new(index, Some(camera_format), backend)
    }

    /// Gets the current Camera's index.
    pub fn index(&self) -> usize {
        self.idx
    }

    /// Sets the current Camera's index. Note that this re-initializes the camera.
    /// # Errors
    /// The Backend may fail to initialize.
    pub fn set_index(&mut self, new_idx: usize) -> Result<(), NokhwaError> {
        {
            self.backend.borrow_mut().stop_stream()?;
        }
        let new_camera_format = self.backend.borrow().camera_format();
        let new_camera = init_camera(new_idx, Some(new_camera_format), self.backend_api)?;
        *self.backend.borrow_mut() = new_camera;
        Ok(())
    }

    /// Gets the current Camera's backend
    pub fn backend(&self) -> CaptureAPIBackend {
        self.backend_api
    }

    /// Sets the current Camera's backend. Note that this re-initializes the camera.
    /// # Errors
    /// The new backend may not exist or may fail to initialize the new camera.
    pub fn set_backend(&mut self, new_backend: CaptureAPIBackend) -> Result<(), NokhwaError> {
        {
            self.backend.borrow_mut().stop_stream()?;
        }
        let new_camera_format = self.backend.borrow().camera_format();
        let new_camera = init_camera(self.idx, Some(new_camera_format), new_backend)?;
        *self.backend.borrow_mut() = new_camera;
        Ok(())
    }

    /// Gets the camera information such as Name and Index as a [`CameraInfo`].
    pub fn info(&self) -> CameraInfo {
        self.backend.borrow().camera_info()
    }
    /// Gets the current [`CameraFormat`].
    pub fn camera_format(&self) -> CameraFormat {
        self.backend.borrow().camera_format()
    }
    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    pub fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().set_camera_format(new_fmt)
    }
    /// A hashmap of [`Resolution`]s mapped to framerates
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a [`UnsupportedOperationError`](crate::NokhwaError::UnsupportedOperationError).
    pub fn compatible_list_by_resolution(
        &self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        self.backend.borrow().compatible_list_by_resolution(fourcc)
    }
    /// A Vector of compatible [`FrameFormat`]s.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a [`UnsupportedOperationError`](crate::NokhwaError::UnsupportedOperationError).
    pub fn compatible_fourcc(&self) -> Result<Vec<FrameFormat>, NokhwaError> {
        self.backend.borrow_mut().compatible_fourcc()
    }
    /// Gets the current camera resolution (See: [`Resolution`], [`CameraFormat`]).
    pub fn resolution(&self) -> Resolution {
        self.backend.borrow().resolution()
    }
    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    pub fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().set_resolution(new_res)
    }
    /// Gets the current camera framerate (See: [`CameraFormat`]).
    pub fn frame_rate(&self) -> u32 {
        self.backend.borrow().frame_rate()
    }
    /// Will set the current framerate
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new framerate, this will return an error.
    pub fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().set_frame_rate(new_fps)
    }
    /// Gets the current camera's frame format (See: [`FrameFormat`], [`CameraFormat`]).
    pub fn frame_format(&self) -> FrameFormat {
        self.backend.borrow().frame_format()
    }
    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame format, this will return an error.
    pub fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().set_frame_format(fourcc)
    }
    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`frame()`](CaptureBackendTrait::frame()) before you call [`open_stream()`](CaptureBackendTrait::open_stream()).
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    pub fn open_stream(&mut self) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().open_stream()
    }
    /// Checks if stream if open. If it is, it will return true.
    pub fn is_stream_open(&self) -> bool {
        self.backend.borrow().is_stream_open()
    }
    /// Will get a frame from the camera as a Raw RGB image buffer. Depending on the backend, if you have not called [`open_stream()`](CaptureBackendTrait::open_stream()) before you called this,
    /// it will either return an error.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), the decoding fails (e.g. MJPEG -> u8), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet,
    /// this will error.
    pub fn frame(&self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        self.backend.borrow_mut().frame()
    }
    /// Will get a frame from the camera **without** any processing applied, meaning you will usually get a frame you need to decode yourself.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet, this will error.
    pub fn frame_raw(&self) -> Result<Vec<u8>, NokhwaError> {
        match self.backend.borrow_mut().frame_raw() {
            Ok(f) => Ok(f.to_vec()),
            Err(why) => Err(why),
        }
    }

    /// The minimum buffer size needed to write the current frame (RGB24). If `rgba` is true, it will instead return the minimum size of the RGBA buffer needed.
    pub fn min_buffer_size(&self, rgba: bool) -> usize {
        let resolution = self.backend.borrow().resolution();
        if rgba {
            return (resolution.width() * resolution.height() * 4) as usize;
        }
        (resolution.width() * resolution.height() * 3) as usize
    }
    /// Directly writes the current frame(RGB24) into said `buffer`. If `convert_rgba` is true, the buffer written will be written as an RGBA frame instead of a RGB frame. Returns the amount of bytes written on successful capture.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet, this will error.
    pub fn frame_to_buffer(
        &self,
        buffer: &mut [u8],
        convert_rgba: bool,
    ) -> Result<usize, NokhwaError> {
        let frame = self.frame()?;
        let mut frame_data = frame.to_vec();
        if convert_rgba {
            let rgba_image: RgbaImage = frame.convert();
            frame_data = rgba_image.to_vec();
        }
        let bytes = frame_data.len();
        buffer.copy_from_slice(&frame_data);
        Ok(bytes)
    }

    #[cfg(feature = "output-wgpu")]
    /// Directly copies a frame to a Wgpu texture. This will automatically convert the frame into a RGBA frame.
    /// # Errors
    /// If the frame cannot be captured or the resolution is 0 on any axis, this will error.
    pub fn frame_texture<'a>(
        &mut self,
        device: &WgpuDevice,
        queue: &WgpuQueue,
        label: Option<&'a str>,
    ) -> Result<WgpuTexture, NokhwaError> {
        use std::{convert::TryFrom, num::NonZeroU32};
        let frame = self.frame()?;
        let rgba_frame: RgbaImage = frame.convert();

        let texture_size = Extent3d {
            width: frame.width(),
            height: frame.height(),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });

        let width_nonzero = match NonZeroU32::try_from(4 * rgba_frame.width()) {
            Ok(w) => Some(w),
            Err(why) => return Err(NokhwaError::ReadFrameError(why.to_string())),
        };

        let height_nonzero = match NonZeroU32::try_from(rgba_frame.height()) {
            Ok(h) => Some(h),
            Err(why) => return Err(NokhwaError::ReadFrameError(why.to_string())),
        };

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba_frame.to_vec(),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: width_nonzero,
                rows_per_image: height_nonzero,
            },
            texture_size,
        );

        Ok(texture)
    }

    /// Will drop the stream.
    /// # Errors
    /// Please check the `Quirks` section of each backend.
    pub fn stop_stream(&self) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().stop_stream()
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
    } else if cfg!(feature = "input-avfoundationn") && platform == "mac" {
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
        $( ($backend:ty, $init_fn:ident, $feature:expr, $backend_name:ident) ),+
    } => {
        $(
            paste::paste! {
                #[cfg(feature = $feature)]
                fn [< init_ $backend_name>](idx: usize, setting: Option<CameraFormat>) -> Option<Result<Box<dyn CaptureBackendTrait>, NokhwaError>> {
                    use crate::backends::capture::$backend;
                    match <$backend>::$init_fn(idx, setting) {
                        Ok(cap) => Some(Ok(Box::new(cap))),
                        Err(why) => Some(Err(why)),
                    }
                }
                #[cfg(not(feature = $feature))]
                fn [< init_ $backend_name>](_idx: usize, _setting: Option<CameraFormat>) -> Option<Result<Box<dyn CaptureBackendTrait>, NokhwaError>> {
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
                                                    "Platform requirements not satisfied.".to_string(),
                                                ));
                                            }
                                        }
                                    }
                                    false => {
                                        return Err(NokhwaError::NotImplementedError(
                                            "Platform requirements not satisfied.".to_string(),
                                        ));
                                    }
                                }
                            }
                        )+
                        _ => {
                            return Err(NokhwaError::NotImplementedError(
                                "Platform requirements not satisfied.".to_string(),
                            ));
                        }
                    }
                    None => {
                        return Err(NokhwaError::NotImplementedError(
                            "Platform requirements not satisfied.".to_string(),
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
                                            "Platform requirements not satisfied.".to_string(),
                                        ));
                                    }
                                }
                            }
                            false => {
                                return Err(NokhwaError::NotImplementedError(
                                    "Platform requirements not satisfied.".to_string(),
                                ));
                            }
                        }
                    }
                )+
                _ => {
                    return Err(NokhwaError::NotImplementedError(
                        "Platform requirements not satisfied.".to_string(),
                    ));
                }
            }
        };
    }
}

cap_impl_fn! {
    (GStreamerCaptureDevice, new, "input-gst", gst),
    (OpenCvCaptureDevice, new_autopref, "input-opencv", opencv),
    (V4LCaptureDevice, new, "input-v4l", v4l),
    // (UVCCaptureDevice, create, "input-uvc", uvc),
    (MediaFoundationCaptureDevice, new, "input-msmf", msmf)
}

fn init_camera(
    index: usize,
    format: Option<CameraFormat>,
    backend: CaptureAPIBackend,
) -> Result<Box<dyn CaptureBackendTrait>, NokhwaError> {
    let camera_backend = cap_impl_matches! {
            backend, index, format,
            ("input-v4l", Video4Linux, init_v4l),
            // ("input-uvc", UniversalVideoClass, init_uvc),
            ("input-gst", GStreamer, init_gst),
            ("input-opencv", OpenCv, init_opencv),
            ("input-msmf", MediaFoundation, init_msmf)
    };
    Ok(camera_backend)
}
