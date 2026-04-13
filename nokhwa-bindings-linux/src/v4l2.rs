use nokhwa_core::camera::CameraTrait;
use nokhwa_core::control::{
    Control as NokhwaControl, ControlDescription, ControlFlags, ControlId, ControlValue,
    ControlValueDescriptor, CustomControlId, Orientation,
};
use nokhwa_core::error::{NokhwaError, NokhwaResult, StreamError};
use nokhwa_core::frame_format::{CustomFrameFormat, FrameFormat};
use nokhwa_core::metadata::{Metadata, MetadataTypes, Time};
use nokhwa_core::platform::PlatformTrait;
use nokhwa_core::ranges::Range;
use nokhwa_core::stream::StreamTrait;
use nokhwa_core::types::{
    Backends, CameraFormat, CameraIndex, CameraInformation, FrameRate, QueriedCamera, Resolution,
};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;
use v4l::context::enum_devices;
use v4l::control::{Description, Flags, MenuItem, Type, Value};
use v4l::frameinterval::FrameIntervalEnum;
use v4l::io::traits::{CaptureStream, Stream};
use v4l::prelude::MmapStream;
use v4l::video::traits::Capture;
use v4l::{Capabilities, Device, FourCC, FrameInterval};
use v4l2_sys_mit::{
    V4L2_CAMERA_ORIENTATION_BACK, V4L2_CAMERA_ORIENTATION_EXTERNAL, V4L2_CAMERA_ORIENTATION_FRONT,
    V4L2_CID_AUTO_EXPOSURE_BIAS, V4L2_CID_AUTO_FOCUS_RANGE, V4L2_CID_AUTO_FOCUS_STATUS,
    V4L2_CID_AUTO_N_PRESET_WHITE_BALANCE, V4L2_CID_AUTO_WHITE_BALANCE, V4L2_CID_CAMERA_ORIENTATION,
    V4L2_CID_EXPOSURE_ABSOLUTE, V4L2_CID_EXPOSURE_AUTO, V4L2_CID_EXPOSURE_METERING,
    V4L2_CID_FLASH_LED_MODE, V4L2_CID_FLASH_STROBE, V4L2_CID_FLASH_STROBE_STATUS,
    V4L2_CID_FLASH_STROBE_STOP, V4L2_CID_FOCUS_ABSOLUTE, V4L2_CID_FOCUS_AUTO,
    V4L2_CID_FOCUS_RELATIVE, V4L2_CID_IRIS_ABSOLUTE, V4L2_CID_IRIS_RELATIVE,
    V4L2_CID_ISO_SENSITIVITY, V4L2_CID_ISO_SENSITIVITY_AUTO, V4L2_CID_ZOOM_ABSOLUTE,
    V4L2_CID_ZOOM_CONTINUOUS, V4L2_CID_ZOOM_RELATIVE,
};

fn index_capabilities_to_camera_info(capabilities: Capabilities) -> CameraInformation {
    let name = capabilities.card;
    let description = capabilities.driver;
    let misc = format!(
        "{} v{}.{}.{} Flags: {}",
        capabilities.bus,
        capabilities.version.0,
        capabilities.version.1,
        capabilities.version.2,
        capabilities.capabilities
    );

    CameraInformation::new(name, description, misc, None)
}

macro_rules! define_back_and_forth {
    ( $($frame_format:path => $fourcc:literal ,)+ ) => {
        fn frame_format_to_fourcc(frame_format: FrameFormat) -> Result<FourCC, NokhwaError> {
            match frame_format {
                $(
                $frame_format => Ok(FourCC::new($fourcc)),
                )+
                FrameFormat::Custom(def) => {
                    if let CustomFrameFormat::FourCC(fcc) = def {
                            // if 4-7 is set (non-null) return an error.
                        Ok(FourCC {
                            repr: fcc,
                        })

                    } else {
                    return Err(NokhwaError::InvalidFrameFormat(frame_format, "Unsupported CustomFrameFormat".to_string()))
                    }
                }
                _ => {
                    return Err(NokhwaError::InvalidFrameFormat(frame_format, "Unsupported FrameFormat".to_string()))
                }
            }
        }

        fn fourcc_to_frame_format(four_cc: FourCC) -> FrameFormat {
            match &four_cc.repr {
                $(
                $fourcc => $frame_format,
                )+
                custom => FrameFormat::Custom(CustomFrameFormat::FourCC(*custom))
            }
        }
    }
}

define_back_and_forth!(
    FrameFormat::H265 => b"HEVC",
    FrameFormat::H264 => b"H264",
    FrameFormat::AVC1 => b"AVC1",
    FrameFormat::H263 => b"H263",
    FrameFormat::AV1 => b"AV1F",
    FrameFormat::MPEG_1 => b"MPG1",
    FrameFormat::MPEG_2 => b"MPG2",
    FrameFormat::MPEG_4 => b"MPG4",
    FrameFormat::MJPEG => b"MJPG",
    FrameFormat::XviD => b"XVID",
    FrameFormat::VP8 => b"VP80",
    FrameFormat::VP9 => b"VP90",
    FrameFormat::Ayuv_32 => b"AYUV",
    FrameFormat::Yuyv_4_2_2 => b"YUYV",
    FrameFormat::Uyvy_4_2_2 => b"UYVY",
    FrameFormat::Yvyu_4_2_2 => b"YVYU",
    FrameFormat::NV12 => b"NV12",
    FrameFormat::NV21 => b"NV21",
    FrameFormat::NV16 => b"NV16",
    FrameFormat::NV61 => b"NV61",
    FrameFormat::NV24 => b"NV24",
    FrameFormat::NV42 => b"NV42",
    FrameFormat::Luma_8 => b"GREY",
    FrameFormat::Luma_16 => b"Y16 ",
    FrameFormat::Depth_16 => b"Z16 ",
    FrameFormat::Rgb_3_3_2 => b"RGB1",
    FrameFormat::Rgb_8_8_8 => b"RGB3",
    FrameFormat::Bgr_8_8_8 => b"BGR3",
    FrameFormat::Bgra_8_8_8_8 => b"RA24",
    FrameFormat::Rgba_8_8_8_8 => b"AB24",
    FrameFormat::Argb_8_8_8_8 => b"BA24",
);

macro_rules! define_control_id_conv {
    ( $($control_id:path => $v4l_cid:ident ,)+ ) => {
        fn control_id_to_cid(control_id: ControlId) -> Result<u32, NokhwaError> {
            match control_id {
                $(
                $control_id => Ok($v4l_cid),
                )+
                ControlId::Custom(custom_id) => {
                    match custom_id {
                        CustomControlId::U32(specific) => Ok(specific),
                        _ => Err(NokhwaError::InvalidControlId(control_id, "expected u32 control id".to_string()))
                    }
                }
                _ => Err(NokhwaError::InvalidControlId(control_id, "Could not match ID".to_string())
                )
            }
        }

        fn control_id_to_cid_ref(control_id: &ControlId) -> Result<u32, NokhwaError> {
            match control_id {
                $(
                $control_id => Ok($v4l_cid),
                )+
                ControlId::Custom(specific_id) => {
                    match specific_id {
                        CustomControlId::U32(specific) => Ok(*specific),
                        _ => Err(NokhwaError::InvalidControlId(*control_id, "expected u32 control id".to_string()))
                    }
                }
                _ => Err(NokhwaError::InvalidControlId(*control_id, "Could not match ID".to_string())
                )
            }
        }

        fn cid_to_control_id(cid: u32) -> ControlId {
            match cid {
                $(
                $v4l_cid => $control_id,
                )+
                other_id => ControlId::Custom(other_id.into())
            }
        }
    }
}

define_control_id_conv!(
    ControlId::FocusMode => V4L2_CID_FOCUS_AUTO,
    ControlId::FocusAutoRange => V4L2_CID_AUTO_FOCUS_RANGE,
    ControlId::FocusAbsolute => V4L2_CID_FOCUS_ABSOLUTE,
    ControlId::FocusRelative => V4L2_CID_FOCUS_RELATIVE,
    ControlId::FocusStatus => V4L2_CID_AUTO_FOCUS_STATUS,

    ControlId::ExposureMode => V4L2_CID_EXPOSURE_AUTO,
    ControlId::ExposureBias => V4L2_CID_AUTO_EXPOSURE_BIAS,
    ControlId::ExposureMetering => V4L2_CID_EXPOSURE_METERING,
    ControlId::ExposureAbsolute =>V4L2_CID_EXPOSURE_ABSOLUTE,

    ControlId::IsoMode =>V4L2_CID_ISO_SENSITIVITY_AUTO,
    ControlId::IsoSensitivity => V4L2_CID_ISO_SENSITIVITY,

    ControlId::ApertureAbsolute => V4L2_CID_IRIS_ABSOLUTE,
    ControlId::ApertureRelative => V4L2_CID_IRIS_RELATIVE,

    ControlId::WhiteBalanceMode => V4L2_CID_AUTO_WHITE_BALANCE,
    ControlId::WhiteBalanceTemperature => V4L2_CID_AUTO_N_PRESET_WHITE_BALANCE,

    ControlId::ZoomContinuous => V4L2_CID_ZOOM_CONTINUOUS,
    ControlId::ZoomRelative => V4L2_CID_ZOOM_RELATIVE,
    ControlId::ZoomAbsolute => V4L2_CID_ZOOM_ABSOLUTE,

    ControlId::LightingMode => V4L2_CID_FLASH_LED_MODE,
    ControlId::LightingStart => V4L2_CID_FLASH_STROBE,
    ControlId::LightingStop => V4L2_CID_FLASH_STROBE_STOP,
    ControlId::LightingStatus => V4L2_CID_FLASH_STROBE_STATUS,

    ControlId::Orientation => V4L2_CID_CAMERA_ORIENTATION,
);

fn flags(flags: Flags) -> HashSet<ControlFlags> {
    let mut output_flags = HashSet::new();

    if flags.intersects(Flags::DISABLED) {
        output_flags.insert(ControlFlags::Disabled);
    }
    if flags.intersects(Flags::GRABBED) {
        output_flags.insert(ControlFlags::Busy);
    }
    if flags.intersects(Flags::READ_ONLY) {
        output_flags.insert(ControlFlags::ReadOnly);
    }
    if flags.intersects(Flags::UPDATE) {
        output_flags.insert(ControlFlags::CascadingUpdates);
    }
    if flags.intersects(Flags::SLIDER) {
        output_flags.insert(ControlFlags::Slider);
    }
    if flags.intersects(Flags::WRITE_ONLY) {
        output_flags.insert(ControlFlags::WriteOnly);
    }
    if flags.intersects(Flags::VOLATILE) {
        output_flags.insert(ControlFlags::Volatile);
    }
    if flags.intersects(Flags::EXECUTE_ON_WRITE) {
        output_flags.insert(ControlFlags::ExecuteOnWrite);
    }

    output_flags
}

fn convert_description_to_ctrl_body(description: &Description) -> Option<ControlDescription> {
    let flags = flags(description.flags);

    let (descriptor, default) = match description.typ {
        Type::Integer | Type::Integer64 => (
            ControlValueDescriptor::Integer(Range::new(
                description.minimum,
                description.maximum,
                Some(description.step as i64),
            )),
            Some(ControlValue::Integer(description.default)),
        ),
        Type::U8 => (
            ControlValueDescriptor::Integer(Range::new(
                0,
                u8::MAX as i64,
                Some(description.step as i64),
            )),
            Some(ControlValue::Integer(description.default)),
        ),
        Type::U16 => (
            ControlValueDescriptor::Integer(Range::new(
                0,
                u16::MAX as i64,
                Some(description.step as i64),
            )),
            Some(ControlValue::Integer(description.default)),
        ),
        Type::U32 => (
            ControlValueDescriptor::Integer(Range::new(
                0,
                u32::MAX as i64,
                Some(description.step as i64),
            )),
            Some(ControlValue::Integer(description.default)),
        ),
        Type::String => (ControlValueDescriptor::String, None),
        Type::Boolean => (
            ControlValueDescriptor::Boolean,
            Some(ControlValue::Boolean(description.default != 0)),
        ),
        Type::Bitmask => (
            ControlValueDescriptor::BitMask,
            Some(ControlValue::BitMask(description.default as u64)),
        ),
        Type::IntegerMenu | Type::Menu => {
            // our keys
            let descriptor = match &description.items {
                Some(items) => ControlValueDescriptor::Menu(
                    items
                        .into_iter()
                        .map(|(idx, menu_item)| {
                            (
                                ControlValue::Integer(*idx as i64),
                                match menu_item {
                                    MenuItem::Name(name) => ControlValue::String(name.clone()),
                                    MenuItem::Value(v) => ControlValue::Integer(*v),
                                },
                            )
                        })
                        .collect::<HashMap<ControlValue, ControlValue>>(),
                ),
                // This can probably never happen so we just immediately return if this bad thing
                // happens somehow
                None => return None,
            };
            (descriptor, Some(ControlValue::Integer(description.default)))
        }
        Type::Button => (ControlValueDescriptor::Null, None),

        // we simply will not support control class.
        // if someone needs it we can fix it later.
        // honestly the whole concept scares me.
        // i also have no idea on what an Area could be
        // v4l2 docs are very sparse with this info. https://docs.kernel.org/userspace-api/media/v4l/ext-ctrls-image-source.html#c.v4l2_area
        _ => return None,
    };

    ControlDescription::new(flags, descriptor, default)
}

fn conv_control_value_to_v4l_value(control: ControlValue) -> Result<Value, NokhwaError> {
    let value = match &control {
        ControlValue::Null => Value::None,
        ControlValue::Integer(i) => Value::Integer(*i),
        ControlValue::BitMask(bm) => Value::Integer(*bm as i64),
        ControlValue::String(s) => Value::String(s.clone()),
        ControlValue::Boolean(t) => Value::Boolean(*t),
        ControlValue::Binary(b) => Value::CompoundU8(b.clone()),
        ControlValue::EnumPick(e) => {
            if let ControlValue::Integer(i) = &**e {
                Value::Integer(*i)
            } else {
                return Err(NokhwaError::InvalidControlValue(control.clone()));
            }
        }
        ControlValue::Orientation(o) => Value::Integer(match o {
            Orientation::User => V4L2_CAMERA_ORIENTATION_FRONT as i64,
            Orientation::Environment => V4L2_CAMERA_ORIENTATION_BACK as i64,
            Orientation::Custom(i) => *i,
            _ => V4L2_CAMERA_ORIENTATION_EXTERNAL as i64,
        }),
        _ => {
            return Err(NokhwaError::InvalidControlValue(control.clone()));
        }
    };

    Ok(value)
}

pub struct V4L2Platform {}

impl PlatformTrait for V4L2Platform {
    const PLATFORM: Backends = Backends::Video4Linux2;
    type Camera = V4L2Camera;

    fn block_on_permission(&mut self) -> NokhwaResult<()> {
        Ok(())
    }

    fn check_permission_given(&mut self) -> bool {
        true
    }

    fn query(&mut self) -> NokhwaResult<Vec<QueriedCamera>> {
        Ok(enum_devices()
            .into_iter()
            .map(|v4l_node| {
                let index = v4l_node.index();
                // open camera for capabilities. if we dont get any, dont return the camera
                Device::new(index)
                    .map(|dev| {
                        dev.query_caps()
                            .map(|caps| QueriedCamera {
                                index: CameraIndex::Index(index as u32),
                                information: index_capabilities_to_camera_info(caps),
                            })
                            .ok()
                    })
                    .ok()
                    .flatten()
            })
            .flatten()
            .collect::<Vec<_>>())
    }

    fn open(&mut self, index: CameraIndex) -> NokhwaResult<Self::Camera> {
        let device = match &index {
            CameraIndex::Index(i) => Device::new(*i as usize),
            CameraIndex::String(path) => Device::with_path(path),
            CameraIndex::Stable(_) => {
                return Err(NokhwaError::UnsupportedOperationError(
                    Backends::Video4Linux2,
                ))
            }
        }
        .map_err(|why| NokhwaError::OpenDeviceError(index, why.to_string()))?;

        let v4l2_camera = V4L2Camera { device };

        Ok(v4l2_camera)
    }
}

pub struct V4L2Camera {
    device: Device,
}

impl CameraTrait for V4L2Camera {
    type Stream = V4L2Stream;

    fn enumerate_formats(&self) -> Result<Vec<CameraFormat>, NokhwaError> {
        let mut formats = vec![];

        for (desc, frame_format) in self
            .device
            .enum_formats()
            .map_err(|why| NokhwaError::ListFourCCError(why.to_string()))?
            .into_iter()
            .map(|desc| {
                let fourcc = fourcc_to_frame_format(desc.fourcc);
                (desc, fourcc)
            })
        {
            let resolutions = self
                .device
                .enum_framesizes(desc.fourcc)
                .map_err(|why| NokhwaError::ListResolutionError(why.to_string()))?
                .into_iter()
                .flat_map(|frame_size| frame_size.size.to_discrete())
                .map(|discrete| Resolution::new(discrete.width, discrete.height))
                .collect::<Vec<Resolution>>();

            let v4l2_frame_intervals = resolutions
                .iter()
                .map(|resolution| {
                    let frame_intervals = self.device.enum_frameintervals(
                        desc.fourcc,
                        resolution.width(),
                        resolution.height(),
                    );

                    frame_intervals.and_then(|ints| Ok((*resolution, ints)))
                })
                .collect::<Result<Vec<(Resolution, Vec<FrameInterval>)>, std::io::Error>>()
                .map_err(|why| NokhwaError::ListFrameRatesError(why.to_string()))?;

            formats.extend(
                v4l2_frame_intervals
                    .iter()
                    .flat_map(|(resolution, interval)| {
                        interval
                            .iter()
                            .flat_map(|frame_interval| match &frame_interval.interval {
                                FrameIntervalEnum::Discrete(fraction) => {
                                    vec![FrameRate::new(fraction.numerator, fraction.denominator)]
                                }
                                FrameIntervalEnum::Stepwise(stepwise) => {
                                    let min = stepwise.min.numerator;
                                    let max = stepwise.max.numerator;

                                    // short circuit if denominators differ
                                    if stepwise.step.denominator != stepwise.max.denominator
                                        || stepwise.step.denominator != stepwise.min.denominator
                                    {
                                        return vec![];
                                    }

                                    (min..max)
                                        .step_by(stepwise.step.numerator as usize)
                                        .map(|num| FrameRate::new(num, stepwise.step.denominator))
                                        .collect()
                                }
                            })
                            .map(|frame_rate| {
                                CameraFormat::new(*resolution, frame_format, frame_rate)
                            })
                    }),
            );
        }
        formats.dedup();
        Ok(formats)
    }

    fn controls(&self) -> Result<Vec<NokhwaControl>, NokhwaError> {
        let controls = self
            .device
            .query_controls()
            .map_err(|why| NokhwaError::ListControlError(why.to_string()))?
            .iter()
            .filter_map(|descriptor| {
                convert_description_to_ctrl_body(descriptor).map(|x| NokhwaControl {
                    id: cid_to_control_id(descriptor.id),
                    description: x,
                })
            })
            .collect::<Vec<NokhwaControl>>();
        Ok(controls)
    }

    fn control_value(&self, id: ControlId) -> Result<ControlValue, NokhwaError> {
        todo!()
    }

    fn set_control(&self, id: ControlId, value: ControlValue) -> Result<(), NokhwaError> {
        todo!()
    }

    fn open_stream<FrameCallback, ErrorCallback>(
        &mut self,
        camera_format: CameraFormat,
        frame_callback: FrameCallback,
        error_callback: ErrorCallback,
    ) -> Result<Self::Stream, NokhwaError>
    where
        FrameCallback: FnMut(nokhwa_core::frame_buffer::FrameBuffer<'_>) + Send + 'static,
        ErrorCallback: FnMut(NokhwaError) + Send + 'static,
    {
        todo!()
    }
}

pub struct V4L2Stream {
    format: CameraFormat,
    join_handle: JoinHandle<()>,
    stop: AtomicBool,

}

impl V4L2Stream {
    fn new<FrameCallback, ErrorCallback>(
        mut stream: MmapStream<'_>,
        format: CameraFormat,
        index: CameraIndex,
        fcb: FrameCallback,
        ecb: ErrorCallback,
    ) -> Result<Self, NokhwaError>
    where
        FrameCallback: FnMut(nokhwa_core::frame_buffer::FrameBuffer<'_>) + Send + 'static,
        ErrorCallback: FnMut(NokhwaError) + Send + 'static,
    {
        Ok(())
    }
}

impl StreamTrait for V4L2Stream {
    fn current_format(&self) -> &CameraFormat {
        &self.format
    }

    fn stop_stream(mut self) -> Result<(), NokhwaError> {
        self.device.stop()?;
        Ok(())
    }
}

fn frame_poller_v4l2<FrameCallback, ErrorCallback>(
    index: CameraIndex,
    mut stream: MmapStream<'_>,
    mut fcb: FrameCallback,
    mut ecb: ErrorCallback,
) where
    FrameCallback: FnMut(nokhwa_core::frame_buffer::FrameBuffer<'_>) + Send + 'static,
    ErrorCallback: FnMut(NokhwaError) + Send + 'static,
{
    loop {
        let (frame_data, metadata) = match stream.next() {
            Ok((f, m)) => (f, m),
            Err(why) => {
                // match why {
                //     Ok(_) => todo!(),
                //     Err(_) => todo!(),
                // }
                ecb(NokhwaError::StreamError(StreamError::StreamInvalidated));
                break;
            },
        };
        let converted_metadata = {
            let mut meta = Metadata::new();
            meta.insert(MetadataTypes::Size(metadata.bytesused as u64));
            let time = Time::
            meta.insert(MetadataTypes::Timestamp());
            meta.insert(MetadataTypes::Size(metadata.bytesused as u64));
        }
    }
}

// pub struct V4L2Camera<'a> {
//     device: Device,
//     camera_format: Option<CameraFormat>,
//     camera_index: CameraIndex,
//     controls: Controls,
//     stream: Option<V4L2Stream<'a>>,
//     _phantom: PhantomData<&'a V4L2Platform>,
// }

// impl<'a> Setting for V4L2Camera<'a> {
//     fn enumerate_formats(&self) -> Result<Vec<CameraFormat>, NokhwaError> {
//         let mut formats = vec![];

//         for frame_format in self
//             .device
//             .enum_formats()
//             .map_err(|why| NokhwaError::GetPropertyError {
//                 property: "enum_formats".to_string(),
//                 error: why.to_string(),
//             })?
//             .into_iter()
//             .map(|desc| fourcc_to_frame_format(desc.fourcc))
//         {
//             formats.extend(
//                 self.enumerate_resolution_and_frame_rates(frame_format)?
//                     .into_iter()
//                     .flat_map(|(resolution, frame_rates)| {
//                         frame_rates.into_iter().map(move |frame_rate| {
//                             CameraFormat::new(resolution, frame_format, frame_rate)
//                         })
//                     }),
//             );
//         }
//         Ok(formats)
//     }

//     fn enumerate_resolution_and_frame_rates(
//         &self,
//         frame_format: FrameFormat,
//     ) -> Result<HashMap<Resolution, Vec<FrameRate>>, NokhwaError> {
//         let fourcc = frame_format_to_fourcc(frame_format)?;
//         let resolutions = self
//             .device
//             .enum_framesizes(fourcc)
//             .map_err(|why| NokhwaError::GetPropertyError {
//                 property: "enum_framesizes".to_string(),
//                 error: why.to_string(),
//             })?
//             .into_iter()
//             .flat_map(|frame_size| frame_size.size.to_discrete())
//             .map(|discrete| Resolution::new(discrete.width, discrete.height))
//             .collect::<Vec<Resolution>>();

//         let v4l2_frame_intervals = resolutions
//             .iter()
//             .map(|resolution| {
//                 let frame_intervals = self.device.enum_frameintervals(
//                     fourcc,
//                     resolution.width(),
//                     resolution.height(),
//                 );

//                 frame_intervals.and_then(|ints| Ok((*resolution, ints)))
//             })
//             .collect::<Result<Vec<(Resolution, Vec<FrameInterval>)>, std::io::Error>>()
//             .map_err(|why| NokhwaError::GetPropertyError {
//                 property: "enum_frameintervals".to_string(),
//                 error: why.to_string(),
//             })?;

//         Ok(v4l2_frame_intervals
//             .iter()
//             .flat_map(|(resolution, interval)| {
//                 interval.iter().map(|int| {
//                     match &int.interval {
//                         FrameIntervalEnum::Discrete(discrete) => {
//                             NonZeroI32::new(discrete.denominator as i32).map(|denominator| {
//                                 (
//                                     *resolution,
//                                     vec![FrameRate::new(discrete.numerator as i32, denominator)],
//                                 )
//                             })
//                         }
//                         FrameIntervalEnum::Stepwise(stepwise) => {
//                             // we have to do this ourselves

//                             // no logic to handle different or zero demoninator
//                             if (stepwise.step.denominator != stepwise.max.denominator)
//                                 || (stepwise.step.denominator != stepwise.min.denominator)
//                             {
//                                 return None;
//                             }

//                             let min = stepwise.min.numerator as i32;
//                             let max = stepwise.max.numerator as i32;
//                             let step = stepwise.step.numerator as i32;
//                             let denominator = stepwise.step.denominator as i32;

//                             NonZeroI32::new(denominator).map(|denominator| {
//                                 (
//                                     *resolution,
//                                     (min..max)
//                                         .step_by(step as usize)
//                                         .map(|numerator| FrameRate::new(numerator, denominator))
//                                         .collect::<Vec<FrameRate>>(),
//                                 )
//                             })
//                         }
//                     }
//                 })
//             })
//             .flatten()
//             .collect::<HashMap<Resolution, Vec<FrameRate>>>())
//     }

//     fn set_format(&mut self, camera_format: CameraFormat) -> Result<(), NokhwaError> {
//         let fourcc = frame_format_to_fourcc(camera_format.format())?;
//         self.device
//             .set_format(&Format::new(
//                 camera_format.width(),
//                 camera_format.height(),
//                 fourcc,
//             ))
//             .map_err(|why| NokhwaError::SetPropertyError {
//                 property: "set_format".to_string(),
//                 value: format!("format: {camera_format} fourcc: {fourcc}"),
//                 error: why.to_string(),
//             })?;
//         self.device
//             .set_params(&Parameters::new(Fraction::new(
//                 camera_format.frame_rate().numerator() as u32,
//                 camera_format.frame_rate().denominator() as u32,
//             )))
//             .map_err(|why| NokhwaError::SetPropertyError {
//                 property: "set_params".to_string(),
//                 value: format!("{}", camera_format.frame_rate()),
//                 error: why.to_string(),
//             })?;
//         self.camera_format = Some(camera_format);
//         Ok(())
//     }

//     fn control_ids(&self) -> Keys<ControlId, ControlDescription> {
//         self.controls.ids()
//     }

//     fn control_descriptions(&self) -> Values<ControlId, ControlDescription> {
//         self.controls.descriptions()
//     }

//     fn control_values(&self) -> Values<ControlId, ControlValue> {
//         self.controls.values()
//     }

//     fn control_value(&self, id: &ControlId) -> Option<&ControlValue> {
//         self.controls.value(id)
//     }

//     fn control_description(&self, id: &ControlId) -> Option<&ControlDescription> {
//         self.controls.description(id)
//     }

//     fn set_control(
//         &mut self,
//         property: &ControlId,
//         value: ControlValue,
//     ) -> Result<(), NokhwaError> {
//         if !self.controls.validate(property, &value)? {
//             return Err(NokhwaError::SetPropertyError {
//                 property: property.to_string(),
//                 value: value.to_string(),
//                 error: "failed to validate".to_string(),
//             });
//         }
//         let cid = control_id_to_cid(*property)?;
//         let v4l_value = conv_control_value_to_v4l_value(value.clone())?;
//         self.device
//             .set_control(Control {
//                 id: cid,
//                 value: v4l_value,
//             })
//             .map_err(|why| NokhwaError::SetPropertyError {
//                 property: cid.to_string(),
//                 value: value.to_string(),
//                 error: why.to_string(),
//             })?;
//         self.controls.set_control_value(property, value)?;
//         Ok(())
//     }

//     fn refresh_controls(&mut self) -> Result<(), NokhwaError> {
//         let descriptions = self
//             .device
//             .query_controls()
//             .map_err(|why| NokhwaError::GetPropertyError {
//                 property: "query_controls".to_string(),
//                 error: why.to_string(),
//             })?
//             .into_iter()
//             .map(|description| {
//                 let id = cid_to_control_id(description.id);

//                 convert_description_to_ctrl_body(description).map(|body| (id, body))
//             })
//             .flatten()
//             .collect::<HashMap<ControlId, ControlDescription>>();

//         let values = descriptions
//             .keys()
//             .into_iter()
//             .copied()
//             .flat_map(|k| control_id_to_cid(k).map(|cid| (k, cid)))
//             .flat_map(|(id, cid)| self.device.control(cid).map(|v| (id, v)))
//             .map(|(id, value)| {
//                 (
//                     id,
//                     match value.value {
//                         Value::None => ControlValue::Null,
//                         Value::Integer(i) => ControlValue::Integer(i),
//                         Value::Boolean(b) => ControlValue::Boolean(b),
//                         Value::String(s) => ControlValue::String(s),
//                         Value::CompoundU8(bin) | Value::CompoundPtr(bin) => {
//                             ControlValue::Binary(bin)
//                         }
//                         Value::CompoundU16(c_16) => ControlValue::Array(
//                             c_16.into_iter()
//                                 .map(|u| ControlValue::Integer(u as i64))
//                                 .collect(),
//                         ),
//                         Value::CompoundU32(c_32) => ControlValue::Array(
//                             c_32.into_iter()
//                                 .map(|u| ControlValue::Integer(u as i64))
//                                 .collect(),
//                         ),
//                     },
//                 )
//             })
//             .collect::<HashMap<ControlId, ControlValue>>();

//         match Controls::new(descriptions, values) {
//             Some(c) => {
//                 self.controls = c;
//             }
//             None => {
//                 return Err(NokhwaError::SetPropertyError {
//                     property: "control".to_string(),
//                     value: format!(""),
//                     error: "Failed to convert to control".to_string(),
//                 })
//             }
//         }

//         Ok(())
//     }
// }

// struct V4L2Stream<'a> {
//     thread: ScopedJoinHandle<'a, ()>,
//     control: Arc<Sender<()>>,
// }

// impl<'a> V4L2Stream<'a> {
//     pub fn join(self) -> Result<(), ()> {
//         self.thread.join().map_err(|_| ())
//     }
// }
