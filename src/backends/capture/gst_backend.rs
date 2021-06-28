use crate::{
    CameraFormat, CameraInfo, CaptureAPIBackend, CaptureBackendTrait, FrameFormat, NokhwaError,
    Resolution,
};
use gstreamer::{
    glib::Cast, Caps, ClockTime, DeviceExt, DeviceMonitor, DeviceMonitorExt,
    DeviceMonitorExtManual, Element, ElementExtManual, GstBinExt, State,
};
use gstreamer_app::AppSink;
use image::{ImageBuffer, Rgb};
use std::{collections::HashMap, str::FromStr};

/// The backend struct that interfaces with GStreamer.
/// To see what this does, please see [`CaptureBackendTrait`].
/// # Quirks
/// - [`FrameFormat`]s are **not** respected.
/// - `Drop`-ing this may cause a `panic`.
pub struct GStreamerCaptureDevice {
    pipeline: Element,
    app_sink: AppSink,
    camera_format: CameraFormat,
    camera_info: CameraInfo,
}

impl GStreamerCaptureDevice {
    pub fn new(index: usize, cam_fmt: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        let camera_format = match cam_fmt {
            Some(fmt) => fmt,
            None => CameraFormat::default(),
        };

        if let Err(why) = gstreamer::init() {
            return Err(NokhwaError::CouldntOpenDevice(format!(
                "Failed to initialize GStreamer: {}",
                why.to_string()
            )));
        }

        let camera_info = {
            let device_monitor = DeviceMonitor::new();
            let video_caps = match Caps::from_str("video/x-raw") {
                Ok(cap) => cap,
                Err(why) => {
                    return Err(NokhwaError::GeneralError(format!(
                        "Failed to generate caps: {}",
                        why.to_string()
                    )))
                }
            };
            let _video_filter_id = match device_monitor
                .add_filter(Some("Video/Source"), Some(&video_caps))
            {
                Some(id) => id,
                None => return Err(NokhwaError::CouldntOpenDevice(
                    "Failed to generate Device Monitor Filter ID with video/x-raw and Video/Source"
                        .to_string(),
                )),
            };
            if let Err(why) = device_monitor.start() {
                return Err(NokhwaError::CouldntOpenDevice(format!(
                    "Failed to start device monitor: {}",
                    why.to_string()
                )));
            }
            let device = match device_monitor.get_devices().get(index) {
                Some(dev) => dev.clone(),
                None => {
                    return Err(NokhwaError::CouldntOpenDevice(format!(
                        "Failed to find device at index {}",
                        index
                    )))
                }
            };
            device_monitor.stop();

            CameraInfo::new(
                DeviceExt::get_display_name(&device).to_string(),
                DeviceExt::get_device_class(&device).to_string(),
                "".to_string(),
                index,
            )
        };

        let pipeline = match gstreamer::parse_launch(&*webcam_pipeline(
            camera_format.width(),
            camera_format.height(),
            camera_format.framerate(),
            format!("{}", index).as_str(),
        )) {
            Ok(pl) => pl,
            Err(why) => {
                return Err(NokhwaError::CouldntOpenDevice(format!(
                    "Failed to initialize GStreamer Pipeline with string {} : {}",
                    webcam_pipeline(
                        camera_format.width(),
                        camera_format.height(),
                        camera_format.framerate(),
                        format!("{}", index).as_str()
                    ),
                    why.to_string(),
                )));
            }
        };

        let sink = {
            let bin = match pipeline.clone().dynamic_cast::<gstreamer::Bin>() {
                Ok(bn) => bn,
                Err(_) => {
                    return Err(NokhwaError::CouldntOpenDevice(
                        "Failed to cast Element to Bin".to_string(),
                    ));
                }
            };

            let snk = match bin.get_by_name("appsink") {
                Some(sk) => sk,
                None => {
                    return Err(NokhwaError::CouldntOpenDevice(
                        "Failed to cast get appsink from pipeline!".to_string(),
                    ));
                }
            };
            snk
        };
        let app_sink = match sink.clone().dynamic_cast::<gstreamer_app::AppSink>() {
            Ok(ap_sk) => ap_sk,
            Err(_) => {
                return Err(NokhwaError::CouldntOpenDevice(
                    "Failed to cast Element to AppSink".to_string(),
                ));
            }
        };

        Ok(GStreamerCaptureDevice {
            pipeline,
            app_sink,
            camera_format,
            camera_info,
        })
    }

    pub fn new_with(index: usize, width: u32, height: u32, fps: u32) -> Result<Self, NokhwaError> {
        let cam_fmt = CameraFormat::new(Resolution::new(width, height), FrameFormat::MJPEG, fps);
        GStreamerCaptureDevice::new(index, Some(cam_fmt))
    }

    /// Regenerates the GStreamer Pipeline. Mostly used by internal functions, although made available for your convenience.
    /// Equivalent to calling [`GStreamerCaptureDevice::new`] but it sets on the current backend object.
    /// # Errors
    /// If the GStreamer fails to capture the object or
    pub fn regenerate_pipeline(&mut self) -> Result<(), NokhwaError> {
        let pipeline = match gstreamer::parse_launch(&*webcam_pipeline(
            self.camera_format.width(),
            self.camera_format.height(),
            self.camera_format.framerate(),
            format!("{}", self.camera_info.index()).as_str(),
        )) {
            Ok(pl) => pl,
            Err(why) => {
                return Err(NokhwaError::CouldntOpenDevice(format!(
                    "Failed to initialize GStreamer Pipeline with string {} : {}",
                    webcam_pipeline(
                        self.camera_format.width(),
                        self.camera_format.height(),
                        self.camera_format.framerate(),
                        format!("{}", self.camera_info.index()).as_str()
                    ),
                    why.to_string(),
                )));
            }
        };

        let sink = {
            let bin = match pipeline.clone().dynamic_cast::<gstreamer::Bin>() {
                Ok(bn) => bn,
                Err(_) => {
                    return Err(NokhwaError::CouldntOpenDevice(
                        "Failed to cast Element to Bin".to_string(),
                    ));
                }
            };

            let snk = match bin.get_by_name("appsink") {
                Some(sk) => sk,
                None => {
                    return Err(NokhwaError::CouldntOpenDevice(
                        "Failed to cast get appsink from pipeline!".to_string(),
                    ));
                }
            };
            snk
        };
        let app_sink = match sink.clone().dynamic_cast::<gstreamer_app::AppSink>() {
            Ok(ap_sk) => ap_sk,
            Err(_) => {
                return Err(NokhwaError::CouldntOpenDevice(
                    "Failed to cast Element to AppSink".to_string(),
                ));
            }
        };

        self.app_sink = app_sink;
        self.pipeline = pipeline;

        Ok(())
    }
}

impl CaptureBackendTrait for GStreamerCaptureDevice {
    fn camera_info(&self) -> CameraInfo {
        self.camera_info.clone()
    }

    fn camera_format(&self) -> CameraFormat {
        self.camera_format
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        self.camera_format = new_fmt;
        self.regenerate_pipeline()?;
        if self.is_stream_open() {
            self.open_stream()?;
        }
        Ok(())
    }

    fn get_compatible_list_by_resolution(
        &self,
        _fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        Err(NokhwaError::UnsupportedOperation(
            CaptureAPIBackend::GStreamer,
        ))
    }

    fn get_compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        Err(NokhwaError::UnsupportedOperation(
            CaptureAPIBackend::GStreamer,
        ))
    }

    fn resolution(&self) -> Resolution {
        self.camera_format.resolution()
    }

    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        let mut new_fmt = self.camera_format;
        new_fmt.set_resolution(new_res);
        self.set_camera_format(new_fmt)
    }

    fn frame_rate(&self) -> u32 {
        self.camera_format.framerate()
    }

    fn set_framerate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        let mut new_fmt = self.camera_format;
        new_fmt.set_framerate(new_fps);
        self.set_camera_format(new_fmt)
    }

    fn frameformat(&self) -> FrameFormat {
        self.camera_format.format()
    }

    fn set_frameformat(&mut self, _fourcc: FrameFormat) -> Result<(), NokhwaError> {
        Err(NokhwaError::UnsupportedOperation(
            CaptureAPIBackend::GStreamer,
        ))
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        if let Err(why) = self.pipeline.set_state(State::Playing) {
            return Err(NokhwaError::CouldntOpenStream(format!(
                "Failed to set appsink to playing: {}",
                why.to_string()
            )));
        }
        Ok(())
    }

    // TODO: someone validate this
    fn is_stream_open(&self) -> bool {
        let (res, state_from, state_to) = self.pipeline.get_state(ClockTime::none());
        return match res {
            Ok(_) => {
                if state_to == State::Playing {
                    return true;
                }
                false
            }
            Err(_) => {
                if state_from == State::Playing {
                    return true;
                }
                false
            }
        };
    }

    fn get_frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        let image_data = self.get_frame_raw()?;
        let cam_fmt = self.camera_format;
        let imagebuf =
            match ImageBuffer::from_vec(cam_fmt.width(), cam_fmt.height(), image_data) {
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

    fn get_frame_raw(&mut self) -> Result<Vec<u8>, NokhwaError> {
        if self.is_stream_open() {
            return Err(NokhwaError::CouldntCaptureFrame(
                "Please call `open_stream` first!".to_string(),
            ));
        }
        if self.app_sink.is_eos() {
            return Err(NokhwaError::CouldntCaptureFrame(
                "Stream is EOS!".to_string(),
            ));
        }
        match self.app_sink.pull_sample() {
            Ok(sample) => match sample.get_buffer_owned() {
                Some(buf) => match buf.into_mapped_buffer_readable() {
                    Ok(readable) => Ok(readable.as_slice().to_vec()),
                    Err(_) => {
                        return Err(NokhwaError::CouldntCaptureFrame(
                            "Sample Buffer get mapped readable fail!".to_string(),
                        ))
                    }
                },
                None => {
                    return Err(NokhwaError::CouldntCaptureFrame(
                        "Sample Buffer get fail!".to_string(),
                    ))
                }
            },
            Err(why) => {
                return Err(NokhwaError::CouldntCaptureFrame(format!(
                    "Failed to pull sample from appsink: {}",
                    why.to_string()
                )))
            }
        }
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        if let Err(why) = self.pipeline.set_state(State::Null) {
            return Err(NokhwaError::CouldntStopStream(format!(
                "Could not change state: {}",
                why.to_string()
            )));
        }
        Ok(())
    }
}

impl Drop for GStreamerCaptureDevice {
    fn drop(&mut self) {
        self.pipeline.set_state(State::Null).unwrap();
    }
}

#[cfg(target_os = "linux")]
fn webcam_pipeline(width: u32, height: u32, fps: u32, camera_location: &str) -> String {
    format!("v4l2src device=/dev/video{} ! video/x-raw,width={},height={},format=RGB,framerate={}/1 ! appsink name=appsink async=false sync=false", camera_location, width, height, fps)
}

#[cfg(target_os = "windows")]
fn webcam_pipeline(width: u32, height: u32, fps: u32, camera_location: &str) -> String {
    format!("ksvideosrc device_index={} ! video/x-raw,width={},height={},format=RGB,framerate={}/1 ! appsink name=appsink async=false sync=false", camera_location, width, height, fps)
}

#[cfg(target_os = "macos")]
fn webcam_pipeline(width: u32, height: u32, fps: u32, camera_location: &str) -> String {
    format!("autovideosrc location=/dev/video{} ! video/x-raw,width={},height={},format=RGB,framerate={}/1 ! appsink name=appsink async=false sync=false", camera_location, width, height, fps)
}
