use crate::{
    tryinto_num, vector, CameraFormat, CameraInfo, CaptureAPIBackend, CaptureBackendTrait,
    FrameFormat, NokhwaError, Resolution,
};
use image::{ImageBuffer, Rgb};
use opencv::core::Vector;
use opencv::{
    core::{Mat, MatTrait, MatTraitManual, Vec3b},
    videoio::{
        VideoCapture, VideoCaptureTrait, VideoWriter, CAP_ANY, CAP_AVFOUNDATION, CAP_DSHOW,
        CAP_PROP_FOURCC, CAP_PROP_FPS, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH, CAP_V4L2,
    },
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
///  - **Some features don't work properly on this backend (yet)! Setting Resolution, FPS, FourCC does not work and will default to 640x480 30FPS. This is being worked on.**
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
    /// If the API u32 -> i32 fails this will error
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

        let mut video_capture = match camera_location.clone() {
            CameraIndexType::Index(idx) => {
                let vid_cap = match VideoCapture::new(tryinto_num!(i32, idx), api) {
                    Ok(vc) => {
                        index = idx;
                        vc
                    }
                    Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
                };
                vid_cap
            }
            CameraIndexType::IPCamera(ip) => match VideoCapture::from_file(&*ip, CAP_ANY) {
                Ok(vc) => vc,
                Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
            },
        };

        set_properties(&mut video_capture, camera_format, &camera_location)?;

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

    /// Creates a new capture device using the `OpenCV` backend.
    /// Indexes are gives to devices by the OS, and usually numbered by order of discovery.
    ///
    /// If `camera_format` is `None`, it will be spawned with with 640x480@15 FPS, MJPEG [`CameraFormat`] default if it is a index camera.
    /// # Errors
    /// If the backend fails to open the camera (e.g. Device does not exist at specified index/ip), Camera does not support specified [`CameraFormat`], and/or other `OpenCV` Error, this will error.
    pub fn new_autopref(index: usize, cfmt: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        let camera_location = CameraIndexType::Index(tryinto_num!(u32, index));
        OpenCvCaptureDevice::new(camera_location, cfmt, None)
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

    /// Gets the RGB24 frame directly read from `OpenCV` without any additional processing.
    /// # Errors
    /// If the frame is failed to be read, this will error.
    #[allow(clippy::cast_sign_loss)]
    pub fn raw_frame_vec(&mut self) -> Result<Vec<u8>, NokhwaError> {
        if !self.is_stream_open() {
            return Err(NokhwaError::CouldntCaptureFrame(
                "Stream is not open!".to_string(),
            ));
        }

        let mut frame = Mat::default();
        match self.video_capture.read(&mut frame) {
            Ok(a) => {
                if !a {
                    return Err(NokhwaError::CouldntCaptureFrame(
                        "Failed to read frame from videocapture: OpenCV return false, camera disconnected?".to_string(),
                    ));
                }
            }
            Err(why) => {
                return Err(NokhwaError::CouldntCaptureFrame(format!(
                    "Failed to read frame from videocapture: {}",
                    why.to_string()
                )))
            }
        }

        let frame_empty = match frame.empty() {
            Ok(e) => e,
            Err(why) => {
                return Err(NokhwaError::CouldntCaptureFrame(format!(
                    "Failed to check for empty OpenCV frame: {}",
                    why.to_string()
                )))
            }
        };

        match frame.size() {
            Ok(size) => {
                if size.width > 0 && !frame_empty {
                    return match frame.is_continuous() {
                        Ok(cont) => {
                            if cont {
                                let mut raw_vec: Vec<u8> = Vec::new();
                                raw_vec.reserve(
                                    (self.resolution().width()
                                        * self.resolution().height()
                                        * (frame.channels().unwrap_or(3)) as u32)
                                        as usize,
                                );

                                let frame_data_vec = match Mat::data_typed::<Vec3b>(&frame) {
                                    Ok(v) => v,
                                    Err(why) => {
                                        return Err(NokhwaError::CouldntCaptureFrame(format!(
                                            "Failed to convert frame into raw Vec3b: {}",
                                            why.to_string()
                                        )))
                                    }
                                };

                                for pixel in frame_data_vec.iter() {
                                    let pixel_slice: &[u8; 3] = &**pixel;
                                    raw_vec.push(pixel_slice[2]);
                                    raw_vec.push(pixel_slice[1]);
                                    raw_vec.push(pixel_slice[0]);
                                }

                                return Ok(raw_vec);
                            }
                            Err(NokhwaError::CouldntCaptureFrame(
                                "Failed to read frame from videocapture: not cont".to_string(),
                            ))
                        }
                        Err(why) => Err(NokhwaError::CouldntCaptureFrame(format!(
                            "Failed to read frame from videocapture: failed to read continuous: {}",
                            why.to_string()
                        ))),
                    };
                }
                Err(NokhwaError::CouldntCaptureFrame(
                    "Frame width is less than zero!".to_string(),
                ))
            }
            Err(why) => {
                return Err(NokhwaError::CouldntCaptureFrame(format!(
                    "Failed to read frame from videocapture: failed to read size: {}",
                    why.to_string()
                )))
            }
        }
    }

    /// Gets the resolution raw as read by `OpenCV`.
    /// # Errors
    /// If the resolution is failed to be read (e.g. invalid or not supported), this will error.
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn raw_resolution(&self) -> Result<Resolution, NokhwaError> {
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

        self.camera_format = new_fmt;

        if let Err(why) = set_properties(&mut self.video_capture, new_fmt, &self.camera_location) {
            self.camera_format = current_format;
            return Err(why);
        }
        if is_opened {
            self.stop_stream()?;
            if let Err(why) = self.open_stream() {
                return Err(NokhwaError::CouldntOpenDevice(why.to_string()));
            }
        }
        Ok(())
    }

    fn compatible_list_by_resolution(
        &self,
        _fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        Err(NokhwaError::UnsupportedOperation(CaptureAPIBackend::OpenCv))
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
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

    #[allow(clippy::cast_possible_wrap)]
    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        match self.camera_location.clone() {
            CameraIndexType::Index(idx) => {
                match self
                    .video_capture
                    .open_1(idx as i32, get_api_pref_int() as i32)
                {
                    Ok(_) => {}
                    Err(why) => {
                        return Err(NokhwaError::CouldntOpenDevice(format!(
                            "Failed to open device: {}",
                            why.to_string()
                        )))
                    }
                }
            }
            CameraIndexType::IPCamera(ip) => {
                match self
                    .video_capture
                    .open_file(&*ip, get_api_pref_int() as i32)
                {
                    Ok(_) => {}
                    Err(why) => {
                        return Err(NokhwaError::CouldntOpenDevice(format!(
                            "Failed to open device: {}",
                            why.to_string()
                        )))
                    }
                }
            }
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

    fn frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        let mut image_data = self.frame_raw()?;
        let camera_resolution = self.camera_format.resolution();
        image_data.resize(
            (camera_resolution.width() * camera_resolution.height() * 3) as usize,
            0_u8,
        );
        let imagebuf =
            match ImageBuffer::from_vec(
                camera_resolution.width(),
                camera_resolution.height(),
                image_data,
            ) {
                Some(buf) => {
                    let rgb: ImageBuffer<Rgb<u8>, Vec<u8>> = buf;
                    rgb
                }
                None => return Err(NokhwaError::CouldntCaptureFrame(
                    "Imagebuffer is not large enough! This is probably a bug, please report it!"
                        .to_string(),
                )),
            };
        Ok(imagebuf)
    }

    fn frame_raw(&mut self) -> Result<Vec<u8>, NokhwaError> {
        let vec = self.raw_frame_vec()?;
        // println!("{:?}", vec);
        Ok(vec)
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
        "windows" => CAP_DSHOW as u32,
        "mac" => CAP_AVFOUNDATION as u32,
        &_ => CAP_ANY as u32,
    }
}

#[allow(clippy::cast_possible_wrap)]
fn set_properties(
    vc: &mut VideoCapture,
    camera_format: CameraFormat,
    camera_location: &CameraIndexType,
) -> Result<(), NokhwaError> {
    let fourcc = match camera_format.format() {
        FrameFormat::MJPEG => {
            match VideoWriter::fourcc('m' as i8, 'j' as i8, 'p' as i8, 'g' as i8) {
                Ok(fmt) => fmt,
                Err(why) => {
                    return Err(NokhwaError::CouldntSetProperty {
                        property: "FrameFormat".to_string(),
                        value: "FourCC MJPG".to_string(),
                        error: why.to_string(),
                    })
                }
            }
        }
        FrameFormat::YUYV => {
            match VideoWriter::fourcc('y' as i8, 'u' as i8, 'y' as i8, 'v' as i8) {
                Ok(fmt) => fmt,
                Err(why) => {
                    return Err(NokhwaError::CouldntSetProperty {
                        property: "FrameFormat".to_string(),
                        value: "FourCC YUYV".to_string(),
                        error: why.to_string(),
                    })
                }
            }
        }
    };

    let properties: &Vector<i32> = &vector!(
        CAP_PROP_FOURCC as i32,
        fourcc,
        CAP_PROP_FRAME_WIDTH as i32,
        camera_format.width() as i32,
        CAP_PROP_FRAME_HEIGHT as i32,
        camera_format.height() as i32,
        CAP_PROP_FPS as i32,
        camera_format.framerate() as i32
    );

    match camera_location {
        CameraIndexType::Index(idx) => {
            match vc.open_2(*idx as i32, get_api_pref_int() as i32, properties) {
                Ok(v) => {
                    if !v {
                        return Err(NokhwaError::CouldntOpenDevice(
                            "Failed to re-open camera, OpenCV Bool return error".to_string(),
                        ));
                    }
                }
                Err(why) => {
                    return Err(NokhwaError::CouldntOpenDevice(format!(
                        "Failed to re-open camera with properties: {}",
                        why.to_string()
                    )))
                }
            }
        }
        CameraIndexType::IPCamera(ip) => {
            match vc.open(ip.as_str(), get_api_pref_int() as i32, properties) {
                Ok(v) => {
                    if !v {
                        return Err(NokhwaError::CouldntOpenDevice(
                            "Failed to re-open camera, OpenCV Bool return error".to_string(),
                        ));
                    }
                }
                Err(why) => {
                    return Err(NokhwaError::CouldntOpenDevice(format!(
                        "Failed to re-open camera with properties: {}",
                        why.to_string()
                    )))
                }
            }
        }
    }

    Ok(())
}
