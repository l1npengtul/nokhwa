use crate::{
    tryinto_num, CameraFormat, CameraInfo, CaptureAPIBackend, CaptureBackendTrait, FrameFormat,
    NokhwaError, Resolution,
};
use image::{ImageBuffer, Rgb};
use opencv::videoio::{CAP_AVFOUNDATION, CAP_MSMF};
use opencv::{
    core::{ToInputArray, ToOutputArray, Vector},
    imgproc::{cvt_color, ColorConversionCodes},
    types::VectorOfu8,
    videoio::{VideoCapture, VideoCaptureTrait, VideoWriter, CAP_ANY, CAP_PROP_FOURCC, CAP_V4L2},
    videoio::{CAP_PROP_FPS, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH},
};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

// TODO: Define behaviour for IPCameras.
/// The backend struct that interfaces with `OpenCV`. Note that an `opencv` matching the version that this was either compiled on must be present on the user's machine. (usually 4.5.2 or greater)
/// For more information, please see [`opencv-rust`](https://github.com/twistedfall/opencv-rust) and [`OpenCV VideoCapture Docs`](https://docs.opencv.org/4.5.2/d8/dfe/classcv_1_1VideoCapture.html).
///
/// To see what this does, please see [`CaptureBackendTrait`]
/// # Quirks
///  - This is a **cross-platform** backend. This means that it will work on most platforms given that `OpenCV` is present.
///  - This backend can also do IP Camera input.
///  - The backend's backend will default to system level APIs on Linux(V4L2), Mac(AVFoundation), and Windows(Media Foundation). Otherwise, it will decide for itself.
///  - If the [`OpenCvCaptureDevice`] is initialized as a `IPCamera`, the [`CameraFormat`]'s `index` value will be [`u32::MAX`](std::u32::MAX) (4294967295).
///  - `OpenCV` does not support camera querying. Camera Name and Camera supported resolution/fps/fourcc is a [`UnsupportedOperation`](NokhwaError::UnsupportedOperation).
/// Note: [`get_resolution()`](CaptureBackendTrait::get_resolution()), [`get_frameformat()`](CaptureBackendTrait::get_frameformat()), and [`get_framerate()`](CaptureBackendTrait::get_framerate()) is not affected.
///  - [`CameraInfo`]'s human name will be "`OpenCV` Capture Device {location}"
///  - [`CameraInfo`]'s description will contain the Camera's Index or IP.
///  - [`get_frame_raw()`](CaptureBackendTrait::get_frame_raw()) returns a BGR24 image instead of \<native format>.
///  - The API Preference order is the native OS API (linux => `v4l2`, mac => `AVFoundation`, windows => `directshow`) than [`CAP_AUTO`](https://docs.opencv.org/4.5.2/d4/d15/group__videoio__flags__base.html#gga023786be1ee68a9105bf2e48c700294da77ab1fe260fd182f8ec7655fab27a31d)
pub struct OpenCvCaptureDevice {
    camera_format: CameraFormat,
    camera_location: CameraIndexType,
    camera_info: CameraInfo,
    api_preference: i32,
    video_capture: VideoCapture,
}

#[allow(clippy::must_use_candidate)]
impl OpenCvCaptureDevice {
    /// Creates a new capture device using the `OpenCV` backend. You can either use an [`Index`](CameraIndexType::Index) or [`IPCamera`](CameraIndexType::IPCamera).
    ///
    /// Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// `IPCameras` follow the format
    /// ```.ignore
    /// <protocol>://<IP>:<port>/
    /// ```
    /// , but please refer to the manufacturer for the actual IP format.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default if it is a index camera.
    /// # Errors
    /// If the backend fails to open the camera (e.g. Device does not exist at specified index/ip), Camera does not support specified [`CameraFormat`], and/or other `OpenCV` Error, this will error.
    /// # Panics
    /// If the API u32 -> i32
    pub fn new(
        camera_location: CameraIndexType,
        cfmt: Option<CameraFormat>,
        api_pref: Option<u32>,
    ) -> Result<Self, NokhwaError> {
        let api = if let Some(a) = api_pref {
            tryinto_num!(i32, a)
        } else {
            tryinto_num!(i32, get_api_pref_int())
        };

        let mut index = i32::MAX as u32;

        let camera_format = match cfmt {
            Some(cam_fmt) => cam_fmt,
            None => CameraFormat::default(),
        };

        let video_capture = match camera_location.clone() {
            CameraIndexType::Index(idx) => {
                let mut vid_cap = match VideoCapture::new(tryinto_num!(i32, idx), api) {
                    Ok(vc) => {
                        index = idx;
                        vc
                    }
                    Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
                };
                set_properties(&mut vid_cap, camera_format)?;
                vid_cap
            }
            CameraIndexType::IPCamera(ip) => match VideoCapture::from_file(&*ip, CAP_ANY) {
                Ok(vc) => vc,
                Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
            },
        };

        let camera_info = CameraInfo::new(
            format!("OpenCV Capture Device {}", camera_location),
            camera_location.to_string(),
            "".to_string(),
            index as usize,
        );

        Ok(OpenCvCaptureDevice {
            camera_format,
            camera_location,
            camera_info,
            api_preference: api,
            video_capture,
        })
    }

    /// Creates a new capture device using the `OpenCV` backend.
    ///
    /// `IPCameras` follow the format
    /// ```.ignore
    /// <protocol>://<IP>:<port>/
    /// ```
    /// , but please refer to the manufacturer for the actual IP format.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default if it is a index camera.
    /// # Errors
    /// If the backend fails to open the camera (e.g. Device does not exist at specified index/ip), Camera does not support specified [`CameraFormat`], and/or other `OpenCV` Error, this will error.
    pub fn new_ip_camera(ip: String) -> Result<Self, NokhwaError> {
        let camera_location = CameraIndexType::IPCamera(ip);
        OpenCvCaptureDevice::new(camera_location, None, None)
    }

    /// Creates a new capture device using the `OpenCV` backend.
    /// Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default if it is a index camera.
    /// # Errors
    /// If the backend fails to open the camera (e.g. Device does not exist at specified index/ip), Camera does not support specified [`CameraFormat`], and/or other `OpenCV` Error, this will error.
    pub fn new_index_camera(
        index: usize,
        cfmt: Option<CameraFormat>,
        api_pref: Option<u32>,
    ) -> Result<Self, NokhwaError> {
        let camera_location = CameraIndexType::Index(tryinto_num!(u32, index));
        OpenCvCaptureDevice::new(camera_location, cfmt, api_pref)
    }

    /// Gets weather said capture device is an `IPCamera`.
    pub fn is_ip_camera(&self) -> bool {
        match self.camera_location {
            CameraIndexType::Index(_) => false,
            CameraIndexType::IPCamera(_) => true,
        }
    }

    /// Gets weather said capture device is an OS-based indexed camera.
    pub fn is_index_camera(&self) -> bool {
        match self.camera_location {
            CameraIndexType::Index(_) => true,
            CameraIndexType::IPCamera(_) => false,
        }
    }

    /// Gets the camera location
    pub fn camera_location(&self) -> CameraIndexType {
        self.camera_location.clone()
    }

    /// Gets the `OpenCV` API Preference number. Please refer to [`OpenCV VideoCapture Flag Docs`](https://docs.opencv.org/4.5.2/d4/d15/group__videoio__flags__base.html).
    pub fn opencv_preference(&self) -> i32 {
        self.api_preference
    }

    /// Gets the BGR24 frame directly read from `OpenCV` without any additional processing
    /// # Errors
    /// If the frame is failed to be read, this will error.
    pub fn get_raw_frame_vector(&mut self) -> Result<Vector<u8>, NokhwaError> {
        let mut image_read_vector = VectorOfu8::new();
        let image_out_arr = &mut match image_read_vector.output_array() {
            Ok(out) => out,
            Err(why) => return Err(NokhwaError::CouldntCaptureFrame(why.to_string())),
        };
        match self.video_capture.read(image_out_arr) {
            Ok(read) => {
                if !read {
                    return Err(NokhwaError::CouldntCaptureFrame(
                        "OpenCV failed to read frame, returned false".to_string(),
                    ));
                }
            }
            Err(why) => return Err(NokhwaError::CouldntCaptureFrame(why.to_string())),
        }

        Ok(image_read_vector)
    }

    /// Gets the resolution raw as read by `OpenCV`.
    /// # Errors
    /// If the resolution is failed to be read (e.g. invalid or not supported), this will error.
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_resolution_raw(&self) -> Result<Resolution, NokhwaError> {
        let width = match self.video_capture.get(CAP_PROP_FRAME_WIDTH) {
            Ok(width) => width as u32,
            Err(why) => {
                return Err(NokhwaError::CouldntQueryDevice {
                    property: "Width".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let height = match self.video_capture.get(CAP_PROP_FRAME_HEIGHT) {
            Ok(height) => height as u32,
            Err(why) => {
                return Err(NokhwaError::CouldntQueryDevice {
                    property: "Height".to_string(),
                    error: why.to_string(),
                })
            }
        };

        Ok(Resolution::new(width, height))
    }

    /// Gets the framerate raw as read by `OpenCV`.
    /// # Errors
    /// If the framerate is failed to be read (e.g. invalid or not supported), this will error.
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_framerate_raw(&self) -> Result<u32, NokhwaError> {
        match self.video_capture.get(CAP_PROP_FPS) {
            Ok(fps) => Ok(fps as u32),
            Err(why) => Err(NokhwaError::CouldntQueryDevice {
                property: "Framerate".to_string(),
                error: why.to_string(),
            }),
        }
    }
}

impl CaptureBackendTrait for OpenCvCaptureDevice {
    fn camera_info(&self) -> CameraInfo {
        self.camera_info.clone()
    }

    fn camera_format(&self) -> CameraFormat {
        self.camera_format
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        let current_format = self.camera_format;
        let is_opened = match self.video_capture.is_opened() {
            Ok(opened) => opened,
            Err(why) => {
                return Err(NokhwaError::CouldntQueryDevice {
                    property: "Is Stream Open".to_string(),
                    error: why.to_string(),
                })
            }
        };
        set_properties(&mut self.video_capture, new_fmt)?;
        if is_opened {
            self.stop_stream()?;
            if let Err(why) = self.open_stream() {
                // revert
                set_properties(&mut self.video_capture, current_format)?;
                return Err(NokhwaError::CouldntOpenDevice(why.to_string()));
            }
        }
        Ok(())
    }

    fn get_compatible_list_by_resolution(
        &self,
        _fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        Err(NokhwaError::UnsupportedOperation(CaptureAPIBackend::OpenCv))
    }

    fn get_compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        Err(NokhwaError::UnsupportedOperation(CaptureAPIBackend::OpenCv))
    }

    fn resolution(&self) -> Resolution {
        self.camera_format.resolution()
    }

    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        let mut current_fmt = self.camera_format;
        current_fmt.set_resolution(new_res);
        self.set_camera_format(current_fmt)
    }

    fn frame_rate(&self) -> u32 {
        self.camera_format.framerate()
    }

    fn set_framerate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        let mut current_fmt = self.camera_format;
        current_fmt.set_framerate(new_fps);
        self.set_camera_format(current_fmt)
    }

    fn frameformat(&self) -> FrameFormat {
        self.camera_format.format()
    }

    fn set_frameformat(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        let mut current_fmt = self.camera_format;
        current_fmt.set_format(fourcc);
        self.set_camera_format(current_fmt)
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        let is_opened = match self.video_capture.is_opened() {
            Ok(opened) => opened,
            Err(why) => {
                return Err(NokhwaError::CouldntQueryDevice {
                    property: "Is Stream Open".to_string(),
                    error: why.to_string(),
                })
            }
        };

        if is_opened {
            match self.video_capture.release() {
                Ok(_) => {}
                Err(why) => return Err(NokhwaError::CouldntOpenStream(why.to_string())),
            }
        }

        self.video_capture = match self.camera_location.clone() {
            CameraIndexType::Index(idx) => {
                let mut vid_cap =
                    match VideoCapture::new(tryinto_num!(i32, idx), self.api_preference as i32) {
                        Ok(vc) => vc,
                        Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
                    };
                set_properties(&mut vid_cap, self.camera_format)?;
                vid_cap
            }
            CameraIndexType::IPCamera(ip) => match VideoCapture::from_file(&*ip, CAP_ANY) {
                Ok(vc) => vc,
                Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
            },
        };

        match self.video_capture.is_opened() {
            Ok(open) => {
                if open {
                    return Ok(());
                }
                Err(NokhwaError::CouldntOpenStream(
                    "Stream is not opened after stream open attempt opencv".to_string(),
                ))
            }
            Err(why) => Err(NokhwaError::CouldntQueryDevice {
                property: "Is Stream Open After Open Stream".to_string(),
                error: why.to_string(),
            }),
        }
    }

    fn is_stream_open(&self) -> bool {
        self.video_capture.is_opened().unwrap_or(false)
    }

    fn get_frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        let image_data = self.get_raw_frame_vector()?;
        let image_input_arr = &match image_data.input_array() {
            Ok(input) => input,
            Err(why) => return Err(NokhwaError::CouldntCaptureFrame(why.to_string())),
        };

        let mut image_rgb_out = VectorOfu8::new();
        let rgb_image_output_arr = &mut match image_rgb_out.output_array() {
            Ok(out) => out,
            Err(why) => return Err(NokhwaError::CouldntCaptureFrame(why.to_string())),
        };

        match cvt_color(
            image_input_arr,
            rgb_image_output_arr,
            ColorConversionCodes::COLOR_BGR2RGB as i32,
            0,
        ) {
            Ok(_) => {
                let rgb_image = image_rgb_out.to_vec();
                let camera_resolution = self.camera_format.resolution();
                let imagebuf =
                    match ImageBuffer::from_vec(camera_resolution.width(), camera_resolution.height(), rgb_image) {
                        Some(buf) => {
                            let rgbbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = buf;
                            rgbbuf
                        }
                        None => return Err(NokhwaError::CouldntCaptureFrame(
                            "Imagebuffer is not large enough! This is probably a bug, please report it!"
                                .to_string(),
                        )),
                    };
                Ok(imagebuf)
            }
            Err(why) => Err(NokhwaError::CouldntCaptureFrame(why.to_string())),
        }
    }

    fn get_frame_raw(&mut self) -> Result<Vec<u8>, NokhwaError> {
        let data = self.get_raw_frame_vector()?;
        Ok(data.to_vec())
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        match self.video_capture.release() {
            Ok(_) => Ok(()),
            Err(why) => Err(NokhwaError::CouldntStopStream(why.to_string())),
        }
    }
}

/// The `OpenCV` backend supports both native cameras and IP Cameras, so this is an enum to differentiate them
/// The `IPCamera`'s string follows the pattern
/// ```.ignore
/// <protocol>://<IP>:<port>/
/// ```
/// but please consult the manufacturer's specification for more details.
/// The index is a standard webcam index.
#[derive(Clone, Debug, PartialEq)]
pub enum CameraIndexType {
    Index(u32),
    IPCamera(String),
}

impl Display for CameraIndexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CameraIndexType::Index(idx) => {
                write!(f, "{}", idx)
            }
            CameraIndexType::IPCamera(ip) => {
                write!(f, "{}", ip)
            }
        }
    }
}

fn get_api_pref_int() -> u32 {
    match std::env::consts::OS {
        "linux" => CAP_V4L2 as u32,
        "windows" => CAP_MSMF as u32,
        "mac" => CAP_AVFOUNDATION as u32,
        &_ => CAP_ANY as u32,
    }
}
fn set_properties(vc: &mut VideoCapture, camera_format: CameraFormat) -> Result<(), NokhwaError> {
    set_property_fourcc(vc, camera_format.format())?;
    set_property_res(vc, camera_format.resolution())?;
    set_property_fps(vc, camera_format.framerate())?;
    Ok(())
}

fn set_property_res(vc: &mut VideoCapture, res: Resolution) -> Result<(), NokhwaError> {
    match vc.set(CAP_PROP_FRAME_HEIGHT as i32, f64::from(res.height())) {
        Ok(r) => {
            if !r {
                return Err(NokhwaError::CouldntSetProperty {
                    property: "Resolution Height".to_string(),
                    value: res.height().to_string(),
                    error: "OpenCV bool assert failure".to_string(),
                });
            }
        }
        Err(why) => {
            return Err(NokhwaError::CouldntSetProperty {
                property: "Resolution Height".to_string(),
                value: res.height().to_string(),
                error: why.to_string(),
            })
        }
    }

    match vc.set(CAP_PROP_FRAME_WIDTH as i32, f64::from(res.width())) {
        Ok(r) => {
            if !r {
                return Err(NokhwaError::CouldntSetProperty {
                    property: "Resolution Width".to_string(),
                    value: res.width().to_string(),
                    error: "OpenCV bool assert failure".to_string(),
                });
            }
        }
        Err(why) => {
            return Err(NokhwaError::CouldntSetProperty {
                property: "Resolution Width".to_string(),
                value: res.width().to_string(),
                error: why.to_string(),
            })
        }
    }

    Ok(())
}

fn set_property_fps(vc: &mut VideoCapture, fps: u32) -> Result<(), NokhwaError> {
    match vc.set(CAP_PROP_FPS as i32, f64::from(fps)) {
        Ok(r) => {
            if !r {
                return Err(NokhwaError::CouldntSetProperty {
                    property: "Framerate".to_string(),
                    value: fps.to_string(),
                    error: "OpenCV bool assert failure".to_string(),
                });
            }
        }
        Err(why) => {
            return Err(NokhwaError::CouldntSetProperty {
                property: "Framerate".to_string(),
                value: fps.to_string(),
                error: why.to_string(),
            })
        }
    }
    Ok(())
}

fn set_property_fourcc(vc: &mut VideoCapture, frame_fmt: FrameFormat) -> Result<(), NokhwaError> {
    let fourcc = match frame_fmt {
        FrameFormat::MJPEG => f64::from(
            match VideoWriter::fourcc('M' as i8, 'J' as i8, 'P' as i8, 'G' as i8) {
                Ok(fcc) => fcc,
                Err(why) => {
                    return Err(NokhwaError::CouldntSetProperty {
                        property: "FourCC".to_string(),
                        value: "MJPG".to_string(),
                        error: why.to_string(),
                    })
                }
            },
        ),
        FrameFormat::YUYV => f64::from(
            match VideoWriter::fourcc('Y' as i8, 'U' as i8, 'Y' as i8, 'V' as i8) {
                Ok(fcc) => fcc,
                Err(why) => {
                    return Err(NokhwaError::CouldntSetProperty {
                        property: "FourCC".to_string(),
                        value: "YUYV".to_string(),
                        error: why.to_string(),
                    })
                }
            },
        ),
    };

    match vc.set(
        CAP_PROP_FOURCC as i32,
        f64::from(VideoWriter::fourcc('M' as i8, 'J' as i8, 'P' as i8, 'G' as i8).unwrap()),
    ) {
        Ok(r) => {
            if !r {
                return Err(NokhwaError::CouldntSetProperty {
                    property: "FourCC".to_string(),
                    value: format!("FrameFormat: {}, OpenCV FourCC {}", frame_fmt, fourcc),
                    error: "OpenCV bool assert failure".to_string(),
                });
            }
        }
        Err(why) => {
            return Err(NokhwaError::CouldntSetProperty {
                property: "FourCC".to_string(),
                value: format!("FrameFormat: {}, OpenCV FourCC {}", frame_fmt, fourcc),
                error: why.to_string(),
            })
        }
    }
    Ok(())
}
