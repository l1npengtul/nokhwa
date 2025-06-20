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

use std::fmt::{Display, Formatter};
use ordered_float::OrderedFloat;
// /// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC`.
// /// Note that endianness is determined by the native machine (or the driver itself).
// #[derive(Clone, Debug, Hash, PartialOrd, PartialEq)]
// #[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
// #[non_exhaustive]
// pub enum FrameFormat {
//     // Compressed Formats
//
//
//     // YCbCr Formats
//
//     // 8 bit per pixel, 4:4:4
//     Ayuv444,
//
//     // -> 4:2:2
//     Yuyv422, // AKA YUY2
//     Uyvy422, // UYUV
//     Yvyu422,
//     Yv12,
//
//     // 4:2:0
//     Nv12,
//     Nv21,
//     I420,
//
//     // 16:1:1
//     Yvu9,
//
//     // Grayscale Formats
//     Luma8,
//     Luma16,
//
//     // Depth
//     Depth16,
//
//     // RGB Formats
//     Rgb332,
//     Rgb888,
//     RgbA8888,
//     ARgb8888,
//     RgbX1010102,
//     RgbA1010102,
//     ARgb1010102,
//
//
//     Bgr888,
//     BgrA8888,
//     Bgr121212,
//     BgrA1212121212,
//
//     Bgr161616,
//     Bgr16161616,
//
//
//     // Bayer Formats
//     Bayer8,
//     Bayer16,
//
//     // Custom
//     Custom(CustomFrameFormat),
// }

macro_rules! define_frame_format_with_groups {
    (
        $(
            $group_name:ident => [
                $($format:ident),* $(,)?
            ]
        ),* $(,)?
    ) => {
        /// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC`.
        /// Note that endianness is determined by the native machine (or the driver itself), unless otherwise
        /// specified.
        ///
        /// Note that compatibility is driver dependant, while some drivers (such as Web) may not respect the setting
        /// at all.
        #[derive(Copy, Clone, Debug, Hash, PartialOrd, PartialEq)]
        #[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
        #[non_exhaustive]
        #[allow(non_camel_case_types)]
        pub enum FrameFormat {
            $(
                $($format,)*
            )*
            Custom(CustomFrameFormat),
        }

        impl FrameFormat {
            $(
                pub const $group_name: &'static [FrameFormat] = &[
                    $(FrameFormat::$format),*
                ];
            )*
            pub const ALL: &'static [FrameFormat] = &[
                $(
                    $(FrameFormat::$format,)*
                )*
            ];
        }
    };
}

define_frame_format_with_groups! {
    COMPRESSED => [
        H265,
        HEVC,
        H264,
        AVC1,
        H263,
        AV1,
        MPEG_1,
        MPEG_2,
        MPEG_4,
        MJPEG,
        XviD,
        VP8,
        VP9,
    ],

    YCBCR_PACKED_444 => [
        Ayuv_32,
    ],
    YCBCR_PLANAR_444 => [],
    YCBCR_SEMI_PLANAR_444 => [
        NV24,
        NV42,
    ],
    YCBCR_PACKED_422 => [
        Yuyv_4_2_2,
        Uyvy_4_2_2,
        Vyuy_4_2_2,
        Yvyu_4_2_2,
        Y210,
        Y216,
    ],
    YCBCR_PLANAR_422 => [],
    YCBCR_SEMI_PLANAR_422 => [
        NV16,
        NV61,
    ],
    YCBCR_PACKED_420 => [],
    YCBCR_PLANAR_420 => [
        Yuv_4_2_0,
        Yvu_4_2_0,
    ],
    YCBCR_SEMI_PLANAR_420 => [
        NV12,
        NV21,
        P010,
        P012,

    ],
    YCBCR_PACKED_411 => [
        Y41Packed
    ],
    YCBCR_PLANAR_411 => [
        Y411Planar
    ],
    YCBCR_SEMI_PLANAR_411 => [
        NV11,
    ],

    LUMA => [
        Luma_8,
        Luma_10,
        Luma_12,
        Luma_14,
        Luma_16,
        Depth_16,
    ],

    RAW_RGB => [
        Rgb_3_3_2,
        Rgb_5_6_5,
        Rgb_5_5_5,
        Rgb_8_8_8,
        Argb_8_8_8_8,
        Rgba_8_8_8_8,
    ],

    RAW_BGR => [
        Bgr_3_3_2,
        Bgr_5_6_5,
        Bgr_5_5_5,
        Bgr_8_8_8,
        Abgr_8_8_8_8,
        Bgra_8_8_8_8,
    ]
}

// define_frame_format_groups! {
//     ALL => [
//             // Compressed Formats
//         H265,
//         H264,
//         Avc1,
//         H263,
//         Av1,
//         Mpeg1,
//         Mpeg2,
//         Mpeg4,
//         MJpeg,
//         XVid,
//         VP8,
//         VP9,
//
//         // YCbCr Formats
//
//         // 8 bit per pixel, 4:4:4
//         Ayuv444,
//
//         // -> 4:2:2
//         Yuyv422, // AKA YUY2
//         Uyvy422, // UYUV
//         Yvyu422,
//         Yv12,
//
//         // 4:2:0
//         Nv12,
//         Nv21,
//         I420,
//
//         // 16:1:1
//         Yvu9,
//
//         // Grayscale Formats
//         Luma8,
//         Luma16,
//
//         // Depth
//         Depth16,
//
//         // RGB Formats
//         Rgb332,
//         Rgb888,
//
//         Bgr888,
//         BgrA8888,
//
//         RgbA8888,
//         ARgb8888,
//
//         // Bayer Formats
//         Bayer8,
//         Bayer16,
//     ],
//     COMPRESSED => [
//         H265,
//         H264,
//         Avc1,
//         H263,
//         Av1,
//         Mpeg1,
//         Mpeg2,
//         Mpeg4,
//         MJpeg,
//         XVid,
//         VP8,
//         VP9,
//     ],
//     YCBCR => [
//         Ayuv444,
//
//         // -> 4:2:2
//         Yuyv422, // AKA YUY2
//         Uyvy422, // UYUV
//         Yvyu422,
//         Yv12,
//
//         // 4:2:0
//         Nv12,
//         Nv21,
//         I420,
//
//         // 16:1:1
//         Yvu9,
//     ],
//     YCBCR_PACKED => [
//         Ayuv444,
//
//         // -> 4:2:2
//         Yuyv422, // AKA YUY2
//         Uyvy422, // UYUV
//         Yvyu422,
//
//         // 4:2:0
//         Nv12,
//         Nv21,
//         I420,
//
//     ],
//     YCBCR_PLANAR => [        Yvu9,
//                 Yv12,
// ],
//     LUMA => [
//         Luma8, Luma16
//     ],
//     RGB => [
//         Rgb332, RgbA8888
//     ],
//     COLOR_FORMATS => [
//         H265, H264, H263, Av1, Avc1, Mpeg1, Mpeg2, Mpeg4, MJpeg, XVid,
//         VP8, VP9, Yuyv422, Uyvy422, Nv12, Nv21, Yv12, Rgb332, RgbA8888
//     ],
//     GRAYSCALE => [
//         Luma8, Luma16, Depth16,
//     ],
//     RAW => [
//         // RGB Formats
//         Rgb332,
//         Rgb888,
//
//         Bgr888,
//         BgrA8888,
//
//         RgbA8888,
//         ARgb8888,],
//     BAYER => [
//     // Bayer Formats
//     Bayer8,
//     Bayer16,],
// }

impl Display for FrameFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialOrd, PartialEq)]
pub enum CustomFrameFormat {
    UUID(u128),
    FourCC([char; 4]),
    U32(u32),
    U64(u64),
    F32(OrderedFloat<f32>),
    F64(OrderedFloat<f64>),
}

#[macro_export]
macro_rules! define_back_and_fourth_frame_format {
    ($fourcc_type:ty, { $( $frame_format:expr => $value:literal, )* }, $func_u8_8_to_fcc:expr, $func_fcc_to_u8_8:expr, $value_to_fcc_type:expr) => {
        pub struct FrameFormatIntermediate(pub $fourcc_type);

        impl FrameFormatIntermediate {
            pub fn from_frame_format(frame_format: FrameFormat) -> Option<Self> {
                match frame_format {
                    $(
                        $frame_format => Some(Self($value_to_fcc_type($value))),
                    )*
                    FrameFormat::Custom(cv) => Some($func_u8_8_to_fcc(cv))
                    _ => None,
                }
            }

            pub fn into_frame_format(fourcc: $fourcc_type) -> FrameFormat {
                match fourcc.0 {
                    $(
                         $value => $frame_format,
                    )*
                    cv => FrameFormat::Custom($func_fcc_to_u8_8(cv)),
                }
            }
        }
    };
}
