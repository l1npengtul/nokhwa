use async_trait::async_trait;
use js_sys::{Array, Object};
use nokhwa_core::buffer::Buffer;
use nokhwa_core::error::NokhwaError;
use nokhwa_core::format_filter::FormatFilter;
use nokhwa_core::frame_format::{FrameFormat, SourceFrameFormat};
use nokhwa_core::traits::{AsyncCaptureTrait, Backend, CaptureTrait};
use nokhwa_core::types::{
    ApiBackend, CameraControl, CameraFormat, CameraIndex, CameraInfo, ControlValueSetter,
    KnownCameraControl, Resolution,
};
use wasm_bindgen_futures::JsFuture;
use std::borrow::Cow;
use std::collections::{HashMap, BTreeMap, HashSet};
use std::future::Future;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, Document, Element, MediaDevices, Navigator, OffscreenCanvas, Window, MediaStream, MediaStreamConstraints, HtmlCanvasElement, MediaDeviceInfo, MediaDeviceKind, MediaStreamTrack,
};

macro_rules! jsv {
    ($value:expr) => {{
        JsValue::from($value)
    }};
}

macro_rules! obj {
    ($(($key:expr, $value:expr)),+ ) => {{
        use js_sys::{Map, Object};
        use wasm_bindgen::JsValue;

        let map = Map::new();
        $(
            map.set(&jsv!($key), &jsv!($value));
        )+
        Object::from(map)
    }};
    ($object:expr, $(($key:expr, $value:expr)),+ ) => {{
        use js_sys::{Map, Object};
        use wasm_bindgen::JsValue;

        let map = Map::new();
        $(
            map.set(&jsv!($key), &jsv!($value));
        )+
        let o = Object::from(map);
        Object::assign(&$object, &o)
    }};
}

fn window() -> Result<Window, NokhwaError> {
    match web_sys::window() {
        Some(win) => Ok(win),
        None => Err(NokhwaError::StructureError {
            structure: "web_sys Window".to_string(),
            error: "None".to_string(),
        }),
    }
}

fn media_devices(navigator: &Navigator) -> Result<MediaDevices, NokhwaError> {
    match navigator.media_devices() {
        Ok(media) => Ok(media),
        Err(why) => Err(NokhwaError::StructureError {
            structure: "MediaDevices".to_string(),
            error: format!("{why:?}"),
        }),
    }
}

fn document(window: &Window) -> Result<Document, NokhwaError> {
    match window.document() {
        Some(doc) => Ok(doc),
        None => Err(NokhwaError::StructureError {
            structure: "web_sys Document".to_string(),
            error: "None".to_string(),
        }),
    }
}

fn document_select_elem(doc: &Document, element: &str) -> Result<Element, NokhwaError> {
    match doc.get_element_by_id(element) {
        Some(elem) => Ok(elem),
        None => {
            return Err(NokhwaError::StructureError {
                structure: format!("Document {element}"),
                error: "None".to_string(),
            })
        }
    }
}

fn element_cast<T: JsCast, U: JsCast>(from: T, name: &str) -> Result<U, NokhwaError> {
    if !from.has_type::<U>() {
        return Err(NokhwaError::StructureError {
            structure: name.to_string(),
            error: "Cannot Cast - No Subtype".to_string(),
        });
    }

    let casted = match from.dyn_into::<U>() {
        Ok(cast) => cast,
        Err(_) => {
            return Err(NokhwaError::StructureError {
                structure: name.to_string(),
                error: "Casting Error".to_string(),
            });
        }
    };
    Ok(casted)
}

fn element_cast_ref<'a, T: JsCast, U: JsCast>(
    from: &'a T,
    name: &'a str,
) -> Result<&'a U, NokhwaError> {
    if !from.has_type::<U>() {
        return Err(NokhwaError::StructureError {
            structure: name.to_string(),
            error: "Cannot Cast - No Subtype".to_string(),
        });
    }

    match from.dyn_ref::<U>() {
        Some(v_e) => Ok(v_e),
        None => Err(NokhwaError::StructureError {
            structure: name.to_string(),
            error: "Cannot Cast".to_string(),
        }),
    }
}

fn create_element(doc: &Document, element: &str) -> Result<Element, NokhwaError> {
    match Document::create_element(doc, element) {
        // ???? thank you intellij
        Ok(new_element) => Ok(new_element),
        Err(why) => Err(NokhwaError::StructureError {
            structure: "Document Video Element".to_string(),
            error: format!("{:?}", why.as_string()),
        }),
    }
}

fn set_autoplay_inline(element: &Element) -> Result<(), NokhwaError> {
    if let Err(why) = element.set_attribute("autoplay", "autoplay") {
        return Err(NokhwaError::SetPropertyError {
            property: "Video-autoplay".to_string(),
            value: "autoplay".to_string(),
            error: format!("{why:?}"),
        });
    }

    if let Err(why) = element.set_attribute("playsinline", "playsinline") {
        return Err(NokhwaError::SetPropertyError {
            property: "Video-playsinline".to_string(),
            value: "playsinline".to_string(),
            error: format!("{why:?}"),
        });
    }

    Ok(())
}

fn media_devices(navigator: &Navigator) -> Result<MediaDevices, NokhwaError> {
    match navigator.media_devices() {
        Ok(media) => Ok(media),
        Err(why) => Err(NokhwaError::StructureError {
            structure: "MediaDevices".to_string(),
            error: format!("{why:?}"),
        }),
    }
}

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
enum JSCameraFacingMode {
    Any,
    Environment,
    User,
    Left,
    Right,
}

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
enum JSCameraResizeMode {
    Any,
    None,
    CropAndScale,
}

enum CanvasType {
    OffScreen(OffscreenCanvas),
    HtmlCanvas(HtmlCanvasElement),
}

/// Quirks:
/// - Regular [`CaptureTrait`] will block, something that is undesired in web applications. Use [`AsyncCaptureTrait`]
/// - [REQUIRES AN UP-TO-DATE BROWSER DUE TO USE OF OFFSCREEN CANVAS.](https://caniuse.com/?search=OffscreenCanvas)
/// - [`SourceFrameFormat`]/[`FrameFormat`] does NOT apply, due to browser non-support. All returned streams will be RGB (autodecoded by browser).
/// - Custom Controls
///     - aspectRatio: 8
///     - facingMode: 16
///     - resizeMode: 32
///     - attachedCanvasMode: 64
pub struct BrowserCamera {
    index: CameraIndex,
    info: CameraInfo,
    format: CameraFormat,
    media_stream: MediaStream,
    init: bool,
    custom_controls: HashMap<u128, CameraControl>,
    controls: HashMap<KnownCameraControl, CameraControl>,
    supported_controls: HashSet<KnownCameraControl>,
    cavnas: Option<CanvasType>,
    context: Option<CanvasRenderingContext2d>,
}

impl BrowserCamera {
    pub fn new(index: &CameraIndex) -> Result<BrowserCamera, NokhwaError> {
        wasm_rs_async_executor::single_threaded::block_on(Self::new_async(index))
    }

    pub async fn new_async(index: &CameraIndex) -> Result<BrowserCamera, NokhwaError> {
        let window = window()?;
        let media_devices = media_devices(&window.navigator())?;

        let stream: MediaStream = match media_devices.get_user_media_with_constraints(&constraints)
        {
            Ok(promise) => {
                let future = JsFuture::from(promise);
                match future.await {
                    Ok(stream) => {
                        let media_stream: MediaStream = MediaStream::from(stream);
                        media_stream
                    }
                    Err(why) => {
                        return Err(NokhwaError::OpenDeviceError(
                            "MediaDevicesGetUserMediaJsFuture".to_string(), format!("{why:?}"),
                        ))
                    }
                }
            }
            Err(why) => {
                return Err(NokhwaError::OpenDeviceError(
                    "MediaDevicesGetUserMedia".to_string(), format!("{why:?}"),
                ))
            }
        };

        let media_info = match media_devices.enumerate_devices() {
            Ok(i) => {
                let future = JsFuture::from(promise);
                match future.await {
                    Ok(devs) => {
                        let arr = Array::from(&devs);
                        match index {
                            CameraIndex::Index(i) => {
                                let dr = arr.get(i as u32);

                                if dr == JsValue::UNDEFINED {
                                    return Err(NokhwaError::StructureError { structure: "MediaDeviceInfo".to_string(), error: "undefined".to_string() })
                                }

                                MediaDeviceInfo::from(dr)
                            }
                            CameraIndex::String(s) => {
                                match arr.iter().map(MediaDeviceInfo::from)
                                .filter(|mdi| {
                                    mdi.device_id() == s
                                }).nth(0) {
                                    Some(i) => i,
                                    None => return Err(NokhwaError::StructureError { structure: "MediaDeviceInfo".to_string(), error: "no id".to_string() })

                                }
                            }
                        }
                    }
                    Err(why) => {
                        return Err(NokhwaError::StructureError { structure: "MediaDeviceInfo Enumerate Devices Promise".to_string(), error: format!("{why:?}") })
                    }
                }
            }
            Err(why) => {
                return Err(NokhwaError::GetPropertyError { property: "MediaDeviceInfo".to_string(), error: format!("{why:?}") })
            },
        };

        let info = CameraInfo {
            human_name: media_info.label(),
            description: media_info.kind().to_string(),
            misc: format!("{}:{}", media_info.group_id().to_string(), media_info.device_id().to_string()),
            index: index.clone(),
        };

        Ok(BrowserCamera { index:  index.clone(), info, format: CameraFormat::default(), init: false, cavnas: None, context: None, media_stream: stream, controls: HashMap::new(), custom_controls: HashMap::new(), supported_controls: HashSet::new() })
    }

    async fn measure_controls(&mut self) -> Result<(), NokhwaError> {
        let tracks = self.media_stream.get_video_tracks().iter().nth(0).ok_or(NokhwaError::GetPropertyError { property: "VideoTrack".to_string(), error: "Not Found".to_string() })?;
        let tracks = MediaStreamTrack::from(tracks);

        let constraints = {
            let c = tracks.get_constraints();
            Object::entries(&c.value_of()).iter().map(|field| {
                let element = Array::from(&field);
                let key = element.get(0);
                let value = element.get(1);
                (key.as_string().unwrap_or_default(), value)
            }).collect::<HashMap<String, JsValue>>()
        };
        
        let settings = {
            let c = tracks.get_settings();
            Object::entries(&c.value_of()).iter().map(|field| {
                let element = Array::from(&field);
                let key = element.get(0);
                let value = element.get(1);
                (key.as_string().unwrap_or_default(), value)
            }).collect::<HashMap<String, JsValue>>()
        };

        let window = window()?;
        let media_devices = media_devices(&window.navigator())?;
        let supported_constraints = Object::keys(&media_devices.get_supported_constraints())
            .iter()
            .map(|item| item.as_string().unwrap())
            .map(|constraint| {
                match constraint.as_str() {
                    "aspectRatio" => KnownCameraControl::Other(8),
                    "facingMode" => KnownCameraControl::Other(16),
                    "resizeMode" => KnownCameraControl::Other(32),
                    "frameRate" => KnownCameraControl::Other(64), // if you're looking through the source to see if i missed anything, ignore these
                    "width" => KnownCameraControl::Other(65),     // they're only here to make sure we support the basics
                    "height" => KnownCameraControl::Other(66),   
                    "deviceId" => KnownCameraControl::Other(67),
                    "groupId" => KnownCameraControl::Other(68),
                    _ => KnownCameraControl::Other(u128::MAX),
                }
            }).collect::<HashSet<KnownCameraControl>>();

        let 

        // set custom control properties
    }
}

impl Backend for BrowserCamera {
    const BACKEND: ApiBackend = ApiBackend::Browser;
}

impl CaptureTrait for BrowserCamera {
    fn init(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    fn init_with_format(&mut self, format: FormatFilter) -> Result<CameraFormat, NokhwaError> {
        todo!()
    }

    fn backend(&self) -> ApiBackend {
        todo!()
    }

    fn camera_info(&self) -> &CameraInfo {
        todo!()
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
        fourcc: SourceFrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        todo!()
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<SourceFrameFormat>, NokhwaError> {
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

    fn frame_format(&self) -> SourceFrameFormat {
        todo!()
    }

    fn set_frame_format(
        &mut self,
        fourcc: impl Into<SourceFrameFormat>,
    ) -> Result<(), NokhwaError> {
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

#[cfg(feature = "output-async")]
#[async_trait::async_trait]
impl AsyncCaptureTrait for BrowserCamera {
    async fn init(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn init_with_format(
        &mut self,
        format: FormatFilter,
    ) -> Result<CameraFormat, NokhwaError> {
        todo!()
    }

    async fn refresh_camera_format(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn set_frame_format(
        &mut self,
        fourcc: impl Into<SourceFrameFormat>,
    ) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn set_camera_control(
        &mut self,
        id: KnownCameraControl,
        value: ControlValueSetter,
    ) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn open_stream(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    async fn frame(&mut self) -> Result<Buffer, NokhwaError> {
        todo!()
    }

    async fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        todo!()
    }

    async fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }
}
