use crate::error::NokhwaError;
use crate::frame_format::FrameFormat;
use crate::types::CameraFormat;
use std::fmt::Debug;

pub trait Codec {
    type Config: Clone + Debug;

    type Input<'a>;

    type Output;

    type WrittenMeta: Clone + Debug;

    /// # Errors
    /// Errors are decoder specific. 
    fn allowed_formats(&self) -> Result<&[FrameFormat], NokhwaError>;

    fn config(&self) -> &Self::Config;

    /// # Errors
    /// Errors are decoder specific. 
    fn set_config(&mut self, config: Self::Config) -> Result<(), NokhwaError>;

    /// # Errors
    /// Errors are decoder specific. 
    fn send_item(&mut self, input: Self::Input<'_>) -> Result<(), NokhwaError>;

    fn receive_decoded_item(
        &mut self,
        writing_output: &mut Self::Output,
    ) -> Result<Self::WrittenMeta, NokhwaError>;

    fn preferred_buffer_min_size(
        &mut self,
        format: &Option<CameraFormat>,
    ) -> Result<Option<usize>, NokhwaError>;

    fn deinitialize(&mut self) -> Result<(), NokhwaError>;
}

#[cfg(feature = "async")]
pub trait CodecAsync: Codec {

    async fn allowed_formats_async(&self) -> Result<&[FrameFormat], NokhwaError> {
        self.allowed_formats()
    }

    async fn set_format_async(&self, format: CameraFormat) -> Result<(), NokhwaError>;

    async fn config_async(&self) -> &Self::Config {
        self.config()
    }

    fn set_config_async(&mut self, config: Self::Config) -> Result<(), NokhwaError> {
        self.set_config(config)
    }

    fn send_item_async(&mut self, input: &Self::Input) -> Result<(), NokhwaError>;

    fn receive_decoded_item_async(
        &mut self,
        writing_output: &mut Self::Output,
    ) -> Result<Option<usize>, NokhwaError>;

    async fn deinitialize_async(&mut self) -> Result<(), NokhwaError> {
        self.deinitialize()
    }
}
