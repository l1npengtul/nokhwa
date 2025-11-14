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
use crate::control::ControlValue;
pub use compact_str::CompactString;
pub use smallmap::Map;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

pub type PlatformSpecificFlag = u32;

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    flags: Map<CompactString, ControlValue>,
}

impl Metadata {
    #[must_use]
    pub fn new() -> Self {
        Self {
            flags: Map::default(),
        }
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&ControlValue> {
        self.flags.get(key)
    }

    pub fn insert(&mut self, key: CompactString, value: ControlValue) {
        self.flags.insert(key, value);
    }
}

impl Hash for Metadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in self.flags.iter() {
            state.write(key.as_bytes());
            value.hash(state);
        }
    }
}

impl Deref for Metadata {
    type Target = Map<CompactString, ControlValue>;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        for (this_key, this_value) in self.flags.iter() {
            if let Some(other_value) = other.flags.get(this_key) {
                if this_value != other_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

/// A buffer returned by a camera to accommodate custom decoding.
/// Contains information of Resolution, the buffer's [`FrameFormat`], and the buffer.
///
/// Note that decoding on the main thread **will** decrease your performance and lead to dropped frames.
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct FrameBuffer<'a> {
    buffer: Cow<'a, [u8]>,
    metadata: Option<Metadata>,
}

impl<'a> FrameBuffer<'a> {
    /// Creates a new buffer with a [`&[u8]`].
    #[must_use]
    pub fn new(buffer: Cow<'a, [u8]>, metadata: Option<Metadata>) -> Self {
        Self { buffer, metadata }
    }

    #[must_use]
    pub fn from_buffer(buffer: &'a [u8], metadata: Option<Metadata>) -> Self {
        FrameBuffer {
            buffer: Cow::Borrowed(buffer),
            metadata,
        }
    }

    #[must_use]
    pub fn from_cow(buffer: Cow<'a, [u8]>, metadata: Option<Metadata>) -> Self {
        FrameBuffer { buffer, metadata }
    }

    #[must_use]
    pub fn from_vec(buffer: Vec<u8>, metadata: Option<Metadata>) -> Self {
        FrameBuffer {
            buffer: Cow::Owned(buffer),
            metadata,
        }
    }

    /// Get the data of this buffer.
    #[must_use]
    pub fn buffer(&'a self) -> &'a [u8] {
        &self.buffer
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[must_use]
    pub fn deep_copy(&self) -> Self {
        Self {
            buffer: match &self.buffer {
                Cow::Borrowed(b) => Cow::Owned(b.to_vec()),
                Cow::Owned(o) => Cow::Owned(o.clone()),
            },
            metadata: self.metadata.clone(),
        }
    }

    #[must_use]
    pub fn consume(self) -> (Cow<'a, [u8]>, Option<Metadata>) {
        (self.buffer, self.metadata)
    }

    #[must_use]
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }
}

impl AsRef<[u8]> for FrameBuffer<'_> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl Deref for FrameBuffer<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.buffer.as_ref()
    }
}

impl<'a> From<&'a [u8]> for FrameBuffer<'a> {
    fn from(value: &'a [u8]) -> Self {
        FrameBuffer {
            buffer: Cow::Borrowed(value),
            metadata: None,
        }
    }
}

impl From<Vec<u8>> for FrameBuffer<'static> {
    fn from(value: Vec<u8>) -> Self {
        FrameBuffer {
            buffer: Cow::Owned(value),
            metadata: None,
        }
    }
}

impl<'a> From<Cow<'a, [u8]>> for FrameBuffer<'a> {
    fn from(value: Cow<'a, [u8]>) -> Self {
        FrameBuffer {
            buffer: value,
            metadata: None,
        }
    }
}
