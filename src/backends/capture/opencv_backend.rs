use crate::{CameraFormat, CameraInfo, FrameFormat, NokhwaError, Resolution};
use opencv::{prelude::Detail_NoBundleAdjusterTrait, videoio::{CAP_ANY, CAP_AVFOUNDATION, CAP_MSMF, CAP_PROP_FOURCC, CAP_V4L2, VideoCapture, VideoCaptureProperties, VideoCaptureTrait, VideoWriter}};
use std::{convert::TryInto, ops::Deref};

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
        let api = match api_pref {
            Some(a) => a as i32,
            None => get_api_pref_int() as i32,
        };
        let mut video_capture = match location {
            CameraIndexType::Index(idx) => match VideoCapture::new(idx as i32, api) {
                Ok(vc) => vc,
                Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
            },
            CameraIndexType::IPCamera(ip) => match VideoCapture::from_file(ip.deref(), api) {
                Ok(vc) => vc,
                Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
            },
        };
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
fn set_properties(
    vc: &mut VideoCapture,
    res: Resolution,
    fps: u32,
    frame_fmt: FrameFormat
) -> Result<(), NokhwaError> {
    set_property_fourcc(vc, frame_fmt)?;
    set_property_res(vc, res)?;
    set_property_fps(vc, fps)?;
    Ok(())
}

fn set_property_res(
    vc: &mut VideoCapture,
    res: Resolution,
) -> Result<(), NokhwaError> {
    match vc.set(
        VideoCaptureProperties::CAP_PROP_FRAME_HEIGHT as i32,
        f64::from(res.height()),
    ) {
        Ok(r) => {
            if !r {
                return Err(NokhwaError::CouldntSetProperty { property: "Resolution Height".to_string(), value: res.height().to_string(), error: "OpenCV bool assert failure".to_string() })
            }
        }
        Err(why) => {
            return Err(NokhwaError::CouldntSetProperty { property: "Resolution Height".to_string(), value: res.height().to_string(), error: why.to_string() })
        }
    }

    match vc.set(
        VideoCaptureProperties::CAP_PROP_FRAME_WIDTH as i32,
        f64::from(res.width()),
    ) {
        Ok(r) => {
            if !r {
                return Err(NokhwaError::CouldntSetProperty { property: "Resolution Width".to_string(), value: res.width().to_string(), error: "OpenCV bool assert failure".to_string() })
            }
        }
        Err(why) => {
            return Err(NokhwaError::CouldntSetProperty { property: "Resolution Width".to_string(), value: res.width().to_string(), error: why.to_string() })
        }
    }

    Ok(())
}

fn set_property_fps(vc: &mut VideoCapture, fps: u32) -> Result<(), NokhwaError> {
    match vc.set(VideoCaptureProperties::CAP_PROP_FPS as i32, f64::from(fps)) {
        Ok(r) => {
            if !r {
                return Err(NokhwaError::CouldntSetProperty { property: "Framerate".to_string(), value: fps.to_string(), error: "OpenCV bool assert failure".to_string() })
            }
        }
        Err(why) => {
            return Err(NokhwaError::CouldntSetProperty { property: "Framerate".to_string(), value: fps.to_string(), error: why.to_string() })
        }
    }
    Ok(())
}

fn set_property_fourcc(vc: &mut VideoCapture, frame_fmt: FrameFormat) -> Result<(), NokhwaError> {
    let fourcc = match frame_fmt {
        FrameFormat::MJPEG => {
            f64::from(
                match VideoWriter::fourcc('M' as i8, 'J' as i8, 'P' as i8, 'G' as i8) {
                    Ok(fcc) => fcc,
                    Err(why) => return Err(NokhwaError::CouldntSetProperty { property: "FourCC".to_string(), value: "MJPG".to_string(), error: why.to_string() })
                }
            )
        }
        FrameFormat::YUYV => {
            f64::from(
                match VideoWriter::fourcc('Y' as i8, 'U' as i8, 'Y' as i8, 'V' as i8) {
                    Ok(fcc) => fcc,
                    Err(why) => return Err(NokhwaError::CouldntSetProperty { property: "FourCC".to_string(), value: "YUYV".to_string(), error: why.to_string() })
                }
            )
        }
    };

    match vc.set(
        CAP_PROP_FOURCC as i32,
        f64::from(VideoWriter::fourcc('M' as i8, 'J' as i8, 'P' as i8, 'G' as i8).unwrap()),
    ) {
        Ok(r) => {
            if !r {
                return Err(NokhwaError::CouldntSetProperty { property: "FourCC".to_string(), value: format!("FrameFormat: {}, OpenCV FourCC {}", frame_fmt, fourcc), error: "OpenCV bool assert failure".to_string() })
            }
        }
        Err(why) => {
            return Err(NokhwaError::CouldntSetProperty { property: "FourCC".to_string(), value: format!("FrameFormat: {}, OpenCV FourCC {}", frame_fmt, fourcc), error: why.to_string() })
        }
    }
    Ok(())
}
