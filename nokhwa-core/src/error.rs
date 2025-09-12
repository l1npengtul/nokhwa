/*
 * Copyright 2022 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
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
use crate::frame_format::{CustomFrameFormat, FrameFormat};
use std::fmt::Debug;
use thiserror::Error;
use crate::types::Backends;

pub type NokhwaResult<T> = Result<T, NokhwaError>;

/// All errors in `nokhwa`.
#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug, Clone)]
pub enum NokhwaError {
    #[error("Could not initialize {backend}: {error}")]
    InitializeError { backend: Backends, error: String },
    #[error("Could not shutdown {backend}: {error}")]
    ShutdownError { backend: Backends, error: String },
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
    UnsupportedOperationError(Backends),
    #[error("This operation is not implemented yet: {0}")]
    NotImplementedError(String),
    #[error("Failed To Convert: {0}")]
    ConversionError(String),
    #[error("Permission denied by user.")]
    PermissionDenied,
    #[error("Failed to decode: {0}")]
    Decoder(String),
    #[error("Unsupported FrameFormat: {0}")]
    DecoderUnsupportedFrameFormat(FrameFormat),
    #[error("The destination frame format from {0} to {1} is not supported.")]
    DecoderUnsupportedCustomFrameFormatDestination(CustomFrameFormat, FrameFormat),
    #[error("Unsupported pixel configuration {0} with width {1}b.")]
    DecoderUnsupportedDestinationPixelFormat(&'static str, u32),
    #[error("Bad decoder configuration: {0}")]
    DecoderInvalidConfiguration(String),
    #[error("Failed to initialize decoder: {0}")]
    DecoderInitializationError(String),
    #[error("Bad frame sent to the decoder: {0}")]
    DecoderInvalidFrameData(String),
    #[error("Bad buffer sent to decoder, did not write: {0}")]
    DecoderInvalidBuffer(String),
    #[error("You need to pass in a destination hint, it is not optional for this decoder.")]
    DecoderDestinationHintRequired,
    #[error("Decoder already deinitialized. Unusable, please make a new decoder.")]
    DecoderAlreadyDeinitialized,
}
//
// pub enum InitializeError {}
//
// pub enum QueryBackendError {}
//
// pub enum OpenDeviceError {}
//
// pub enum QueryDeviceError {}
//
// pub enum GetPropertyError {}
//
// pub enum SetPropertyError {}
//
// pub enum OpenStreamError {}
//
// pub enum CloseStreamError {}
//
// pub enum FrameError {}
//
// pub enum DecoderError {}
