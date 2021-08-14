/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use thiserror::Error;

use crate::{CaptureAPIBackend, FrameFormat};

/// All errors in `nokhwa`.
#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug, Clone)]
pub enum NokhwaError {
    #[error("Could not initialize {backend}: {error}")]
    InitializeError {
        backend: CaptureAPIBackend,
        error: String,
    },
    #[error("Could not shutdown {backend}: {error}")]
    ShutdownError {
        backend: CaptureAPIBackend,
        error: String,
    },
    #[error("Error: {0}")]
    GeneralError(String),
    #[error("Could not generate required structure {structure}: {error}")]
    StructureError { structure: String, error: String },
    #[error("Could not open device {0}: {1}")]
    OpenDeviceError(String, String),
    #[error("Could not get device property {property}: {error}")]
    GetPropertyError { property: String, error: String },
    #[error("Could not set device property {property} with value {value}: {error}")]
    SetPropertyError {
        property: String,
        value: String,
        error: String,
    },
    #[error("Could not open device stream: {0}")]
    OpenStreamError(String),
    #[error("Could not capture frame: {0}")]
    ReadFrameError(String),
    #[error("Could not process frame {src} to {destination}: {error}")]
    ProcessFrameError {
        src: FrameFormat,
        destination: String,
        error: String,
    },
    #[error("Could not stop stream: {0}")]
    StreamShutdownError(String),
    #[error("This operation is not supported by backend {0}.")]
    UnsupportedOperationError(CaptureAPIBackend),
    #[error("This operation is not implemented yet: {0}")]
    NotImplementedError(String),
}
#[cfg(feature = "input-msmf")]
use nokhwa_bindings_windows::BindingError;

#[cfg(feature = "input-msmf")]
impl From<nokhwa_bindings_windows::BindingError> for NokhwaError {
    fn from(err: BindingError) -> Self {
        match err {
            BindingError::InitializeError(error) => NokhwaError::InitializeError {
                backend: CaptureAPIBackend::MediaFoundation,
                error,
            },
            BindingError::DeInitializeError(error) => NokhwaError::ShutdownError {
                backend: CaptureAPIBackend::MediaFoundation,
                error,
            },
            BindingError::GUIDSetError(property, value, error) => NokhwaError::SetPropertyError {
                property,
                value,
                error,
            },
            BindingError::GUIDReadError(property, error) => {
                NokhwaError::GetPropertyError { property, error }
            }
            BindingError::AttributeError(error) => NokhwaError::StructureError {
                structure: "IMFAttribute".to_string(),
                error,
            },
            BindingError::EnumerateError(error) => NokhwaError::GetPropertyError {
                property: "Devices".to_string(),
                error,
            },
            BindingError::DeviceOpenFailError(device, error) => {
                NokhwaError::OpenDeviceError(device.to_string(), error)
            }
            BindingError::ReadFrameError(error) => NokhwaError::ReadFrameError(error),
            BindingError::NotImplementedError => {
                NokhwaError::NotImplementedError("Docs-Only MediaFoundation".to_string())
            }
        }
    }
}
