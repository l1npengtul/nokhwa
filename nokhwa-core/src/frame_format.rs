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

use crate::{buffer::Buffer, types::ApiBackend};
use image::{ImageBuffer, Pixel};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    ops::Deref,
};

/// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC`.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameFormat {
    // Compressed Formats
    H265,
    H264,
    H263,
    Avc1,
    Mpeg1,
    Mpeg2,
    Mpeg4,
    MJpeg,
    XVid,
    VP8,
    VP9,

    // YCbCr formats

    // -> 422 16 BPP
    Yuv422,
    Uyv422,

    // 420
    Nv12,
    Nv21,
    Yv12,

    // Grayscale Formats
    Luma8,

    // RGB Formats
    Rgb8,
    RgbA8,

    // Custom
    Custom(u128),
}

impl FrameFormat {
    pub const ALL: &'static [FrameFormat] = &[
        FrameFormat::H263,
        FrameFormat::H264,
        FrameFormat::H265,
        FrameFormat::Avc1,
        FrameFormat::Mpeg1,
        FrameFormat::Mpeg2,
        FrameFormat::Mpeg4,
        FrameFormat::MJpeg,
        FrameFormat::XVid,
        FrameFormat::VP8,
        FrameFormat::VP9,
        FrameFormat::Yuv422,
        FrameFormat::Uyv422,
        FrameFormat::Nv12,
        FrameFormat::Nv21,
        FrameFormat::Yv12,
        FrameFormat::Imc2,
        FrameFormat::Imc4,
        FrameFormat::Luma8,
        FrameFormat::Rgb8,
        FrameFormat::RgbA8,
    ];

    pub const COMPRESSED: &'static [FrameFormat] = &[
        FrameFormat::H263,
        FrameFormat::H264,
        FrameFormat::H265,
        FrameFormat::Avc1,
        FrameFormat::Mpeg1,
        FrameFormat::Mpeg2,
        FrameFormat::Mpeg4,
        FrameFormat::MJpeg,
        FrameFormat::XVid,
        FrameFormat::VP8,
        FrameFormat::VP9,
    ];

    pub const CHROMA: &'static [FrameFormat] = &[
        FrameFormat::Yuv422,
        FrameFormat::Uyv422,
        FrameFormat::Nv12,
        FrameFormat::Nv21,
        FrameFormat::Yv12,
        FrameFormat::Imc2,
        FrameFormat::Imc4,
    ];

    pub const LUMA: &'static [FrameFormat] = &[FrameFormat::Luma8];

    pub const RGB: &'static [FrameFormat] = &[FrameFormat::Rgb8, FrameFormat::RgbA8];
}

impl Display for FrameFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlatformFrameFormat {
    backend: ApiBackend,
    format: u128,
}

impl PlatformFrameFormat {
    pub fn new(backend: ApiBackend, format: u128) -> Self {
        Self { backend, format }
    }

    pub fn backend(&self) -> ApiBackend {
        self.backend
    }

    pub fn format(&self) -> u128 {
        self.format
    }

    pub fn as_tuple(&self) -> (ApiBackend, u128) {
        (self.backend, self.format)
    }
}

impl From<(ApiBackend, u128)> for PlatformFrameFormat {
    fn from(value: (ApiBackend, u128)) -> Self {
        PlatformFrameFormat::new(value.0, value.1)
    }
}

impl From<PlatformFrameFormat> for (ApiBackend, u128) {
    fn from(value: PlatformFrameFormat) -> Self {
        value.as_tuple()
    }
}

impl PartialEq<(ApiBackend, u128)> for PlatformFrameFormat {
    fn eq(&self, other: &(ApiBackend, u128)) -> bool {
        &self.as_tuple() == other
    }
}

impl AsRef<(ApiBackend, u128)> for PlatformFrameFormat {
    fn as_ref(&self) -> &(ApiBackend, u128) {
        &self.as_tuple()
    }
}

impl Display for PlatformFrameFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// The Source Format of a [`Buffer`].
///
/// May either be a platform specific FourCC, or a FrameFormat
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SourceFrameFormat {
    FrameFormat(FrameFormat),
    PlatformSpecific(PlatformFrameFormat),
}

impl From<FrameFormat> for SourceFrameFormat {
    fn from(value: FrameFormat) -> Self {
        SourceFrameFormat::FrameFormat(value)
    }
}

impl From<(ApiBackend, u128)> for SourceFrameFormat {
    fn from(value: (ApiBackend, u128)) -> Self {
        SourceFrameFormat::PlatformSpecific(value.into())
    }
}

impl From<PlatformFrameFormat> for SourceFrameFormat {
    fn from(value: PlatformFrameFormat) -> Self {
        SourceFrameFormat::PlatformSpecific(value)
    }
}

impl PartialEq<FrameFormat> for SourceFrameFormat {
    fn eq(&self, other: &FrameFormat) -> bool {
        if let SourceFrameFormat::FrameFormat(ff) = self {
            ff == other
        } else {
            false
        }
    }
}

impl PartialEq<(ApiBackend, u128)> for SourceFrameFormat {
    fn eq(&self, other: &(ApiBackend, u128)) -> bool {
        if let SourceFrameFormat::PlatformSpecific(pff) = self {
            pff == other
        }  else {
            false
        }
    }
}
impl PartialEq<PlatformFrameFormat> for SourceFrameFormat {
    fn eq(&self, other: &PlatformFrameFormat) -> bool {
        if let SourceFrameFormat::PlatformSpecific(pff) = self {
            pff == other
        }  else {
            false
        }
    }
}

impl Display for SourceFrameFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub trait FormatDecoders<T: Pixel, E: Error>: Send + Sync {
    const NAME: &'static str;

    const PRIMARY: &'static [FrameFormat];

    const PLATFORM_ACCEPTABLE: &'static [(ApiBackend, &'static [u128])];

    type Container: Deref<Target = [T::Subpixel]>;

    fn decode(&self, buffer: &Buffer) -> Result<ImageBuffer<T, Self::Container>, E>;
}

// TODO: Wgpu Decoder

// TODO: OpenCV Mat Decoder
