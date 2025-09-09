use crate::camera::Camera;
use crate::error::NokhwaResult;
use crate::types::{Backends, CameraIndex, CameraInformation, QueriedCamera};
use std::fmt::{Display, Formatter};


pub trait PlatformTrait {
    const PLATFORM: Backends;
    type Camera: Camera;

    fn block_on_permission(&mut self) -> NokhwaResult<()>;

    fn check_permission_given(&mut self) -> bool;

    fn query(&mut self) -> NokhwaResult<Vec<QueriedCamera>>;

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

    async fn query_async(&mut self) -> NokhwaResult<Vec<QueriedCamera>>;

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
