use crate::error::NokhwaError;
use crate::frame_buffer::FrameBuffer;
use crate::image::{DecodedImage, NonFloatScalarWidth};
use crate::types::{CameraFormat, Resolution};
pub use image::{ImageBuffer, Pixel, Primitive};
use std::fmt::Debug;

pub trait Decoder {
    type Config: Clone + Debug;
    type OutputMeta: Clone + Debug;
    type DestinationFormatHint: Clone + Debug;

    fn config(&self) -> &Self::Config;

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError>;

    fn decode_to_buffer(
        &mut self,
        to_decode: FrameBuffer,
        buffer: impl AsMut<[u8]>,
        destination_format_hint: Option<Self::DestinationFormatHint>,
    ) -> Result<Self::OutputMeta, NokhwaError>;

    fn decode_to_pixel_buffer<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
        buffer: impl AsMut<[P::Subpixel]>,
    ) -> Result<Self::OutputMeta, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth;

    fn decode<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
    ) -> Result<DecodedImage<P, Self::OutputMeta>, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth;
    
    fn output_decoder_min_size_pixel<P>(&self, resolution: Resolution) -> usize where
        P: Pixel,
        <P as Pixel>::Subpixel: NonFloatScalarWidth {
        let channels = P::CHANNEL_COUNT as usize;
        let width_bytes = <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES;
        let resolution_mult = resolution.height() * resolution.width();
        (resolution_mult as usize) * (width_bytes as usize) * channels
    }
    
    fn output_decoder_min_size(&self, resolution: Resolution, destination_format: Self::DestinationFormatHint) -> usize;

    fn buffer_takes_destination_hint(&self) -> bool;
}
