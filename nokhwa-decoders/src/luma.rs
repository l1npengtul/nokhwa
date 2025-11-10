use bytemuck::{cast_slice, cast_slice_mut};
use itermore::IterArrayChunks;
use nokhwa_core::decoder::{ConfigHasResolution, Decoder};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_buffer::FrameBuffer;
use nokhwa_core::frame_format::{CustomFrameFormat, FrameFormat};
use nokhwa_core::pixel_destination::PixelDestination;
use nokhwa_core::types::{CameraFormat, Resolution};
use nokhwa_iter_extensions::duplicate::IterDuplicateConst;
use nokhwa_iter_extensions::interweave::IterInterweave;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub struct LumaDecoder {
    luma_config: LumaConfig,
}

impl Decoder for LumaDecoder {
    type Config = LumaConfig;
    type OutputMeta = ();
    const SUPPORTED_DESTINATIONS: &'static [PixelDestination] = &[
        PixelDestination::Luma8,
        PixelDestination::LumaA8,
        PixelDestination::Luma16,
        PixelDestination::LumaA16,
        PixelDestination::Rgb8,
        PixelDestination::Rgba8,
        PixelDestination::Rgb16,
        PixelDestination::Rgba16,
    ];

    fn config(&self) -> &Self::Config {
        &self.luma_config
    }

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        self.luma_config = config;
        Ok(())
    }

    fn decode_to_buffer(
        &mut self,
        to_decode: FrameBuffer,
        mut buffer: impl AsMut<[u8]>,
        destination_format: PixelDestination,
    ) -> Result<Self::OutputMeta, NokhwaError> {
        let format = self
            .config()
            .custom_frame_format_map
            .as_ref()
            .and_then(|m| match self.config().format {
                FrameFormat::Custom(cfmt) => m.get(&cfmt).copied(),
                _ => None,
            })
            .unwrap_or(self.config().format);

        let r = filter_to_u8(self.config().channel_filters.red);
        let g = filter_to_u8(self.config().channel_filters.green);
        let b = filter_to_u8(self.config().channel_filters.blue);
        let a = filter_to_u8(self.config().channel_filters.alpha);

        let r_u16 = filter_to_u16(self.config().channel_filters.red);
        let g_u16 = filter_to_u16(self.config().channel_filters.green);
        let b_u16 = filter_to_u16(self.config().channel_filters.blue);
        let a_u16 = filter_to_u16(self.config().channel_filters.alpha);

        let buffer = buffer.as_mut();

        match format {
            FrameFormat::Luma_8 => match destination_format {
                PixelDestination::Luma8 => {
                    if to_decode.len() != buffer.len() {
                        return Err(NokhwaError::DecoderInvalidBuffer(
                            "Lengths differ!".to_string(),
                        ));
                    }

                    match to_decode.consume().0 {
                        Cow::Borrowed(data) => {
                            buffer.copy_from_slice(data);
                            Ok(())
                        }
                        Cow::Owned(mut owned) => {
                            buffer.swap_with_slice(owned.as_mut_slice());
                            Ok(())
                        }
                    }
                }
                PixelDestination::LumaA8 => {
                    let default_alpha = u8::MAX * a;

                    if (to_decode.len() * 2) != buffer.len() {
                        return Err(NokhwaError::DecoderInvalidBuffer(
                            "Lengths differ!".to_string(),
                        ));
                    }

                    to_decode
                        .buffer()
                        .iter()
                        .interweave::<0>(&default_alpha, false)
                        .enumerate()
                        .for_each(|(len, data)| unsafe {
                            *buffer.get_unchecked_mut(len) = *data;
                        });
                    Ok(())
                }
                PixelDestination::Rgb8 => {
                    if (to_decode.len() * 3) != buffer.len() {
                        return Err(NokhwaError::DecoderInvalidBuffer(
                            "Lengths differ!".to_string(),
                        ));
                    }

                    to_decode
                        .buffer()
                        .iter()
                        .duplicate_const::<3>()
                        .arrays::<3>()
                        .flat_map(|pixel| {
                            let px_r = *pixel[0_usize] * r;
                            let px_g = *pixel[1_usize] * g;
                            let px_b = *pixel[2_usize] * b;
                            [px_r, px_g, px_b]
                        })
                        .enumerate()
                        .for_each(|(len, data)| unsafe {
                            *buffer.get_unchecked_mut(len) = data;
                        });
                    Ok(())
                }
                PixelDestination::Rgba8 => {
                    if (to_decode.len() * 4) != buffer.len() {
                        return Err(NokhwaError::DecoderInvalidBuffer(
                            "Lengths differ!".to_string(),
                        ));
                    }

                    to_decode
                        .buffer()
                        .iter()
                        .duplicate_const::<3>()
                        .arrays::<3>()
                        .flat_map(|pixel| {
                            let px_r = *pixel[0_usize] * r;
                            let px_g = *pixel[1_usize] * g;
                            let px_b = *pixel[2_usize] * b;
                            let px_a = 255 * b;
                            [px_r, px_g, px_b, px_a]
                        })
                        .enumerate()
                        .for_each(|(len, data)| unsafe {
                            *buffer.get_unchecked_mut(len) = data;
                        });
                    Ok(())
                }
                PixelDestination::Rgb16 => {
                    if (to_decode.len() * 6) != buffer.len() {
                        return Err(NokhwaError::DecoderInvalidBuffer(
                            "Lengths differ!".to_string(),
                        ));
                    }

                    let temp_buffer = cast_slice_mut::<u8, u16>(buffer);
                    let factor = match self.config().mode {
                        ConvertMode::Scaled => u16::MAX / (u8::MAX as u16),
                        ConvertMode::Clipped => 1_u16,
                    };

                    to_decode
                        .buffer()
                        .iter()
                        .duplicate_const::<3>()
                        .arrays::<3>()
                        .flat_map(|pixel| {
                            let px_r = (*pixel[0_usize] as u16) * r_u16 * factor;
                            let px_g = (*pixel[1_usize] as u16) * g_u16 * factor;
                            let px_b = (*pixel[2_usize] as u16) * b_u16 * factor;
                            [px_r, px_g, px_b]
                        })
                        .enumerate()
                        .for_each(|(len, data)| unsafe {
                            *temp_buffer.get_unchecked_mut(len) = data;
                        });
                    Ok(())
                }
                PixelDestination::Rgba16 => {
                    if (to_decode.len() * 8) != buffer.len() {
                        return Err(NokhwaError::DecoderInvalidBuffer(
                            "Lengths differ!".to_string(),
                        ));
                    }

                    let temp_buffer = cast_slice_mut::<u8, u16>(buffer);
                    let factor = match self.config().mode {
                        ConvertMode::Scaled => u16::MAX / (u8::MAX as u16),
                        ConvertMode::Clipped => 1_u16,
                    };

                    to_decode
                        .buffer()
                        .iter()
                        .duplicate_const::<3>()
                        .arrays::<3>()
                        .flat_map(|pixel| {
                            let px_r = (*pixel[0_usize] as u16) * r_u16 * factor;
                            let px_g = (*pixel[1_usize] as u16) * g_u16 * factor;
                            let px_b = (*pixel[2_usize] as u16) * b_u16 * factor;
                            let px_a = u16::MAX * a_u16;
                            [px_r, px_g, px_b, px_a]
                        })
                        .enumerate()
                        .for_each(|(len, data)| unsafe {
                            *temp_buffer.get_unchecked_mut(len) = data;
                        });
                    Ok(())
                }
                PixelDestination::Luma16 => {
                    let buffer_u16 = cast_slice_mut::<u8, u16>(buffer);

                    if to_decode.len() != buffer_u16.len() {
                        return Err(NokhwaError::DecoderInvalidBuffer(
                            "Lengths differ!".to_string(),
                        ));
                    }

                    let factor = match self.config().mode {
                        ConvertMode::Scaled => 8,
                        ConvertMode::Clipped => 0,
                    };

                    to_decode
                        .buffer()
                        .iter()
                        .map(|px| (*px as u16) << factor)
                        .enumerate()
                        .for_each(|(len, data)| unsafe {
                            *buffer_u16.get_unchecked_mut(len) = data;
                        });
                    Ok(())
                }
                PixelDestination::LumaA16 => {
                    let default_alpha = u16::MAX * a_u16;
                    let buffer_u16 = cast_slice_mut::<u8, u16>(buffer);

                    if (to_decode.len() * 2) != buffer_u16.len() {
                        return Err(NokhwaError::DecoderInvalidBuffer(
                            "Lengths differ!".to_string(),
                        ));
                    }

                    let factor = match self.config().mode {
                        ConvertMode::Scaled => 8,
                        ConvertMode::Clipped => 0,
                    };

                    to_decode
                        .buffer()
                        .iter()
                        .map(|px| (*px as u16) << factor)
                        .interweave::<0>(default_alpha, false)
                        .enumerate()
                        .for_each(|(len, data)| unsafe {
                            *buffer_u16.get_unchecked_mut(len) = data;
                        });
                    Ok(())
                }
                fmt => Err(NokhwaError::DecoderUnsupportedDestinationPixelFormat(fmt)),
            },
            FrameFormat::Luma_10 => convert_u16_type_buffers(
                to_decode,
                buffer,
                destination_format,
                self.config().mode,
                self.config().channel_filters,
                10,
            ),
            FrameFormat::Luma_12 => convert_u16_type_buffers(
                to_decode,
                buffer,
                destination_format,
                self.config().mode,
                self.config().channel_filters,
                12,
            ),
            FrameFormat::Luma_14 => convert_u16_type_buffers(
                to_decode,
                buffer,
                destination_format,
                self.config().mode,
                self.config().channel_filters,
                14,
            ),
            FrameFormat::Luma_16 | FrameFormat::Depth_16 => convert_u16_type_buffers(
                to_decode,
                buffer,
                destination_format,
                self.config().mode,
                self.config().channel_filters,
                16,
            ),
            fmt => Err(NokhwaError::DecoderUnsupportedFrameFormat(fmt)),
        }
    }
}

fn filter_to_u8(filter: bool) -> u8 {
    if filter { 1_u8 } else { 0_u8 }
}

fn filter_to_u16(filter: bool) -> u16 {
    if filter { 1_u16 } else { 0_u16 }
}

fn convert_u16_type_buffers(
    to_decode: FrameBuffer,
    destination: &mut [u8],
    hint: PixelDestination,
    mode: ConvertMode,
    channel_filters: ChannelFilters,
    original_bit_num: u32,
) -> Result<(), NokhwaError> {
    let r_u16 = filter_to_u16(channel_filters.red);
    let g_u16 = filter_to_u16(channel_filters.green);
    let b_u16 = filter_to_u16(channel_filters.blue);
    let a_u16 = filter_to_u16(channel_filters.alpha);

    // let r = filter_to_u8(channel_filters.red);
    // let g = filter_to_u8(channel_filters.green);
    // let b = filter_to_u8(channel_filters.blue);
    let a = filter_to_u8(channel_filters.alpha);

    match hint {
        PixelDestination::Luma8 => {
            let to_decode_u16 = cast_slice::<u8, u16>(to_decode.buffer());

            if to_decode_u16.len() != destination.len() {
                return Err(NokhwaError::DecoderInvalidBuffer(
                    "sizes differ!".to_string(),
                ));
            }

            let factor = match mode {
                ConvertMode::Scaled => original_bit_num - 8_u32,
                ConvertMode::Clipped => 0,
            };

            to_decode_u16
                .iter()
                .map(|px| (*px >> factor) as u8)
                .enumerate()
                .for_each(|(len, data)| unsafe {
                    *destination.get_unchecked_mut(len) = data;
                });
            Ok(())
        }
        PixelDestination::LumaA8 => {
            let to_decode_u16 = cast_slice::<u8, u16>(to_decode.buffer());

            if (to_decode_u16.len() * 2) != destination.len() {
                return Err(NokhwaError::DecoderInvalidBuffer(
                    "sizes differ!".to_string(),
                ));
            }

            let factor = match mode {
                ConvertMode::Scaled => original_bit_num - 8_u32,
                ConvertMode::Clipped => 0,
            };

            to_decode_u16
                .iter()
                .map(|px| (*px >> factor) as u8)
                .interweave::<0>(255 * a, false)
                .enumerate()
                .for_each(|(len, data)| unsafe {
                    *destination.get_unchecked_mut(len) = data;
                });
            Ok(())
        }
        PixelDestination::Rgb8 => {
            let to_decode_u16 = cast_slice::<u8, u16>(to_decode.buffer());

            if (to_decode_u16.len() * 3) != destination.len() {
                return Err(NokhwaError::DecoderInvalidBuffer(
                    "sizes differ!".to_string(),
                ));
            }

            let factor = match mode {
                ConvertMode::Scaled => original_bit_num - 8_u32,
                ConvertMode::Clipped => 0,
            };

            to_decode_u16
                .iter()
                .duplicate_const::<3>()
                .arrays::<3>()
                .flat_map(|px| {
                    let px_r = (*px[0_usize] >> factor) * r_u16;
                    let px_g = (*px[1_usize] >> factor) * g_u16;
                    let px_b = (*px[2_usize] >> factor) * b_u16;
                    [px_r as u8, px_g as u8, px_b as u8]
                })
                .enumerate()
                .for_each(|(len, data)| unsafe {
                    *destination.get_unchecked_mut(len) = data;
                });
            Ok(())
        }
        PixelDestination::Rgba8 => {
            let to_decode_u16 = cast_slice::<u8, u16>(to_decode.buffer());

            if (to_decode_u16.len() * 4) != destination.len() {
                return Err(NokhwaError::DecoderInvalidBuffer(
                    "sizes differ!".to_string(),
                ));
            }

            let factor = match mode {
                ConvertMode::Scaled => original_bit_num - 8_u32,
                ConvertMode::Clipped => 0,
            };

            to_decode_u16
                .iter()
                .duplicate_const::<3>()
                .arrays::<3>()
                .flat_map(|px| {
                    let px_r = (*px[0_usize] >> factor) * r_u16;
                    let px_g = (*px[1_usize] >> factor) * g_u16;
                    let px_b = (*px[2_usize] >> factor) * b_u16;
                    let px_a = u8::MAX * a;
                    [px_r as u8, px_g as u8, px_b as u8, px_a]
                })
                .enumerate()
                .for_each(|(len, data)| unsafe {
                    *destination.get_unchecked_mut(len) = data;
                });
            Ok(())
        }
        PixelDestination::Rgb16 => {
            let to_decode_u16 = cast_slice::<u8, u16>(to_decode.buffer());
            let destination_buffer_u16 = cast_slice_mut::<u8, u16>(destination);

            if (to_decode_u16.len() * 3) != destination_buffer_u16.len() {
                return Err(NokhwaError::DecoderInvalidBuffer(
                    "sizes differ!".to_string(),
                ));
            }

            let factor = match mode {
                ConvertMode::Scaled => original_bit_num - 8_u32,
                ConvertMode::Clipped => 0,
            };

            to_decode_u16
                .iter()
                .duplicate_const::<3>()
                .arrays::<3>()
                .flat_map(|px| {
                    let px_r = (*px[0_usize] >> factor) * r_u16;
                    let px_g = (*px[1_usize] >> factor) * g_u16;
                    let px_b = (*px[2_usize] >> factor) * b_u16;
                    [px_r, px_g, px_b]
                })
                .enumerate()
                .for_each(|(len, data)| unsafe {
                    *destination_buffer_u16.get_unchecked_mut(len) = data;
                });
            Ok(())
        }
        PixelDestination::Rgba16 => {
            let to_decode_u16 = cast_slice::<u8, u16>(to_decode.buffer());
            let destination_buffer_u16 = cast_slice_mut::<u8, u16>(destination);

            if (to_decode_u16.len() * 3) != destination_buffer_u16.len() {
                return Err(NokhwaError::DecoderInvalidBuffer(
                    "sizes differ!".to_string(),
                ));
            }

            let factor = match mode {
                ConvertMode::Scaled => original_bit_num - 8_u32,
                ConvertMode::Clipped => 0,
            };

            to_decode_u16
                .iter()
                .duplicate_const::<3>()
                .arrays::<3>()
                .flat_map(|px| {
                    let px_r = (*px[0_usize] >> factor) * r_u16;
                    let px_g = (*px[1_usize] >> factor) * g_u16;
                    let px_b = (*px[2_usize] >> factor) * b_u16;
                    let px_a = u16::MAX * a_u16;
                    [px_r, px_g, px_b, px_a]
                })
                .enumerate()
                .for_each(|(len, data)| unsafe {
                    *destination_buffer_u16.get_unchecked_mut(len) = data;
                });
            Ok(())
        }
        PixelDestination::Luma16 => {
            if to_decode.len() != destination.len() {
                return Err(NokhwaError::DecoderInvalidBuffer(
                    "sizes differ!".to_string(),
                ));
            }

            match to_decode.consume().0 {
                Cow::Borrowed(data) => destination.copy_from_slice(data),
                Cow::Owned(mut owned) => destination.swap_with_slice(owned.as_mut_slice()),
            }
            Ok(())
        }
        PixelDestination::LumaA16 => {
            if (to_decode.len() * 2) != destination.len() {
                return Err(NokhwaError::DecoderInvalidBuffer(
                    "sizes differ!".to_string(),
                ));
            }

            let to_decode_u16 = cast_slice::<u8, u16>(to_decode.buffer());
            let destination_buffer_u16 = cast_slice_mut::<u8, u16>(destination);

            to_decode_u16
                .iter()
                .interweave::<0>(&(u16::MAX * a_u16), false)
                .enumerate()
                .for_each(|(len, data)| unsafe {
                    *destination_buffer_u16.get_unchecked_mut(len) = *data;
                });

            Ok(())
        }
        _ => Err(NokhwaError::DecoderUnsupportedDestinationPixelFormat(hint)),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LumaConfig {
    pub mode: ConvertMode,
    pub channel_filters: ChannelFilters,
    pub format: FrameFormat,
    pub custom_frame_format_map: Option<HashMap<CustomFrameFormat, FrameFormat>>,
    pub resolution: Resolution,
}

impl ConfigHasResolution for LumaConfig {
    fn resolution(&self) -> Resolution {
        self.resolution
    }
}

impl TryFrom<CameraFormat> for LumaConfig {
    type Error = NokhwaError;

    fn try_from(value: CameraFormat) -> Result<Self, Self::Error> {
        if !FrameFormat::BRIGHTNESS_LUMA.contains(&value.format()) {
            return Err(NokhwaError::DecoderUnsupportedFrameFormat(value.format()));
        }

        Ok(LumaConfig {
            mode: ConvertMode::default(),
            channel_filters: ChannelFilters::default(),
            format: value.format(),
            custom_frame_format_map: None,
            resolution: value.resolution(),
        })
    }
}

#[derive(Copy, Clone, Default, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum ConvertMode {
    #[default]
    Scaled,
    Clipped,
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

#[cfg(test)]
mod test {
    #[test]
    pub fn luma8() {}
}
