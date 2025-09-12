use std::collections::HashMap;
use bytemuck::{cast_slice, cast_slice_mut, try_cast_slice_mut};

use nokhwa_core::decoder::{Decoder, ImageBuffer, Pixel, Primitive};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_buffer::FrameBuffer;
use nokhwa_core::frame_format::{CustomFrameFormat, FrameFormat};
use nokhwa_core::image::{DecodedImage, NonFloatScalarWidth};
use nokhwa_core::types::{CameraFormat, Resolution};
use yuv::{
    YuvBiPlanarImage, YuvConversionMode, YuvPackedImage, YuvPlanarImage, YuvRange,
    YuvStandardMatrix, ayuv_to_rgb, ayuv_to_rgba, p010_to_bgr, p010_to_bgra, p010_to_rgb,
    p010_to_rgb10, p010_to_rgba, p010_to_rgba10, p012_to_rgb12, p012_to_rgba12, uyvy422_to_bgr,
    uyvy422_to_bgra, uyvy422_to_rgb, uyvy422_to_rgb_p16, uyvy422_to_rgba, uyvy422_to_rgba_p16,
    vyuy422_to_bgr, vyuy422_to_bgra, vyuy422_to_rgb, vyuy422_to_rgb_p16, vyuy422_to_rgba,
    vyuy422_to_rgba_p16, yuv_nv12_to_bgr, yuv_nv12_to_bgra, yuv_nv12_to_rgb, yuv_nv12_to_rgba,
    yuv_nv16_to_bgr, yuv_nv16_to_bgra, yuv_nv16_to_rgb, yuv_nv16_to_rgba, yuv_nv21_to_bgr,
    yuv_nv21_to_bgra, yuv_nv21_to_rgb, yuv_nv21_to_rgba, yuv_nv24_to_bgr, yuv_nv24_to_bgra,
    yuv_nv24_to_rgb, yuv_nv24_to_rgba, yuv_nv42_to_bgr, yuv_nv42_to_bgra, yuv_nv42_to_rgb,
    yuv_nv42_to_rgba, yuv_nv61_to_bgr, yuv_nv61_to_bgra, yuv_nv61_to_rgb, yuv_nv61_to_rgba,
    yuv420_to_bgr, yuv420_to_bgra, yuv420_to_rgb, yuv420_to_rgba, yuyv422_to_bgr, yuyv422_to_bgra,
    yuyv422_to_rgb, yuyv422_to_rgb_p16, yuyv422_to_rgba, yuyv422_to_rgba_p16, yvyu422_to_bgr,
    yvyu422_to_bgra, yvyu422_to_rgb, yvyu422_to_rgb_p16, yvyu422_to_rgba, yvyu422_to_rgba_p16,
};

pub struct YUVDecoder {
    config: YUVConfig,
}

impl YUVDecoder {
    pub fn new(config: <Self as Decoder>::Config) -> Self {
        Self { config }
    }
}

impl Decoder for YUVDecoder {
    type Config = YUVConfig;
    type OutputMeta = ();
    type DestinationFormatHint = YUVDestination;

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        if !FrameFormat::YCBCR.contains(&config.yuv_type) {
            return Err(NokhwaError::DecoderUnsupportedFrameFormat(config.yuv_type));
        }

        if let Some(custom_map) = &config.custom_frame_format_map {
            if let Some((src, dest)) = custom_map.iter().find(|(_, value)| {
                FrameFormat::YCBCR.contains(value)
            }) {
                return Err(NokhwaError::DecoderUnsupportedCustomFrameFormatDestination(*src, *dest))
            }
        }

        self.config = config;
        Ok(())
    }

    fn decode_to_buffer(
        &mut self,
        to_decode: FrameBuffer<'_>,
        mut buffer: impl AsMut<[u8]>,
        destination_format: Option<Self::DestinationFormatHint>,
    ) -> Result<Self::OutputMeta, NokhwaError> {
        let destination_format = match destination_format {
            Some(df) => df,
            None => return Err(NokhwaError::DecoderDestinationHintRequired),
        };


        let yuv_format = self.config().custom_frame_format_map.as_ref().map(|m| {
            match self.config.yuv_type {
                FrameFormat::Custom(cfmt) => {
                    m.get(&cfmt).copied()
                }
                _ => None,
            }
        }).flatten().unwrap_or(self.config.yuv_type);


        let buffer = buffer.as_mut();
        if buffer.len() < self.output_decoder_min_size(self.config.resolution, destination_format) {
            return Err(NokhwaError::DecoderInvalidBuffer("Too small!".to_string()));
        }

        let stride = figure_out_stride(yuv_format).ok_or(NokhwaError::DecoderUnsupportedFrameFormat(yuv_format))?;
        let byte_width = figure_out_byte_width(yuv_format).ok_or(NokhwaError::DecoderUnsupportedFrameFormat(yuv_format))?;


        let stride_3px = 3 *
            self.config.resolution.width();
        let stride_4px = 4 * self.config.resolution.width();
        let stride_3px_2w = 6 * self.config.resolution.width();
        let stride_4px_2w = 8 * self.config.resolution.width();

        // todo: clean up ts into a macro </3
         let decode_status = match stride {
            Stride::Packed(stride) => {
                let image = prepare_to_packed_image(&to_decode, self.config.resolution, byte_width, stride);
                match yuv_format {
                    FrameFormat::Ayuv_32 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(ayuv_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.premultiply_alpha)),
                            YUVDestination::Rgba8 => Some(ayuv_to_rgba(&image, buffer, stride_4px, self.config.range, self.config.matrix, self.config.premultiply_alpha)),
                            _ => None,
                        }
                    }
                    FrameFormat::Yuyv_4_2_2 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuyv422_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(yuyv422_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                            YUVDestination::Rgb16 => Some(yuyv422_to_rgb_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), stride_3px_2w, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(yuyv422_to_rgba_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), stride_3px_2w, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(yuyv422_to_bgr(&image, buffer, stride_4px_2w, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(yuyv422_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                        }
                    }
                    FrameFormat::Uyvy_4_2_2 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(uyvy422_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(uyvy422_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                            YUVDestination::Rgb16 => Some(uyvy422_to_rgb_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), stride_3px_2w, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(uyvy422_to_rgba_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), stride_3px_2w, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(uyvy422_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(uyvy422_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                        }
                    }
                    FrameFormat::Vyuy_4_2_2 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(vyuy422_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(vyuy422_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                            YUVDestination::Rgb16 => Some(vyuy422_to_rgb_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), stride_3px_2w, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(vyuy422_to_rgba_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), stride_3px_2w, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(vyuy422_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(vyuy422_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                        }
                    }
                    FrameFormat::Yvyu_4_2_2 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yvyu422_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(yvyu422_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                            YUVDestination::Rgb16 => Some(yvyu422_to_rgb_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), stride_3px_2w, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(yvyu422_to_rgba_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), stride_3px_2w, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(yvyu422_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(yvyu422_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                        }
                    }
                    _ => {
                        if FrameFormat::YCBCR_PACKED.contains(&self.config.yuv_type) {
                            return Err(NokhwaError::NotImplementedError("etto blehhh! ()".to_string()))
                        }
                        // shouldnt happen
                        return Err(NokhwaError::DecoderUnsupportedFrameFormat(yuv_format))
                    }
                }
            }
            Stride::Semi(y_stride, uv_stride) => {
                let image = prepare_to_semi_planar_image(&to_decode, self.config.resolution, byte_width, y_stride, uv_stride);
                match yuv_format {
                    FrameFormat::NV24 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv24_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv24_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv24_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv24_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV42 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv42_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv42_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv42_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv42_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV16 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv16_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv16_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv16_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv16_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV61 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv61_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv61_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv61_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv61_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV12 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv12_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv12_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv12_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv12_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV21 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv21_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv21_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv21_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv21_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::P010 => {
                        let a = convert_bi_planar_image_to_u16(image);
                        match destination_format {
                            YUVDestination::Rgb8 => Some(p010_to_rgb(&a, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(p010_to_rgba(&a, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(p010_to_bgr(&a, buffer, stride_3px, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(p010_to_bgra(&a, buffer, stride_4px,  self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgb16 => Some(p010_to_rgb10(&a, cast_slice_mut(buffer), stride_3px_2w, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(p010_to_rgba10(&a, cast_slice_mut(buffer), stride_4px, self.config.range, self.config.matrix)),
                            // _ => None,
                        }
                    }
                    FrameFormat::P012 => {
                        match destination_format {
                            YUVDestination::Rgb16 => Some(p012_to_rgb12(&convert_bi_planar_image_to_u16(image), cast_slice_mut(buffer), stride_3px_2w, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(p012_to_rgba12(&convert_bi_planar_image_to_u16(image), cast_slice_mut(buffer), stride_4px_2w, self.config.range, self.config.matrix)),
                            _ => None,
                        }
                    }
                    _ => {
                        if FrameFormat::YCBCR_SEMI.contains(&yuv_format) {
                            return Err(NokhwaError::NotImplementedError("etto blehhh!".to_string()))
                        }
                        // shouldnt happen
                        return Err(NokhwaError::DecoderUnsupportedFrameFormat(yuv_format))
                    }
                }
            }
            Stride::Planar(y_stride, u_stride, v_stride, line_ratio) => {
                let image = prepare_to_planar_image(&to_decode, self.config.resolution, byte_width, y_stride, u_stride, v_stride, line_ratio);
                match yuv_format {
                    FrameFormat::Yuv_4_2_0 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv420_to_rgb(&image, buffer, stride_3px, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(yuv420_to_rgba(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(yuv420_to_bgr(&image, buffer, stride_3px, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(yuv420_to_bgra(&image, buffer, stride_4px,  self.config.range, self.config.matrix)),
                            _ => None,
                        }
                    }
                    _ => {
                        if FrameFormat::YCBCR_PLANAR.contains(&yuv_format) {
                            return Err(NokhwaError::NotImplementedError("etto blehhh!".to_string()))
                        }
                        // shouldnt happen
                        return Err(NokhwaError::DecoderUnsupportedFrameFormat(yuv_format))
                    }
                }
            }
        };
        match decode_status {
            Some(Ok(_)) => Ok(()),
            Some(Err(why)) => Err(NokhwaError::Decoder(why.to_string())),
            None => Err(NokhwaError::DecoderUnsupportedFrameFormat(
                yuv_format,
            )),
        }
    }

    fn decode_to_pixel_buffer<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer<'_>,
        mut buffer: impl AsMut<[P::Subpixel]>,
    ) -> Result<Self::OutputMeta, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        let destination = match YUVDestination::get_by_pixel::<P>() {
            None => {
                return Err(NokhwaError::DecoderUnsupportedDestinationPixelFormat(
                    P::COLOR_MODEL,
                    <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES,
                ));
            }
            Some(d) => d,
        };
        let buffer = buffer.as_mut();

        let cast_slice = try_cast_slice_mut::<P::Subpixel, u8>(buffer)
            .map_err(|why| NokhwaError::DecoderInvalidBuffer(why.to_string()))?;

        self.decode_to_buffer(to_decode, cast_slice, Some(destination))
    }

    fn decode<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer<'_>,
    ) -> Result<DecodedImage<P, Self::OutputMeta>, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        let min_size_alloc = self.output_decoder_min_size_pixel::<P>(self.config.resolution);
        let mut out_buffer: Vec<P::Subpixel> = vec![P::Subpixel::DEFAULT_MIN_VALUE; min_size_alloc];
        self.decode_to_pixel_buffer::<P>(to_decode, &mut out_buffer)?;
        Ok(DecodedImage::new(
            ImageBuffer::from_vec(
                self.config.resolution.width(),
                self.config.resolution.height(),
                out_buffer,
            )
            .ok_or(NokhwaError::Decoder(
                "failed to convert into an image buffer".to_string(),
            ))?,
            (),
        ))
    }

    fn output_decoder_min_size(
        &self,
        resolution: Resolution,
        destination_format: Self::DestinationFormatHint,
    ) -> usize {
        let px_size = match destination_format {
            YUVDestination::Rgb8 | YUVDestination::Bgr8 => 3,
            YUVDestination::Rgba8 | YUVDestination::Bgra8 => 4,
            YUVDestination::Rgb16 => 3 * 2,
            YUVDestination::Rgba16 => 4 * 2,
        };
        let reso = resolution.width() * resolution.height();
        (reso as usize) * (px_size as usize)
    }

    fn buffer_takes_destination_hint(&self) -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum YUVDestination {
    Rgb8,
    Rgba8,
    Rgb16,
    Rgba16,
    Bgr8,
    Bgra8,
}

impl YUVDestination {
    pub fn get_by_pixel<P>() -> Option<Self>
    where
        P: Pixel,
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        match P::COLOR_MODEL {
            "RGB" => match <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES {
                1 => Some(YUVDestination::Rgb8),
                2 => Some(YUVDestination::Rgb16),
                _ => None,
            },
            "RGBA" => match <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES {
                1 => Some(YUVDestination::Rgba8),
                2 => Some(YUVDestination::Rgba16),
                _ => None,
            },
            "BGR" => match <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES {
                1 => Some(YUVDestination::Bgr8),
                // 2 => Some(YUVDestination::Bgr16),
                _ => None,
            },
            "BGRA" => match <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES {
                1 => Some(YUVDestination::Bgra8),
                // 2 => Some(YUVDestination::Bgra16),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct YUVConfig {
    pub resolution: Resolution,
    pub yuv_type: FrameFormat,
    pub range: YuvRange,
    pub matrix: YuvStandardMatrix,
    pub mode: YuvConversionMode,
    pub premultiply_alpha: bool,
    pub custom_frame_format_map: Option<HashMap<CustomFrameFormat, FrameFormat>>,
}

impl TryFrom<CameraFormat> for YUVConfig {
    type Error = NokhwaError;

    fn try_from(value: CameraFormat) -> Result<Self, Self::Error> {
        if !FrameFormat::YCBCR.contains(&value.format()) {
            return Err(NokhwaError::DecoderUnsupportedFrameFormat(value.format()));
        }
        Ok(YUVConfig {
            resolution: value.resolution(),
            yuv_type: value.format(),
            range: YuvRange::Full,
            matrix: YuvStandardMatrix::Bt601,
            mode: YuvConversionMode::Balanced,
            premultiply_alpha: false,
            custom_frame_format_map: None,
        })
    }
}

#[derive(Copy, Clone, Debug)]
enum Stride {
    Packed(u32),
    Semi(u32, u32),
    Planar(u32, u32, u32, u32),
}

fn figure_out_stride(frame_format: FrameFormat) -> Option<Stride> {
    if let Some(yuy) = packed_stride_component(frame_format) {
        return Some(Stride::Packed(yuy));
    }
    if let Some((y, uv)) = semiplanar_stride(frame_format) {
        return Some(Stride::Semi(y, uv));
    }
    // l here means the ratio of luma lines to chroma lines
    // u_r and v_r are defined as _ratios_ to the luma stride, i.e. how many luma components
    // per chroma component, u_r = 2 means 2 luma per 1 u chroma
    if let Some((y, u_r, v_r, l)) = planar_stride(frame_format) {
        return Some(Stride::Planar(y, u_r, v_r, l));
    }
    None
}

fn figure_out_byte_width(format: FrameFormat) -> Option<u32> {
    match format {
        FrameFormat::Ayuv_32 | FrameFormat::Yuyv_4_2_2
        | FrameFormat::Uyvy_4_2_2
        | FrameFormat::Vyuy_4_2_2
        | FrameFormat::Yvyu_4_2_2 => Some(1),
        FrameFormat::NV24 | FrameFormat::NV42 | FrameFormat::NV16 | FrameFormat::NV61 | FrameFormat::NV12 | FrameFormat::NV21  => Some(1),
        FrameFormat::P010 | FrameFormat::P012 => Some(2),
        FrameFormat::Yuv_4_2_0 => Some(1),
        _ => None,
    }
}

fn packed_stride_component(format: FrameFormat) -> Option<u32> {
    match format {
        FrameFormat::Ayuv_32 => Some(4),
        FrameFormat::Yuyv_4_2_2
        | FrameFormat::Uyvy_4_2_2
        | FrameFormat::Vyuy_4_2_2
        | FrameFormat::Yvyu_4_2_2 => Some(2),
        _ => None,
    }
}

fn semiplanar_stride(format: FrameFormat) -> Option<(u32, u32)> {
    match format {
        FrameFormat::NV24 | FrameFormat::NV42 => Some((1, 2)),
        FrameFormat::NV16 | FrameFormat::NV61 => Some((1, 1)),
        FrameFormat::P010 | FrameFormat::P012 => Some((1, 1)),
        FrameFormat::NV12 | FrameFormat::NV21 => Some((1, 1)),
        _ => None,
    }
}

fn planar_stride(format: FrameFormat) -> Option<(u32, u32, u32, u32)> {
    match format {
        // if you are here wondering if I will ever add another planar format
        // the answer is no.
        // do not bother opening an issue or a pr i will never merge it use a sane format like nv12
        FrameFormat::Yuv_4_2_0 => Some((1, 2, 2, 2)),
        _ => None,
    }
}

fn prepare_to_packed_image<'a>(
    frame_buffer: &'a FrameBuffer<'a>,
    resolution: Resolution,
    byte_width: u32,
    yuy_stride: u32,
) -> YuvPackedImage<'a, u8> {
    YuvPackedImage {
        yuy: frame_buffer.buffer(),
        yuy_stride: yuy_stride * resolution.width(),
        width: resolution.width(),
        height: resolution.height(),
    }
}

fn prepare_to_semi_planar_image<'a>(
    frame_buffer: &'a FrameBuffer<'a>,
    resolution: Resolution,
    byte_width: u32,
    y_stride: u32,
    uv_stride: u32,
) -> YuvBiPlanarImage<'a, u8> {
    let uv_start = (resolution.width() * resolution.height() * byte_width) as usize;
    YuvBiPlanarImage {
        y_plane: &frame_buffer.buffer()[0..uv_start],
        y_stride: y_stride * resolution.width(),
        uv_plane: &frame_buffer.buffer()[uv_start..frame_buffer.len()],
        uv_stride: uv_stride * resolution.width(),
        width: resolution.width(),
        height: resolution.height(),
    }
}

// fn planar_stride_per_4px(format: FrameFormat) -> Option<(u32, u32, u32)> {
//     match format {
//         FrameFormat::Yuv_4_2_0 => Some((stride_ 1, 1)),
//         _ => None,
//     }
// }

// does this work? idk
fn prepare_to_planar_image<'a>(
    frame_buffer: &'a FrameBuffer<'a>,
    resolution: Resolution,
    byte_width: u32,
    y_stride_base: u32,
    u_stride_ratio: u32,
    v_stride_ratio: u32,
    line_ratio: u32,
) -> YuvPlanarImage<'a, u8> {

    let y_stride = resolution.width() * y_stride_base;
    let u_stride = y_stride / u_stride_ratio;
    let v_stride = y_stride / v_stride_ratio;
    let chroma_lines = resolution.height() / line_ratio;

    // size of y area
    let u_start = resolution.height() * resolution.width() * byte_width;
    let v_start =  u_start + u_stride * chroma_lines * byte_width;

    let us = u_start as usize;
    let vs = v_start as usize;

    YuvPlanarImage {
        y_plane: &frame_buffer.buffer()[0..us],
        y_stride,
        u_plane: &frame_buffer.buffer()[us..vs],
        u_stride,
        v_plane: &frame_buffer.buffer()[vs..frame_buffer.len()],
        v_stride,
        width: resolution.width(),
        height: resolution.height(),
    }
}

fn convert_packed_image_to_u16(yuv_packed_image: YuvPackedImage<u8>) -> YuvPackedImage<u16> {
    let buf = cast_slice(yuv_packed_image.yuy);
    YuvPackedImage {
        yuy: buf,
        yuy_stride: yuv_packed_image.yuy_stride,
        width: yuv_packed_image.width,
        height: yuv_packed_image.height,
    }
}

fn convert_bi_planar_image_to_u16(
    yuv_bi_planar_image: YuvBiPlanarImage<u8>,
) -> YuvBiPlanarImage<u16> {
    let buf_y = cast_slice(yuv_bi_planar_image.y_plane);
    let buf_uv = cast_slice(yuv_bi_planar_image.uv_plane);
    YuvBiPlanarImage {
        y_plane: buf_y,
        y_stride: yuv_bi_planar_image.y_stride,
        uv_plane: buf_uv,
        uv_stride: yuv_bi_planar_image.uv_stride,
        width: yuv_bi_planar_image.width,
        height: yuv_bi_planar_image.height,
    }
}

#[cfg(test)]
mod test {
    use crate::yuv::{YUVConfig, YUVDecoder};
    use image::{DynamicImage, EncodableLayout, ImageBuffer, ImageFormat, Pixel, PixelWithColorType, Rgb, Rgba};
    use nokhwa_core::decoder::Decoder;
    use nokhwa_core::frame_buffer::FrameBuffer;
    use nokhwa_core::frame_format::FrameFormat;
    use nokhwa_core::image::NonFloatScalarWidth;
    use nokhwa_core::types::Resolution;
    use std::borrow::Cow;
    use std::fs::File;
    use std::io::{BufReader, Read};
    use yuv::{YuvConversionMode, YuvRange, YuvStandardMatrix};

    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum PixelFormat {
        Rgb8,
        RgbA8,
        Rgb16,
        RgbA16,
    }

    fn base_test<P: Pixel>(filename: String, resolution: Resolution, format: FrameFormat) -> ImageBuffer<P, Vec<P::Subpixel>>
where
        <P as Pixel>::Subpixel: NonFloatScalarWidth {
        let mut file = File::open(filename).unwrap();
        let mut nv12_data = Vec::new();
        file.read_to_end(&mut nv12_data).unwrap();

        let mut decoder = YUVDecoder::new(
            YUVConfig {
                resolution,
                yuv_type: format,
                range: YuvRange::Full,
                matrix: YuvStandardMatrix::Bt601,
                mode: YuvConversionMode::Balanced,
                premultiply_alpha: false,
                custom_frame_format_map: None,
            }
        );

        let frame_buffer = FrameBuffer::new(Cow::Owned(nv12_data), None);
        decoder.decode::<P>(frame_buffer).unwrap().buffer
    }

    fn write_image<P: Pixel + PixelWithColorType>(image: ImageBuffer<P, Vec<P::Subpixel>>, filename: String) where
    [<P as Pixel>::Subpixel]: EncodableLayout
    {
        image.save_with_format(filename, ImageFormat::Png).unwrap();
    }

    fn load_image(filename: String, format: ImageFormat) -> DynamicImage {
        let file = File::open(filename).unwrap();
        let image = image::load(BufReader::new(file), format).unwrap();
        image
    }
    
    #[test]
    fn test_nv12() {
        let base_filename = "test_images/yuv/nv12/crimeandsekai";
        let resolution = Resolution::new(1024, 1520);
        let format = FrameFormat::NV12;
        let img_format = ImageFormat::Png;

        // test nv12 rgb8
        {
            let image_rgb8 = load_image(format!("{base_filename}.rgb8.png"), img_format).to_rgb8();

            let out = base_test::<Rgb<u8>>(format!("{base_filename}.yuv"), resolution, format);
            assert_eq!(out.as_raw(), image_rgb8.as_raw());
        }
        // test nv12 rgba8
        {
            let image_rgba8 = load_image(format!("{base_filename}.rgba8.png"), img_format).to_rgba8();

            let out = base_test::<Rgba<u8>>(format!("{base_filename}.yuv"), resolution, format);

            assert_eq!(out.as_raw(), image_rgba8.as_raw());
        }
    }

    // FIXME: Can anyone make a 32bit AYUV image? I can't. Reenable this test after doing so.
    // #[test]
    // fn test_ayuv() {
    //     let base_filename = "test_images/yuv/ayuv/lhzlings";
    //     let resolution = Resolution::new(1000, 1080);
    //     let format = FrameFormat::Ayuv_32;
    //     let img_format = ImageFormat::Png;
    //
    //     {
    //         // let image_rgb8 = load_image(format!("{base_filename}.rgb"))
    //         let out = base_test::<Rgb<u8>>(format!("{base_filename}.yuv"), resolution, format);
    //
    //         write_image(out, format!("{base_filename}.rgb8.png"))
    //     }
    //
    //     {
    //         let out = base_test::<Rgba<u8>>(format!("{base_filename}.yuv"), resolution, format);
    //
    //         write_image(out, format!("{base_filename}.rgba8.png"))
    //     }
    // }

    #[test]
    fn test_nv24() {
        let base_filename = "test_images/yuv/nv24/larihole";
        let resolution = Resolution::new(800, 600);
        let format = FrameFormat::NV24;
        let img_format = ImageFormat::Png;

        {
            let image_rgb8 = load_image(format!("{base_filename}.rgb8.png"), img_format).to_rgb8();
            let out = base_test::<Rgb<u8>>(format!("{base_filename}.yuv"), resolution, format);
            assert_eq!(image_rgb8.as_raw(), out.as_raw());
            // write_image(out, format!("{base_filename}.rgb8.png"));
        }

        {
            let image_rgba8 = load_image(format!("{base_filename}.rgba8.png"), img_format).to_rgba8();
            let out = base_test::<Rgba<u8>>(format!("{base_filename}.yuv"), resolution, format);
            assert_eq!(image_rgba8.as_raw(), out.as_raw());
            // write_image(out, format!("{base_filename}.rgba8.png"));

        }
    }

    #[test]
    fn test_nv16() {
        let base_filename = "test_images/yuv/nv16/aeq";
        let resolution = Resolution::new(802, 602);
        let format = FrameFormat::NV16;
        let img_format = ImageFormat::Png;

        {
            let image_rgb8 = load_image(format!("{base_filename}.rgb8.png"), img_format).to_rgb8();
            let out = base_test::<Rgb<u8>>(format!("{base_filename}.yuv"), resolution, format);
            assert_eq!(image_rgb8.as_raw(), out.as_raw());
            // write_image(out, format!("{base_filename}.rgb8.png"));
        }

        {
            let image_rgba8 = load_image(format!("{base_filename}.rgba8.png"), img_format).to_rgba8();
            let out = base_test::<Rgba<u8>>(format!("{base_filename}.yuv"), resolution, format);
            assert_eq!(image_rgba8.as_raw(), out.as_raw());
            // write_image(out, format!("{base_filename}.rgba8.png"));
        }
    }

    #[test]
    fn test_p010() {
        let base_filename = "test_images/yuv/p010/crimesagainstvalor";
        let resolution = Resolution::new(128, 128);
        let format = FrameFormat::P010;
        let img_format = ImageFormat::Png;

        {
            let image_rgb8 = load_image(format!("{base_filename}.rgb8.png"), img_format).to_rgb8();
            let out = base_test::<Rgb<u8>>(format!("{base_filename}.yuv"), resolution, format);
            assert_eq!(image_rgb8.as_raw(), out.as_raw());
            // write_image(out, format!("{base_filename}.rgb8.png"));
        }

        {
            let image_rgba8 = load_image(format!("{base_filename}.rgba8.png"), img_format).to_rgba8();
            let out = base_test::<Rgba<u8>>(format!("{base_filename}.yuv"), resolution, format);
            assert_eq!(image_rgba8.as_raw(), out.as_raw());
            // write_image(out, format!("{base_filename}.rgba8.png"));
        }

        // there is some kind of bug in image-rs or smth idk cba to figure it out
        // {
        //     let out = base_test::<Rgb<u16>>(format!("{base_filename}.yuv"), resolution, format);
        //     // write_image(out, format!("{base_filename}.rgb16.png"));
        //     out.save_with_format(format!("{base_filename}.rgb16.avif"), ImageFormat::Avif).unwrap();
        // }
    }

    #[test]
    fn test_yuv420() {
        let base_filename = "test_images/yuv/yuv420/youarethemurderr";
        let resolution = Resolution::new(460, 460);
        let format = FrameFormat::Yuv_4_2_0;
        let img_format = ImageFormat::Png;

        {
            let image_rgb8 = load_image(format!("{base_filename}.rgb8.png"), img_format).to_rgb8();
            let out = base_test::<Rgb<u8>>(format!("{base_filename}.yuv"), resolution, format);
            // write_image(out, format!("{base_filename}.rgb8.png"));
            assert_eq!(image_rgb8.as_raw(), out.as_raw());
        }

        {
            let image_rgba8 = load_image(format!("{base_filename}.rgba8.png"), img_format).to_rgba8();
            let out = base_test::<Rgba<u8>>(format!("{base_filename}.yuv"), resolution, format);
            // write_image(out, format!("{base_filename}.rgba8.png"));
            assert_eq!(image_rgba8.as_raw(), out.as_raw());
        }
    }

    #[test]
    fn test_yuyv() {
        let base_filename = "test_images/yuv/yuyv/20250530_224336";
        let resolution = Resolution::new(3024, 4032);
        let format = FrameFormat::Yuyv_4_2_2;
        let img_format = ImageFormat::Png;

        {
            let image_rgb8 = load_image(format!("{base_filename}.rgb8.png"), img_format).to_rgb8();
            let out = base_test::<Rgb<u8>>(format!("{base_filename}.yuv"), resolution, format);
            assert_eq!(image_rgb8.as_raw(), out.as_raw());
        }

        {
            let image_rgba8 = load_image(format!("{base_filename}.rgba8.png"), img_format).to_rgba8();
            let out = base_test::<Rgba<u8>>(format!("{base_filename}.yuv"), resolution, format);
            assert_eq!(image_rgba8.as_raw(), out.as_raw());
        }
    }


}
