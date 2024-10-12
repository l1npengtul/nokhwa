use std::collections::HashMap;
use crate::buffer::Buffer;
use crate::controls::{CameraProperties, CameraPropertyId, CameraPropertyValue};
use crate::error::NokhwaError;
use crate::types::{CameraFormat, CameraIndex, Resolution};

pub trait Open {
    fn open(index: CameraIndex) -> Self; 
}

#[cfg(feature = "async")]
pub trait AsyncOpen {
    async fn open_async(index: CameraIndex) -> Self;
}

macro_rules! def_camera_props {
    ( $($property:ident, )* ) => {
        $(
        fn paste::paste! { [<$property:snake>] } (&self) -> Option<&CameraPropertyDescriptor> {
            self.properties().paste::paste! { [<$property:snake>] }
        }
        
        fn paste::paste! { [<set_ $property:snake>] } (&mut self, value: CameraPropertyValue) -> Result<(), NokhwaError>;
        )*
    };
}

macro_rules! def_camera_props_async {
    ( $($property:ident, )* ) => {
        $(
        async fn paste::paste! { [<set_ $property:snake>] } (&mut self, value: CameraPropertyValue) -> Result<(), NokhwaError>;
        )*
    };
}

pub trait Setting {
    fn enumerate_formats(&self) -> Vec<CameraFormat>;
    
    fn enumerate_formats_by_resolution(&self) -> HashMap<Resolution, CameraFormat>;
    
    fn set_format(&self, camera_format: CameraFormat) -> Result<(), NokhwaError>;
    
    fn properties(&self) -> &CameraProperties;
    
    fn set_property(&mut self, property: &CameraPropertyId, value: CameraPropertyValue) -> Result<(), NokhwaError>; 

    def_camera_props!(
        Brightness,
        Contrast,
        Hue,
        Saturation,
        Sharpness,
        Gamma,
        WhiteBalance,
        BacklightCompensation,
        Gain,
        Pan,
        Tilt,
        Zoom,
        Exposure,
        Iris,
        Focus,
        Facing, 
    );
}

#[cfg(feature = "async")]
pub trait AsyncSetting {
    async fn set_format(&self, camera_format: CameraFormat) -> Result<(), NokhwaError>;

    async fn set_property(&mut self, property: &CameraPropertyId, value: CameraPropertyValue) -> Result<(), NokhwaError>;

    def_camera_props_async!(
        Brightness,
        Contrast,
        Hue,
        Saturation,
        Sharpness,
        Gamma,
        WhiteBalance,
        BacklightCompensation,
        Gain,
        Pan,
        Tilt,
        Zoom,
        Exposure,
        Iris,
        Focus,
        Facing, 
    );
}

pub trait Stream {
    fn open_stream(&mut self) -> Result<(), NokhwaError>;
    
    fn poll_frame(&mut self) -> Result<Buffer, NokhwaError>;
    
    fn close_stream(&mut self) -> Result<(), NokhwaError>;
}

#[cfg(feature = "async")]
pub trait AsyncStream {
    async fn open_stream(&mut self) -> Result<(), NokhwaError>;

    async fn poll_frame(&mut self) -> Result<Buffer, NokhwaError>;

    async fn close_stream(&mut self) -> Result<(), NokhwaError>;}

pub trait Capture: Open + Setting + Stream {}

#[cfg(feature = "async")]
pub trait AsyncCapture: Capture + AsyncOpen + AsyncSetting + AsyncStream {}
