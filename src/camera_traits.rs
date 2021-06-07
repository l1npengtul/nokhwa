use crate::{
    error::NokhwaError,
    utils::{CameraFormat, CameraInfo, FrameFormat, Resolution},
};
use image::{buffer::ConvertBuffer, ImageBuffer, Rgb, RgbaImage};
use std::{collections::HashMap, convert::TryFrom, num::NonZeroU32};

#[cfg(feature = "output-wgpu")]
use wgpu::{
    Device as WgpuDevice, Extent3d, ImageCopyTexture, ImageDataLayout, Queue as WgpuQueue,
    Texture as WgpuTexture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
};

/// This trait is for any backend that allows you to grab and take frames from a camera.
/// Many of the backends are **blocking**, if the camera is occupied the library will block while it waits for it to become available.
///
/// **Note**:
/// - Backends, if not provided with a camera format, will be spawned with 640x480@15 FPS, MJPEG [`CameraFormat`].
/// - Behaviour can differ from backend to backend. While the [`Camera`](crate::camera::Camera) struct abstracts most of this away, if you plan to use the raw backend structs please read the `Quirks` section of each backend.
/// - If you call [`stop_stream()`](CaptureBackendTrait::stop_stream()), you will usually need to call [`open_stream()`](CaptureBackendTrait::open_stream()) to get more frames from the camera.
pub trait CaptureBackendTrait {
    /// Gets the camera information such as Name and Index as a [`CameraInfo`].
    fn camera_info(&self) -> CameraInfo;

    /// Gets the current [`CameraFormat`].
    fn camera_format(&self) -> CameraFormat;

    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError>;

    /// A hashmap of [`Resolution`]s mapped to framerates
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a Unsupported Operation ([`UnsupportedOperation`](crate::NokhwaError::UnsupportedOperation)).
    fn get_compatible_list_by_resolution(
        &self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError>;

    /// A Vector of compatible [`FrameFormat`]s.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a Unsupported Operation ([`UnsupportedOperation`](crate::NokhwaError::UnsupportedOperation)).
    fn get_compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError>;

    /// Gets the current camera resolution (See: [`Resolution`], [`CameraFormat`]).
    fn resolution(&self) -> Resolution;

    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError>;

    /// Gets the current camera framerate (See: [`CameraFormat`]).
    fn frame_rate(&self) -> u32;

    /// Will set the current framerate
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new framerate, this will return an error.
    fn set_framerate(&mut self, new_fps: u32) -> Result<(), NokhwaError>;

    /// Gets the current camera's frame format (See: [`FrameFormat`], [`CameraFormat`]).
    fn frameformat(&self) -> FrameFormat;

    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame format, this will return an error.
    fn set_frameformat(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError>;

    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`get_frame()`](CaptureBackendTrait::get_frame()) before you call [`open_stream()`](CaptureBackendTrait::open_stream()).
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    fn open_stream(&mut self) -> Result<(), NokhwaError>;

    /// Checks if stream if open. If it is, it will return true.
    fn is_stream_open(&self) -> bool;

    /// Will get a frame from the camera as a Raw RGB image buffer. Depending on the backend, if you have not called [`open_stream()`](CaptureBackendTrait::open_stream()) before you called this,
    /// it will either return an error.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), the decoding fails (e.g. MJPEG -> u8), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet,
    /// this will error.
    fn get_frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError>;

    /// Will get a frame from the camera **without** any processing applied, meaning you will usually get a frame you need to decode yourself.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet, this will error.
    fn get_frame_raw(&mut self) -> Result<Vec<u8>, NokhwaError>;

    /// The minimum buffer size needed to write the current frame (RGB24). If `rgba` is true, it will instead return the minimum size of the RGBA buffer needed.
    fn min_buffer_size(&self, rgba: bool) -> usize {
        let resolution = self.resolution();
        if rgba {
            return (resolution.width() * resolution.height() * 4) as usize;
        }
        (resolution.width() * resolution.height() * 3) as usize
    }

    /// Directly writes the current frame(RGB24) into said `buffer`. If `convert_rgba` is true, the buffer written will be written as an RGBA frame instead of a RGB frame. Returns the amount of bytes written on successful capture.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet, this will error.
    fn get_frame_to_buffer(
        &mut self,
        buffer: &mut [u8],
        convert_rgba: bool,
    ) -> Result<usize, NokhwaError> {
        let frame = self.get_frame()?;
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
    fn get_frame_texture<'a>(
        &mut self,
        device: &WgpuDevice,
        queue: &WgpuQueue,
        label: Option<&'a str>,
    ) -> Result<WgpuTexture, NokhwaError> {
        let frame = self.get_frame()?;
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
            Err(why) => return Err(NokhwaError::CouldntCaptureFrame(why.to_string())),
        };

        let height_nonzero = match NonZeroU32::try_from(rgba_frame.height()) {
            Ok(h) => Some(h),
            Err(why) => return Err(NokhwaError::CouldntCaptureFrame(why.to_string())),
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
    fn stop_stream(&mut self) -> Result<(), NokhwaError>;
}

pub trait VirtualBackendTrait {}
