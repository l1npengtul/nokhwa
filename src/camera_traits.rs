use std::collections::HashMap;

use image::{ImageBuffer, Rgb};

use crate::{
    error::NokhwaError,
    utils::{CameraFormat, CameraInfo, FrameFormat, Resolution},
};

/// This trait is for any backend that allows you to grab and take frames from a camera.
pub trait CaptureBackendTrait {
    /// Gets the camera information such as Name and Index as a [`CameraInfo`].
    fn get_info(&self) -> CameraInfo;
    /// Assigns a sensible default to the backend's `CameraFormat`. Usually 640x480 @ 15 FPS + MJPEG.
    /// If there is already a value assigned to the `CameraFormat`, it will only be overwritten if `overwrite` is set to `true`.
    /// If false, this function will do nothing (NO-OP).
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    fn init_camera_format_default(&mut self, overwrite: bool) -> Result<(), NokhwaError>;
    /// Gets the current [`CameraFormat`]. Will return none if no format has been set yet.
    fn get_camera_format(&self) -> Option<CameraFormat>;
    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError>;
    /// Gets the current camera resolution (See: [`Resolution`]). Will return none if no resolution has been set yet.
    fn get_resolution(&self) -> Option<Resolution>;
    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError>;
    /// Gets the current camera framerate. Will return none if no framerate has been set yet.
    fn get_framerate(&self) -> Option<u32>;
    /// Will set the current framerate
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new framerate, this will return an error.
    fn set_framerate(&mut self, new_fps: u32) -> Result<(), NokhwaError>;
    /// Gets the current camera's frame format (See: [`FrameFormat`]). Will return none if no frame format has been set yet.
    fn get_frameformat(&self) -> Option<FrameFormat>;
    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame foramt, this will return an error.
    fn set_frameformat(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError>;
    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`get_frame()`](CaptureBackendTrait::get_frame()) before you call [`open_stream()`](CaptureBackendTrait::open_stream()).
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    fn open_stream(&mut self) -> Result<(), NokhwaError>;
    /// Checks if stream if open. If it is, it will return true.
    fn is_stream_open(&self) -> bool;
    /// Will get a frame from the camera as a Raw RGB image buffer. Depending on the backend, if you have not called [`open_stream()`](CaptureBackendTrait::open_stream()) before you called this,
    /// it will either return an error.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), the decoding fails (e.g. MJPEG -> u8), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet,
    /// this will error.
    fn get_frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError>;
    /// Will get a frame from the camera **without** any processing applied, meaning you will usually get a frame you need to decode yourself.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet, this will error.
    fn get_frame_raw(&mut self) -> Result<Vec<u8>, NokhwaError>;
    /// Will drop the stream.
    /// # Errors
    /// Please check the `Quirks` section of each backend.
    fn stop_stream(&mut self) -> Result<(), NokhwaError>;
}

/// This is for any backend that allows you to query a camera for its compatible resolutions/fourcc/framerates.
pub trait QueryBackendTrait: CaptureBackendTrait {
    /// A hashmap of [`Resolution`]s mapped to framerates
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed.
    fn get_compatible_list_by_resolution(
        &self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError>;
    /// Gets the supported camera formats.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed.
    fn get_resolution_list(&self, fourcc: FrameFormat) -> Result<Vec<Resolution>, NokhwaError>;
}

pub trait VirtualBackendTrait {}
