use bytemuck::try_cast_slice_mut;
use core::mem::transmute;
use ffmpeg_the_third::codec::{Context, Id, Parameters};
use ffmpeg_the_third::decoder::Video;
use ffmpeg_the_third::ffi::{
    AVChromaLocation, AVCodecID, AVCodecParameters, AVColorPrimaries, AVColorRange, AVColorSpace,
    AVColorTransferCharacteristic, AVFieldOrder, AVMediaType, AVPacket,
    AVPixelFormat, AVRational, SwsContext, av_frame_alloc, av_frame_move_ref,
    av_image_copy_to_buffer, av_image_fill_arrays, av_image_get_buffer_size, avcodec_free_context,
    avcodec_parameters_alloc, avcodec_parameters_free, sws_freeContext, sws_getContext,
    sws_scale_frame,
};
use ffmpeg_the_third::packet::{Borrow, Ref};
use ffmpeg_the_third::{Frame, decoder, packet::Packet};
use nokhwa_core::codec::Codec;
use nokhwa_core::decoder::{Decoder, ImageBuffer, Pixel, Primitive};
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_buffer::FrameBuffer;
use nokhwa_core::frame_format::{CustomFrameFormat, FrameFormat};
use nokhwa_core::image::{DecodedImage, NonFloatScalarWidth};
use nokhwa_core::types::{CameraFormat, FrameRate, Resolution};

pub struct FfmpegDecoder {
    codec: FfmpegCodec,
    sws: Option<Sws>,
}

impl FfmpegDecoder {
    pub fn new(
        config: <FfmpegDecoder as nokhwa_core::decoder::Decoder>::Config,
    ) -> Result<Self, NokhwaError> {
        let codec = FfmpegCodec::new(config)?;
        Ok(Self { codec, sws: None })
    }

    pub fn with_format(format: &CameraFormat) -> Result<Self, NokhwaError> {
        let config = FfmpegDecoderConfig::with_camera_format(format);
        let codec = FfmpegCodec::new(config)?;
        Ok(Self { codec, sws: None })
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
    type Config = <FfmpegCodec as Codec>::Config;
    type OutputMeta = <FfmpegCodec as Codec>::WrittenMeta;
    type DestinationFormatHint = AVPixelFormat;

    fn config(&self) -> &Self::Config {
        self.codec.config()
    }

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        self.codec.set_config(config)?;
        Ok(())
    }

    fn decode_to_buffer(
        &mut self,
        to_decode: FrameBuffer,
        mut buffer: impl AsMut<[u8]>,
        _destination_format: Option<Self::DestinationFormatHint>
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
        let destination_format =
            pixel_to_destination_px_fmt::<P>()
                .ok_or(NokhwaError::DecoderInvalidBuffer("Unsupported Pixel Type".to_string()))?;

        let buffer = buffer.as_mut();
        let estimated_size =
            self.codec
                .preferred_buffer_min_size(&None)?
                .ok_or(NokhwaError::DecoderInvalidBuffer(
                    "failed to estimate decoder buffer.length".to_string(),
                ))?;
        if buffer.len() < estimated_size {
            return Err(NokhwaError::DecoderInvalidBuffer("buffer too small!".to_string()));
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

    fn decode<P: Pixel>(
        &mut self,
        to_decode: FrameBuffer,
    ) -> Result<DecodedImage<P, Self::OutputMeta>, NokhwaError>
    where
        <P as Pixel>::Subpixel: NonFloatScalarWidth,
    {
        let min_size = self.output_decoder_min_size_pixel::<P>(self.config().resolution);
        let mut buffer: Vec<P::Subpixel> = vec![<P::Subpixel>::DEFAULT_MIN_VALUE; min_size];
        let meta = self.decode_to_buffer(to_decode, try_cast_slice_mut(&mut buffer).map_err(|why| NokhwaError::DecoderInvalidBuffer(why.to_string()))?, None)?;
        Ok(DecodedImage::new(
            ImageBuffer::from_vec(
                self.codec.config.resolution.width(),
                self.codec.config.resolution.height(),
                buffer,
            )
            .ok_or(NokhwaError::Decoder(
                "Failed to create Image Buffer".to_string(),
            ))?,
            meta,
        ))
    }

    fn output_decoder_min_size(&self, resolution: Resolution, destination_format: Self::DestinationFormatHint) -> usize {
        let size =
            unsafe { av_image_get_buffer_size(destination_format, resolution.width() as i32, resolution.height() as i32, 1) };
        size as usize
    }

    fn buffer_takes_destination_hint(&self) -> bool {
        false
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

fn pixel_to_destination_px_fmt<P: Pixel>() -> Option<AVPixelFormat>
where
    <P as Pixel>::Subpixel: NonFloatScalarWidth,
{
    match P::COLOR_MODEL {
        "RGB" => match <<P as Pixel>::Subpixel>::WIDTH_BYTES {
            1 => Some(AVPixelFormat::AV_PIX_FMT_RGB24),
            _ => None,
        },

        "RGBA" => match <<P as Pixel>::Subpixel>::WIDTH_BYTES {
            1 => Some(AVPixelFormat::AV_PIX_FMT_RGBA),
            2 => Some(switch_endian(
                AVPixelFormat::AV_PIX_FMT_RGBA64LE,
                AVPixelFormat::AV_PIX_FMT_RGBA64BE,
            )),
            _ => None,
        },
        "BGR" => match <<P as Pixel>::Subpixel>::WIDTH_BYTES {
            1 => Some(AVPixelFormat::AV_PIX_FMT_BGR24),
            _ => None,
        },

        "BGRA" => match <<P as Pixel>::Subpixel>::WIDTH_BYTES {
            1 => Some(AVPixelFormat::AV_PIX_FMT_BGRA),
            2 => Some(switch_endian(
                AVPixelFormat::AV_PIX_FMT_BGRA64LE,
                AVPixelFormat::AV_PIX_FMT_BGRA64BE,
            )),
            _ => None,
        },
        "Y" => match <<P as Pixel>::Subpixel>::WIDTH_BYTES {
            1 => Some(AVPixelFormat::AV_PIX_FMT_GRAY8),
            2 => Some(switch_endian(
                AVPixelFormat::AV_PIX_FMT_GRAY16LE,
                AVPixelFormat::AV_PIX_FMT_GRAY16BE,
            )),
            _ => None,
        },
        "YA" => match <<P as Pixel>::Subpixel>::WIDTH_BYTES {
            1 => Some(AVPixelFormat::AV_PIX_FMT_GRAY8A),
            _ => None,
        },
        _ => None,
    }
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
    config: FfmpegDecoderConfig,
    temp_cfg: Option<*mut AVCodecParameters>,
    deinitialized: bool,
}

impl FfmpegCodec {
    fn new(config: <FfmpegCodec as Codec>::Config) -> Result<Self, NokhwaError> {
        let id = convert_format_to_codec_id(&config.frame_format)
            .ok_or(NokhwaError::DecoderUnsupportedFrameFormat(config.frame_format))?;

        let codec =
            decoder::find(id).ok_or(NokhwaError::DecoderInitializationError("Failed to find codec".to_string()))?;

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
                Parameters::from_raw(config.as_ptr()?).ok_or(NokhwaError::DecoderInitializationError(
                    "Failed to convert parameters".to_string(),
                ))?
            })
            .map_err(|why| NokhwaError::Decoder(why.to_string()))?;

        Ok(Self {
            decoder: video,
            config,
            temp_cfg: None,
            deinitialized: false,
        })
    }
}

impl Codec for FfmpegCodec {
    type Config = FfmpegDecoderConfig;
    type Input<'a> = PacketOrRef<'a>;
    type Output<'a> = Frame;
    type WrittenMeta = FfmpegFrameMetadata;

    fn allowed_formats(&self) -> Result<&[FrameFormat], NokhwaError> {
        if self.deinitialized {
            return Err(NokhwaError::DecoderAlreadyDeinitialized);
        }

        Ok(FrameFormat::ALL)
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        if self.deinitialized {
            return Err(NokhwaError::DecoderAlreadyDeinitialized);
        }
        dealloc_av_params(&mut self.temp_cfg);
        let mut temp_config = config.as_avcodec_params()?;
        self.decoder
            .set_parameters(unsafe {
                Parameters::from_raw(&mut temp_config).ok_or(NokhwaError::DecoderInvalidConfiguration(
                    "Failed to convert parameters".to_string(),
                ))?
            })
            .map_err(|why| NokhwaError::DecoderInvalidConfiguration(why.to_string()))?;
        self.config = config;
        self.temp_cfg = Some(&mut temp_config);
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
                convert_frame_format_to_pixfmt(&fmt.format()),
            ),
            None => (
                self.config().resolution.width(),
                self.config.resolution.height(),
                convert_frame_format_to_pixfmt(&self.config().frame_format),
            ),
        };

        let size =
            unsafe { av_image_get_buffer_size(pixel_format, width as i32, height as i32, 1) };
        Ok(Some(size as usize))
    }

    fn deinitialize(&mut self) -> Result<(), NokhwaError> {
        dealloc_av_params(&mut self.temp_cfg);
        if self.deinitialized {
            return Ok(());
        }

        self.deinitialized = true;
        Ok(())
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

fn convert_frame_format_to_pixfmt(frame_format: &FrameFormat) -> AVPixelFormat {
    match frame_format {
        // does FFMPEG not support 32bpp 4:4:4 packed AYUV?
        FrameFormat::NV24 => AVPixelFormat::AV_PIX_FMT_NV24,
        FrameFormat::NV42 => AVPixelFormat::AV_PIX_FMT_NV42,
        FrameFormat::Yuyv_4_2_2 => AVPixelFormat::AV_PIX_FMT_YUYV422,
        FrameFormat::Uyvy_4_2_2 => AVPixelFormat::AV_PIX_FMT_UYVY422,
        FrameFormat::Yvyu_4_2_2 => AVPixelFormat::AV_PIX_FMT_YVYU422,
        FrameFormat::NV16 => AVPixelFormat::AV_PIX_FMT_NV16,
        FrameFormat::Yuv_4_2_0 => AVPixelFormat::AV_PIX_FMT_YUV420P,
        FrameFormat::NV12 => AVPixelFormat::AV_PIX_FMT_NV12,
        FrameFormat::NV21 => AVPixelFormat::AV_PIX_FMT_NV21,
        FrameFormat::P010 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_P010LE
            } else {
                AVPixelFormat::AV_PIX_FMT_P010BE
            }
        }
        FrameFormat::P012 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_P012LE
            } else {
                AVPixelFormat::AV_PIX_FMT_P012BE
            }
        }
        FrameFormat::Luma_8 => AVPixelFormat::AV_PIX_FMT_GRAY8,
        FrameFormat::Luma_10 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_GRAY10LE
            } else {
                AVPixelFormat::AV_PIX_FMT_GRAY10BE
            }
        }
        FrameFormat::Luma_12 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_GRAY12LE
            } else {
                AVPixelFormat::AV_PIX_FMT_GRAY12BE
            }
        }
        FrameFormat::Luma_14 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_GRAY14LE
            } else {
                AVPixelFormat::AV_PIX_FMT_GRAY14BE
            }
        }
        FrameFormat::Luma_16 | FrameFormat::Depth_16 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_GRAY16LE
            } else {
                AVPixelFormat::AV_PIX_FMT_GRAY16BE
            }
        }
        FrameFormat::Rgb_3_3_2 => AVPixelFormat::AV_PIX_FMT_RGB8,
        FrameFormat::Rgb_5_6_5 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_RGB565LE
            } else {
                AVPixelFormat::AV_PIX_FMT_RGB565BE
            }
        }
        FrameFormat::Rgb_5_5_5 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_RGB555LE
            } else {
                AVPixelFormat::AV_PIX_FMT_RGB565BE
            }
        }
        FrameFormat::Rgb_8_8_8 => AVPixelFormat::AV_PIX_FMT_RGB24,
        FrameFormat::Argb_8_8_8_8 => AVPixelFormat::AV_PIX_FMT_ARGB,
        FrameFormat::Rgba_8_8_8_8 => AVPixelFormat::AV_PIX_FMT_RGBA,
        FrameFormat::Bgr_3_3_2 => AVPixelFormat::AV_PIX_FMT_BGR8,
        FrameFormat::Bgr_5_6_5 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_BGR565LE
            } else {
                AVPixelFormat::AV_PIX_FMT_BGR565BE
            }
        }
        FrameFormat::Bgr_5_5_5 => {
            if is_little_endian() {
                AVPixelFormat::AV_PIX_FMT_BGR555LE
            } else {
                AVPixelFormat::AV_PIX_FMT_BGR555BE
            }
        }
        FrameFormat::Bgr_8_8_8 => AVPixelFormat::AV_PIX_FMT_BGR24,
        FrameFormat::Abgr_8_8_8_8 => AVPixelFormat::AV_PIX_FMT_ABGR,
        FrameFormat::Bgra_8_8_8_8 => AVPixelFormat::AV_PIX_FMT_BGRA,
        _ => AVPixelFormat::AV_PIX_FMT_RGB24,
    }
}

#[derive(Clone, Debug)]
pub struct FfmpegDecoderConfig {
    pub frame_format: FrameFormat,
    pub codec_tag: u32,
    #[doc = " - video: the pixel format, the value corresponds to enum AVPixelFormat.\n - audio: the sample format, the value corresponds to enum AVSampleFormat."]
    pub pix_fmt: Option<AVPixelFormat>,
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

impl FfmpegDecoderConfig {
    pub fn with_camera_format(camera_format: &CameraFormat) -> Self {
        FfmpegDecoderConfig::from(*camera_format)
    }

    pub fn as_avcodec_params(&self) -> Result<AVCodecParameters, NokhwaError> {
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

            if let Some(pixfmt) = self.pix_fmt {
                av_codec_params.format = pixfmt as i32;
            } else {
                let pixel_format = convert_frame_format_to_pixfmt(&self.frame_format);
                av_codec_params.format = pixel_format as i32;
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

        Ok(av_codec_params)
    }

    /// SAFETY: the user is responsible for freeing this return value
    pub fn as_ptr(&self) -> Result<*mut AVCodecParameters, NokhwaError> {
        Ok(&mut self.as_avcodec_params()?)
    }
}

impl From<CameraFormat> for FfmpegDecoderConfig {
    fn from(value: CameraFormat) -> Self {
        Self {
            frame_format: value.format(),
            codec_tag: 0,
            // extra_data: None,
            // extra_data_size: 0,
            // coded_side_data: None,
            // coded_side_data_size: 0,
            pix_fmt: None,
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

#[cfg(target_endian = "little")]
const fn is_little_endian() -> bool {
    true
}
#[cfg(not(target_endian = "little"))]
const fn is_little_endian() -> bool {
    false
}
