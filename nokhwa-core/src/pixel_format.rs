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
use crate::buffer::Buffer;
use crate::frame_format::FrameFormat;
use image::{ImageBuffer, Luma, LumaA, Primitive, Rgb, Rgba};
use std::ops::Deref;

pub trait FormatDecoders: Send + Sync {
    const NAME: &'static str;

    const PRIMARY: FrameFormat;

    const ACCEPTABLE: &'static [FrameFormat];

    const PLATFORM_ACCEPTABLE: &'static [(&'static str, &'static [u128])];

    type Primitive: Primitive;

    type Container: Deref<Target = [Self::Primitive]>;
}

pub trait RgbDecoder: FormatDecoders {
    fn decode_rgb(&self, buffer: &Buffer) -> ImageBuffer<Rgb<Self::Primitive>, Self::Container>;
}

pub trait RgbADecoder: FormatDecoders {
    fn decode_rgba(&self, buffer: &Buffer) -> ImageBuffer<Rgba<Self::Primitive>, Self::Container>;
}

pub trait LumaDecoder: FormatDecoders {
    fn decode_luma(&self, buffer: &Buffer) -> ImageBuffer<Luma<Self::Primitive>, Self::Container>;
}

pub trait LumaADecoder: FormatDecoders {
    fn decode_luma_a(
        &self,
        buffer: &Buffer,
    ) -> ImageBuffer<LumaA<Self::Primitive>, Self::Container>;
}

// TODO: Wgpu Decoder
