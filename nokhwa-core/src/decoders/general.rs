use dcv_color_primitives::PixelFormat;
use image::{ExtendedColorType, ImageBuffer, Pixel, PixelWithColorType};
use crate::buffer::Buffer;
use crate::decoders::Decoder;
use crate::error::NokhwaError;
use crate::frame_format::FrameFormat;

pub struct GeneralPurposeDecoder<D> where D: PixelWithColorType;

impl<D> Decoder for GeneralPurposeDecoder<D> where D: PixelWithColorType {
    const ALLOWED_FORMATS: &'static [FrameFormat] = &[
        FrameFormat::MJpeg, FrameFormat::Luma8, FrameFormat::Luma16, FrameFormat::Rgb8, FrameFormat::RgbA8,
        FrameFormat::Nv12, FrameFormat::Nv21, FrameFormat::Uyvy_422, FrameFormat::Yuy2_422, FrameFormat::Yv12,
        FrameFormat::Yuv444, FrameFormat::I420, FrameFormat::I422, FrameFormat::I444
    ];

    type OutputPixels = D;
    type PixelContainer = Vec<D::Subpixel>;
    type Error = NokhwaError;

    fn decode(&mut self, buffer: Buffer) -> Result<ImageBuffer<Self::OutputPixels, Self::PixelContainer>, Self::Error> {
        todo!()
    }

    fn decode_buffer(&mut self, buffer: &Buffer, output: &mut [<<Self as Decoder>::OutputPixels as Pixel>::Subpixel]) -> Result<(), Self::Error> {
        if !Self::ALLOWED_FORMATS.contains(&buffer.source_frame_format()) {
            return Err(NokhwaError::ConversionError(format!("Invaid frame format {} (allowed formats: {:?})", buffer.source_frame_format(), Self::ALLOWED_FORMATS)))
        }
        
        let destination = match D::COLOR_TYPE {
            ExtendedColorType::Rgb8 => PixelFormat::Rgb,
            ExtendedColorType::Rgba8 => PixelFormat::Rgba,
            ExtendedColorType::Bgr8 => PixelFormat::Bgr,
            ExtendedColorType::Bgra8 => PixelFormat::Bgra,
            _ => return Err(())
        };

        // some extra processing needed for some formats

        let source = match buffer.source_frame_format() {
            FrameFormat::MJpeg => PixelFormat::Rgb, // => JPEG decoder
            FrameFormat::Yuy2_422 => PixelFormat::I422,
            FrameFormat::Uyvy_422 => PixelFormat::I422,
            FrameFormat::Yuv444 => PixelFormat::I444,
            FrameFormat::Nv12 => PixelFormat::Nv12,
            FrameFormat::Nv21 => PixelFormat::Nv12,
            FrameFormat::Yv12 => PixelFormat::I420,
            FrameFormat::I420 => PixelFormat::I420,
            // already decoded
            FrameFormat::Rgb8 => PixelFormat::Rgb,
            FrameFormat::RgbA8 => {
                PixelFormat::Rgba
            }
            _ => return Err(()),
        };
        
    }
}
