use flume::{bounded, unbounded, Sender};
use nokhwa_core::camera::{Camera, Capture, Setting};
use nokhwa_core::control::{
    ControlDescription, ControlFlags, ControlId, ControlValue, ControlValueDescriptor, Controls,
    Orientation,
};
use nokhwa_core::error::{NokhwaError, NokhwaResult};
use nokhwa_core::frame_buffer::{CompactString, FrameBuffer, Metadata};
use nokhwa_core::frame_format::FrameFormat;
use nokhwa_core::platform::{Backends, PlatformTrait};
use nokhwa_core::ranges::Range;
use nokhwa_core::stream::{Event, StreamBounds, StreamConfiguration, StreamHandle};
use nokhwa_core::types::{CameraFormat, CameraIndex, CameraInformation, FrameRate, Resolution};
use std::borrow::Cow;
use std::collections::hash_map::{Keys, Values};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroI32;
use std::sync::Arc;
use std::thread::JoinHandle;
use v4l::context::enum_devices;
use v4l::control::{Description, Flags, MenuItem, Type, Value};
use v4l::frameinterval::FrameIntervalEnum;
use v4l::io::traits::OutputStream;
use v4l::prelude::MmapStream;
use v4l::video::output::Parameters;
use v4l::video::Output;
use v4l::{Capabilities, Control, Device, Format, FourCC, Fraction, FrameInterval};
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

fn index_capabilities_to_camera_info(index: u32, capabilities: Capabilities) -> CameraInformation {
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

    CameraInformation::new(name, description, misc, CameraIndex::Index(index))
}

macro_rules! define_back_and_forth {
    ( $($frame_format:expr => $fourcc:expr ,)+ ) => {
        fn frame_format_to_fourcc(frame_format: FrameFormat) -> Result<FourCC, NokhwaError> {
            match frame_format {
                $(
                $frame_format => Ok(FourCC::new($fourcc)),
                )+
            FrameFormat::Custom(def) => {
            // if 4-7 is set (non-null) return an error.
            if def[4..=7] != [0x00, 0x00, 0x00, 0x00] {
                return Err(NokhwaError::ConversionError("Invalid: Custom bytes 4-7 are set (linux only uses 0-3)".to_string()))
            }
            Ok(FourCC::new(&[def[0], def[1], def[2], def[3]]))
        }
        _ => {
            return Err(NokhwaError::ConversionError("Unsupported FrameFormat".to_string()))
        }}
        }

        fn fourcc_to_frame_format(four_cc: FourCC) -> FrameFormat {
            match &four_cc.repr {
                $(
                $fourcc => $frame_format
                )+
                custom => FrameFormat::Custom([ custom[0], custom[1], custom[2], custom[3], 0x00, 0x00, 0x00, 0x00 ])
            }
        }
    }
}

define_back_and_forth!(
    FrameFormat::H265 => b"HEVC",
    FrameFormat::H264 => b"H264",
    FrameFormat::Avc1 => b"AVC1",
    FrameFormat::H263 => b"H263",
    FrameFormat::Av1 => b"AV1F",
    FrameFormat::Mpeg1 => b"MPG1",
    FrameFormat::Mpeg2 => b"MPG2",
    FrameFormat::Mpeg4 => b"MPG4",
    FrameFormat::MJpeg => b"MJPG",
    FrameFormat::XVid => b"XVID",
    FrameFormat::VP8 => b"VP80",
    FrameFormat::VP9 => b"VP90",
    FrameFormat::Ayuv444 => b"AYUV",
    FrameFormat::Yuyv422 => b"YUYV",
    FrameFormat::Uyvy422 => b"UYVY",
    FrameFormat::Yvyu422 => b"YVYU",
    FrameFormat::Yv12 => b"YV12",
    FrameFormat::Nv12 => b"NV12",
    FrameFormat::Nv21 => b"NV21",
    FrameFormat::I420 => b"YU12",
    FrameFormat::Yvu9 => b"YVU9",
    FrameFormat::Luma8 => b"GREY",
    FrameFormat::Luma16 => b"Y16 ",
    FrameFormat::Depth16 => b"Z16 ",
    FrameFormat::Rgb332 => b"RGB1",
    FrameFormat::Rgb888 => b"RGB3",
    FrameFormat::Bgr888 => b"BGR3",
    FrameFormat::BgrA8888 => b"RA24",
    FrameFormat::RgbA8888 => b"AB24",
    FrameFormat::ARgb8888 => b"BA24",
    FrameFormat::Bayer8 => b"BA81",
    FrameFormat::Bayer16 => b"BYR2",
);

macro_rules! define_control_id_conv {
    ( $($control_id:expr => $v4l_cid:expr ,)+ ) => {
        fn control_id_to_cid(control_id: ControlId) -> Result<u32, NokhwaError> {
            match control_id {
                $(
                $control_id => Ok($v4l_cid)
                )+
                ControlId::PlatformSpecific(specific_id) => {
                    u32::try_from(specific_id).map_err(|why| {
                        NokhwaError::ConversionError("ID must be a u32".to_string())
                    })
                }
                _ => Err(NokhwaError::ConversionError("Could not match ID".to_string())
                )
            }
        }

        fn control_id_to_cid_ref(control_id: &ControlId) -> Result<u32, NokhwaError> {
            match control_id {
                $(
                $control_id => Ok($v4l_cid)
                )+
                ControlId::PlatformSpecific(specific_id) => {
                    u32::try_from(specific_id).map_err(|why| {
                        NokhwaError::ConversionError("ID must be a u32".to_string())
                    })
                }
                _ => Err(NokhwaError::ConversionError("Could not match ID".to_string())
                )
            }
        }

        fn cid_to_control_id(cid: u32) -> ControlId {
            match cid {
                $(
                $v4l_cid => $control_id
                )+
                other_id => ControlId::PlatformSpecific(other_id as u64)
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

fn convert_description_to_ctrl_body(description: Description) -> Option<ControlDescription> {
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
                u8::MAX_VALUE as i64,
                Some(description.step as i64),
            )),
            Some(ControlValue::Integer(description.default)),
        ),
        Type::U16 => (
            ControlValueDescriptor::Integer(Range::new(
                0,
                u16::MAX_VALUE as i64,
                Some(description.step as i64),
            )),
            Some(ControlValue::Integer(description.default)),
        ),
        Type::U32 => (
            ControlValueDescriptor::Integer(Range::new(
                0,
                u32::MAX_VALUE as i64,
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
            let descriptor = match description.items {
                Some(items) => ControlValueDescriptor::Menu(
                    items
                        .into_iter()
                        .map(|(idx, menu_item)| {
                            (
                                ControlValue::Integer(idx as i64),
                                match menu_item {
                                    MenuItem::Name(name) => ControlValue::String(name),
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
    let value = match control {
        ControlValue::Null => Value::None,
        ControlValue::Integer(i) | ControlValue::BitMask(i) => Value::Integer(i),
        ControlValue::String(s) => Value::String(s),
        ControlValue::Boolean(t) => Value::Boolean(t),
        ControlValue::Binary(b) => Value::CompoundU8(b),
        ControlValue::EnumPick(e) => {
            if let ControlValue::Integer(i) = &e {
                Value::Integer(*i)
            } else {
                return Err(NokhwaError::ConversionError(
                    "could not convert non integer enum pick".to_string(),
                ));
            }
        }
        ControlValue::Orientation(o) => Value::Integer(match o {
            Orientation::User => V4L2_CAMERA_ORIENTATION_FRONT as i64,
            Orientation::Environment => V4L2_CAMERA_ORIENTATION_BACK as i64,
            Orientation::Custom(i) => i,
            _ => V4L2_CAMERA_ORIENTATION_EXTERNAL as i64,
        }),
        _ => {
            return Err(NokhwaError::ConversionError(
                "Conversion not supported for this data type.".to_string(),
            ))
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

    fn query(&mut self) -> NokhwaResult<Vec<CameraInformation>> {
        Ok(enum_devices()
            .into_iter()
            .map(|v4l_node| {
                let index = v4l_node.index();
                // open camera for capabilities. if we dont get any, dont return the camera
                Device::new(index)
                    .map(|dev| {
                        dev.query_caps()
                            .map(|caps| index_capabilities_to_camera_info(index as u32, caps))
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
        }
        .map_err(|why| NokhwaError::OpenDeviceError(index.to_string(), why.to_string()))?;

        let mut v4l2_camera = V4L2Camera {
            device,
            camera_format: None,
            camera_index: index,
            controls: Default::default(),
            stream: None,
        };

        v4l2_camera.refresh_controls()?;

        Ok(v4l2_camera)
    }
}

pub struct V4L2Camera {
    device: Device,
    camera_format: Option<CameraFormat>,
    camera_index: CameraIndex,
    controls: Controls,
    stream: Option<V4L2Stream>,
}

impl Setting for V4L2Camera {
    fn enumerate_formats(&self) -> Result<Vec<CameraFormat>, NokhwaError> {
        let mut formats = vec![];

        for frame_format in self
            .device
            .enum_formats()
            .map_err(|why| NokhwaError::GetPropertyError {
                property: "enum_formats".to_string(),
                error: why.to_string(),
            })?
            .into_iter()
            .map(|desc| fourcc_to_frame_format(desc.fourcc))
        {
            formats.extend(
                self.enumerate_resolution_and_frame_rates(frame_format)?
                    .into_iter()
                    .flat_map(|(resolution, frame_rates)| {
                        frame_rates.into_iter().map(|frame_rate| {
                            CameraFormat::new(resolution, frame_format, frame_rate)
                        })
                    }),
            );
        }
        Ok(formats)
    }

    fn enumerate_resolution_and_frame_rates(
        &self,
        frame_format: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<FrameRate>>, NokhwaError> {
        let fourcc = frame_format_to_fourcc(frame_format)?;
        let resolutions = self
            .device
            .enum_framesizes(fourcc)
            .map_err(|why| NokhwaError::GetPropertyError {
                property: "enum_framesizes".to_string(),
                error: why.to_string(),
            })?
            .into_iter()
            .flat_map(|frame_size| frame_size.size.to_discrete())
            .map(|discrete| Resolution::new(discrete.width, discrete.height))
            .collect::<Vec<Resolution>>();

        let v4l2_frame_intervals = resolutions
            .iter()
            .map(|resolution| {
                (
                    *resolution,
                    self.device.enum_frameintervals(
                        fourcc,
                        resolution.width(),
                        resolution.height(),
                    ),
                )
            })
            .collect::<Result<Vec<(Resolution, Vec<FrameInterval>)>, std::io::Error>>()
            .map_err(|why| NokhwaError::GetPropertyError {
                property: "enum_frameintervals".to_string(),
                error: why.to_string(),
            })?;

        Ok(v4l2_frame_intervals
            .into_iter()
            .flatten()
            .flat_map(|(resolution, interval)| {
                match interval.interval {
                    FrameIntervalEnum::Discrete(discrete) => {
                        NonZeroI32::new(discrete.denominator as i32).map(|denominator| {
                            (
                                resolution,
                                vec![FrameRate::new(discrete.numerator as i32, denominator)],
                            )
                        })
                    }
                    FrameIntervalEnum::Stepwise(stepwise) => {
                        // we have to do this ourselves

                        // no logic to handle different or zero demoninator
                        if (stepwise.step.denominator != stepwise.max.denominator)
                            || (stepwise.step.denominator != stepwise.min.denominator)
                        {
                            return None;
                        }

                        let min = stepwise.min.numerator as i32;
                        let max = stepwise.max.numerator as i32;
                        let step = stepwise.step.numerator as i32;
                        let denominator = stepwise.step.denominator as i32;

                        NonZeroI32::new(denominator).map(|denominator| {
                            (
                                resolution,
                                (min..max)
                                    .step_by(step as usize)
                                    .map(|numerator| FrameRate::new(numerator, denominator))
                                    .collect::<Vec<FrameRate>>(),
                            )
                        })
                    }
                }
            })
            .flatten()
            .collect::<HashMap<Resolution, Vec<FrameRate>>>())
    }

    fn set_format(&mut self, camera_format: CameraFormat) -> Result<(), NokhwaError> {
        let fourcc = frame_format_to_fourcc(*camera_format.format())?;
        self.device
            .set_format(&Format::new(
                camera_format.width(),
                camera_format.height(),
                fourcc,
            ))
            .map_err(|why| NokhwaError::SetPropertyError {
                property: "set_format".to_string(),
                value: format!("format: {camera_format} fourcc: {fourcc}"),
                error: why.to_string(),
            })?;
        self.device
            .set_params(&Parameters::new(Fraction::new(
                *camera_format.frame_rate().numerator() as u32,
                *camera_format.frame_rate().denominator() as u32,
            )))
            .map_err(|why| NokhwaError::SetPropertyError {
                property: "set_params".to_string(),
                value: format!("{}", camera_format.frame_rate()),
                error: why.to_string(),
            })?;
        self.camera_format = Some(camera_format);
        Ok(())
    }

    fn control_ids(&self) -> Keys<ControlId, ControlDescription> {
        self.controls.ids()
    }

    fn control_descriptions(&self) -> Values<ControlId, ControlDescription> {
        self.controls.descriptions()
    }

    fn control_values(&self) -> Values<ControlId, ControlValue> {
        self.controls.values()
    }

    fn control_value(&self, id: &ControlId) -> Option<&ControlValue> {
        self.controls.value(id)
    }

    fn control_description(&self, id: &ControlId) -> Option<&ControlDescription> {
        self.controls.description(id)
    }

    fn set_control(
        &mut self,
        property: &ControlId,
        value: ControlValue,
    ) -> Result<(), NokhwaError> {
        if !self.controls.validate(property, &value)? {
            return Err(NokhwaError::SetPropertyError {
                property: property.to_string(),
                value: value.to_string(),
                error: "failed to validate".to_string(),
            });
        }
        let cid = control_id_to_cid(*property)?;
        let v4l_value = conv_control_value_to_v4l_value(value.clone())?;
        self.device
            .set_control(Control {
                id: cid,
                value: v4l_value,
            })
            .map_err(|why| {
                Err(NokhwaError::SetPropertyError {
                    property: cid.to_string(),
                    value: value.to_string(),
                    error: why.to_string(),
                })
            })?;
        self.controls.set_control_value(property, value)?;
        Ok(())
    }

    fn refresh_controls(&mut self) -> Result<(), NokhwaError> {
        let descriptions = self
            .device
            .query_controls()
            .map_err(|why| NokhwaError::GetPropertyError {
                property: "query_controls".to_string(),
                error: why.to_string(),
            })?
            .into_iter()
            .map(|description| {
                let id = cid_to_control_id(description.id);

                convert_description_to_ctrl_body(description).map(|body| (id, body))
            })
            .flatten()
            .collect::<HashMap<ControlId, ControlDescription>>();

        let values = descriptions
            .keys()
            .into_iter()
            .copied()
            .flat_map(|k| control_id_to_cid(k).map(|cid| (k, cid)))
            .flat_map(|(id, cid)| self.device.control(cid).map(|v| (id, v)))
            .map(|(id, value)| {
                (
                    id,
                    match value.value {
                        Value::None => ControlValue::Null,
                        Value::Integer(i) => ControlValue::Integer(i),
                        Value::Boolean(b) => ControlValue::Boolean(b),
                        Value::String(s) => ControlValue::String(s),
                        Value::CompoundU8(bin) | Value::CompoundPtr(bin) => {
                            ControlValue::Binary(bin)
                        }
                        Value::CompoundU16(u) | Value::CompoundU32(u) => ControlValue::Array(
                            u.into_iter()
                                .map(|u| ControlValue::Integer(u as i64))
                                .collect(),
                        ),
                    },
                )
            })
            .collect::<HashMap<ControlId, ControlValue>>();

        match Controls::new(descriptions, values) {
            Some(c) => {
                self.controls = c;
            }
            None => {
                return Err(NokhwaError::SetPropertyError {
                    property: "control".to_string(),
                    value: format!("{:?} {:?}", descriptions, values),
                    error: "Failed to convert to control".to_string(),
                })
            }
        }

        Ok(())
    }
}

struct V4L2Stream {
    thread: JoinHandle<()>,
    control: Arc<Sender<()>>,
}

impl Drop for V4L2Stream {
    fn drop(&mut self) {
        let _ = self.control.send(());
    }
}

impl Capture for V4L2Camera {
    fn open_stream(
        &mut self,
        stream_configuration: Option<StreamConfiguration>,
    ) -> Result<StreamHandle, NokhwaError> {
        if let Some(_) = self.stream {
            return Err(NokhwaError::OpenStreamError(
                "StreamAlreadyOpen".to_string(),
            ));
        }

        let stream_config = stream_configuration.unwrap_or_default();

        let format = match self.camera_format {
            Some(fmt) => fmt,
            None => return Err(NokhwaError::OpenStreamError("No Format".to_string())),
        };

        let (control, ctrl_recv) = bounded(1);
        let (sender, receiver) = match stream_config.bound {
            StreamBounds::Bounded(b) => bounded(b as usize),
            StreamBounds::Unbounded => unbounded(),
        };

        let control = Arc::new(control);

        let mut mmap_stream = MmapStream::new(&self.device, v4l::buffer::Type::VideoCapture)
            .map_err(|why| return NokhwaError::OpenStreamError(why.to_string()))?;

        let stream_handle = StreamHandle::new(receiver, control.clone(), stream_config, format);

        let thread = std::thread::spawn(move || loop {
            if ctrl_recv.is_disconnected() || sender.is_disconnected() {
                return;
            }
            if let Ok(_) = ctrl_recv.try_recv() {
                let _ = sender.send(Event::Closed);
                return;
            }

            match mmap_stream.next() {
                Ok((data, meta)) => {
                    let data = Cow::Owned(data.to_owned());
                    let mut metadata = Metadata::new();

                    metadata.insert(
                        CompactString::from("flags"),
                        ControlValue::BitMask(meta.flags.bits() as u64),
                    );
                    metadata.insert(
                        CompactString::from("time_secs"),
                        ControlValue::Integer(meta.timestamp.sec),
                    );
                    metadata.insert(
                        CompactString::from("time_usecs"),
                        ControlValue::Integer(meta.timestamp.usec),
                    );
                    metadata.insert(
                        CompactString::from("size"),
                        ControlValue::Integer(meta.bytesused as i64),
                    );
                    metadata.insert(
                        CompactString::from("sequence"),
                        ControlValue::Integer(meta.sequence as i64),
                    );
                    metadata.insert(
                        CompactString::from("field"),
                        ControlValue::Integer(meta.field as i64),
                    );

                    let _ = sender.send(Event::NewFrame(FrameBuffer::new(data, Some(metadata))));
                }
                Err(why) => {
                    let _ = sender.send(Event::Error(Box::new(why)));
                }
            }
        });

        self.stream = Some(V4L2Stream { thread, control });

        Ok(stream_handle)
    }

    fn close_stream(&mut self) -> Result<(), NokhwaError> {
        let mut stream = match std::mem::take(&mut self.stream) {
            Some(s) => s,
            None => {
                return Err(NokhwaError::StreamShutdownError(
                    "No stream to shutdown".to_string(),
                ))
            }
        };

        let _ = stream.control.send(());

        stream
            .thread
            .join()
            .map_err(|why| NokhwaError::StreamShutdownError(format!("{why:?}")))?;

        Ok(())
    }
}

impl Camera for V4L2Camera {}
