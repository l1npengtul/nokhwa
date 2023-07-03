use image::ImageBuffer;
use nokhwa_core::buffer::Buffer;
use nokhwa_core::decoder::{Decoder, IdemptDecoder, StaticDecoder};
use nokhwa_core::frame_format::SourceFrameFormat;

// For those maintaining this, I recommend you read: https://docs.microsoft.com/en-us/windows/win32/medfound/recommended-8-bit-yuv-formats-for-video-rendering#yuy2
// https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
// and this too: https://stackoverflow.com/questions/16107165/convert-from-yuv-420-to-imagebgr-byte
// The YUY2(Yuv422) format is a 16 bit format. We read 4 bytes at a time to get 6 bytes of RGB888.
// First, the YUY2 is converted to YCbCr 4:4:4 (4:2:2 -> 4:4:4)
// then it is converted to 6 bytes (2 pixels) of RGB888
/// Converts a Yuv422 4:2:2 datastream to a RGB888 Stream. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
/// # Errors
/// This may error when the data stream size is not divisible by 4, a i32 -> u8 conversion fails, or it fails to read from a certain index.
pub struct YUYVDecoder {}

impl Decoder for YUYVDecoder {
    const ALLOWED_FORMATS: &'static [SourceFrameFormat] = &[];
    type Pixel = ();
    type Container = ();
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

impl StaticDecoder for YUYVDecoder {
    fn decode_static(buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }

    fn decode_static_to_buffer(buffer: &mut [Pixel::Subpixel]) -> Result<(), Self::Error> {
        todo!()
    }
}

impl IdemptDecoder for YUYVDecoder {
    fn decode_nm(&self, buffer: Buffer) -> Result<ImageBuffer<Self::Pixel, Self::Container>, Self::Error> {
        todo!()
    }

    fn decode_nm_to_buffer(&self, buffer: &mut [Pixel::Subpixel]) -> Result<(), Self::Error> {
        todo!()
    }
}
