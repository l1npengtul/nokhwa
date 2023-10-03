use image::{ImageBuffer, Rgb};
use nokhwa_core::buffer::Buffer;
use nokhwa_core::decoder::{Decoder, IdemptDecoder, StaticDecoder};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_format::{FrameFormat, SourceFrameFormat};

#[inline]
fn decompress(
    data: &[u8],
    rgba: bool,
) -> Result<, NokhwaError> {
    use mozjpeg::Decompress;

    match Decompress::new_mem(data) {
        Ok(decompress) => {
            let decompressor_res = if rgba {
                decompress.rgba()
            } else {
                decompress.rgb()
            };
            match decompressor_res {
                Ok(decompressor) => Ok(decompressor),
                Err(why) => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::MJpeg,
                        destination: "RGB888".to_string(),
                        error: why.to_string(),
                    })
                }
            }
        }
        Err(why) => {
            return Err(NokhwaError::ProcessFrameError {
                src: FrameFormat::MJpeg,
                destination: "RGB888".to_string(),
                error: why.to_string(),
            })
        }
    }
}


pub struct MJPegDecoder;

impl Decoder for MJPegDecoder {
    const ALLOWED_FORMATS: &'static [SourceFrameFormat] = &[SourceFrameFormat::FrameFormat(FrameFormat::MJpeg)];
    type Pixel = Rgb<u8>;
    type Container = Vec<u8>;
    type Error = NokhwaError;

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

impl StaticDecoder for MJPegDecoder {
    fn decode_static(buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }

    fn decode_static_to_buffer(buffer: &mut [Pixel::Subpixel]) -> Result<(), Self::Error> {
        todo!()
    }
}

impl IdemptDecoder for MJPegDecoder {
    fn decode_nm(&self, buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }

    fn decode_nm_to_buffer(&self, buffer: &mut [Pixel::Subpixel]) -> Result<(), Self::Error> {
        todo!()
    }
}
