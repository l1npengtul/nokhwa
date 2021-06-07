use crate::{backends::capture::OpenCvCaptureDevice, CaptureBackendTrait, NokhwaError};
use image::{buffer::ConvertBuffer, Bgr, ImageBuffer, Rgb, RgbaImage};
use std::{cell::RefCell, convert::TryFrom, num::NonZeroU32};
#[cfg(feature = "output-wgpu")]
use wgpu::{
    Device as WgpuDevice, Extent3d, ImageCopyTexture, ImageDataLayout, Queue as WgpuQueue,
    Texture as WgpuTexture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
};

/// A struct that supports IP Cameras via the `OpenCV` backend.
pub struct NetworkCamera {
    ip: String,
    opencv_backend: RefCell<OpenCvCaptureDevice>,
}

impl NetworkCamera {
    /// Creates a new [`NetworkCamera`] from an IP.
    /// # Errors
    /// If the IP is invalid or `OpenCV` fails to open the IP, this will error
    pub fn new(ip: String) -> Result<Self, NokhwaError> {
        let opencv_camera = OpenCvCaptureDevice::new_ip_camera(ip.clone())?;
        Ok(NetworkCamera {
            ip,
            opencv_backend: RefCell::new(opencv_camera),
        })
    }

    /// Gets the IP string
    pub fn ip(&self) -> String {
        self.ip.clone()
    }

    /// Sets the IP. Will restart stream if already started.
    /// # Errors
    /// If the IP is invalid or `OpenCV` fails to open the IP, this will error
    pub fn set_ip(&mut self, ip: String) -> Result<(), NokhwaError> {
        *self.opencv_backend.borrow_mut() = OpenCvCaptureDevice::new_ip_camera(ip.clone())?;
        self.ip = ip;
        Ok(())
    }

    /// Opens stream.
    /// # Errors
    /// If the backend fails to capture the stream this will error
    pub fn open_stream(&self) -> Result<(), NokhwaError> {
        self.opencv_backend.borrow_mut().open_stream()
    }

    /// Gets the frame decoded as a RGB24 frame
    /// # Errors
    /// If the backend fails to capture the stream, or if the decoding fails this will error
    pub fn get_frame(&self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        self.opencv_backend.borrow_mut().get_frame()
    }

    /// The minimum buffer size needed to write the current frame (RGB24). If `rgba` is true, it will instead return the minimum size of the RGBA buffer needed.
    pub fn min_buffer_size(&self, rgba: bool) -> usize {
        let resolution = self.opencv_backend.borrow().resolution();
        if rgba {
            return (resolution.width() * resolution.height() * 4) as usize;
        }
        (resolution.width() * resolution.height() * 3) as usize
    }
    /// Directly writes the current frame(RGB24) into said `buffer`. If `convert_rgba` is true, the buffer written will be written as an RGBA frame instead of a RGB frame. Returns the amount of bytes written on successful capture.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet, this will error.
    pub fn get_frame_to_buffer(
        &self,
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
    pub fn get_frame_texture<'a>(
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
    pub fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        self.opencv_backend.borrow_mut().stop_stream()
    }
}
