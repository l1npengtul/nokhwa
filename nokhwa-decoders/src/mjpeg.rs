use bytemuck::cast_slice_mut;
use nokhwa_core::decoder::{Decoder, ImageBuffer, Pixel};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_buffer::FrameBuffer;
use nokhwa_core::image::Primitive;
use nokhwa_core::image::{DecodedImage, NonFloatScalarWidth};
use nokhwa_core::types::Resolution;
use zune_core::bytestream::ZCursor;
use zune_core::colorspace::ColorSpace;
pub use zune_core::options::DecoderOptions;
pub use zune_jpeg::ImageInfo;
use zune_jpeg::JpegDecoder;
use zune_jpeg::errors::DecodeErrors;

#[derive(Clone, Debug)]
pub struct MJpegDecoder {
    config: MJpegOptions,
}

impl Decoder for MJpegDecoder {
    type Config = MJpegOptions;
    type OutputMeta = ImageMeta;
    type DestinationFormatHint = OutputColor;

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
        destination_format_hint: Option<Self::DestinationFormatHint>,
    ) -> Result<Self::OutputMeta, NokhwaError> {
        let buffer = buffer.as_mut();
        let cursor = ZCursor::new(to_decode.as_ref());

        let mut decoder = JpegDecoder::new(cursor);

        let mut config = self.config.decoder_options;
        if let Some(dest_hint) = destination_format_hint {
            config = self
                .config
                .decoder_options
                .jpeg_set_out_colorspace(dest_hint.into());
        }

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

    fn decode_to_pixel_buffer<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
        mut buffer: impl AsMut<[P::Subpixel]>,
    ) -> Result<Self::OutputMeta, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        let hint = match pixel_to_colorspace::<P>() {
            Some(cs) => cs,
            None => {
                return Err(NokhwaError::DecoderUnsupportedDestinationPixelFormat(
                    P::COLOR_MODEL,
                    <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES,
                ));
            }
        };
        let meta = self.decode_to_buffer(to_decode, cast_slice_mut(buffer.as_mut()), Some(hint))?;
        Ok(meta)
    }

    fn decode<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
    ) -> Result<DecodedImage<P, Self::OutputMeta>, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        let min_size = self.output_decoder_min_size_pixel::<P>(self.config.resolution);
        let mut out_buffer: Vec<P::Subpixel> = vec![P::Subpixel::DEFAULT_MAX_VALUE; min_size];
        let output_metadata =
            self.decode_to_pixel_buffer::<P>(to_decode, out_buffer.as_mut_slice())?;
        let image_buffer = ImageBuffer::from_raw(
            output_metadata.resolution.width(),
            output_metadata.resolution.height(),
            out_buffer,
        )
        .ok_or(NokhwaError::Decoder(
            "Failed to make imagebuffer".to_string(),
        ))?;
        Ok(DecodedImage::new(image_buffer, output_metadata))
    }

    fn output_decoder_min_size(
        &self,
        resolution: Resolution,
        destination_format: Self::DestinationFormatHint,
    ) -> usize {
        let stride = match destination_format {
            OutputColor::Rgb => 3,
            OutputColor::RgbA => 4,
            OutputColor::Bgr => 3,
            OutputColor::BgrA => 4,
            OutputColor::Luma => 1,
            OutputColor::LumaA => 2,
        };
        (resolution.width() * resolution.height() * stride) as usize
    }

    fn buffer_takes_destination_hint(&self) -> bool {
        true
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
pub struct MJpegOptions {
    pub resolution: Resolution,
    pub decoder_options: DecoderOptions,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum OutputColor {
    Rgb,
    RgbA,
    Bgr,
    BgrA,
    Luma,
    LumaA,
}

impl From<OutputColor> for ColorSpace {
    fn from(val: OutputColor) -> Self {
        match val {
            OutputColor::Rgb => ColorSpace::RGB,
            OutputColor::RgbA => ColorSpace::RGBA,
            OutputColor::Bgr => ColorSpace::BGR,
            OutputColor::BgrA => ColorSpace::BGRA,
            OutputColor::Luma => ColorSpace::Luma,
            OutputColor::LumaA => ColorSpace::LumaA,
        }
    }
}

fn pixel_to_colorspace<P>() -> Option<OutputColor>
where
    P: Pixel,
    <P as Pixel>::Subpixel: NonFloatScalarWidth,
{
    match P::COLOR_MODEL {
        "RGBA" => Some(OutputColor::RgbA),
        "RGB" => Some(OutputColor::Rgb),
        "BGR" => Some(OutputColor::Bgr),
        "BGRA" => Some(OutputColor::BgrA),
        "Y" => Some(OutputColor::Luma),
        "YA" => Some(OutputColor::LumaA),
        _ => None,
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
