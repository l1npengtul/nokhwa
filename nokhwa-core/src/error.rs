use crate::control::{ControlId, ControlValue};
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
use crate::pixel_destination::PixelDestination;
use crate::types::{Backends, CameraIndex};
use std::fmt::{Debug, Display};
use std::num::ParseIntError;
use thiserror::Error;

pub type NokhwaResult<T> = Result<T, NokhwaError>;

/// All errors in `nokhwa`.
#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug, Clone)]
pub enum NokhwaError {
    // NokhwaCore Errors
    #[error("Failed to parse string index to u32: {0}")]
    IndexParsingFailed(ParseIntError),

    // Platform Errors
    #[error("Could not initialize {backend}: {error}")]
    InitializeError { backend: Backends, error: String },
    #[error("Could not open device {0}: {1}")]
    OpenDeviceError(CameraIndex, String),
    #[error("Failed to query for cameras: {0}")]
    QueryError(String),

    // Camera Errors
    #[error("Failed to list FrameFormats: {0}")]
    ListFourCCError(String),
    #[error("Failed to list Resolutions: {0}")]
    ListResolutionError(String),
    #[error("Failed to list FrameRates: {0}")]
    ListFrameRatesError(String),
    #[error("Failed to list Controls: {0}")]
    ListControlError(String),
    #[error("{0:?} is an invalid control id: {1}")]
    InvalidControlId(ControlId, String),
    #[error("{0:?} is an invalid control value.")]
    InvalidControlValue(ControlValue),
    #[error("{0:?} is an invalid frame format: {1}")]
    InvalidFrameFormat(FrameFormat, String),
    #[error("Failed to get control descriptor for {0}: {1}")]
    FailedToGetControlDescriptor(ControlId, String),
    #[error("Could not shutdown {backend} device {device}: {error}")]
    ShutdownError {
        backend: Backends,
        device: CameraIndex,
        error: String,
    },

    // Stream Related Errors
    #[error("Could not open device stream: {0}")]
    OpenStreamError(String),
    #[error("Error occured during stream: {0:?}")]
    StreamError(StreamError),
    #[error("Could not stop stream: {0}")]
    StreamShutdownError(String),
    #[error("Device no longer exists: {0}")]
    DeviceNoLongerExists(CameraIndex),

    // Not-Implemented Errors
    #[error("This operation is not supported by backend {0}.")]
    UnsupportedOperationError(Backends),
    #[error("This operation is not implemented yet: {0}")]
    NotImplementedError(String),

    // Decoders
    #[error("Failed to decode: {0}")]
    Decoder(String),
    #[error("Unsupported FrameFormat: {0}")]
    DecoderUnsupportedFrameFormat(FrameFormat),
    #[error("The destination frame format from {0} to {1} is not supported.")]
    DecoderUnsupportedCustomFrameFormatDestination(CustomFrameFormat, FrameFormat),
    #[error("Unknown pixel configuration {0} with width {1}b.")]
    DecoderUnknownDestinationPixelFormat(&'static str, u32),
    #[error("Unsupported pixel configuration {0}.")]
    DecoderUnsupportedDestinationPixelFormat(PixelDestination),
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
    #[error(
        "Decoder requires more data to process - not actual error - please send more data to decode: {0}"
    )]
    DecoderNeedsMoreData(String),
}

impl NokhwaError {
    #[must_use]
    pub fn is_needs_more(&self) -> bool {
        matches!(self, NokhwaError::DecoderNeedsMoreData(_))
    }
}

/// Errors that may occur during a stream
#[derive(Error, Debug, Clone)]
pub enum StreamError {
    NotReady,
    NoLongerExists,
    StreamInvalidated,
    Other(String),
}

impl Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
