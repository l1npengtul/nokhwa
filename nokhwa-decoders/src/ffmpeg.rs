use bytemuck::try_cast_slice_mut;
use core::mem::transmute;
use std::collections::HashMap;
use ffmpeg_the_third::codec::{Context, Id, Parameters, ParametersMut};
use ffmpeg_the_third::decoder::Video;
use ffmpeg_the_third::ffi::{
    AVChromaLocation, AVCodecID, AVCodecParameters, AVColorPrimaries, AVColorRange, AVColorSpace,
    AVColorTransferCharacteristic, AVFieldOrder, AVMediaType, AVPacket, AVPixelFormat, AVRational,
    SwsContext, av_frame_alloc, av_frame_move_ref, av_image_copy_to_buffer, av_image_fill_arrays,
    av_image_get_buffer_size, avcodec_free_context, avcodec_parameters_alloc,
    avcodec_parameters_free, sws_freeContext, sws_getContext, sws_scale_frame,
};
use ffmpeg_the_third::packet::{Borrow, Ref};
use ffmpeg_the_third::{Frame, decoder, packet::Packet, AsMutPtr};
use nokhwa_core::codec::Codec;
use nokhwa_core::decoder::{ConfigHasResolution, Decoder, Pixel};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_buffer::FrameBuffer;
use nokhwa_core::frame_format::{CustomFrameFormat, FrameFormat};
use nokhwa_core::image::{NonFloatScalarWidth};
use nokhwa_core::pixel_destination::PixelDestination;
use nokhwa_core::types::{CameraFormat, FrameRate, Resolution};

pub struct FfmpegDecoder {
    codec: FfmpegCodec,
    settings: FfmpegConfig,
    sws: Option<Sws>,
}

impl FfmpegDecoder {
    pub fn new(
        config: <FfmpegDecoder as nokhwa_core::decoder::Decoder>::Config,
    ) -> Result<Self, NokhwaError> {
        let codec = FfmpegCodec::new(config.ffmpeg_settings.clone())?;
        Ok(Self { codec, settings: config, sws: None })
    }

    pub fn with_format(format: &CameraFormat) -> Result<Self, NokhwaError> {
        let config = FfmpegCodecConfig::with_camera_format(format);
        let codec = FfmpegCodec::new(config.clone())?;
        Ok(Self { codec, settings: , sws: None })
    }

    pub fn receive_decoded_frame(
        &mut self,
        to_decode: FrameBuffer,
    ) -> Result<(Frame, <FfmpegDecoder as Decoder>::OutputMeta), NokhwaError> {
        let new_pkt = Packet::borrow(to_decode.buffer());
        self.codec.send_item(new_pkt.into())?;
        let mut new_frame = unsafe { Frame::empty() };
        let metadata = self
            .codec
            .receive_decoded_item(&mut new_frame)
            .map_err(|why| NokhwaError::Decoder(why.to_string()))?;
        Ok((new_frame, metadata))
    }
}

impl Decoder for FfmpegDecoder {
    type Config = FfmpegConfig;
    type OutputMeta = <FfmpegCodec as Codec>::WrittenMeta;
    const SUPPORTED_DESTINATIONS: &'static [PixelDestination] = &[];


    fn config(&self) -> &Self::Config {
        &self.settings
    }

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        self.codec.set_config(config.ffmpeg_settings.clone())?;
        self.settings = config;
        Ok(())
    }

    fn decode_to_buffer(
        &mut self,
        to_decode: FrameBuffer,
        mut buffer: impl AsMut<[u8]>,
        _destination_format: PixelDestination,
    ) -> Result<Self::OutputMeta, NokhwaError> {
        // TODO: add an extra zippy happy path for rgb/bgr/luma
        let (frame, metadata) = self.receive_decoded_frame(to_decode)?;
        let av_frame_data = unsafe {
            frame.as_ref().ok_or(NokhwaError::Decoder(
                "No Frame Data from Decoder".to_string(),
            ))?
        };
        let buffer = buffer.as_mut();
        let result = unsafe {
            av_image_copy_to_buffer(
                buffer.as_mut().as_mut_ptr(),
                buffer.as_mut().len() as i32,
                av_frame_data.data.as_ptr() as *const *const u8,
                av_frame_data.linesize.as_ptr(),
                metadata.pixel_format,
                self.config().resolution.width() as i32,
                self.config().resolution.height() as i32,
                1,
            )
        };
        if result.is_negative() {
            return Err(NokhwaError::Decoder(format!("Error Code {result}")));
        }
        Ok(metadata)
    }

    fn decode_to_pixel_buffer<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
        mut buffer: impl AsMut<[P::Subpixel]>,
    ) -> Result<Self::OutputMeta, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        let destination_format = pixel_to_destination_px_fmt::<P>().ok_or(
            NokhwaError::DecoderInvalidBuffer("Unsupported Pixel Type".to_string()),
        )?;

        let buffer = buffer.as_mut();
        let estimated_size = self.codec.preferred_buffer_min_size(&None)?.ok_or(
            NokhwaError::DecoderInvalidBuffer(
                "failed to estimate decoder buffer.length".to_string(),
            ),
        )?;
        if buffer.len() < estimated_size {
            return Err(NokhwaError::DecoderInvalidBuffer(
                "buffer too small!".to_string(),
            ));
        }

        let (mut frame, decoded_meta) = self.receive_decoded_frame(to_decode)?;
        let source_format = decoded_meta.pixel_format;

        let cast_slice = try_cast_slice_mut::<P::Subpixel, u8>(buffer)
            .map_err(|why| NokhwaError::DecoderInvalidBuffer(why.to_string()))?;
        let receiving_buffer = unsafe {
            let av_frame = av_frame_alloc();
            let result = av_image_fill_arrays(
                (*av_frame).data.as_mut_ptr(),
                (*av_frame).linesize.as_mut_ptr(),
                cast_slice.as_ptr(),
                destination_format,
                self.codec.config.resolution.width() as i32,
                self.codec.config.resolution.height() as i32,
                1,
            );
            if result < 0 {
                return Err(NokhwaError::Decoder("Failed to fill avimage".to_string()));
            }
            av_frame
        };

        if source_format != destination_format {
            let sws_scaler = match &mut self.sws {
                None => {
                    let context = create_sws_context(
                        source_format,
                        destination_format,
                        self.codec.config.resolution,
                    )?;
                    self.sws = Some(Sws {
                        sws: context,
                        source_pixel_format: source_format,
                        dest_pixel_format: destination_format,
                    });
                    self.sws.as_mut().unwrap()
                }
                Some(v) => {
                    if v.source_pixel_format != source_format
                        || v.dest_pixel_format != destination_format
                    {
                        v.source_pixel_format = source_format;
                        v.dest_pixel_format = destination_format;
                        v.sws = create_sws_context(
                            source_format,
                            destination_format,
                            self.codec.config.resolution,
                        )?;
                    }
                    v
                }
            };

            let scaled =
                unsafe { sws_scale_frame(sws_scaler.sws, receiving_buffer, frame.as_mut_ptr()) };
            if scaled < 0 {
                Err(NokhwaError::Decoder("Failed to scale sws".to_string()))
            } else {
                Ok(decoded_meta)
            }
        } else {
            unsafe { av_frame_move_ref(receiving_buffer, frame.as_mut_ptr()) };
            Ok(decoded_meta)
        }
    }
}

fn create_sws_context(
    src_format: AVPixelFormat,
    dest: AVPixelFormat,
    resolution: Resolution,
) -> Result<*mut SwsContext, NokhwaError> {
    let param = 0_f64;
    let new_sws = unsafe {
        sws_getContext(
            resolution.width() as i32,
            resolution.height() as i32,
            src_format,
            resolution.width() as i32,
            resolution.height() as i32,
            dest,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &param,
        )
    };
    Ok(new_sws)
}

pub struct Sws {
    pub sws: *mut SwsContext,
    pub source_pixel_format: AVPixelFormat,
    pub dest_pixel_format: AVPixelFormat,
    // pub filter_a: SwsFilter,
    // pub filter_b: SwsFilter,
}

impl Drop for Sws {
    fn drop(&mut self) {
        unsafe {
            sws_freeContext(self.sws);
            // sws_freeFilter(&mut self.filter_a);
            // sws_freeFilter(&mut self.filter_b);
        }
    }
}

#[derive(Clone, Debug)]
pub struct FfmpegFrameMetadata {
    pub color_range: AVColorRange,
    pub pixel_format: AVPixelFormat,
    pub resolution: Resolution,
}

pub enum PacketOrRef<'a> {
    Packet(Packet),
    Ref(Borrow<'a>),
}

impl Ref for PacketOrRef<'_> {
    fn as_ptr(&self) -> *const AVPacket {
        match self {
            PacketOrRef::Packet(p) => p.as_ptr(),
            PacketOrRef::Ref(r) => r.as_ptr(),
        }
    }
}

impl<'a> From<Borrow<'a>> for PacketOrRef<'a> {
    fn from(value: Borrow<'a>) -> Self {
        Self::Ref(value)
    }
}

impl From<Packet> for PacketOrRef<'_> {
    fn from(value: Packet) -> Self {
        Self::Packet(value)
    }
}

pub struct FfmpegCodec {
    decoder: Video,
    deinitialized: bool,
}

impl FfmpegCodec {
    fn new(config: <FfmpegCodec as Codec>::Config) -> Result<Self, NokhwaError> {
        let id = convert_format_to_codec_id(&config.frame_format).ok_or(
            NokhwaError::DecoderUnsupportedFrameFormat(config.frame_format),
        )?;

        let codec = decoder::find(id).ok_or(NokhwaError::DecoderInitializationError(
            "Failed to find codec".to_string(),
        ))?;

        let context = unsafe {
            let ptr = ffmpeg_the_third::ffi::avcodec_alloc_context3(codec.as_ptr());
            if ptr.is_null() {
                return Err(NokhwaError::DecoderInitializationError(
                    "ffmpeg returned a null context".to_string(),
                ));
            }
            Context::wrap(ptr, None)
        };

        let mut video = context
            .decoder()
            .video()
            .map_err(|why| NokhwaError::Decoder(why.to_string()))?;
        video
            .set_parameters(unsafe {
                Parameters::from_raw(config.as_ptr()?).ok_or(
                    NokhwaError::DecoderInitializationError(
                        "Failed to convert parameters".to_string(),
                    ),
                )?
            })
            .map_err(|why| NokhwaError::Decoder(why.to_string()))?;

        Ok(Self {
            decoder: video,
            deinitialized: false,
        })
    }
}

impl Codec for FfmpegCodec {
    type Config = FfmpegCodecConfig;
    type Input<'a> = PacketOrRef<'a>;
    type Output<'a> = Frame;
    type WrittenMeta = FfmpegFrameMetadata;

    fn allowed_formats(&self) -> Result<&[FrameFormat], NokhwaError> {
        if self.deinitialized {
            return Err(NokhwaError::DecoderAlreadyDeinitialized);
        }

        Ok(FrameFormat::ALL)
    }

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        unsafe {
            avcodec_free_context(&mut self.decoder.as_mut_ptr())
        }

        self.deinitialized = true;

        let params = config.as_parameters()?;

        let codec = Context::from_parameters(params).map_err(|why| {
            NokhwaError::DecoderInitializationError(why.to_string())
        })?;

        let video = codec.decoder().video().map_err(|why| {
            NokhwaError::DecoderInitializationError(why.to_string())
        })?;

        self.decoder = video;
        self.deinitialized = false;

        Ok(())
    }

    fn send_item(&mut self, input: Self::Input<'_>) -> Result<(), NokhwaError> {
        if self.deinitialized {
            return Err(NokhwaError::DecoderAlreadyDeinitialized);
        }
        self.decoder
            .send_packet(&input)
            .map_err(|why| NokhwaError::Decoder(why.to_string()))
    }

    fn receive_decoded_item(
        &mut self,
        writing_output: &mut Self::Output<'_>,
    ) -> Result<FfmpegFrameMetadata, NokhwaError> {
        if self.deinitialized {
            return Err(NokhwaError::DecoderAlreadyDeinitialized);
        }
        self.decoder
            .receive_frame(writing_output)
            .map_err(|why| NokhwaError::Decoder(why.to_string()))?;
        let avframe = match unsafe { writing_output.as_ref() } {
            Some(r) => r,
            None => {
                return Err(NokhwaError::Decoder(
                    "failed to get refrenece to decoded frame".to_string(),
                ));
            }
        };
        let meta = FfmpegFrameMetadata {
            color_range: avframe.color_range,
            pixel_format: unsafe { transmute::<i32, AVPixelFormat>(avframe.format) },
            resolution: Resolution::new(avframe.height as u32, avframe.width as u32),
        };
        Ok(meta)
    }

    fn preferred_buffer_min_size(
        &mut self,
        camera_format: &Option<CameraFormat>,
    ) -> Result<Option<usize>, NokhwaError> {
        let (width, height, pixel_format) = match camera_format {
            Some(fmt) => (
                fmt.width(),
                fmt.height(),
                convert_frame_format_to_pixfmt(&fmt.format()).ok_or(NokhwaError::DecoderUnsupportedFrameFormat(fmt.format()))?,
            ),
            None => (
                self.decoder.width(),
                self.decoder.height(),
                unsafe { self.decoder.as_ptr() }.pix_fmt
            ),
        };

        let size =
            unsafe { av_image_get_buffer_size(pixel_format, width as i32, height as i32, 1) };
        Ok(Some(size as usize))
    }
}

impl Drop for FfmpegCodec {
    fn drop(&mut self) {
        unsafe {
            avcodec_free_context(&mut self.decoder.as_mut_ptr())
        }
    }
}

fn convert_format_to_codec_id(frame_format: &FrameFormat) -> Option<Id> {
    match frame_format {
        FrameFormat::H265 => Some(Id::H265),
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
        FrameFormat::Ayuv_32 => Some(Id::RAWVIDEO),
        FrameFormat::Yuyv_4_2_2 => Some(Id::RAWVIDEO),
        FrameFormat::Yvyu_4_2_2 => Some(Id::RAWVIDEO),
        FrameFormat::Uyvy_4_2_2 => Some(Id::RAWVIDEO),
        FrameFormat::NV12 => Some(Id::RAWVIDEO),
        FrameFormat::NV21 => Some(Id::RAWVIDEO),
        FrameFormat::Luma_8 => Some(Id::RAWVIDEO),
        FrameFormat::Luma_10 => Some(Id::RAWVIDEO),
        FrameFormat::Luma_12 => Some(Id::RAWVIDEO),
        FrameFormat::Luma_14 => Some(Id::RAWVIDEO),
        FrameFormat::Luma_16 => Some(Id::RAWVIDEO),
        FrameFormat::Rgb_3_3_2 => Some(Id::RAWVIDEO),
        FrameFormat::Rgb_5_5_5 => Some(Id::RAWVIDEO),
        FrameFormat::Rgb_5_6_5 => Some(Id::RAWVIDEO),
        FrameFormat::Rgb_8_8_8 => Some(Id::RAWVIDEO),
        FrameFormat::Argb_8_8_8_8 => Some(Id::RAWVIDEO),
        FrameFormat::Rgba_8_8_8_8 => Some(Id::RAWVIDEO),
        FrameFormat::Bgr_3_3_2 => Some(Id::RAWVIDEO),
        FrameFormat::Bgr_5_5_5 => Some(Id::RAWVIDEO),
        FrameFormat::Bgr_5_6_5 => Some(Id::RAWVIDEO),
        FrameFormat::Bgr_8_8_8 => Some(Id::RAWVIDEO),
        FrameFormat::Abgr_8_8_8_8 => Some(Id::RAWVIDEO),
        FrameFormat::Bgra_8_8_8_8 => Some(Id::RAWVIDEO),
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

fn convert_frame_format_to_pixfmt(frame_format: &FrameFormat) -> Option<AVPixelFormat> {
    match frame_format {
        // does FFMPEG not support 32bpp 4:4:4 packed AYUV?
        FrameFormat::NV24 => Some(AVPixelFormat::AV_PIX_FMT_NV24),
        FrameFormat::NV42 => Some(AVPixelFormat::AV_PIX_FMT_NV42),
        FrameFormat::Yuyv_4_2_2 => Some(AVPixelFormat::AV_PIX_FMT_YUYV422),
        FrameFormat::Uyvy_4_2_2 => Some(AVPixelFormat::AV_PIX_FMT_UYVY422),
        FrameFormat::Yvyu_4_2_2 => Some(AVPixelFormat::AV_PIX_FMT_YVYU422),
        FrameFormat::NV16 => Some(AVPixelFormat::AV_PIX_FMT_NV16),
        FrameFormat::Yuv_4_2_0 => Some(AVPixelFormat::AV_PIX_FMT_YUV420P),
        FrameFormat::NV12 => Some(AVPixelFormat::AV_PIX_FMT_NV12),
        FrameFormat::NV21 => Some(AVPixelFormat::AV_PIX_FMT_NV21),
        FrameFormat::P010 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_P010LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_P010BE)
            }
        }
        FrameFormat::P012 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_P012LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_P012BE)
            }
        }
        FrameFormat::Luma_8 => Some(AVPixelFormat::AV_PIX_FMT_GRAY8),
        FrameFormat::Luma_10 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_GRAY10LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_GRAY10BE)
            }
        }
        FrameFormat::Luma_12 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_GRAY12LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_GRAY12BE)
            }
        }
        FrameFormat::Luma_14 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_GRAY14LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_GRAY14BE)
            }
        }
        FrameFormat::Luma_16 | FrameFormat::Depth_16 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_GRAY16LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_GRAY16BE)
            }
        }
        FrameFormat::Rgb_3_3_2 => Some(AVPixelFormat::AV_PIX_FMT_RGB8),
        FrameFormat::Rgb_5_6_5 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_RGB565LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_RGB565BE)
            }
        }
        FrameFormat::Rgb_5_5_5 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_RGB555LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_RGB565BE)
            }
        }
        FrameFormat::Rgb_8_8_8 => Some(AVPixelFormat::AV_PIX_FMT_RGB24),
        FrameFormat::Argb_8_8_8_8 => Some(AVPixelFormat::AV_PIX_FMT_ARGB),
        FrameFormat::Rgba_8_8_8_8 => Some(AVPixelFormat::AV_PIX_FMT_RGBA),
        FrameFormat::Bgr_3_3_2 => Some(AVPixelFormat::AV_PIX_FMT_BGR8),
        FrameFormat::Bgr_5_6_5 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_BGR565LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_BGR565BE)
            }
        }
        FrameFormat::Bgr_5_5_5 => {
            if is_little_endian() {
                Some(AVPixelFormat::AV_PIX_FMT_BGR555LE)
            } else {
                Some(AVPixelFormat::AV_PIX_FMT_BGR555BE)
            }
        }
        FrameFormat::Bgr_8_8_8 => Some(AVPixelFormat::AV_PIX_FMT_BGR24),
        FrameFormat::Abgr_8_8_8_8 => Some(AVPixelFormat::AV_PIX_FMT_ABGR),
        FrameFormat::Bgra_8_8_8_8 => Some(AVPixelFormat::AV_PIX_FMT_BGRA),
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct FfmpegConfig {
    pub custom_frame_format_map: Option<HashMap<CustomFrameFormat, FrameFormat>>,
    pub ffmpeg_settings: FfmpegCodecConfig,
}

impl ConfigHasResolution for FfmpegConfig {
    fn resolution(&self) -> Resolution {
        self.ffmpeg_settings.resolution
    }
}

#[derive(Clone, Debug)]
pub struct FfmpegCodecConfig {
    pub frame_format: FrameFormat,
    pub codec_tag: u32,
    #[doc = " Codec-specific bitstream restrictions that the stream conforms to."]
    pub profile: i32,
    pub level: i32,
    pub resolution: Resolution,
    #[doc = " Video only. The aspect ratio (width / height) which a single pixel\n should have when displayed.\n\n When the aspect ratio is unknown / undefined, the numerator should be\n set to 0 (the denominator may have any value)."]
    pub sample_aspect_ratio: AVRational,
    #[doc = " Video only. Number of frames per second, for streams with constant frame\n durations. Should be set to { 0, 1 } when some frames have differing\n durations or if the value is not known.\n\n @note This field correponds to values that are stored in codec-level\n headers and is typically overridden by container/transport-layer\n timestamps, when available. It should thus be used only as a last resort,\n when no higher-level timing information is available."]
    pub frame_rate: FrameRate,
    #[doc = " Video only. The order of the fields in interlaced video."]
    pub field_order: AVFieldOrder,
    #[doc = " Video only. Additional colorspace characteristics."]
    pub color_range: AVColorRange,
    pub color_primaries: AVColorPrimaries,
    pub color_trc: AVColorTransferCharacteristic,
    pub color_space: AVColorSpace,
    pub chroma_location: AVChromaLocation,
    #[doc = " Video only. Number of delayed frames."]
    pub video_delay: i32,
}

impl ConfigHasResolution for FfmpegCodecConfig {
    fn resolution(&self) -> Resolution {
        self.resolution
    }
}

impl FfmpegCodecConfig {
    pub fn with_camera_format(camera_format: &CameraFormat) -> Self {
        FfmpegCodecConfig::from(*camera_format)
    }
    pub fn as_parameters(&self) -> Result<Parameters, NokhwaError> {
        let mut av_codec_params = unsafe { avcodec_parameters_alloc().read() };
        av_codec_params.codec_type = AVMediaType::AVMEDIA_TYPE_VIDEO;
        // if let Some(extra_data) = self.extra_data {
        //     if extra_data.is_null() {
        //         return Err(NokhwaError::Decoder("extra data is nullptr!".to_string()))
        //     }
        //     av_codec_params.extradata = extra_data;
        //     av_codec_params.extradata_size = self.extra_data_size as i32;
        // }
        // if let Some(side_data) = self.coded_side_data {
        //     if side_data.is_null() {
        //         return Err(NokhwaError::Decoder("side data is nullptr!".to_string()))
        //     }
        //     av_codec_params.coded_side_data = side_data;
        //     av_codec_params.nb_coded_side_data = self.coded_side_data_size as i32;
        // }

        if let Some(id) = convert_format_to_codec_id(&self.frame_format) {
            av_codec_params.codec_id = id.into();

            if let Some(pixfmt) = convert_frame_format_to_pixfmt(&self.frame_format) {
                av_codec_params.format = pixfmt as i32;
            }
        } else {
            return Err(NokhwaError::DecoderInvalidConfiguration(
                "Failed to convert frameformat to id".to_string(),
            ));
        }

        av_codec_params.profile = self.profile;
        av_codec_params.level = self.level;

        av_codec_params.width = self.resolution.width() as i32;
        av_codec_params.height = self.resolution.height() as i32;

        av_codec_params.sample_aspect_ratio = self.sample_aspect_ratio;

        av_codec_params.framerate = AVRational {
            num: self.frame_rate.numerator(),
            den: self.frame_rate.numerator(),
        };

        av_codec_params.field_order = self.field_order;

        av_codec_params.color_range = self.color_range;

        av_codec_params.color_primaries = self.color_primaries;

        av_codec_params.color_trc = self.color_trc;

        av_codec_params.color_space = self.color_space;

        av_codec_params.chroma_location = self.chroma_location;

        av_codec_params.video_delay = self.video_delay;

        match unsafe {Parameters::from_raw(&mut av_codec_params)} {
            Some(p) => Ok(p),
            None => Err(NokhwaError::Decoder("Failed to convert into parameters".to_string()))
        }
    }
}

impl From<CameraFormat> for FfmpegCodecConfig {
    fn from(value: CameraFormat) -> Self {
        Self {
            frame_format: value.format(),
            codec_tag: 0,
            // extra_data: None,
            // extra_data_size: 0,
            // coded_side_data: None,
            // coded_side_data_size: 0,
            profile: 0,
            level: 0,
            resolution: value.resolution(),
            sample_aspect_ratio: AVRational { num: 0, den: 1 },
            frame_rate: value.frame_rate(),
            field_order: AVFieldOrder::AV_FIELD_UNKNOWN,
            color_range: AVColorRange::AVCOL_RANGE_UNSPECIFIED,
            color_primaries: AVColorPrimaries::AVCOL_PRI_RESERVED0,
            color_trc: AVColorTransferCharacteristic::AVCOL_TRC_RESERVED0,
            color_space: AVColorSpace::AVCOL_SPC_RGB,
            chroma_location: AVChromaLocation::AVCHROMA_LOC_UNSPECIFIED,
            video_delay: 0,
        }
    }
}

unsafe impl Send for FfmpegCodec {}

unsafe impl Sync for FfmpegCodec {}

fn dealloc_av_params(avcodec_parameters: &mut Option<*mut AVCodecParameters>) {
    if let Some(param) = avcodec_parameters {
        if !param.is_null() {
            unsafe {
                let mut_param = param as *mut *mut AVCodecParameters;
                avcodec_parameters_free(mut_param)
            }
        }
        *avcodec_parameters = None;
    }
}

impl Drop for FfmpegCodec {
    fn drop(&mut self) {
        let _ = self.deinitialize();
        dealloc_av_params(&mut self.temp_cfg);
        unsafe {
            let mut ctx = self.decoder.as_mut_ptr();
            if !ctx.is_null() {
                avcodec_free_context(&mut ctx)
            }

            // if let Some(mut sws) = self.sws_context {
            //     sws_freeContext(&mut sws)
            // }
        }
    }
}

fn switch_endian<T>(little: T, not: T) -> T {
    match is_little_endian() {
        true => little,
        false => not,
    }
}

fn convert_destination_pixel_format_to_av_pxfmt(pixel_destination: PixelDestination) -> AVPixelFormat {
    match pixel_destination {
        PixelDestination::Rgb8 => AVPixelFormat::AV_PIX_FMT_RGB24,
        PixelDestination::Rgba8 => AVPixelFormat::AV_PIX_FMT_RGBA,
        PixelDestination::Rgb16 => if is_little_endian() { AVPixelFormat::AV_PIX_FMT_RGB48LE }  else { AVPixelFormat::AV_PIX_FMT_RGB48BE },
        PixelDestination::Rgba16 => if is_little_endian() { AVPixelFormat::AV_PIX_FMT_RGBA64LE }  else { AVPixelFormat::AV_PIX_FMT_RGBA64LE },
        PixelDestination::Bgr8 => AVPixelFormat::AV_PIX_FMT_BGR24,
        PixelDestination::Bgra8 => AVPixelFormat::AV_PIX_FMT_BGRA,
        PixelDestination::Bgr16 => if is_little_endian() { AVPixelFormat::AV_PIX_FMT_BGR48LE }  else { AVPixelFormat::AV_PIX_FMT_BGR48BE },
        PixelDestination::Bgra16 => if is_little_endian() { AVPixelFormat::AV_PIX_FMT_BGRA64LE }  else { AVPixelFormat::AV_PIX_FMT_BGRA64BE },
        PixelDestination::Luma8 => AVPixelFormat::AV_PIX_FMT_GRAY8,
        PixelDestination::LumaA8 => AVPixelFormat::AV_PIX_FMT_YA8,
        PixelDestination::Luma16 => if is_little_endian() { AVPixelFormat::AV_PIX_FMT_GRAY16LE }  else { AVPixelFormat::AV_PIX_FMT_GRAY16BE },
        PixelDestination::LumaA16 => if is_little_endian() { AVPixelFormat::AV_PIX_FMT_YA16LE }  else { AVPixelFormat::AV_PIX_FMT_YA16BE },
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
