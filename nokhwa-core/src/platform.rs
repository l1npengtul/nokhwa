use crate::camera::Camera;
use crate::error::NokhwaResult;
use crate::types::{CameraIndex, CameraInformation};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Backends {
    Video4Linux2,
    WebWASM,
    AVFoundation,
    MicrosoftMediaFoundation,
    OpenCV,
    Custom(&'static str),
}

impl Display for Backends {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub trait PlatformTrait {
    const PLATFORM: Backends;
    type Camera: Camera;

    fn block_on_permission(&mut self) -> NokhwaResult<()>;

    fn check_permission_given(&mut self) -> bool;

    fn query(&mut self) -> NokhwaResult<Vec<CameraInformation>>;

    fn open(&mut self, index: CameraIndex) -> NokhwaResult<Self::Camera>;

    fn open_dynamic(&mut self, index: CameraIndex) -> NokhwaResult<Box<dyn Camera>>
    where
        <Self as PlatformTrait>::Camera: 'static,
    {
        self.open(index).map(|cam| Box::new(cam) as Box<dyn Camera>)
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait AsyncPlatformTrait: PlatformTrait {
    const PLATFORM: Backends;
    type AsyncCamera: crate::camera::AsyncCamera;

    async fn await_permission(&mut self) -> NokhwaResult<()>;

    async fn query_async(&mut self) -> NokhwaResult<Vec<CameraInformation>>;

    async fn open_async(&mut self, index: &CameraIndex) -> NokhwaResult<Self::AsyncCamera>;

    async fn open_dynamic_async(&mut self, index: &CameraIndex) -> NokhwaResult<Box<dyn Camera>>
    where
        <Self as AsyncPlatformTrait>::AsyncCamera: 'static,
    {
        self.open_async(index)
            .await
            .map(|cam| Box::new(cam) as Box<dyn Camera>)
    }
}
