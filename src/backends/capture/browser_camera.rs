use std::borrow::Cow;
use std::collections::HashMap;
use js_sys::wasm_bindgen::{JsCast, JsValue};
use js_sys::{Array, Map, Object, Promise};
use nokhwa_core::format_request::FormatRequest;
use serde::{de, Serialize};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, MediaDeviceInfo, MediaDevices, MediaStream, MediaStreamConstraints, MediaStreamTrack, MediaTrackConstraints, Navigator};
use nokhwa_core::buffer::Buffer;
use nokhwa_core::error::NokhwaError;
use nokhwa_core::frame_format::FrameFormat;
use nokhwa_core::traits::{AsyncCaptureTrait, AsyncOpenCaptureTrait, CaptureTrait, OpenCaptureTrait};
use nokhwa_core::types::{ApiBackend, CameraControl, CameraFormat, CameraIndex, CameraInfo, ControlValueSetter, FrameRate, KnownCameraControl, Resolution};

async fn resolve_to<T: JsCast>(promise: Promise) -> Result<T, NokhwaError> {
    let future = JsFuture::from(promise);
    let jsv = match future.await {
        Ok(v) => v,
        Err(why) => NokhwaError::ConversionError(why.as_string().unwrap_or_default())
    };
    // we do a little checking
    if !T::has_type(&jsv) {
        return Err(NokhwaError::ConversionError("Bad Conversion - No Type".to_string()))
    }
    Ok(unsafe { cast_js_value(v) })
}

fn checked_js_cast<T: JsCast>(from: JsValue) -> Result<T, NokhwaError> {
    // we do a little checking
    if !T::has_type(&jsv) {
        return Err(NokhwaError::ConversionError("Bad Conversion - No Type".to_string()))
    }
    Ok(unsafe { cast_js_value(v) })
}

// PLEASE CHECK WHAT YOU'RE DOING OH MY GOD
unsafe fn cast_js_value<T: JsCast>(from: JsValue) -> T {
    JsCast::unchecked_from_js(from)
}

// wasm-bindgen doesnt allow us to access internal attributes for some reason
// because of this, we turn objects into Map. (a JS HashMap)
fn make_jsobj_map(from: impl AsRef<Object>) -> Result<Map, NokhwaError> {
    let kvpairs = Object::entries(from.as_ref());
    // we get the constructor for a map
    let map_constructor = Map::new().constructor();
    // pray we arnt in strict mode
    match map_constructor.call1(&JsValue::null(), &kvpairs) {
        Ok(m) => unsafe { Ok(cast_js_value::<Map>(m)) },
        Err(why) => Err(NokhwaError::ConversionError("failed to construct map to access int. values.".to_string())),
    }

}

#[derive(Serialize)]
struct ConstrainedDouble {
    pub min: Option<f64>,
    pub ideal: Option<f64>,
    pub max: Option<f64>,
    pub exact: Option<f64>,
}

impl Default for ConstrainedDouble {
    fn default() -> Self {
        Self { min: None, ideal: None, max: None, exact: None }
    }
}

impl From<&ConstrainedDouble> for JsValue {
    fn from(value: &ConstrainedDouble) -> Self {
        serde_wasm_bindgen::to_value(value).unwrap()
    }
}

#[derive(Serialize)]
struct ConstrainedULong {
    pub min: Option<u64>,
    pub ideal: Option<u64>,
    pub max: Option<u64>,
    pub exact: Option<u64>,
}


pub struct BrowserCaptureDevice {
    info: CameraInfo,
    group_id: String,
    device_id: String,
    format: CameraFormat,
    media_devices: MediaDevices,
    media_stream: MediaStream
}

impl BrowserCaptureDevice {
    pub async fn new(index: &CameraIndex, camera_fmt: FormatRequest) -> Result<Self, NokhwaError>{
        let nav = window().map(|x| x.navigator()).ok_or(NokhwaError::InitializeError { backend: ApiBackend::Browser, error: "No Window Object!".to_string() })?;
        let media_devices = match nav.media_devices() {
            Ok(m) => m,
            Err(why) => return Err(NokhwaError::InitializeError { backend: ApiBackend::Browser, error: why.as_string().unwrap_or_default() }),
        };

        let (group_id, device_id) = match index {
            CameraIndex::Index(i) => {
                return Err(NokhwaError::OpenDeviceError(i.to_string(), "Invalid Index".to_string()))
            },
            CameraIndex::String(s) => {
                match s.split_once(" ") {
                    Some((g, d)) => (g.to_string(), d.to_string()),
                    None => return Err(NokhwaError::OpenDeviceError(s.to_string(), "Invalid Index".to_string())) ,
                }
            },
        };

        let mut device_info = None;
        for enumed_dev in resolve_to::<Array>(media_devices.enumerate_devices()).await? {
            let dev_info = unsafe { 
                checked_js_cast::<MediaDeviceInfo>(enumed_dev)?
             };
             if dev_info.device_id() == device_id && dev_info.group_id() == group_id {
                device_info = Some(dev_info)
             }
        };

        let info = match device_info {
            Some(v) => {
                CameraInfo::new(&v.label(), v.kind(), &v.device_id(), index)
            }
            None => return Err(NokhwaError::OpenDeviceError(index.to_string(), "failed to find MediaDeviceInfo".to_string())),
        };

        let mut constraint = MediaStreamConstraints::new();
        let mut video_constraint = MediaTrackConstraints::new();

        video_constraint = video_constraint.device_id(&JsValue::from_str(&device_id));

        match camera_fmt {
            FormatRequest::Closest { resolution, frame_rate, frame_format } => {
                let (_aspect_ratio, width, height) = match resolution {
                    Some(res_range) => (
                        ConstrainedDouble {
                            min: None,
                            ideal: None,
                            max: None,
                            exact: Some(res_range.preferred().aspect_ratio()),
                        },
                        ConstrainedDouble {
                            min: res_range.minimum().map(|x| x.width() as f64),
                            ideal: Some(res_range.preferred().width() as f64),
                            max: res_range.maximum().map(|x| x.width() as f64),
                            exact: None,
                        },
                        ConstrainedDouble {
                            min: res_range.minimum().map(|x: Resolution| x.height() as f64),
                            ideal: Some(res_range.preferred().width() as f64),
                            max: res_range.maximum().map(|x| x.height() as f64),
                            exact: None,
                        },
                    ),
                    None => (
                        ConstrainedDouble::default(), ConstrainedDouble::default(), ConstrainedDouble::default()
                    ),
                };

                let frame_rate = match frame_rate {
                    Some(f) => ConstrainedDouble {
                        min: f.minimum().map(|x| x.frame_rate() as f64),
                        ideal: Some(f.preferred().frame_rate() as f64),
                        max: f.maximum().map(|x| x.frame_rate() as f64),
                        exact: None,
                    },
                    None => ConstrainedDouble::default(),
                };

                video_constraint = video_constraint.width(width.into());
                video_constraint = video_constraint.height(height.into());
                video_constraint = video_constraint.frame_rate(frame_rate.into());
            }
            FormatRequest::HighestFrameRate { frame_rate, frame_format } => {
                let frame_rate = match frame_rate {
                    Some(f) => ConstrainedDouble {
                        min: f.minimum().map(|x| x.frame_rate() as f64),
                        ideal: Some(f.preferred().frame_rate() as f64),
                        max: f.maximum().map(|x| x.frame_rate() as f64),
                        exact: None,
                    },
                    None => ConstrainedDouble::default(),
                };

                video_constraint = video_constraint.frame_rate(frame_rate.into());
            }
            FormatRequest::HighestResolution { resolution, frame_format } => {
                let (_aspect_ratio, width, height) = match resolution {
                    Some(res_range) => (
                        ConstrainedDouble {
                            min: None,
                            ideal: None,
                            max: None,
                            exact: Some(res_range.preferred().aspect_ratio()),
                        },
                        ConstrainedDouble {
                            min: res_range.minimum().map(|x| x.width() as f64),
                            ideal: Some(res_range.preferred().width() as f64),
                            max: res_range.maximum().map(|x| x.width() as f64),
                            exact: None,
                        },
                        ConstrainedDouble {
                            min: res_range.minimum().map(|x: Resolution| x.height() as f64),
                            ideal: Some(res_range.preferred().width() as f64),
                            max: res_range.maximum().map(|x| x.height() as f64),
                            exact: None,
                        },
                    ),
                    None => (
                        ConstrainedDouble::default(), ConstrainedDouble::default(), ConstrainedDouble::default()
                    ),
                };

                video_constraint = video_constraint.width(width.into());
                video_constraint = video_constraint.height(height.into());
            }
            FormatRequest::Exact { resolution, frame_rate, frame_format } => {
                let (_aspect_ratio, width, height) = match resolution {
                    Some(res_range) => (
                        ConstrainedDouble {
                            min: None,
                            ideal: None,
                            max: None,
                            exact: Some(res_range.preferred().aspect_ratio()),
                        },
                        ConstrainedDouble {
                            min: None,
                            ideal: None,
                            max: None,
                            exact: Some(res_range.preferred().width() as f64),
                        },
                        ConstrainedDouble {
                            min: None,
                            ideal: None,
                            max: None,
                            exact: Some(res_range.preferred().width() as f64),
                        },
                    ),
                    None => (
                        ConstrainedDouble::default(), ConstrainedDouble::default(), ConstrainedDouble::default()
                    ),
                };

                let frame_rate: ConstrainedDouble = match frame_rate {
                    Some(f) => ConstrainedDouble {
                        min: None,
                        ideal: None,
                        max: None,
                        exact: Some(f.preferred().frame_rate() as f64),
                    },
                    None => ConstrainedDouble::default(),
                };

                video_constraint = video_constraint.width(width.into());
                video_constraint = video_constraint.height(height.into());
                video_constraint = video_constraint.frame_rate(frame_rate.into());
            }
        }

        constraint = constraint.video(&video_constraint);

        let media_stream: MediaStream = resolve_to(media_devices.get_user_media_with_constraints(&constraint)).await?;

        let mut video_track: MediaStreamTrack = checked_js_cast(media_stream.get_video_tracks().get(0))?;

        resolve_to::<()>(video_track.apply_constraints_with_constraints(&video_constraint)).await?;

        let track_settings = video_track.get_settings();
        let track_settings_map = make_jsobj_map(track_settings)?;

        let format = {
            let frame_rate = track_settings_map.get("frameRate").as_f64().ok_or(NokhwaError::ConversionError("failed to get frameRate as f64".to_string()))?;
            let resolution_width = u32::from(track_settings_map.get("width").as_f64().ok_or(NokhwaError::ConversionError("failed to get width as f64".to_string()))?);
            let resolution_length = u32::from(track_settings_map.get("length").as_f64().ok_or(NokhwaError::ConversionError("failed to get length as f64".to_string()))?);
            CameraFormat::new(Resolution::new(resolution_width, resolution_length), FrameFormat::Rgb8, frame_rate)
        };

        Ok(BrowserCaptureDevice { info, media_devices, media_stream, group_id, device_id, format })
    }

}

impl CaptureTrait for BrowserCaptureDevice {
    fn backend(&self) -> ApiBackend {
        ApiBackend::Browser
    }

    fn camera_info(&self) -> &CameraInfo {
        &self.info
    }

    fn refresh_camera_format(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    fn camera_format(&self) -> Option<CameraFormat> {
        todo!()
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        todo!()
    }

    fn compatible_list_by_resolution(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<FrameRate>>, NokhwaError> {
        todo!()
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        todo!()
    }

    fn resolution(&self) -> Option<Resolution> {
        todo!()
    }

    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        todo!()
    }

    fn frame_rate(&self) -> Option<u32> {
        todo!()
    }

    fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        todo!()
    }

    fn frame_format(&self) -> FrameFormat {
        todo!()
    }

    fn set_frame_format(&mut self, fourcc: FrameFormat)
        -> Result<(), NokhwaError> {
        todo!()
    }

    fn camera_control(&self, control: KnownCameraControl) -> Result<CameraControl, NokhwaError> {
        todo!()
    }

    fn camera_controls(&self) -> Result<Vec<CameraControl>, NokhwaError> {
        todo!()
    }

    fn set_camera_control(
        &mut self,
        id: KnownCameraControl,
        value: ControlValueSetter,
    ) -> Result<(), NokhwaError> {
        todo!()
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    fn is_stream_open(&self) -> bool {
        todo!()
    }

    fn frame(&mut self) -> Result<Buffer, NokhwaError> {
        todo!()
    }

    fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        todo!()
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }
}


#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl AsyncCaptureTrait for BrowserCaptureDevice {
    async fn refresh_camera_format_async(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn set_camera_format_async(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn compatible_list_by_resolution_async(&mut self, fourcc: FrameFormat) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        todo!()
    }

    async fn set_resolution_async(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn set_frame_rate_async(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn set_frame_format_async(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn set_camera_control_async(&mut self, id: KnownCameraControl, value: ControlValueSetter) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn open_stream_async(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn frame_async(&mut self) -> Result<Buffer, NokhwaError> {
        todo!()
    }

    async fn frame_raw_async(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        todo!()
    }

    async fn stop_stream_async(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }
}


#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl AsyncOpenCaptureTrait for AsyncCaptureTrait {
    async fn open(index: &CameraIndex, camera_fmt: FormatRequest) -> Result<Self, NokhwaError> where Self: Sized {
        Self::open(index, camera_fmt)
    }
}
