use crate::error::NokhwaError;
use crate::frame_buffer::FrameBuffer;
use crate::types::CameraFormat;
use std::fmt::Debug;
pub use image::{ImageBuffer, Pixel, Primitive};
use crate::image::{DecodedImage, NonFloatScalarWidth};

pub trait Decoder {
    type Config: Clone + Debug + TryFrom<CameraFormat>;
    type OutputMeta: Debug;

    fn config(&self) -> &Self::Config;

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError>;

    fn decode_to_buffer(
        &mut self,
        to_decode: FrameBuffer,
        buffer: impl AsMut<[u8]>,
    ) -> Result<Self::OutputMeta, NokhwaError>;

    fn decode_to_pixel_buffer<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
        buffer: impl AsMut<[P::Subpixel]>,
    ) -> Result<Self::OutputMeta, NokhwaError> 
    where <P as Pixel>::Subpixel: NonFloatScalarWidth; 

    fn decode<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
    ) -> Result<DecodedImage<P, Self::OutputMeta>, NokhwaError>
    where <P as Pixel>::Subpixel: NonFloatScalarWidth;
}
