use image::{ImageBuffer, Rgb};
use nokhwa_core::buffer::Buffer;
use nokhwa_core::decoder::{Decoder, IdemptDecoder, StaticDecoder};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_format::{FrameFormat, SourceFrameFormat};

pub struct MJPegDecoder;

impl Decoder for MJPegDecoder {
    const ALLOWED_FORMATS: &'static [SourceFrameFormat] = &[SourceFrameFormat::FrameFormat(FrameFormat::MJpeg)];
    type Pixel = Rgb<u8>;
    type Container = Vec<u8>;
    type Error = NokhwaError;

    fn decode(&mut self, buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }
}

impl StaticDecoder for MJPegDecoder {
    fn decode_static(buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }
}

impl IdemptDecoder for MJPegDecoder {
    fn decode_nm(buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }
}
