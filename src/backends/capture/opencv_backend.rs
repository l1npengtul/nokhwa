use crate::{CameraFormat, CameraInfo, NokhwaError};
use opencv::videoio::{VideoCapture, CAP_ANY, CAP_AVFOUNDATION, CAP_MSMF, CAP_V4L2};

/// The backend struct that interfaces with `OpenCV`. Note that an `opencv` matching the version that this was either compiled on must be present on the user's machine. (usually 4.5.2 or greater)
/// For more information, please see [`opencv-rust`](https://github.com/twistedfall/opencv-rust) and [`OpenCV VideoCapture Docs`](https://docs.opencv.org/4.5.2/d8/dfe/classcv_1_1VideoCapture.html).
///
/// To see what this does, please see [`CaptureBackendTrait`]
/// # Quirks
///  - This is a **cross-platform** backend. This means that it will work on most platforms given that `OpenCV` is present.
///  - This backend can also do IP Camera input.
///  - The backend's backend will default to system level APIs on Linux(V4L2), Mac(AVFoundation), and Windows(Media Foundation). Otherwise, it will decide for itself.
///  - If the [`OpenCvCaptureDevice`] is initialized as a IPCamera, the [`CameraFormat`]'s `index` value will be [`u32::MAX`](std::u32::MAX) (4294967295).
pub struct OpenCvCaptureDevice {
    camera_format: CameraFormat,
    camera_location: CameraIndexType,
    camera_info: CameraInfo,
    video_capture: VideoCapture,
}

impl OpenCvCaptureDevice {
    pub fn new(
        location: CameraIndexType,
        camera_format: Option<CameraFormat>,
        api_pref: Option<u32>,
    ) -> Result<Self, NokhwaError> {
    }
}

/// The `OpenCV` backend supports both native cameras and IP Cameras, so this is an enum to differentiate them
/// The IPCamera's string follows the pattern
/// ```.ignore
/// <protocol>://<IP>:<port>/
/// ```
/// but please consult the manufacturer's specification for more details.
/// The index is a standard webcam index.
pub enum CameraIndexType {
    Index(u32),
    IPCamera(String),
}

fn get_api_pref_int() -> u32 {
    match std::env::consts::OS {
        "linux" => CAP_V4L2 as u32,
        "windows" => CAP_MSMF as u32,
        "mac" => CAP_AVFOUNDATION as u32,
        &_ => CAP_ANY as u32,
    }
}
