use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::swap;
use image::Pixel;
use itertools::Itertools;
use nokhwa_core::decoder::Decoder;
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_buffer::FrameBuffer;
use nokhwa_core::frame_format::{CustomFrameFormat, FrameFormat};
use nokhwa_core::image::{DecodedImage, NonFloatScalarWidth};
use nokhwa_core::types::{CameraFormat, Resolution};

#[derive(Clone, Debug, PartialEq)]
pub struct LumaDecoder {
    luma_config: LumaConfig,
}

impl Decoder for LumaDecoder {
    type Config = LumaConfig;
    type OutputMeta = ();
    type DestinationFormatHint = LumaDestination;

    fn config(&self) -> &Self::Config {
        &self.luma_config
    }

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        self.luma_config = config;
        Ok(())
    }

    fn decode_to_buffer(&mut self, mut to_decode: FrameBuffer, mut buffer: impl AsMut<[u8]>, destination_format_hint: Option<Self::DestinationFormatHint>) -> Result<Self::OutputMeta, NokhwaError> {
        let destination_hint = match destination_format_hint {
            Some(h) => h,
            None => return Err(NokhwaError::DecoderDestinationHintRequired)
        };

        let (width, max_value) = match self.config().format {
            FrameFormat::Luma_8 => (1, u8::MAX as u32),
            FrameFormat::Luma_10 => (2, 2_u32.pow(10)),
            FrameFormat::Luma_12 => (2, 2_u32.pow(12)),
            FrameFormat::Luma_14 => (2, 2_u32.pow(14)),
            FrameFormat::Luma_16 | FrameFormat::Depth_16 => (2, u16::MAX as u32),
            fmt => return Err(NokhwaError::DecoderUnsupportedFrameFormat(fmt))
        };

        let format = self.config().custom_frame_format_map.as_ref().map(|m| {
            match self.config().format {
                FrameFormat::Custom(cfmt) => {
                    m.get(&cfmt).copied()
                }
                _ => None,
            }
        }).flatten().unwrap_or(self.config().format);

        let r = filter_to_u8(self.config().channel_filters.red);
        let g = filter_to_u8(self.config().channel_filters.green);
        let b = filter_to_u8(self.config().channel_filters.blue);
        let a = filter_to_u8(self.config().channel_filters.alpha);

        let r_u16 = filter_to_u16(self.config().channel_filters.red);
        let g_u16 = filter_to_u16(self.config().channel_filters.green);
        let b_u16 = filter_to_u16(self.config().channel_filters.blue);
        let a_u16 = filter_to_u16(self.config().channel_filters.alpha);

        match format {
            FrameFormat::Luma_8 => {
                match destination_hint {
                    LumaDestination::Luma8 => {
                        swap(to_decode.as_mut(), buffer.as_mut())
                    }
                    LumaDestination::LumaA8 => {
                        let default_alpha = u8::MAX * a;

                        to_decode.buffer().into_iter().intersperse(default_alpha).co
                    }
                    LumaDestination::Rgb8 => {}
                    LumaDestination::Rgba8 => {}
                    LumaDestination::Rgb16 => {}
                    LumaDestination::Rgba16 => {}
                }
            }
            FrameFormat::Luma_10 => {}
            FrameFormat::Luma_12 => {}
            FrameFormat::Luma_14 => {}
            FrameFormat::Luma_16 | FrameFormat::Depth_16 => {}
            fmt => {
                return Err(NokhwaError::DecoderUnsupportedFrameFormat(fmt))
            }
        }
    }

    fn decode_to_pixel_buffer<P: Pixel>(&mut self, to_decode: FrameBuffer, buffer: impl AsMut<[P::Subpixel]>) -> Result<Self::OutputMeta, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth
    {
        todo!()
    }

    fn decode<P: Pixel>(&mut self, to_decode: FrameBuffer) -> Result<DecodedImage<P, Self::OutputMeta>, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth
    {
        todo!()
    }

    fn output_decoder_min_size(&self, resolution: Resolution, destination_format: Self::DestinationFormatHint) -> usize {
        todo!()
    }

    fn buffer_takes_destination_hint(&self) -> bool {
        todo!()
    }
}

fn filter_to_u8(filter: bool) -> u8 {
    if filter {
        1_u8
    } else {
        0_u8
    }
}


fn filter_to_u16(filter: bool) -> u16 {
    if filter {
        1_u16
    } else {
        0_u16
    }
}



#[derive(Clone, Debug, PartialEq)]
pub struct LumaConfig {
    pub mode: ConvertMode,
    pub scaling_functions: ScalingFunctions,
    pub channel_filters: ChannelFilters,
    pub format: FrameFormat,
    pub custom_frame_format_map: Option<HashMap<CustomFrameFormat, FrameFormat>>
}

impl TryFrom<FrameFormat> for LumaConfig {
    type Error = NokhwaError;

    fn try_from(value: FrameFormat) -> Result<Self, Self::Error> {
        if !FrameFormat::BRIGHTNESS_LUMA.contains(&value) {
            return Err(NokhwaError::DecoderUnsupportedFrameFormat(value))
        }

        Ok(LumaConfig {
            mode: ConvertMode::default(),
            scaling_functions: ScalingFunctions::default(),
            channel_filters: ChannelFilters::default(),
            format: value,
            custom_frame_format_map: None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConvertMode {
    Scaled,
    Clipped
}

impl Default for ConvertMode {
    fn default() -> Self {
        ConvertMode::Scaled
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScalingFunctions {
    pub scale_up_u8_to_u16: Box<dyn FnMut(u8, u32) -> u16>,
    pub scale_down_u8_to_u16: Box<dyn FnMut(u16, u32) -> u8>,
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ChannelFilters {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
    pub alpha: bool,
}

impl Default for ChannelFilters {
    fn default() -> Self {
        ChannelFilters {
            red: true,
            green: true,
            blue: true,
            alpha: true,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum LumaDestination {
    Luma8,
    LumaA8,
    Rgb8,
    Rgba8,
    Rgb16,
    Rgba16,
}

struct ConstIter<T> where T: Copy + Clone + Debug + Default + Eq + Ord + PartialEq + PartialOrd {
    pub val: T
}

impl<T> Iterator for ConstIter<T> where T: Copy + Clone + Debug + Default + Eq + Ord + PartialEq + PartialOrd {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.val)
    }
}

impl<T> IntoIterator for ConstIter<T> where  T: Copy + Clone + Debug + Default + Eq + Ord + PartialEq + PartialOrd {
    type Item = T;
    type IntoIter = ConstIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self
    }
}