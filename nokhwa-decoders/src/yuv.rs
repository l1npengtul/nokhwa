use bytemuck::{cast_slice, cast_slice_mut, try_cast_slice_mut};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_buffer::FrameBuffer;
use nokhwa_core::frame_format::FrameFormat;
use nokhwa_core::types::{CameraFormat, Resolution};
use yuv::{ayuv_to_rgb, ayuv_to_rgba, p010_to_bgr, p010_to_bgra, p010_to_rgb, p010_to_rgb10, p010_to_rgba, p010_to_rgba10, p012_to_rgb12, p012_to_rgba12, uyvy422_to_bgr, uyvy422_to_bgra, uyvy422_to_rgb, uyvy422_to_rgb_p16, uyvy422_to_rgba, uyvy422_to_rgba_p16, vyuy422_to_bgr, vyuy422_to_bgra, vyuy422_to_rgb, vyuy422_to_rgb_p16, vyuy422_to_rgba, vyuy422_to_rgba_p16, yuv420_to_bgr, yuv420_to_bgra, yuv420_to_rgb, yuv420_to_rgba, yuv_nv12_to_bgr, yuv_nv12_to_bgra, yuv_nv12_to_rgb, yuv_nv12_to_rgba, yuv_nv16_to_bgr, yuv_nv16_to_bgra, yuv_nv16_to_rgb, yuv_nv16_to_rgba, yuv_nv21_to_bgr, yuv_nv21_to_bgra, yuv_nv21_to_rgb, yuv_nv21_to_rgba, yuv_nv24_to_bgr, yuv_nv24_to_bgra, yuv_nv24_to_rgb, yuv_nv24_to_rgba, yuv_nv42_to_bgr, yuv_nv42_to_bgra, yuv_nv42_to_rgb, yuv_nv42_to_rgba, yuv_nv61_to_bgr, yuv_nv61_to_bgra, yuv_nv61_to_rgb, yuv_nv61_to_rgba, yuyv422_to_bgr, yuyv422_to_bgra, yuyv422_to_rgb, yuyv422_to_rgb_p16, yuyv422_to_rgba, yuyv422_to_rgba_p16, yvyu422_to_bgr, yvyu422_to_bgra, yvyu422_to_rgb, yvyu422_to_rgb_p16, yvyu422_to_rgba, yvyu422_to_rgba_p16, YuvBiPlanarImage, YuvConversionMode, YuvPackedImage, YuvPlanarImage, YuvRange, YuvStandardMatrix};
use nokhwa_core::decoder::{Decoder, ImageBuffer, Pixel, Primitive};
use nokhwa_core::image::{DecodedImage, NonFloatScalarWidth};

pub struct YUVDecoder {
    config: YUVConfig,
    stride_cache: Option<CachedStride>,
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
            return Err(NokhwaError::DecoderUnsupportedFrameFormat(config.yuv_type))
        }
        self.config = config;
        self.stride_cache = None;
        Ok(())
    }

    fn decode_to_buffer(&mut self, to_decode: FrameBuffer<'_>, mut buffer: impl AsMut<[u8]>, destination_format: Option<Self::DestinationFormatHint>) -> Result<Self::OutputMeta, NokhwaError> {
        let destination_format = match destination_format {
            Some(df) => df,
            None => return Err(NokhwaError::DecoderDestinationHintRequired)
        };
        
        let buffer = buffer.as_mut();
        if buffer.len() < self.output_decoder_min_size(self.config.resolution, destination_format) {
            return Err(NokhwaError::DecoderInvalidBuffer("Too small!".to_string()))
        }

        let stride = match self.stride_cache {
            Some(c) => c,
            None => {
                self.stride_cache = figure_out_stride(self.config.yuv_type);
                match self.stride_cache {
                    Some(s) => s,
                    None => return Err(NokhwaError::DecoderUnsupportedFrameFormat(self.config.yuv_type)),
                }
            }
        };

        // todo: clean up ts into a macro </3
        let decode_status = match stride {
            CachedStride::Packed(stride) => {
                let image = prepare_to_packed_image(&to_decode, self.config.resolution, stride);
                match self.config.yuv_type {
                    FrameFormat::Ayuv_32 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(ayuv_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix, self.config.premultiply_alpha)),
                            YUVDestination::Rgba8 => Some(ayuv_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix, self.config.premultiply_alpha)),
                            _ => None,
                        }
                    }
                    FrameFormat::Yuyv_4_2_2 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuyv422_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(yuyv422_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix)),
                            YUVDestination::Rgb16 => Some(yuyv422_to_rgb_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), 6, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(yuyv422_to_rgba_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), 6, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(yuyv422_to_bgr(&image, buffer, 8, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(yuyv422_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix)),
                        }
                    }
                    FrameFormat::Uyvy_4_2_2 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(uyvy422_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(uyvy422_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix)),
                            YUVDestination::Rgb16 => Some(uyvy422_to_rgb_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), 6, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(uyvy422_to_rgba_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), 6, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(uyvy422_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(uyvy422_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix)),
                        }
                    }
                    FrameFormat::Vyuy_4_2_2 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(vyuy422_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(vyuy422_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix)),
                            YUVDestination::Rgb16 => Some(vyuy422_to_rgb_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), 6, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(vyuy422_to_rgba_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), 6, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(vyuy422_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(vyuy422_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix)),
                        }
                    }
                    FrameFormat::Yvyu_4_2_2 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yvyu422_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(yvyu422_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix)),
                            YUVDestination::Rgb16 => Some(yvyu422_to_rgb_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), 6, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(yvyu422_to_rgba_p16(&convert_packed_image_to_u16(image), cast_slice_mut(buffer), 6, 16, self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(yvyu422_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(yvyu422_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix)),
                        }
                    }
                    _ => {
                        if FrameFormat::YCBCR_PACKED.contains(&self.config.yuv_type) {
                            return Err(NokhwaError::NotImplementedError("etto blehhh!".to_string()))
                        }
                        // shouldnt happen
                        return Err(NokhwaError::DecoderUnsupportedFrameFormat(self.config.yuv_type))
                    }
                }
            }
            CachedStride::SemiPlanar(y_stride, uv_stride) => {
                let image = prepare_to_semi_planar_image(&to_decode, self.config.resolution, y_stride, uv_stride);
                match self.config.yuv_type {
                    FrameFormat::NV24 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv24_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv24_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv24_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv24_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV42 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv42_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv42_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv42_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv42_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV16 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv16_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv16_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv16_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv16_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV61 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv61_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv61_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv61_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv61_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV12 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv12_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv12_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv12_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv12_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::NV21 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv_nv21_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(yuv_nv21_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(yuv_nv21_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(yuv_nv21_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            _ => None,
                        }
                    }
                    FrameFormat::P010 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(p010_to_rgb(&convert_bi_planar_image_to_u16(image), buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgba8 => Some(p010_to_rgba(&convert_bi_planar_image_to_u16(image), buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgr8 => Some(p010_to_bgr(&convert_bi_planar_image_to_u16(image), buffer, 3, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Bgra8 => Some(p010_to_bgra(&convert_bi_planar_image_to_u16(image), buffer, 4, self.config.range, self.config.matrix, self.config.mode)),
                            YUVDestination::Rgb16 => Some(p010_to_rgb10(&convert_bi_planar_image_to_u16(image), cast_slice_mut(buffer), 6, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(p010_to_rgba10(&convert_bi_planar_image_to_u16(image), cast_slice_mut(buffer), 8, self.config.range, self.config.matrix)),
                            // _ => None,
                        }
                    }
                    FrameFormat::P012 => {
                        match destination_format {
                            YUVDestination::Rgb16 => Some(p012_to_rgb12(&convert_bi_planar_image_to_u16(image), cast_slice_mut(buffer), 6, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba16 => Some(p012_to_rgba12(&convert_bi_planar_image_to_u16(image), cast_slice_mut(buffer), 8, self.config.range, self.config.matrix)),
                            _ => None,
                        }
                    }
                    _ => {
                        if FrameFormat::YCBCR_SEMI.contains(&self.config.yuv_type) {
                            return Err(NokhwaError::NotImplementedError("etto blehhh!".to_string()))
                        }
                        // shouldnt happen
                        return Err(NokhwaError::DecoderUnsupportedFrameFormat(self.config.yuv_type))
                    }
                }
            }
            CachedStride::Planar(y_stride, u_stride, v_stride) => {
                let image = prepare_to_planar_image(&to_decode, self.config.resolution, y_stride, u_stride, v_stride);
                match self.config.yuv_type {
                    FrameFormat::Yuv_4_2_0 => {
                        match destination_format {
                            YUVDestination::Rgb8 => Some(yuv420_to_rgb(&image, buffer, 3, self.config.range, self.config.matrix)),
                            YUVDestination::Rgba8 => Some(yuv420_to_rgba(&image, buffer, 4, self.config.range, self.config.matrix)),
                            YUVDestination::Bgr8 => Some(yuv420_to_bgr(&image, buffer, 3, self.config.range, self.config.matrix)),
                            YUVDestination::Bgra8 => Some(yuv420_to_bgra(&image, buffer, 4, self.config.range, self.config.matrix)),
                            _ => None,
                        }
                    }
                    _ => {
                        if FrameFormat::YCBCR_PLANAR.contains(&self.config.yuv_type) {
                            return Err(NokhwaError::NotImplementedError("etto blehhh!".to_string()))
                        }
                        // shouldnt happen
                        return Err(NokhwaError::DecoderUnsupportedFrameFormat(self.config.yuv_type))
                    }
                }
            }
        };
        match decode_status {
            Some(Ok(_)) => Ok(()),
            Some(Err(why)) => Err(NokhwaError::Decoder(why.to_string())),
            None => Err(NokhwaError::DecoderUnsupportedFrameFormat(self.config.yuv_type)),
        }
    }

    fn decode_to_pixel_buffer<P: Pixel>(&mut self, to_decode: FrameBuffer<'_>, mut buffer: impl AsMut<[P::Subpixel]>) -> Result<Self::OutputMeta, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth
    {
        let destination = match YUVDestination::get_by_pixel::<P>() {
            None => return Err(NokhwaError::DecoderUnsupportedDestinationPixelFormat(P::COLOR_MODEL, <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES)),
            Some(d) => d,
        };
        let buffer = buffer.as_mut();

        let cast_slice = try_cast_slice_mut::<P::Subpixel, u8>(buffer)
            .map_err(|why| NokhwaError::DecoderInvalidBuffer(why.to_string()))?;
        
        self.decode_to_buffer(to_decode, cast_slice, Some(destination))
    }

    fn decode<P: Pixel>(&mut self, to_decode: FrameBuffer<'_>) -> Result<DecodedImage<P, Self::OutputMeta>, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth
    {
        let min_size_alloc = self.output_decoder_min_size_pixel::<P>(self.config.resolution);
        let mut out_buffer: Vec<P::Subpixel> = vec![P::Subpixel::DEFAULT_MIN_VALUE; min_size_alloc];
        self.decode_to_pixel_buffer::<P>(to_decode, &mut out_buffer)?;
        Ok(
            DecodedImage::new(
                ImageBuffer::from_vec(self.config.resolution.width(), self.config.resolution.height(), out_buffer)
                    .ok_or(NokhwaError::Decoder("failed to convert into an image buffer".to_string()))?,
                ()
            )
        )
    }

    fn output_decoder_min_size(&self, resolution: Resolution, destination_format: Self::DestinationFormatHint) -> usize {
        let px_size = match destination_format {
            YUVDestination::Rgb8 | YUVDestination::Bgr8 => 3,
            YUVDestination::Rgba8  | YUVDestination::Bgra8 => 4,
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
    pub fn get_by_pixel<P>() -> Option<Self> where P: Pixel, <P as Pixel>::Subpixel: NonFloatScalarWidth {
        match P::COLOR_MODEL {
            "RGB" => {
                match <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES {
                    1 => Some(YUVDestination::Rgb8),
                    2 => Some(YUVDestination::Rgb16),
                    _ => None,
                }
            }
            "RGBA" =>  match <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES {
                1 => Some(YUVDestination::Rgba8),
                2 => Some(YUVDestination::Rgba16),
                _ => None,
            }
            "BGR" =>match <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES {
                1 => Some(YUVDestination::Bgr8),
                // 2 => Some(YUVDestination::Bgr16),
                _ => None,
            }
            "BGRA" => match <<P as Pixel>::Subpixel as NonFloatScalarWidth>::WIDTH_BYTES {
                1 => Some(YUVDestination::Bgra8),
                // 2 => Some(YUVDestination::Bgra16),
                _ => None,
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub struct YUVConfig {
    pub resolution: Resolution,
    pub yuv_type: FrameFormat,
    pub range: YuvRange,
    pub matrix: YuvStandardMatrix,
    pub mode: YuvConversionMode,
    pub premultiply_alpha: bool,
}

impl TryFrom<CameraFormat> for YUVConfig {
    type Error = NokhwaError;

    fn try_from(value: CameraFormat) -> Result<Self, Self::Error> {
        if !FrameFormat::YCBCR.contains(&value.format()) {
            return Err(NokhwaError::DecoderUnsupportedFrameFormat(value.format()))
        }
        Ok(YUVConfig {
            resolution: value.resolution(),
            yuv_type: value.format(),
            range: YuvRange::Full,
            matrix: YuvStandardMatrix::Bt601,
            mode: YuvConversionMode::Balanced,
            premultiply_alpha: false,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
enum CachedStride {
    Packed(u32),
    SemiPlanar(u32, u32),
    Planar(u32, u32, u32),
}

fn figure_out_stride(frame_format: FrameFormat) -> Option<CachedStride> {
    if let Some(s) = packed_stride_component_per_4px(frame_format) {
        return Some(CachedStride::Packed(s));
    }
    if let Some((s1, s2)) = semiplanar_stride_per_4px(frame_format) {
        return Some(CachedStride::SemiPlanar(s1, s2));
    }
    if let Some((s1, s2, s3)) = planar_stride_per_4px(frame_format) {
        return Some(CachedStride::Planar(s1, s2, s3));
    }
    None
}

fn packed_stride_component_per_4px(format: FrameFormat) -> Option<u32> {
    match format {
        FrameFormat::Ayuv_32 => Some(64),
        FrameFormat::Yuyv_4_2_2
        | FrameFormat::Uyvy_4_2_2
        | FrameFormat::Vyuy_4_2_2
        | FrameFormat::Yvyu_4_2_2 => Some(16),
        _ => None,
    }
}

fn prepare_to_packed_image<'a>(
    frame_buffer: &'a FrameBuffer<'a>,
    resolution: Resolution,
    yuy_stride: u32,
) -> YuvPackedImage<'a, u8> {
    YuvPackedImage {
        yuy: frame_buffer.buffer(),
        yuy_stride,
        width: resolution.width(),
        height: resolution.height(),
    }
}

fn semiplanar_stride_per_4px(format: FrameFormat) -> Option<(u32, u32)> {
    match format {
        FrameFormat::NV24 | FrameFormat::NV42 => Some((4, 8)),
        FrameFormat::NV16 | FrameFormat::NV61 => Some((4, 4)),
        FrameFormat::NV12 | FrameFormat::NV21 | FrameFormat::P010 | FrameFormat::P012 => {
            Some((4, 4))
        }
        _ => None,
    }
}

fn prepare_to_semi_planar_image<'a>(
    frame_buffer: &'a FrameBuffer<'a>,
    resolution: Resolution,
    y_stride: u32,
    uv_stride: u32,
) -> YuvBiPlanarImage<'a, u8> {
    let uv_start = (resolution.width() * resolution.height()) as usize;
    YuvBiPlanarImage {
        y_plane: &frame_buffer.buffer()[0..uv_start],
        y_stride,
        uv_plane: &frame_buffer.buffer()[uv_start..frame_buffer.len()],
        uv_stride,
        width: resolution.width(),
        height: resolution.height(),
    }
}

fn planar_stride_per_4px(format: FrameFormat) -> Option<(u32, u32, u32)> {
    match format {
        FrameFormat::Yuv_4_2_0 => Some((4, 1, 1)),
        _ => None,
    }
}

fn prepare_to_planar_image<'a>(
    frame_buffer: &'a FrameBuffer<'a>,
    resolution: Resolution,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
) -> YuvPlanarImage<'a, u8> {
    let u_start = (resolution.width() * resolution.height()) as usize;
    let v_start =
        u_start + ((resolution.width() / 4) * y_stride * (resolution.height() * u_stride)) as usize;
    YuvPlanarImage {
        y_plane: &frame_buffer.buffer()[0..u_start],
        y_stride,
        u_plane: &frame_buffer.buffer()[u_start..v_start],
        u_stride,
        v_plane: &frame_buffer.buffer()[v_start..frame_buffer.len()],
        v_stride,
        width: resolution.width(),
        height: resolution.height(),
    }
}

fn convert_packed_image_to_u16(
    yuv_packed_image: YuvPackedImage<u8>,
) -> YuvPackedImage<u16> {
    let buf= cast_slice(yuv_packed_image.yuy);
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
    let buf_y= cast_slice(yuv_bi_planar_image.y_plane);
    let buf_uv= cast_slice(yuv_bi_planar_image.uv_plane);
    YuvBiPlanarImage {
        y_plane: buf_y,
        y_stride: yuv_bi_planar_image.y_stride,
        uv_plane: buf_uv,
        uv_stride: yuv_bi_planar_image.uv_stride,
        width: yuv_bi_planar_image.width,
        height: yuv_bi_planar_image.height,
    }
}
