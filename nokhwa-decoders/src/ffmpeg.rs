use core::mem::transmute;
use ffmpeg_the_third::codec::{Context, Id};
use ffmpeg_the_third::color::{Range, Space};
use ffmpeg_the_third::decoder::{Video, find};
use ffmpeg_the_third::ffi::{
    AVCodecID, AVCodecParameters, AVMediaType, AVPacketSideData, AVPixelFormat, AVRational,
    av_frame_alloc, av_image_fill_arrays, av_packet_side_data_free, avcodec_parameters_alloc,
    avcodec_parameters_free,
};
use ffmpeg_the_third::format::Pixel as FfmpegPixel;
use ffmpeg_the_third::packet::Packet;
use ffmpeg_the_third::software::scaling::context::Context as ScalerContext;
use ffmpeg_the_third::{AsPtr, frame};
use nokhwa_core::decoder::{ConfigHasResolution, Decoder};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_buffer::FrameBuffer;
use nokhwa_core::frame_format::{CustomFrameFormat, FrameFormat};
use nokhwa_core::pixel_destination::PixelDestination;
use nokhwa_core::types::{CameraFormat, FrameRate, Resolution};
use std::collections::HashMap;
use std::ptr::NonNull;

// heh heh
// ffmpreg

pub use ffmpeg_the_third::{
    FieldOrder, Rational,
    chroma::Location,
    codec::Flags as CodecFlags,
    codec::threading::{Config as ThreadingConfig, Type as ThreadingType},
    color::{Primaries, TransferCharacteristic},
    frame::flag::Flags as FrameFlags,
    frame::side_data::SideData as FrameSideData,
    frame::side_data::Type as SideDataType,
    software::scaling::Flags as ScalingFlags,
    util::dictionary::Owned as DictionaryOwned,
};

fn create_video(config: &FfmpegConfig) -> Result<Video, NokhwaError> {
    let id = convert_format_to_codec_id(&config.frame_format).ok_or(
        NokhwaError::DecoderUnsupportedFrameFormat(config.frame_format),
    )?;
    let codec = find(id).ok_or(NokhwaError::DecoderInvalidConfiguration(format!(
        "Could not find a suitable codec for {id:?}"
    )))?;

    let mut video = Context::new_with_codec(codec)
        .decoder()
        .video()
        .map_err(|why| {
            NokhwaError::DecoderInvalidConfiguration(format!(
                "Could not get a video decoder: {why}"
            ))
        })?;

    let frame_rate = AVRational {
        num: config.frame_rate.numerator() as i32,
        den: config.frame_rate.denominator() as i32,
    };
    let codec_i32 = unsafe { transmute::<AVCodecID, i32>(AVCodecID::from(id)) };
    let intermediate_config = IntermediateDecoderConfig {
        config: config.ffmpeg_codec_low_level.clone(),
        resolution: config.resolution,
        frame_rate,
        format: codec_i32,
    };

    video
        .set_parameters(intermediate_config)
        .map_err(|why| NokhwaError::DecoderInvalidConfiguration(why.to_string()))?;

    let threading_kind = config.parallelism.behavior;

    let threads = match config.parallelism.threads {
        Threads::Set(t) => t,
        Threads::System => std::thread::available_parallelism()
            .map(|parallelism| parallelism.get())
            .map_err(|why| {
                NokhwaError::DecoderInvalidConfiguration(format!(
                    "Failed to get availible parallelism: {why}"
                ))
            })?,
        Threads::One => 1_usize,
    };

    video.set_threading(ThreadingConfig {
        kind: threading_kind,
        count: threads,
    });
    if let Some(flags) = config.ffmpeg_codec_low_level.flags {
        let flags = CodecFlags::from_bits(flags).ok_or(
            NokhwaError::DecoderInvalidConfiguration("Bad Flags!".to_string()),
        )?;
        video.set_flags(flags);
    }
    if let Some(field_order) = config.ffmpeg_codec_low_level.field_order {
        video.set_field_order(field_order);
    }

    Ok(video)
}

fn create_sws(
    config: &FfmpegConfig,
    source_pixel: FfmpegPixel,
    pixel_destination: FfmpegPixel,
) -> Result<ScalerContext, NokhwaError> {
    let dest_resolution = config
        .ffmpeg_scaler_low_level
        .destionation_resolution
        .unwrap_or(config.resolution);

    let source_resolution = config.resolution;
    let flags = config.ffmpeg_scaler_low_level.flags;

    ScalerContext::get(
        source_pixel,
        source_resolution.width(),
        source_resolution.height(),
        pixel_destination,
        dest_resolution.width(),
        dest_resolution.height(),
        ScalingFlags::from_bits(flags).unwrap(),
    )
    .map_err(|why| NokhwaError::DecoderInitializationError(why.to_string()))
}

#[derive(Clone, Debug)]
pub struct FfmpegOutputMeta<'a> {
    pub metadata: DictionaryOwned<'a>,

    pub timestamp: Option<i64>,
    pub pts: Option<i64>,

    pub is_corrupt: bool,
    pub is_empty: bool,

    pub color_space: Space,
    pub color_range: Range,
    pub color_primaries: Primaries,
    pub color_transfer_characteristic: TransferCharacteristic,
    pub chroma_location: Location,
    pub aspect_ratio: Rational,
    pub repeat: f64,
    pub planes: usize,

    pub quality: usize,
    pub flags: i32,
}

pub struct FfmpegDecoder {
    decoder: Video,
    sws: ScalerContext,
    config: FfmpegConfig,
    last_used_out_format: FfmpegPixel,
    last_used_in_format: FfmpegPixel,
}

impl FfmpegDecoder {
    pub fn new(config: FfmpegConfig) -> Result<Self, NokhwaError> {
        let decoder = create_video(&config)?;
        let last_used_in_format = match decoder.format() {
            FfmpegPixel::None => FfmpegPixel::RGB24,
            px => px,
        };
        let last_used_out_format = FfmpegPixel::RGB24;
        let sws = create_sws(&config, last_used_in_format, last_used_out_format)?;
        Ok(FfmpegDecoder {
            decoder,
            sws,
            config,
            last_used_out_format,
            last_used_in_format,
        })
    }

    pub fn with_format(format: &CameraFormat) -> Result<Self, NokhwaError> {
        let config = FfmpegConfig {
            custom_frame_format_map: None,
            frame_format: format.format(),
            resolution: format.resolution(),
            frame_rate: format.frame_rate(),
            parallelism: Parallelism::default(),
            ffmpeg_codec_low_level: FfmpegDecoderConfig::default(),
            ffmpeg_scaler_low_level: FfmpegScalerConfig::default(),
            side_data_check_list: Vec::new(),
        };

        Self::new(config)
    }
}

impl Decoder for FfmpegDecoder {
    type Config = FfmpegConfig;
    type OutputMeta = FfmpegOutputMeta<'static>;
    const SUPPORTED_DESTINATIONS: &'static [PixelDestination] = PixelDestination::ALL;

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        self.decoder = create_video(&config)?;
        let last_used_in_format = match self.decoder.format() {
            FfmpegPixel::None => FfmpegPixel::RGB24,
            px => px,
        };
        self.sws = create_sws(&config, last_used_in_format, self.last_used_out_format)?;
        self.last_used_in_format = last_used_in_format;
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

        let dest_as_ffmpeg = convert_destination_pixel_format_to_av_pxfmt(destination_format);

        let packet = Packet::borrow(to_decode.buffer());
        self.decoder
            .send_packet(&packet)
            .map_err(|why| NokhwaError::DecoderInvalidBuffer(why.to_string()))?;

        let mut frame = frame::Video::new(
            self.decoder.format(),
            self.decoder.width(),
            self.decoder.width(),
        );

        if let Err(why) = self.decoder.receive_frame(&mut frame) {
            if let ffmpeg_the_third::util::error::Error::Other { errno: 11 } = why {
                return Err(NokhwaError::DecoderNeedsMoreData(why.to_string()));
            } else {
                return Err(NokhwaError::Decoder(why.to_string()));
            }
        }

        let receiver_avframe = unsafe { av_frame_alloc() };
        let dest_resolution = self
            .config
            .ffmpeg_scaler_low_level
            .destionation_resolution
            .unwrap_or(self.config.resolution);
        let av_pixel_format: AVPixelFormat = dest_as_ffmpeg.into();
        unsafe {
            (*receiver_avframe).width = dest_resolution.width() as i32;
            (*receiver_avframe).height = dest_resolution.height() as i32;
            (*receiver_avframe).format = av_pixel_format as i32;
        }
        let imgbuf = unsafe {
            av_image_fill_arrays(
                (&mut (*receiver_avframe).data) as *mut *mut u8,
                (&mut (*receiver_avframe).linesize) as *mut i32,
                buffer.as_ptr(),
                av_pixel_format,
                dest_resolution.width() as i32,
                dest_resolution.height() as i32,
                1,
            )
        };
        if imgbuf.is_negative() {
            return Err(NokhwaError::DecoderInvalidBuffer(format!(
                "Failed to av_image_fill_arrays: {imgbuf}"
            )));
        }
        let mut receiver_frame = unsafe { frame::Video::wrap(receiver_avframe) };

        // remake swscontext in case we are being gooned
        let new_in_format = match frame.format() {
            FfmpegPixel::None => {
                return Err(NokhwaError::Decoder(
                    "Received None format from ffmpeg buffer. Invalid - cannot use sws".to_string(),
                ));
            }
            fmt => fmt,
        };

        if new_in_format != self.last_used_in_format
            || receiver_frame.format() != self.last_used_out_format
        {
            self.sws = create_sws(&self.config, new_in_format, receiver_frame.format())?;
            self.last_used_in_format = new_in_format;
            self.last_used_out_format = receiver_frame.format();
        }
        self.sws
            .run(&frame, &mut receiver_frame)
            .map_err(|why| NokhwaError::Decoder(why.to_string()))?;
        let metadata = receiver_frame.metadata().to_owned();
        let timestamp = receiver_frame.timestamp();
        let pts = receiver_frame.pts();
        let is_corrupt = receiver_frame.is_corrupt();
        let is_empty = unsafe { receiver_frame.is_empty() };
        let color_space = receiver_frame.color_space();
        let color_range = receiver_frame.color_range();
        let color_primaries = receiver_frame.color_primaries();
        let color_transfer_characteristic = receiver_frame.color_transfer_characteristic();
        let chroma_location = receiver_frame.chroma_location();
        let aspect_ratio = receiver_frame.aspect_ratio();
        let repeat = receiver_frame.repeat();
        let planes = receiver_frame.planes();
        let quality = receiver_frame.quality();
        let flags = receiver_frame.flags().bits();

        Ok(FfmpegOutputMeta {
            metadata,
            timestamp,
            pts,
            is_corrupt,
            is_empty,
            color_space,
            color_range,
            color_primaries,
            color_transfer_characteristic,
            chroma_location,
            aspect_ratio,
            repeat,
            planes,
            quality,
            flags,
        })
    }
}

fn convert_format_to_codec_id(frame_format: &FrameFormat) -> Option<Id> {
    if FrameFormat::RAW.contains(frame_format)
        || FrameFormat::YCBCR.contains(frame_format)
        || FrameFormat::BRIGHTNESS.contains(frame_format)
    {
        return Some(Id::RAWVIDEO);
    }

    match frame_format {
        FrameFormat::H265 => Some(Id::HEVC),
        FrameFormat::H264 => Some(Id::H264),
        FrameFormat::AVC1 => Some(Id::H264),
        FrameFormat::H263 => Some(Id::H263),
        FrameFormat::AV1 => Some(Id::AV1),
        FrameFormat::MPEG_1 => Some(Id::MPEG1VIDEO),
        FrameFormat::MPEG_2 => Some(Id::MPEG2VIDEO),
        FrameFormat::MPEG_4 => Some(Id::MPEG4),
        FrameFormat::MJPEG => Some(Id::MJPEG),
        FrameFormat::XviD => Some(Id::MPEG4),
        FrameFormat::VP8 => Some(Id::VP8),
        FrameFormat::VP9 => Some(Id::VP9),
        FrameFormat::Custom(c) => {
            if let CustomFrameFormat::U32(c_id) = c {
                // SAFETY: /shrug
                let av_codec_id: AVCodecID = unsafe { transmute(*c_id) };
                Some(av_codec_id.into())
            } else {
                None
            }
        }
        _ => None,
    }
}

// fn convert_frame_format_to_pixfmt(frame_format: &FrameFormat) -> Option<FfmpegPixel> {
//     match frame_format {
//         FrameFormat::Ayuv_32 => Some(FfmpegPixel::AYUV64), // i think these are the same?
//         FrameFormat::NV24 => Some(FfmpegPixel::NV24),
//         FrameFormat::NV42 => Some(FfmpegPixel::NV42),
//         FrameFormat::Yuyv_4_2_2 => Some(FfmpegPixel::YUYV422),
//         FrameFormat::Uyvy_4_2_2 => Some(FfmpegPixel::UYVY422),
//         FrameFormat::Yvyu_4_2_2 => Some(FfmpegPixel::YVYU422),
//         FrameFormat::NV16 => Some(FfmpegPixel::NV16),
//         FrameFormat::Yuv_4_2_0 => Some(FfmpegPixel::YUV420P),
//         FrameFormat::NV12 => Some(FfmpegPixel::NV12),
//         FrameFormat::NV21 => Some(FfmpegPixel::NV21),
//         FrameFormat::P010 => {
//             if is_little_endian() {
//                 Some(FfmpegPixel::P010LE)
//             } else {
//                 Some(FfmpegPixel::P010BE)
//             }
//         }
//         FrameFormat::P012 => {
//             if is_little_endian() {
//                 Some(FfmpegPixel::P012LE)
//             } else {
//                 Some(FfmpegPixel::P012BE)
//             }
//         }
//         FrameFormat::Luma_8 => Some(FfmpegPixel::GRAY8),
//         FrameFormat::Luma_10 => {
//             if is_little_endian() {
//                 Some(FfmpegPixel::GRAY10LE)
//             } else {
//                 Some(FfmpegPixel::GRAY10BE)
//             }
//         }
//         FrameFormat::Luma_12 => {
//             if is_little_endian() {
//                 Some(FfmpegPixel::GRAY12LE)
//             } else {
//                 Some(FfmpegPixel::GRAY12BE)
//             }
//         }
//         FrameFormat::Luma_14 => {
//             if is_little_endian() {
//                 Some(FfmpegPixel::GRAY14LE)
//             } else {
//                 Some(FfmpegPixel::GRAY14BE)
//             }
//         }
//         FrameFormat::Luma_16 | FrameFormat::Depth_16 => Some(FfmpegPixel::GRAY16),
//         FrameFormat::Rgb_3_3_2 => Some(FfmpegPixel::RGB8),
//         FrameFormat::Rgb_5_6_5 => Some(FfmpegPixel::RGB565),
//         FrameFormat::Rgb_5_5_5 => Some(FfmpegPixel::RGB555),
//         FrameFormat::Rgb_8_8_8 => Some(FfmpegPixel::RGB24),
//         FrameFormat::Argb_8_8_8_8 => Some(FfmpegPixel::ARGB),
//         FrameFormat::Rgba_8_8_8_8 => Some(FfmpegPixel::RGBA),
//         FrameFormat::Bgr_3_3_2 => Some(FfmpegPixel::BGR8),
//         FrameFormat::Bgr_5_6_5 => Some(FfmpegPixel::BGR565),
//         FrameFormat::Bgr_5_5_5 => Some(FfmpegPixel::BGR555),
//         FrameFormat::Bgr_8_8_8 => Some(FfmpegPixel::BGR24),
//         FrameFormat::Abgr_8_8_8_8 => Some(FfmpegPixel::ABGR),
//         FrameFormat::Bgra_8_8_8_8 => Some(FfmpegPixel::BGRA),
//         _ => None,
//     }
// }

#[derive(Copy, Clone, Debug)]
pub struct SideData<T>
where
    T: Sized,
{
    pub pointer: NonNull<T>,
    pub length: usize,
}

#[derive(Clone, Debug, Default)]
pub struct FfmpegDecoderConfig {
    pub codec_tag: Option<u32>,
    pub bit_rate: Option<i64>,
    pub profile: Option<i32>,
    pub level: Option<i32>,
    pub aspect_ratio: Option<Rational>,
    pub field_order: Option<FieldOrder>,
    pub color_range: Option<Range>,
    pub color_primaries: Option<Primaries>,
    pub color_transfer_characteristics: Option<TransferCharacteristic>,
    pub chroma_location: Option<Location>,
    // Unsupported!
    // pub video_delay: Option<i32>,
    pub flags: Option<u32>,

    pub coded_side_data: Option<SideData<AVPacketSideData>>,
    pub extra_data: Option<SideData<u8>>,
}

#[derive(Clone, Debug)]
struct IntermediateDecoderConfig {
    pub(crate) config: FfmpegDecoderConfig,
    pub(crate) resolution: Resolution,
    pub(crate) frame_rate: AVRational,
    pub(crate) format: i32,
}

impl AsPtr<AVCodecParameters> for IntermediateDecoderConfig {
    fn as_ptr(&self) -> *const AVCodecParameters {
        let parameters = unsafe { avcodec_parameters_alloc() };

        unsafe {
            (*parameters).width = self.resolution.width() as i32;
            (*parameters).height = self.resolution.height() as i32;
            (*parameters).format = self.format;
            (*parameters).codec_type = AVMediaType::AVMEDIA_TYPE_VIDEO;
            (*parameters).framerate = self.frame_rate;

            if let Some(codec_tag) = self.config.codec_tag {
                (*parameters).codec_tag = codec_tag;
            }

            if let Some(bit_rate) = self.config.bit_rate {
                (*parameters).bit_rate = bit_rate;
            }

            if let Some(profile) = self.config.profile {
                (*parameters).profile = profile;
            }

            if let Some(level) = self.config.level {
                (*parameters).level = level;
            }

            if let Some(aspect_ratio) = self.config.aspect_ratio {
                (*parameters).sample_aspect_ratio = aspect_ratio.into();
            }

            if let Some(field_order) = self.config.field_order {
                (*parameters).field_order = field_order.into();
            }

            if let Some(color_range) = self.config.color_range {
                (*parameters).color_range = color_range.into();
            }

            if let Some(color_primaries) = self.config.color_primaries {
                (*parameters).color_primaries = color_primaries.into();
            }

            if let Some(color_transfer) = self.config.color_transfer_characteristics {
                (*parameters).color_trc = color_transfer.into();
            }

            if let Some(chroma_location) = self.config.chroma_location {
                (*parameters).chroma_location = chroma_location.into();
            }

            if let Some(coded_side_data) = self.config.coded_side_data {
                (*parameters).coded_side_data = coded_side_data.pointer.as_ptr();
                (*parameters).nb_coded_side_data = coded_side_data.length as i32;
            }

            if let Some(extra_data) = self.config.extra_data {
                (*parameters).extradata = extra_data.pointer.as_ptr();
                (*parameters).extradata_size = extra_data.length as i32;
            }
        }

        parameters as *const AVCodecParameters
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FfmpegScalerConfig {
    pub destionation_resolution: Option<Resolution>,
    pub flags: i32,
}

impl Default for FfmpegScalerConfig {
    fn default() -> Self {
        Self {
            destionation_resolution: None,
            flags: ScalingFlags::FAST_BILINEAR.bits(),
        }
    }
}

impl Drop for FfmpegDecoderConfig {
    fn drop(&mut self) {
        if let Some(coded_side_data) = self.coded_side_data {
            unsafe {
                av_packet_side_data_free(
                    &mut coded_side_data.pointer.as_ptr(),
                    &mut (coded_side_data.length as i32),
                );
            }
        }
        // im supposed to free extra data
        // but there is no dedicated function to free extra data
        // so we do this
        if let Some(extra_data) = self.extra_data {
            let mut temp_context = unsafe { avcodec_parameters_alloc() };
            unsafe {
                (*temp_context).extradata = extra_data.pointer.as_ptr();
                (*temp_context).extradata_size = extra_data.length as i32;
                avcodec_parameters_free(&mut temp_context);
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub enum Threads {
    Set(usize),
    #[default]
    System,
    One,
}

pub use ffmpeg_the_third::threading::Type as ParallelismBehavior;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Parallelism {
    pub behavior: ParallelismBehavior,
    pub threads: Threads,
}

impl Default for Parallelism {
    fn default() -> Self {
        Self {
            behavior: ParallelismBehavior::Slice,
            threads: Threads::One,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FfmpegConfig {
    pub custom_frame_format_map: Option<HashMap<CustomFrameFormat, FrameFormat>>,
    pub frame_format: FrameFormat,
    pub resolution: Resolution,
    pub frame_rate: FrameRate,
    pub parallelism: Parallelism,
    pub ffmpeg_codec_low_level: FfmpegDecoderConfig,
    pub ffmpeg_scaler_low_level: FfmpegScalerConfig,
    pub side_data_check_list: Vec<SideDataType>,
}

impl ConfigHasResolution for FfmpegConfig {
    fn resolution(&self) -> Resolution {
        self.resolution
    }
}

fn convert_destination_pixel_format_to_av_pxfmt(
    pixel_destination: PixelDestination,
) -> FfmpegPixel {
    match pixel_destination {
        PixelDestination::Rgb8 => FfmpegPixel::RGB24,
        PixelDestination::Rgba8 => FfmpegPixel::RGBA,
        PixelDestination::Rgb16 => FfmpegPixel::RGB48,
        PixelDestination::Rgba16 => {
            if is_little_endian() {
                FfmpegPixel::RGBA64LE
            } else {
                FfmpegPixel::RGBA64BE
            }
        }
        PixelDestination::Bgr8 => FfmpegPixel::BGR24,
        PixelDestination::Bgra8 => FfmpegPixel::BGR32,
        PixelDestination::Bgr16 => FfmpegPixel::BGR48,
        PixelDestination::Bgra16 => {
            if is_little_endian() {
                FfmpegPixel::BGRA64LE
            } else {
                FfmpegPixel::BGRA64BE
            }
        }
        PixelDestination::Luma8 => FfmpegPixel::GRAY8,
        PixelDestination::LumaA8 => FfmpegPixel::YA8,
        PixelDestination::Luma16 => FfmpegPixel::GRAY16,
        PixelDestination::LumaA16 => FfmpegPixel::YA16,
    }
}

#[cfg(target_endian = "little")]
const fn is_little_endian() -> bool {
    true
}
#[cfg(not(target_endian = "little"))]
const fn is_little_endian() -> bool {
    false
}

#[cfg(test)]
mod test {
    use ffmpeg_the_third::format::input;
    use image::{ImageFormat, Rgb};
    use nokhwa_core::{
        decoder::Decoder,
        frame_buffer::FrameBuffer,
        frame_format::FrameFormat,
        types::{FrameRate, Resolution},
    };

    use crate::ffmpeg::{
        FfmpegConfig, FfmpegDecoder, FfmpegDecoderConfig, FfmpegScalerConfig, Parallelism,
    };

    #[test]
    pub fn test_h264() {
        ffmpeg_the_third::init().unwrap();

        let file = "test_images/ffmpeg/h264/test.h264";
        let output_dir = "test_images/ffmpeg/h264/out";

        let decoder_cfg = FfmpegConfig {
            custom_frame_format_map: None,
            frame_format: FrameFormat::AVC1,
            resolution: Resolution::new(498, 348),
            frame_rate: FrameRate::from_fps(30),
            parallelism: Parallelism::default(),
            ffmpeg_codec_low_level: FfmpegDecoderConfig::default(),
            ffmpeg_scaler_low_level: FfmpegScalerConfig::default(),
            side_data_check_list: Vec::default(),
        };

        let mut decoder = FfmpegDecoder::new(decoder_cfg).unwrap();

        let mut ictx = input(file).unwrap();
        for maybe_frame in ictx.packets() {
            let (_stream, packet) = maybe_frame.unwrap();
            let decoded =
                decoder.decode::<Rgb<u8>>(FrameBuffer::from_buffer(packet.data().unwrap(), None));
            match decoded {
                Ok(decoded) => {
                    let index = packet.position();
                    decoded
                        .save_with_format(format!("{output_dir}/{index}.png"), ImageFormat::Png)
                        .unwrap();
                }
                Err(why) => {
                    if why.is_needs_more() {
                        continue;
                    } else {
                        panic!("aaa {why}");
                    }
                }
            }
        }
    }

    #[test]
    pub fn test_h265() {
        ffmpeg_the_third::init().unwrap();

        let file = "test_images/ffmpeg/h265/bitstream.265";
        let output_dir = "test_images/ffmpeg/h265/out";

        let decoder_cfg = FfmpegConfig {
            custom_frame_format_map: None,
            frame_format: FrameFormat::H265,
            resolution: Resolution::new(480, 270),
            frame_rate: FrameRate::from_fps(30),
            parallelism: Parallelism::default(),
            ffmpeg_codec_low_level: FfmpegDecoderConfig::default(),
            ffmpeg_scaler_low_level: FfmpegScalerConfig::default(),
            side_data_check_list: Vec::default(),
        };

        let mut decoder = FfmpegDecoder::new(decoder_cfg).unwrap();

        let mut ictx = input(file).unwrap();
        for maybe_frame in ictx.packets() {
            let (_stream, packet) = maybe_frame.unwrap();
            let decoded =
                decoder.decode::<Rgb<u8>>(FrameBuffer::from_buffer(packet.data().unwrap(), None));
            match decoded {
                Ok(decoded) => {
                    let index = packet.position();
                    decoded
                        .save_with_format(format!("{output_dir}/{index}.png"), ImageFormat::Png)
                        .unwrap();
                }
                Err(why) => {
                    if why.is_needs_more() {
                        continue;
                    } else {
                        panic!("aaa {why}");
                    }
                }
            }
        }
    }

    #[test]
    pub fn test_av1() {
        ffmpeg_the_third::init().unwrap();

        let file = "test_images/ffmpeg/av1/test.ivf";
        let output_dir = "test_images/ffmpeg/av1/out";

        let decoder_cfg = FfmpegConfig {
            custom_frame_format_map: None,
            frame_format: FrameFormat::AV1,
            resolution: Resolution::new(320, 320),
            frame_rate: FrameRate::from_fps(10),
            parallelism: Parallelism::default(),
            ffmpeg_codec_low_level: FfmpegDecoderConfig::default(),
            ffmpeg_scaler_low_level: FfmpegScalerConfig::default(),
            side_data_check_list: Vec::default(),
        };

        let mut decoder = FfmpegDecoder::new(decoder_cfg).unwrap();

        let mut ictx = input(file).unwrap();
        for maybe_frame in ictx.packets() {
            let (_stream, packet) = maybe_frame.unwrap();
            let decoded =
                decoder.decode::<Rgb<u8>>(FrameBuffer::from_buffer(packet.data().unwrap(), None));
            match decoded {
                Ok(decoded) => {
                    let index = packet.position();
                    decoded
                        .save_with_format(format!("{output_dir}/{index}.png"), ImageFormat::Png)
                        .unwrap();
                }
                Err(why) => {
                    if why.is_needs_more() {
                        continue;
                    } else {
                        panic!("aaa {why}");
                    }
                }
            }
        }
    }
}
