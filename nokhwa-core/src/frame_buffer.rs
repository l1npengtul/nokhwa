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
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use crate::frame_format::FrameFormat;
use small_map::{FxSmallMap, Iter};
use crate::control::ControlValue;

pub use compact_str::CompactString;

pub type PlatformSpecificFlag = u32;

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    flags: FxSmallMap<8, CompactString, ControlValue>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            flags: Default::default(),
        }
    }

    pub fn get(&self, key: CompactString) -> Option<&ControlValue> {
        self.flags.get(&key)
    }

    pub fn insert(&mut self, key: CompactString, value: ControlValue) {
        self.flags.insert(key, value);
    }

    pub fn iter(&self) -> Iter<'_, 8, CompactString, ControlValue> {
        self.flags.iter()
    }
}

impl Hash for Metadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in self.flags {
            state.write(key.as_bytes());
            value.hash(state);
        }
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        for (this_key, this_value) in &self.flags {
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
pub struct FrameBuffer {
    buffer: Cow<'static, [u8]>,
    metadata: Option<Metadata>,
}

impl FrameBuffer {
    /// Creates a new buffer with a [`&[u8]`].
    #[must_use]
    #[inline]
    pub fn new(buffer: Cow<'static, [u8]>, metadata: Option<Metadata>) -> Self {
        Self {
            buffer,
            metadata,
        }
    }

    /// Get the data of this buffer.
    #[must_use]
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn consume(self) -> (Cow<'static, [u8]>, Option<Metadata>) {
        return (self.buffer, self.metadata)
    }

    #[must_use]
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

}
