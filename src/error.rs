use thiserror::Error;

use crate::{CaptureAPIBackend, FrameFormat};

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::pub_enum_variant_names)]
#[derive(Error, Debug, Clone)]
pub enum NokhwaError {
    #[error("Could not open device: {0}")]
    CouldntOpenDevice(String),
    #[error("Could not query device property {property}: {error}")]
    CouldntQueryDevice { property: String, error: String },
    #[error("Could not set device property {property} with value {value}: {error}")]
    CouldntSetProperty {
        property: String,
        value: String,
        error: String,
    },
    #[error("Could not open device stream: {0}")]
    CouldntOpenStream(String),
    #[error("Could not capture frame: {0}")]
    CouldntCaptureFrame(String),
    #[error("Could not decompress frame {src} to {destination}: {error}")]
    CouldntDecompressFrame {
        src: FrameFormat,
        destination: String,
        error: String,
    },
    #[error("This operation is not supported by backend {0}.")]
    UnsupportedOperation(CaptureAPIBackend),
}
