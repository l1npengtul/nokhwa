use crate::control::{ControlDescription, ControlId, ControlValue};
use crate::error::NokhwaError;
use crate::frame_format::FrameFormat;
use crate::stream::{StreamConfiguration, StreamHandle};
use crate::types::{CameraFormat, FrameRate, Resolution};
use std::collections::HashMap;
use std::collections::hash_map::{Keys, Values};
use std::sync::Arc;

pub trait Setting {
    /// # Errors
    /// Will error on 
    fn enumerate_formats(&self) -> Result<Vec<CameraFormat>, NokhwaError>;

    /// # Errors
    /// Will error on 
    fn enumerate_resolution_and_frame_rates(
        &self,
        frame_format: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<FrameRate>>, NokhwaError>;

    /// # Errors
    /// Will error on 
    fn set_format(&mut self, camera_format: CameraFormat) -> Result<(), NokhwaError>;

    fn control_ids(&self) -> Keys<ControlId, ControlDescription>;

    fn control_descriptions(&self) -> Values<ControlId, ControlDescription>;

    fn control_values(&self) -> Values<ControlId, ControlValue>;

    fn control_value(&self, id: &ControlId) -> Option<&ControlValue>;

    fn control_description(&self, id: &ControlId) -> Option<&ControlDescription>;

    /// # Errors
    /// Will error on 
    fn set_control(&mut self, property: &ControlId, value: ControlValue)
    -> Result<(), NokhwaError>;

    /// # Errors
    /// Will error on 
    fn refresh_controls(&mut self) -> Result<(), NokhwaError>;
}

#[cfg(feature = "async")]
pub trait AsyncSetting {
    async fn enumerate_formats_async(&self) -> Result<Vec<CameraFormat>, NokhwaError>;

    async fn enumerate_resolution_and_frame_rates_async(
        &self,
        frame_format: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<FrameRate>>, NokhwaError>;

    async fn set_format_async(&self, camera_format: CameraFormat) -> Result<(), NokhwaError>;
    
    async fn set_property_async(
        &mut self,
        property: &ControlId,
        value: ControlValue,
    ) -> Result<(), NokhwaError>;
}

pub trait Capture {
    /// Implementations MUST guarantee that there can only ever be one stream open at once.
    /// # Errors
    /// Errors are driver specific
    fn open_stream(
        &mut self,
        stream_configuration: Option<StreamConfiguration>,
    ) -> Result<Arc<StreamHandle>, NokhwaError>;

    // Implementations MUST be multi-close tolerant.
    /// # Errors
    /// Errors are driver specific
    fn close_stream(&mut self) -> Result<(), NokhwaError>;
}

#[cfg(feature = "async")]
pub trait AsyncStream {
    async fn open_stream_async(
        &mut self,
        stream_configuration: Option<StreamConfiguration>,
    ) -> Result<StreamHandle, NokhwaError>;

    async fn close_stream_async(&mut self) -> Result<(), NokhwaError>;
}

pub trait Camera: Setting + Capture {}

#[cfg(feature = "async")]
pub trait AsyncCamera: Camera + AsyncSetting + AsyncStream {}
