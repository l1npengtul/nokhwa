use image::{ImageBuffer, Pixel};
use crate::buffer::Buffer;
use crate::frame_format::FrameFormat;

/// Trait to define a struct that can decode a [`Buffer`]
pub trait Decoder {
    /// Formats that the decoder can decode.
    const ALLOWED_FORMATS: &'static FrameFormat;
    /// Output pixel type (e.g. [`Rgb<u8>`](image::Rgb))
    type OutputPixels: Pixel; 
    
    type PixelContainer;
    /// Error that the decoder will output (use [`NokhwaError`] if you're not sure)
    type Error;

    /// Decode function.
    fn decode(&mut self, buffer: Buffer) -> Result<ImageBuffer<Self::OutputPixels, Self::PixelContainer>, Self::Error>;

    /// Decode to user-provided Buffer
    ///
    /// Incase that the buffer is not large enough this should error.
    fn decode_buffer(&mut self, buffer: &mut [<<Self as Decoder>::OutputPixels as Pixel>::Subpixel]) -> Result<(), Self::Error>;

    /// Decoder Predicted Size
    fn predicted_size_of_frame(&mut self, ) -> Option<usize>;
}

/// Decoder that can be used statically (struct contains no state)
///
/// This is useful for times that a simple function is all that is required.
pub trait StaticDecoder: Decoder {
    fn decode_static(buffer: Buffer) -> Result<ImageBuffer<Self::OutputPixels, Self::PixelContainer>, Self::Error>;

    fn decode_static_to_buffer(buffer: &mut [<<Self as Decoder>::OutputPixels as Pixel>::Subpixel]) -> Result<(), Self::Error>;
}

/// Decoder that does not change its internal state.
pub trait IdemptDecoder: Decoder {
    /// Decoder that does not change its internal state.
    fn decode_nm(&self, buffer: Buffer) -> Result<ImageBuffer<Self::OutputPixels, Self::PixelContainer>, Self::Error>;

    /// Decoder that does not change its internal state, decoding to a user provided buffer.
    fn decode_nm_to_buffer(&self, buffer: &mut [<<Self as Decoder>::OutputPixels as Pixel>::Subpixel]) -> Result<(), Self::Error>;
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait AsyncDecoder: Decoder {
    /// Asynchronous decoder
    async fn decode_async(&mut self, buffer: Buffer) -> Result<ImageBuffer<Self::OutputPixels, Self::PixelContainer>, Self::Error>;

    /// Asynchronous decoder to user buffer.
    async fn decode_buffer(&mut self, buffer: &mut [Self::OutputPixels::Subpixel]) -> Result<(), Self::Error>;
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait AsyncStaticDecoder: Decoder {
    /// Asynchronous decoder
        async fn decode_static_async(buffer: Buffer) -> Result<ImageBuffer<Self::OutputPixels, Self::PixelContainer>, Self::Error>;

    /// Asynchronous decoder to user buffer.
    async fn decode_static_buffer(buffer: &mut [Self::OutputPixels::Subpixel]) -> Result<(), Self::Error>;
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait AsyncIdemptDecoder: Decoder {
    /// Asynchronous decoder
    async fn decode_nm_async(&self, buffer: Buffer) -> Result<ImageBuffer<Self::OutputPixels, Self::PixelContainer>, Self::Error>;

    /// Asynchronous decoder to user buffer.
    async fn decode_nm_buffer(&self, buffer: &mut [Self::OutputPixels::Subpixel]) -> Result<(), Self::Error>;
}
