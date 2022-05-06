/*
 * Copyright 2021 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::{CaptureAPIBackend, FrameFormat};
use thiserror::Error;

/// All errors in `nokhwa`.
#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug, Clone)]
pub enum NokhwaError {
    #[error("Unitialized Camera. Call `init()` first!")]
    UnitializedError,
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

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
use nokhwa_bindings_windows::BindingError;

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<BindingError> for NokhwaError {
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
                NokhwaError::OpenDeviceError(device, error)
            }
            BindingError::ReadFrameError(error) => NokhwaError::ReadFrameError(error),
            BindingError::NotImplementedError => {
                NokhwaError::NotImplementedError("Docs-Only MediaFoundation".to_string())
            }
        }
    }
}

#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
))]
use nokhwa_bindings_macos::AVFError;

#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
))]
impl From<AVFError> for NokhwaError {
    fn from(avf_error: AVFError) -> Self {
        match avf_error {
            AVFError::InvalidType { expected, found } => NokhwaError::GetPropertyError {
                property: format!("type of {}", expected),
                error: format!("Invalid type, found {}", found),
            },
            AVFError::InvalidValue { found } => NokhwaError::GetPropertyError {
                property: found,
                error: "Invalid Value".to_string(),
            },
            AVFError::AlreadyBusy(why) => {
                NokhwaError::GeneralError(format!("Already Busy: {}", why))
            }
            AVFError::FailedToOpenDevice { index, why } => {
                NokhwaError::OpenDeviceError(index.to_string(), why)
            }
            AVFError::ConfigNotAccepted => NokhwaError::SetPropertyError {
                property: "Configuration".to_string(),
                value: "Invalid".to_string(),
                error: "Rejected by AVFoundation".to_string(),
            },
            AVFError::General(why) => {
                NokhwaError::GeneralError(format!("AVFoundation Error: {}", why))
            }
            AVFError::RejectedInput => {
                NokhwaError::OpenStreamError("AVFoundation Input Rejection".to_string())
            }
            AVFError::RejectedOutput => {
                NokhwaError::OpenStreamError("AVFoundation Output Rejection".to_string())
            }
            AVFError::StreamOpen(why) => NokhwaError::OpenStreamError(why),
            AVFError::ReadFrame(why) => NokhwaError::ReadFrameError(why),
            AVFError::NotSupported => {
                NokhwaError::UnsupportedOperationError(CaptureAPIBackend::AVFoundation)
            }
        }
    }
}
