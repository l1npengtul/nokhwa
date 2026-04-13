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

use ordered_float::OrderedFloat;
use std::fmt::{Display, Formatter};
pub use uuid::Uuid;

macro_rules! define_frame_format_with_groups {
    (
        $(
            $classifier:ident [
                $(
                    $sub_classifier:ident [
                        $(
                            $group_name:expr => [
                                $($format:ident),* $(,)?
                            ]
                        ),* $(,)?
                    ]
                ),* $(,)?
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
                $(
                    $($($format,)*)*
                )*
            )*
            Custom(CustomFrameFormat),
        }

        paste::paste! {
            impl FrameFormat {
            $(
                    pub const $classifier: &'static [FrameFormat] = &[
                        $($($(FrameFormat::$format,)*)*)*
                    ];
                    $(

                        pub const [<$classifier _ $sub_classifier>]: &'static [FrameFormat] = &[
                            $($(FrameFormat::$format,)*)*
                        ];
                        $(
                        pub const [<$classifier _ $sub_classifier _ $group_name>]: &'static [FrameFormat] = &[
                                $(FrameFormat::$format,)*
                            ];
                        )*
                    )*
                )*
                pub const ALL: &'static [FrameFormat] = &[
                    $($(
                        $($(FrameFormat::$format,)*)*
                    )*)*
                ];
            }
        }
    };
}

define_frame_format_with_groups! {
    COMPRESSED [
        MPEG [
            H => [
                H265,
                HEVC,
                H264,
                H263,
            ],
            MPEG4 => [
                AVC1,
                MPEG_4,
                XviD,
            ],
            MPEG => [
                MPEG_1,
                MPEG_2,
            ],
        ],
        IMAGE [
            MJPEG => [
                MJPEG,
            ]
        ],
        OPEN [
            AOM => [
                AV1
            ],
            WEB => [
                VP8,
                VP9
            ],
        ]
    ],

    YCBCR [
        PACKED [
            444 => [
                Ayuv_32,
            ],
            422 => [
                Yuyv_4_2_2,
                Uyvy_4_2_2,
                Vyuy_4_2_2,
                Yvyu_4_2_2,
            ],
            420 => [],
            411 => [
            ]
        ],
        PLANAR [
            444 => [],
            422 => [],
            420 => [
                Yuv_4_2_0,
            ],
            411 => [
            ]
        ],
        SEMI [
            444 => [
                NV24,
                NV42,
            ],
            422 => [
                NV16,
                NV61,
            ],
            420 => [
                NV12,
                NV21,
                P010,
                P012,
            ],
            411 => [
            ]
        ],
    ],

    BRIGHTNESS [
        LUMA [
            SMALL => [
                Luma_8,
            ],
            LARGE => [
                Luma_10,
                Luma_12,
                Luma_14,
                Luma_16,
            ],
        ],
        DEPTH [
            SMALL => [],
            LARGE => [Depth_16],
        ]
    ],

    RAW [
        RGB [
            NO_ALPHA => [
                Rgb_3_3_2,
                Rgb_5_6_5,
                Rgb_5_5_5,
                Rgb_8_8_8,
            ],
            WITH_ALPHA => [
                Argb_8_8_8_8,
                Rgba_8_8_8_8,
            ]
        ],
        BGR [
            NO_ALPHA => [
                Bgr_3_3_2,
                Bgr_5_6_5,
                Bgr_5_5_5,
                Bgr_8_8_8,
            ],
            WITH_ALPHA => [
                Abgr_8_8_8_8,
                Bgra_8_8_8_8,
            ]
        ]
    ]
}

impl FrameFormat {
    #[must_use]
    pub fn is_custom(&self) -> bool {
        if let FrameFormat::Custom(_) = self {
            return true;
        }
        false
    }
}

impl Display for FrameFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum CustomFrameFormat {
    UUID(Uuid),
    FourCC([u8; 4]),
    U32(u32),
    U64(u64),
    F32(OrderedFloat<f32>),
    F64(OrderedFloat<f64>),
}

impl Display for CustomFrameFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
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
