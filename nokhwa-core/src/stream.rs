use crate::{error::NokhwaError, types::CameraFormat};

pub trait StreamTrait {
    fn current_format(&self) -> &CameraFormat;

    fn stop_stream(self) -> Result<(), NokhwaError>;
}
