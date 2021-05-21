use thiserror::Error;

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
}
