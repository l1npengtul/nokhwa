#[cfg(feature = "input_uvc")]
use crate::backends::capture::UVCCaptureDevice;
#[cfg(feature = "input_v4l")]
use crate::backends::capture::V4LCaptureDevice;
use crate::{
    CameraFormat, CameraInfo, CaptureAPIBackend, CaptureBackendTrait, FrameFormat, NokhwaError,
    Resolution,
};
use image::{ImageBuffer, Rgb};
use std::{cell::RefCell, collections::HashMap};

/// The main `Camera` struct. This is the struct that abstracts over all the backends, providing a simplified interface for use.
/// For more details, please refer to [`CaptureBackendTrait`]
pub struct Camera {
    idx: usize,
    backend: RefCell<Box<dyn CaptureBackendTrait>>,
    backend_api: CaptureAPIBackend,
}

impl Camera {
    /// Create a new camera from an `index`, `format`, and `backend`. `format` can be `None`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input_v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn new(
        index: usize,
        format: Option<CameraFormat>,
        backend: CaptureAPIBackend,
    ) -> Result<Self, NokhwaError> {
        let platform = std::env::consts::OS;
        let use_backend = match backend {
            CaptureAPIBackend::AUTO => {
                let mut cap = CaptureAPIBackend::AUTO;
                if cfg!(feature = "input_v4l") && platform == "linux" {
                    cap = CaptureAPIBackend::V4L2
                } else if cfg!(feature = "input_uvc") {
                    cap = CaptureAPIBackend::UVC;
                }
                if cap == CaptureAPIBackend::AUTO {
                    return Err(NokhwaError::NotImplemented(
                        "Platform requirements not satisfied.".to_string(),
                    ));
                }
                cap
            }
            CaptureAPIBackend::V4L2 => {
                if !(cfg!(feature = "input_v4l") && platform == "linux") {
                    return Err(NokhwaError::NotImplemented(
                        "V4L Requirements: Linux and `input_v4l`.".to_string(),
                    ));
                }
                CaptureAPIBackend::V4L2
            }
            CaptureAPIBackend::UVC => {
                if !(cfg!(feature = "input_uvc")) {
                    return Err(NokhwaError::NotImplemented(
                        "UVC Requirements: `input_uvc`.".to_string(),
                    ));
                }
                CaptureAPIBackend::UVC
            }
            _ => return Err(NokhwaError::NotImplemented(backend.to_string())),
        };

        let capture_backend = match use_backend {
            CaptureAPIBackend::V4L2 => match init_v4l(index, format) {
                Some(capture) => match capture {
                    Ok(cap_back) => cap_back,
                    Err(why) => return Err(why),
                },
                None => {
                    return Err(NokhwaError::NotImplemented(
                        "Platform requirements not satisfied.".to_string(),
                    ));
                }
            },
            CaptureAPIBackend::UVC => match init_uvc(index, format) {
                Some(capture) => match capture {
                    Ok(cap_back) => cap_back,
                    Err(why) => return Err(why),
                },
                None => {
                    return Err(NokhwaError::NotImplemented(
                        "Platform requirements not satisfied.".to_string(),
                    ));
                }
            },
            _ => {
                return Err(NokhwaError::NotImplemented(
                    "Platform requirements not satisfied.".to_string(),
                ));
            }
        };

        Ok(Camera {
            idx: index,
            backend: RefCell::new(capture_backend),
            backend_api: use_backend,
        })
    }

    /// Create a new `Camera` from raw values.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input_v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn new_with(
        index: usize,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
        backend: CaptureAPIBackend,
    ) -> Result<Self, NokhwaError> {
        let camera_format = CameraFormat::new_from(width, height, fourcc, fps);
        Camera::new(index, Some(camera_format), backend)
    }

    /// Gets the current Camera's index.
    pub fn index(&self) -> usize {
        self.idx
    }

    /// Sets the current Camera's index. Note that this re-initializes the camera.
    /// # Errors
    /// The Backend may fail to initialize.
    pub fn set_index(self, new_idx: usize) -> Result<Self, NokhwaError> {
        self.backend.borrow_mut().stop_stream()?;
        let new_camera_format = self.backend.borrow().get_camera_format();
        Camera::new(new_idx, Some(new_camera_format), self.backend_api)
    }

    /// Gets the current Camera's backend
    pub fn backend(&self) -> CaptureAPIBackend {
        self.backend_api
    }

    /// Sets the current Camera's backend. Note that this re-initializes the camera.
    /// # Errors
    /// The new backend may not exist or may fail to initialize the new camera.
    pub fn set_backend(self, new_backend: CaptureAPIBackend) -> Result<Self, NokhwaError> {
        self.backend.borrow_mut().stop_stream()?;
        let new_camera_format = self.backend.borrow().get_camera_format();
        Camera::new(self.idx, Some(new_camera_format), new_backend)
    }

    /// Gets the camera information such as Name and Index as a [`CameraInfo`].
    pub fn get_info(&self) -> CameraInfo {
        self.backend.borrow().get_info()
    }
    /// Gets the current [`CameraFormat`].
    pub fn get_camera_format(&self) -> CameraFormat {
        self.backend.borrow().get_camera_format()
    }
    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    pub fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().set_camera_format(new_fmt)
    }
    /// A hashmap of [`Resolution`]s mapped to framerates
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a Unsupported Operation ([`NokhwaError::UnsupportedOperation`]).
    pub fn get_compatible_list_by_resolution(
        &self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        self.backend
            .borrow()
            .get_compatible_list_by_resolution(fourcc)
    }
    /// A Vector of compatible [`FrameFormat`]s.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a Unsupported Operation ([`NokhwaError::UnsupportedOperation`]).
    pub fn get_compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        self.backend.borrow_mut().get_compatible_fourcc()
    }
    /// Gets the current camera resolution (See: [`Resolution`], [`CameraFormat`]).
    pub fn get_resolution(&self) -> Resolution {
        self.backend.borrow().get_resolution()
    }
    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    pub fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().set_resolution(new_res)
    }
    /// Gets the current camera framerate (See: [`CameraFormat`]).
    pub fn get_framerate(&self) -> u32 {
        self.backend.borrow().get_framerate()
    }
    /// Will set the current framerate
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new framerate, this will return an error.
    pub fn set_framerate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().set_framerate(new_fps)
    }
    /// Gets the current camera's frame format (See: [`FrameFormat`], [`CameraFormat`]).
    pub fn get_frameformat(&self) -> FrameFormat {
        self.backend.borrow().get_frameformat()
    }
    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame foramt, this will return an error.
    pub fn set_frameformat(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().set_frameformat(fourcc)
    }
    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`get_frame()`](CaptureBackendTrait::get_frame()) before you call [`open_stream()`](CaptureBackendTrait::open_stream()).
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    pub fn open_stream(&mut self) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().open_stream()
    }
    /// Checks if stream if open. If it is, it will return true.
    pub fn is_stream_open(&self) -> bool {
        self.backend.borrow().is_stream_open()
    }
    /// Will get a frame from the camera as a Raw RGB image buffer. Depending on the backend, if you have not called [`open_stream()`](CaptureBackendTrait::open_stream()) before you called this,
    /// it will either return an error.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), the decoding fails (e.g. MJPEG -> u8), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet,
    /// this will error.
    pub fn get_frame(&self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        self.backend.borrow_mut().get_frame()
    }
    /// Will get a frame from the camera **without** any processing applied, meaning you will usually get a frame you need to decode yourself.
    /// # Errors
    /// If the backend fails to get the frame (e.g. already taken, busy, doesn't exist anymore), or [`open_stream()`](CaptureBackendTrait::open_stream()) has not been called yet, this will error.
    pub fn get_frame_raw(&self) -> Result<Vec<u8>, NokhwaError> {
        self.backend.borrow_mut().get_frame_raw()
    }
    /// Will drop the stream.
    /// # Errors
    /// Please check the `Quirks` section of each backend.
    pub fn stop_stream(&self) -> Result<(), NokhwaError> {
        self.backend.borrow_mut().stop_stream()
    }
}

#[cfg(feature = "input_v4l")]
#[allow(clippy::unnecessary_wraps)]
fn init_v4l(
    idx: usize,
    setting: Option<CameraFormat>,
) -> Option<Result<Box<dyn CaptureBackendTrait>, NokhwaError>> {
    match V4LCaptureDevice::new(idx, setting) {
        Ok(cap) => Some(Ok(Box::new(cap))),
        Err(why) => Some(Err(why)),
    }
}

#[cfg(not(feature = "input_v4l"))]
#[allow(clippy::unnecessary_wraps)]
fn init_v4l(
    idx: usize,
    setting: Option<CameraFormat>,
) -> Option<Result<Box<dyn CaptureBackendTrait>, NokhwaError>> {
    None
}

#[cfg(feature = "uvc")]
#[allow(clippy::unnecessary_wraps)]
fn init_uvc(
    idx: usize,
    setting: Option<CameraFormat>,
) -> Option<Result<Box<dyn CaptureBackendTrait>, NokhwaError>> {
    match UVCCaptureDevice::create(idx, setting) {
        Ok(cap) => Some(Ok(Box::new(cap))),
        Err(why) => Some(Err(why)),
    }
}

#[cfg(not(feature = "input_uvc"))]
#[allow(clippy::unnecessary_wraps)]
fn init_uvc(
    idx: usize,
    setting: Option<CameraFormat>,
) -> Option<Result<Box<dyn CaptureBackendTrait>, NokhwaError>> {
    None
}
