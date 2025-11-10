use nokhwa_core::decoder::{ConfigHasResolution, Decoder};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_buffer::FrameBuffer;
use nokhwa_core::frame_format::FrameFormat;
use nokhwa_core::pixel_destination::PixelDestination;
use nokhwa_core::types::{CameraFormat, Resolution};
use zune_core::bytestream::ZCursor;
use zune_core::colorspace::ColorSpace;
pub use zune_core::options::DecoderOptions;
pub use zune_jpeg::ImageInfo;
use zune_jpeg::JpegDecoder;
use zune_jpeg::errors::DecodeErrors;

#[derive(Clone, Debug)]
pub struct MJpegDecoder {
    config: MJpegConfig,
}

impl MJpegDecoder {
    pub fn new(config: MJpegConfig) -> Self {
        MJpegDecoder { config }
    }

    pub fn from_camera_format(camera_format: CameraFormat) -> Result<Self, NokhwaError> {
        let resolution = camera_format.resolution();
        if camera_format.format() != FrameFormat::MJPEG {
            return Err(NokhwaError::DecoderInvalidFrameData(
                "Not MJPEG!".to_string(),
            ));
        }
        let decoder_options = DecoderOptions::new_safe();
        let config = MJpegConfig {
            resolution,
            decoder_options,
        };
        Ok(MJpegDecoder { config })
    }
}

impl Decoder for MJpegDecoder {
    type Config = MJpegConfig;
    type OutputMeta = ImageMeta;
    const SUPPORTED_DESTINATIONS: &'static [PixelDestination] = &[
        PixelDestination::Rgb8,
        PixelDestination::Rgba8,
        PixelDestination::Bgr8,
        PixelDestination::Bgra8,
        PixelDestination::Luma8,
        PixelDestination::LumaA8,
    ];

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        self.config = config;
        Ok(())
    }

    fn decode_to_buffer(
        &mut self,
        to_decode: FrameBuffer,
        mut buffer: impl AsMut<[u8]>,
        destination_format: PixelDestination,
    ) -> Result<Self::OutputMeta, NokhwaError> {
        let buffer = buffer.as_mut();
        let cursor = ZCursor::new(to_decode.as_ref());

        let mut decoder = JpegDecoder::new(cursor);

        let colorspace = convert_destination_to_colorspace(destination_format).ok_or(
            NokhwaError::DecoderUnsupportedDestinationPixelFormat(destination_format),
        )?;

        let config = self
            .config
            .decoder_options
            .jpeg_set_out_colorspace(colorspace);

        decoder.set_options(config);
        decoder.decode_into(buffer).map_err(err_to_err)?;

        let info = match decoder.info() {
            Some(i) => i,
            None => {
                return Err(NokhwaError::Decoder(
                    "??????? how did we get here".to_string(),
                ));
            }
        };

        Ok(info.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageMeta {
    pub resolution: Resolution,
    pub pixel_density: u8,
    pub x_density: u16,
    pub y_density: u16,
    pub components: u8,
    pub multi_picture_information: Option<Vec<u8>>,
}

impl From<ImageInfo> for ImageMeta {
    fn from(value: ImageInfo) -> Self {
        Self {
            resolution: Resolution::new(value.width as u32, value.width as u32),
            pixel_density: value.pixel_density,
            x_density: value.x_density,
            y_density: value.y_density,
            components: value.components,
            multi_picture_information: value.multi_picture_information,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MJpegConfig {
    pub resolution: Resolution,
    pub decoder_options: DecoderOptions,
}

impl ConfigHasResolution for MJpegConfig {
    fn resolution(&self) -> Resolution {
        self.resolution
    }
}

fn err_to_err(decode_errors: DecodeErrors) -> NokhwaError {
    match decode_errors {
        DecodeErrors::Format(fmt) => NokhwaError::Decoder(fmt),
        DecodeErrors::FormatStatic(fmt) => NokhwaError::Decoder(fmt.to_string()),
        DecodeErrors::IllegalMagicBytes(b) => {
            NokhwaError::DecoderInvalidFrameData(format!("bad magic bytes: {b}"))
        }
        DecodeErrors::HuffmanDecode(huff) => {
            NokhwaError::DecoderInvalidFrameData(format!("bad huffman tables: {huff}"))
        }
        DecodeErrors::ZeroError => {
            NokhwaError::DecoderInvalidBuffer("image has zero width.".to_string())
        }
        DecodeErrors::DqtError(e)
        | DecodeErrors::MCUError(e)
        | DecodeErrors::SosError(e)
        | DecodeErrors::SofError(e) => NokhwaError::Decoder(format!("error decoding: {e}")),
        DecodeErrors::Unsupported(why) => {
            NokhwaError::DecoderInvalidConfiguration(format!("not supported: {why:?}"))
        }
        DecodeErrors::ExhaustedData => {
            NokhwaError::DecoderInvalidBuffer("image data exhausted".to_string())
        }
        DecodeErrors::LargeDimensions(size) => {
            NokhwaError::DecoderInvalidBuffer(format!("too large: {size}"))
        }
        DecodeErrors::TooSmallOutput(a, b) => {
            NokhwaError::DecoderInvalidBuffer(format!("too small: {a},{b}"))
        }
        DecodeErrors::IoErrors(io) => NokhwaError::Decoder(format!("io error: {io:?}")),
    }
}

fn convert_destination_to_colorspace(pixel_destination: PixelDestination) -> Option<ColorSpace> {
    match pixel_destination {
        PixelDestination::Rgb8 => Some(ColorSpace::RGB),
        PixelDestination::Rgba8 => Some(ColorSpace::RGBA),
        PixelDestination::Bgr8 => Some(ColorSpace::BGR),
        PixelDestination::Bgra8 => Some(ColorSpace::BGRA),
        PixelDestination::Luma8 => Some(ColorSpace::Luma),
        PixelDestination::LumaA8 => Some(ColorSpace::LumaA),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use image::{DynamicImage, ImageFormat, Rgb};
    use nokhwa_core::{decoder::Decoder, frame_buffer::FrameBuffer, types::Resolution};
    use zune_core::options::DecoderOptions;

    use crate::mjpeg::{MJpegConfig, MJpegDecoder};

    fn load_image(filename: String, format: ImageFormat) -> DynamicImage {
        let file = File::open(filename).unwrap();
        let image = image::load(BufReader::new(file), format).unwrap();
        image
    }

    #[test]
    pub fn decode_mjpeg_rgb8() {
        let mut source_file = File::open("test_images/mjpeg/iwillquit.mjpeg").unwrap();
        let mut data = Vec::new();

        let test_image = load_image(
            "test_images/mjpeg/iwillquit.rgb8.png".to_string(),
            ImageFormat::Png,
        )
        .to_rgb8();
        source_file.read_to_end(&mut data).unwrap();
        let resolution = Resolution::new(1044, 409);
        let decoder_options = DecoderOptions::new_fast();
        let config = MJpegConfig {
            resolution,
            decoder_options,
        };

        let decode_buffer = FrameBuffer::from(data);

        let mut decoder = MJpegDecoder::new(config);
        let out = decoder.decode::<Rgb<u8>>(decode_buffer).unwrap();

        assert_eq!(out.as_raw(), test_image.as_raw());
    }
}
