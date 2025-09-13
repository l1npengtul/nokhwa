use crate::error::NokhwaError;
use crate::frame_buffer::FrameBuffer;
use crate::image::{DecodedImage, NonFloatScalarWidth};
use crate::types::{Resolution};
pub use image::{ImageBuffer, Pixel, Primitive};
use std::fmt::Debug;
use bytemuck::try_cast_slice_mut;
use crate::pixel_destination::PixelDestination;

pub trait Decoder {
    type Config: Clone + Debug;
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

    fn decode_to_pixel_buffer<P: Pixel>(&mut self, to_decode: FrameBuffer, mut buffer: impl AsMut<[P::Subpixel]>) -> Result<Self::OutputMeta, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth
    {
        let destination = match PixelDestination::get_by_pixel::<P>() {
            Some(dest) => dest,
            None => return Err(NokhwaError::DecoderUnknownDestinationPixelFormat(P::COLOR_MODEL, P::Subpixel::WIDTH_BYTES))
        };

        if !Self::supports_destination(destination) {
            return Err(NokhwaError::DecoderUnsupportedDestinationPixelFormat(destination))
        }

        let buffer = buffer.as_mut();

        let cast_slice = try_cast_slice_mut::<P::Subpixel, u8>(buffer)
            .map_err(|why| NokhwaError::DecoderInvalidBuffer(why.to_string()))?;

        self.decode_to_buffer(to_decode, cast_slice, destination)
    }

    fn decode<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
    ) -> Result<DecodedImage<P, Self::OutputMeta>, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth;
    
    fn output_decoder_min_size_pixel<P>(&self, resolution: Resolution) -> Result<usize, NokhwaError> where
        P: Pixel,
        <P as Pixel>::Subpixel: NonFloatScalarWidth {
        PixelDestination::get_by_pixel::<P>().map(|dest| self.output_decoder_min_size(resolution, dest)).ok_or(NokhwaError::DecoderUnknownDestinationPixelFormat(P::COLOR_MODEL, P::Subpixel::WIDTH_BYTES))?

    }
    
    fn output_decoder_min_size(&self, resolution: Resolution, destination_format: PixelDestination) -> Result<usize, NokhwaError>;

    fn supports_destination(pixel_destination: PixelDestination) -> bool {
        Self::SUPPORTED_DESTINATIONS.contains(&pixel_destination)
    }
}
