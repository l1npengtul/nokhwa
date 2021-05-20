use thiserror::Error;

#[allow(clippy::clippy::module_name_repetitions)]
#[derive(Error, Debug, Clone)]
pub enum NokhwaError {
    #[error("Could not open device: {0}")]
    CouldntOpenDevice(String),
    #[error("Could not query device property {property}: {error}")]
    CouldntQueryDevice {
        property: String,
        error: String,
    }
}