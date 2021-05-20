use image::{ImageBuffer, Rgb};

use crate::{error::NokhwaError, utils::{CameraFormat, CameraInfo}};

pub trait CaptureBackendTrait {
    fn info(&self) -> CameraInfo;
    fn set_camera_format(&self, new_fmt: CameraFormat) -> Result<(), NokhwaError>;
    fn open_stream(&self) -> Result<(), NokhwaError>;
    fn get_frame(&self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError>;
}

pub trait VirtualBackendTrait {}
