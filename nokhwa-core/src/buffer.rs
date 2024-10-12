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

use crate::types::Resolution;
use bytes::Bytes;
use crate::frame_format::FrameFormat;

/// A buffer returned by a camera to accommodate custom decoding.
/// Contains information of Resolution, the buffer's [`FrameFormat`], and the buffer.
///
/// Note that decoding on the main thread **will** decrease your performance and lead to dropped frames.
#[derive(Clone, Debug, Hash, PartialOrd, PartialEq, Eq)]
pub struct Buffer {
    resolution: Resolution,
    buffer: Bytes,
    source_frame_format: FrameFormat,
}

impl Buffer {
    /// Creates a new buffer with a [`&[u8]`].
    #[must_use]
    #[inline]
    pub fn new(res: Resolution, buf: &[u8], source_frame_format: FrameFormat) -> Self {
        Self {
            resolution: res,
            buffer: Bytes::copy_from_slice(buf),
            source_frame_format,
        }
    }

    /// Get the [`Resolution`] of this buffer.
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Get the data of this buffer.
    #[must_use]
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Get an owned version of this buffer. Note: This is the equivalent 
    #[must_use]
    pub fn buffer_bytes(&self) -> Bytes {
        self.buffer.clone()
    }

    /// Get the [`SourceFrameFormat`] of this buffer.
    #[must_use]
    pub fn source_frame_format(&self) -> FrameFormat {
        self.source_frame_format
    }
}
