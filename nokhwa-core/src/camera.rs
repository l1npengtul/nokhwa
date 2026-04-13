use crate::control::{Control, ControlId, ControlValue};
use crate::error::NokhwaError;
use crate::frame_buffer::FrameBuffer;
use crate::stream::StreamTrait;
use crate::types::CameraFormat;

pub trait CameraTrait {
    type Stream: StreamTrait;

    fn enumerate_formats(&self) -> Result<Vec<CameraFormat>, NokhwaError>;

    fn controls(&self) -> Result<Vec<Control>, NokhwaError>;

    fn control_value(&self, id: ControlId) -> Result<ControlValue, NokhwaError>;

    fn set_control(&self, id: ControlId, value: ControlValue) -> Result<(), NokhwaError>;

    // fn open_stream<FrameCallback, ErrorCallback>(
    //     &mut self,
    //     camera_format: CameraFormat,
    //     frame_callback: FrameCallback,
    //     error_callback: ErrorCallback,
    // ) -> Result<Self::Stream, NokhwaError>
    // where
    //     FrameCallback: FnMut(FrameBuffer<'_>) + Send + 'static,
    //     ErrorCallback: FnMut(StreamEvent) + Send + 'static;

    fn open_stream<FrameCallback, ErrorCallback>(
        &mut self,
        camera_format: CameraFormat,
        frame_callback: FrameCallback,
        error_callback: ErrorCallback,
    ) -> Result<Self::Stream, NokhwaError>
    where
        FrameCallback: FnMut(FrameBuffer<'_>) + Send + 'static,
        ErrorCallback: FnMut(NokhwaError) + Send + 'static;
}

// #[cfg(feature = "async")]
// #[cfg_attr(feature = "async", async_trait::async_trait)]
// pub trait AsyncSetting {
//     async fn enumerate_formats_async(&self) -> Result<Vec<CameraFormat>, NokhwaError>;

//     async fn enumerate_resolution_and_frame_rates_async(
//         &self,
//         frame_format: FrameFormat,
//     ) -> Result<HashMap<Resolution, Vec<FrameRate>>, NokhwaError>;

//     async fn set_format_async(&self, camera_format: CameraFormat) -> Result<(), NokhwaError>;

//     async fn set_property_async(
//         &mut self,
//         property: &ControlId,
//         value: ControlValue,
//     ) -> Result<(), NokhwaError>;
// }

// #[cfg(feature = "async")]
// #[cfg_attr(feature = "async", async_trait::async_trait)]
// pub trait AsyncStream {
//     async fn open_stream_async<'a>(
//         &mut self,
//         stream_configuration: Option<StreamConfiguration>,
//     ) -> Result<StreamHandle<'a>, NokhwaError>;

//     async fn close_stream_async(&mut self) -> Result<(), NokhwaError>;
// }

// // #[cfg(feature = "async")]
// // pub trait AsyncCamera: Camera + AsyncSetting + AsyncStream {}
