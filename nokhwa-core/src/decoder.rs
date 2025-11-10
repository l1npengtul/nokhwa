use crate::error::NokhwaError;
use crate::frame_buffer::FrameBuffer;
use crate::image::{DecodedImage, NonFloatScalarWidth};
use crate::pixel_destination::PixelDestination;
use crate::types::Resolution;
use bytemuck::try_cast_slice_mut;
pub use image::{ImageBuffer, Pixel, Primitive};
use std::fmt::Debug;

pub trait Decoder {
    type Config: Clone + Debug + ConfigHasResolution;
    type OutputMeta: Clone + Debug;
    const SUPPORTED_DESTINATIONS: &'static [PixelDestination];

    fn config(&self) -> &Self::Config;

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError>;

    fn decode_to_buffer(
        &mut self,
        to_decode: FrameBuffer,
        buffer: impl AsMut<[u8]>,
        destination_format: PixelDestination,
    ) -> Result<Self::OutputMeta, NokhwaError>;

    fn decode_to_pixel_buffer<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
        mut buffer: impl AsMut<[P::Subpixel]>,
    ) -> Result<Self::OutputMeta, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        let Some(destination) = PixelDestination::get_by_pixel::<P>() else {
            return Err(NokhwaError::DecoderUnknownDestinationPixelFormat(
                P::COLOR_MODEL,
                P::Subpixel::WIDTH_BYTES,
            ));
        };

        if !Self::supports_destination(destination) {
            return Err(NokhwaError::DecoderUnsupportedDestinationPixelFormat(
                destination,
            ));
        }

        let buffer = buffer.as_mut();

        let cast_slice = try_cast_slice_mut::<P::Subpixel, u8>(buffer)
            .map_err(|why| NokhwaError::DecoderInvalidBuffer(why.to_string()))?;

        self.decode_to_buffer(to_decode, cast_slice, destination)
    }
    fn decode<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer<'_>,
    ) -> Result<DecodedImage<P, Self::OutputMeta>, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        let resolution = self.config().resolution();
        let min_size_alloc = self.output_decoder_min_size_pixel::<P>(resolution)?;
        let mut out_buffer: Vec<P::Subpixel> = vec![P::Subpixel::DEFAULT_MIN_VALUE; min_size_alloc];
        let meta = self.decode_to_pixel_buffer::<P>(to_decode, &mut out_buffer)?;
        Ok(DecodedImage::new(
            ImageBuffer::from_vec(resolution.width(), resolution.height(), out_buffer).ok_or(
                NokhwaError::Decoder("failed to convert into an image buffer".to_string()),
            )?,
            meta,
        ))
    }

    fn output_decoder_min_size_pixel<P>(&self, resolution: Resolution) -> Result<usize, NokhwaError>
    where
        P: Pixel,
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        PixelDestination::get_by_pixel::<P>()
            .map(|dest| self.output_decoder_min_size(resolution, dest))
            .ok_or(NokhwaError::DecoderUnknownDestinationPixelFormat(
                P::COLOR_MODEL,
                P::Subpixel::WIDTH_BYTES,
            ))?
    }

    fn output_decoder_min_size(
        &self,
        resolution: Resolution,
        destination_format: PixelDestination,
    ) -> Result<usize, NokhwaError> {
        if !Self::supports_destination(destination_format) {
            return Err(NokhwaError::DecoderUnsupportedDestinationPixelFormat(
                destination_format,
            ));
        }

        let px_size = match destination_format {
            PixelDestination::Rgb8 | PixelDestination::Bgr8 => 3_u32,
            PixelDestination::Rgba8 | PixelDestination::Bgra8 | PixelDestination::LumaA16 => 4_u32,
            PixelDestination::Rgb16 | PixelDestination::Bgr16 => 3_u32 * 2_u32,
            PixelDestination::Rgba16 | PixelDestination::Bgra16 => 4_u32 * 2_u32,
            PixelDestination::Luma8 => 1_u32,
            PixelDestination::LumaA8 | PixelDestination::Luma16 => 2_u32,
        };
        let reso = resolution.width() * resolution.height();
        Ok((reso as usize) * (px_size as usize))
    }

    #[must_use]
    fn supports_destination(pixel_destination: PixelDestination) -> bool {
        Self::SUPPORTED_DESTINATIONS.contains(&pixel_destination)
    }
}

pub trait ConfigHasResolution {
    fn resolution(&self) -> Resolution;
}
