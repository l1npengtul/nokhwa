use image::{ImageBuffer, Rgb};
use nokhwa_core::buffer::Buffer;
use nokhwa_core::decoder::{Decoder, IdemptDecoder, StaticDecoder};
use nokhwa_core::frame_format::SourceFrameFormat;

pub struct NV12Decoder {}

impl Decoder for NV12Decoder {
    const ALLOWED_FORMATS: &'static [SourceFrameFormat] = &[];
    type Pixel = Rgb<u8>;
    type Container = Vec<u8>;
    type Error = ();

    fn decode(&mut self, buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }

    fn decode_buffer(&mut self, buffer: &mut [Pixel::Subpixel]) -> Result<(), Self::Error> {
        todo!()
    }

    fn predicted_size_of_frame(&mut self) -> Option<usize> {
        todo!()
    }
}

impl StaticDecoder for NV12Decoder {
    fn decode_static(buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }

    fn decode_static_to_buffer(buffer: &mut [Pixel::Subpixel]) -> Result<(), Self::Error> {
        todo!()
    }
}

impl IdemptDecoder for NV12Decoder {
    fn decode_nm(buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }
}