use async_trait::async_trait;
use nokhwa_core::buffer::Buffer;
use nokhwa_core::error::NokhwaError;
use nokhwa_core::format_filter::FormatFilter;
use nokhwa_core::frame_format::{FrameFormat, SourceFrameFormat};
use nokhwa_core::traits::{AsyncCaptureTrait, Backend, CaptureTrait};
use nokhwa_core::types::{
    ApiBackend, CameraControl, CameraFormat, CameraIndex, CameraInfo, ControlValueSetter,
    KnownCameraControl, Resolution,
};
use std::borrow::Cow;
use std::collections::HashMap;
use web_sys::{CanvasRenderingContext2d, OffscreenCanvas};

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

struct CustomControls {
    pub(crate) min_aspect_ratio: Option<f64>,
    pub(crate) aspect_ratio: f64,
    pub(crate) max_aspect_ratio: Option<f64>,
    pub(crate) facing_mode: JSCameraFacingMode,
    pub(crate) facing_mode_exact: bool,
    pub(crate) resize_mode: JSCameraResizeMode,
    pub(crate) resize_mode_exact: bool,
    pub(crate) device_id: String,
    pub(crate) device_id_exact: bool,
    pub(crate) group_id: String,
    pub(crate) group_id_exact: bool,
}

/// Quirks:
/// - Regular [`CaptureTrait`] will block. Use [``]
/// - [REQUIRES AN UP-TO-DATE BROWSER DUE TO USE OF OFFSCREEN CANVAS.](https://caniuse.com/?search=OffscreenCanvas)
/// - [`SourceFrameFormat`]/[`FrameFormat`] does NOT apply, due to browser non-support. All returned streams will be RGB.
pub struct BrowserCamera {
    index: CameraIndex,
    info: CameraInfo,
    format: CameraFormat,
    init: bool,
    controls: CustomControls,
    cavnas: Option<OffscreenCanvas>,
    context: Option<CanvasRenderingContext2d>,
}

impl BrowserCamera {
    pub fn new(index: &CameraIndex) -> Result<BrowserCamera> {}

    pub async fn new_async(index: &CameraIndex) -> Result<BrowserCamera> {
        //
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
