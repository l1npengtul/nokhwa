use std::ops::Deref;
use image::{ImageBuffer, Pixel};
use serde::de::Error;
use crate::buffer::Buffer;
use crate::frame_format::{SourceFrameFormat};

/// Trait to define a struct that can decode a [`Buffer`]
pub trait Decoder {
    /// Formats that the decoder can decode.
    const ALLOWED_FORMATS: &'static [SourceFrameFormat];
    /// Output pixel type (e.g. [`Rgb<u8>`](image::Rgb))
    type Pixel: Pixel;
    /// Container for [`Self::Pixel`] - must have the same [`Pixel::Subpixel`]
    type Container: Deref<Target = [Pixel::Subpixel]>;
    /// Error that the decoder will output (use [`NokhwaError`] if you're not sure)
    type Error: Error;

    /// Decode function.
    fn decode(&mut self, buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error>;
}

/// Decoder that can be used statically (struct contains no state)
/// 
/// This is useful for times that a simple function is all that is required. 
pub trait StaticDecoder: Decoder {
    fn decode_static(buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error>;
}

/// Decoder that does not change its internal state. 
pub trait IdemptDecoder: Decoder {
    fn decode_nm(buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error>;
}

