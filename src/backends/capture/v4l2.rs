use crate::{error::NokhwaError, utils::{CameraFormat, CameraInfo}};
use v4l::prelude::*;

pub struct V4LCaptureDevice {
    camera_format: Option<CameraFormat>,
    camera_info: CameraInfo,
    device: Device,
}

impl V4LCaptureDevice {
    /// Creates a new capture device using the V4L2 backend
    /// # Errors
    /// This function will error if the camera is currently busy or if V4L2 can't read device information. 
    pub fn new(index: usize) -> Result<Self, NokhwaError> {
        let device = match Device::new(index) {
            Ok(dev) => dev,
            Err(why) => {
                return Err(NokhwaError::CouldntOpenDevice(format!("V4L2 Error: {}", why.to_string())))
            }
        };

        let camera_info = match device.query_caps() {
            Ok(caps) => {
                CameraInfo::new(caps.card, "".to_string(), caps.driver, index)
            }
            Err(why) => {
                return Err(NokhwaError::CouldntQueryDevice{ property: "Capabilities".to_string(), error: why.to_string() })
            }
        };

        Ok(V4LCaptureDevice {
            camera_format: None,
            camera_info,
            device,
        })
        
    }
}
