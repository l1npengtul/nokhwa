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
use crate::error::NokhwaError;
use crate::types::{
    buf_mjpeg_to_rgb, buf_yuyv422_to_rgb, mjpeg_to_rgb, yuyv422_to_rgb, FrameFormat,
};
use image::{Luma, LumaA, Pixel, Rgb, Rgba};
use std::fmt::Debug;

/// Trait that has methods to convert raw data from the webcam to a proper raw image.
pub trait FormatDecoder: Clone + Sized + Send + Sync {
    type Output: Pixel<Subpixel = u8>;
    const FORMATS: &'static [FrameFormat];

    /// Allocates and returns a `Vec`
    /// # Errors
    /// If the data is malformed, or the source [`FrameFormat`] is incompatible, this will error.
    fn write_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError>;

    /// Writes to a user provided buffer.
    /// # Errors
    /// If the data is malformed, the source [`FrameFormat`] is incompatible, or the user-alloted buffer is not large enough, this will error.
    fn write_output_buffer(
        fcc: FrameFormat,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError>;
}

/// A Zero-Size-Type that contains the definition to convert a given image stream to an RGB888 in the [`Buffer`](crate::buffer::Buffer)'s [`.to_image()`](crate::buffer::Buffer::to_image)
///
/// ```.ignore
/// use image::{ImageBuffer, Rgb};
/// let image: ImageBuffer<Rgb<u8>, Vec<u8>> = buffer.to_image::<RgbFormat>();
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct RgbFormat;

impl FormatDecoder for RgbFormat {
    type Output = Rgb<u8>;
    const FORMATS: &'static [FrameFormat] = &[FrameFormat::MJPEG, FrameFormat::YUYV];

    fn write_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => mjpeg_to_rgb(data, false),
            FrameFormat::YUYV => yuyv422_to_rgb(data, false),
            FrameFormat::GRAY => Ok(data
                .iter()
                .flat_map(|x| {
                    let pxv = *x;
                    [pxv, pxv, pxv]
                })
                .collect()),
        }
    }

    fn write_output_buffer(
        fcc: FrameFormat,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => buf_mjpeg_to_rgb(data, dest, false),
            FrameFormat::YUYV => buf_yuyv422_to_rgb(data, dest, false),
            FrameFormat::GRAY => {
                if dest.len() != data.len() * 3 {
                    return Err(NokhwaError::ProcessFrameError {
                        src: fcc,
                        destination: "Luma => RGB".to_string(),
                        error: "Bad buffer length".to_string(),
                    });
                }

                data.iter().enumerate().for_each(|(idx, pixel_value)| {
                    let index = idx * 3;
                    dest[index] = *pixel_value;
                    dest[index + 1] = *pixel_value;
                    dest[index + 2] = *pixel_value;
                });
                Ok(())
            }
        }
    }
}

/// A Zero-Size-Type that contains the definition to convert a given image stream to an RGBA8888 in the [`Buffer`](crate::buffer::Buffer)'s [`.to_image()`](crate::buffer::Buffer::to_image)
///
/// ```.ignore
/// use image::{ImageBuffer, Rgba};
/// let image: ImageBuffer<Rgba<u8>, Vec<u8>> = buffer.to_image::<RgbAFormat>();
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct RgbAFormat;

impl FormatDecoder for RgbAFormat {
    type Output = Rgba<u8>;

    const FORMATS: &'static [FrameFormat] = &[FrameFormat::MJPEG, FrameFormat::YUYV];

    fn write_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => mjpeg_to_rgb(data, true),
            FrameFormat::YUYV => yuyv422_to_rgb(data, true),
            FrameFormat::GRAY => Ok(data
                .iter()
                .flat_map(|x| {
                    let pxv = *x;
                    [pxv, pxv, pxv, 255]
                })
                .collect()),
        }
    }

    fn write_output_buffer(
        fcc: FrameFormat,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => buf_mjpeg_to_rgb(data, dest, true),
            FrameFormat::YUYV => buf_yuyv422_to_rgb(data, dest, true),
            FrameFormat::GRAY => {
                if dest.len() != data.len() * 4 {
                    return Err(NokhwaError::ProcessFrameError {
                        src: fcc,
                        destination: "Luma => RGBA".to_string(),
                        error: "Bad buffer length".to_string(),
                    });
                }

                data.iter().enumerate().for_each(|(idx, pixel_value)| {
                    let index = idx * 4;
                    dest[index] = *pixel_value;
                    dest[index + 1] = *pixel_value;
                    dest[index + 2] = *pixel_value;
                    dest[index + 3] = 255;
                });
                Ok(())
            }
        }
    }
}

/// A Zero-Size-Type that contains the definition to convert a given image stream to an Luma8(Grayscale 8-bit) in the [`Buffer`](crate::buffer::Buffer)'s [`.to_image()`](crate::buffer::Buffer::to_image)
///
/// ```.ignore
/// use image::{ImageBuffer, Luma};
/// let image: ImageBuffer<Luma<u8>, Vec<u8>> = buffer.to_image::<LumaFormat>();
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct LumaFormat;

impl FormatDecoder for LumaFormat {
    type Output = Luma<u8>;

    const FORMATS: &'static [FrameFormat] =
        &[FrameFormat::MJPEG, FrameFormat::YUYV, FrameFormat::GRAY];

    #[allow(clippy::cast_possible_truncation)]
    fn write_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => Ok(mjpeg_to_rgb(data, false)?
                .as_slice()
                .chunks_exact(3)
                .map(|x| {
                    let mut avg = 0;
                    x.iter().for_each(|v| avg += u16::from(*v));
                    (avg / 3) as u8
                })
                .collect()),
            FrameFormat::YUYV => Ok(yuyv422_to_rgb(data, false)?
                .as_slice()
                .chunks_exact(3)
                .map(|x| {
                    let mut avg = 0;
                    x.iter().for_each(|v| avg += u16::from(*v));
                    (avg / 3) as u8
                })
                .collect()),
            FrameFormat::GRAY => Ok(data.to_vec()),
        }
    }

    fn write_output_buffer(
        fcc: FrameFormat,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => {
                // FIXME: implement!
                Err(NokhwaError::ProcessFrameError {
                    src: fcc,
                    destination: "Luma => RGB".to_string(),
                    error: "Conversion Error".to_string(),
                })
            }
            FrameFormat::YUYV => Err(NokhwaError::ProcessFrameError {
                src: fcc,
                destination: "Luma => RGB".to_string(),
                error: "Conversion Error".to_string(),
            }),
            FrameFormat::GRAY => {
                data.iter().zip(dest.iter_mut()).for_each(|(pxv, d)| {
                    *d = *pxv;
                });
                Ok(())
            }
        }
    }
}

/// A Zero-Size-Type that contains the definition to convert a given image stream to an LumaA8(Grayscale 8-bit with 8-bit alpha) in the [`Buffer`](crate::buffer::Buffer)'s [`.to_image()`](crate::buffer::Buffer::to_image)
///
/// ```.ignore
/// use image::{ImageBuffer, LumaA};
/// let image: ImageBuffer<LumaA<u8>, Vec<u8>> = buffer.to_image::<LumaAFormat>();
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct LumaAFormat;

impl FormatDecoder for LumaAFormat {
    type Output = LumaA<u8>;

    const FORMATS: &'static [FrameFormat] =
        &[FrameFormat::MJPEG, FrameFormat::YUYV, FrameFormat::GRAY];

    #[allow(clippy::cast_possible_truncation)]
    fn write_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => Ok(mjpeg_to_rgb(data, false)?
                .as_slice()
                .chunks_exact(3)
                .flat_map(|x| {
                    let mut avg = 0;
                    x.iter().for_each(|v| avg += u16::from(*v));
                    [(avg / 3) as u8, 255]
                })
                .collect()),
            FrameFormat::YUYV => Ok(yuyv422_to_rgb(data, false)?
                .as_slice()
                .chunks_exact(3)
                .flat_map(|x| {
                    let mut avg = 0;
                    x.iter().for_each(|v| avg += u16::from(*v));
                    [(avg / 3) as u8, 255]
                })
                .collect()),
            FrameFormat::GRAY => Ok(data.iter().flat_map(|x| [*x, 255]).collect()),
        }
    }

    fn write_output_buffer(
        fcc: FrameFormat,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => {
                // FIXME: implement!
                Err(NokhwaError::ProcessFrameError {
                    src: fcc,
                    destination: "MJPEG => LumaA".to_string(),
                    error: "Conversion Error".to_string(),
                })
            }
            FrameFormat::YUYV => Err(NokhwaError::ProcessFrameError {
                src: fcc,
                destination: "YUYV => LumaA".to_string(),
                error: "Conversion Error".to_string(),
            }),
            FrameFormat::GRAY => {
                if dest.len() != data.len() * 2 {
                    return Err(NokhwaError::ProcessFrameError {
                        src: fcc,
                        destination: "GRAY8 => LumaA".to_string(),
                        error: "Conversion Error".to_string(),
                    });
                }

                data.iter()
                    .zip(dest.chunks_exact_mut(2))
                    .enumerate()
                    .for_each(|(idx, (pxv, d))| {
                        let index = idx * 2;
                        d[index] = *pxv;
                        d[index + 1] = 255;
                    });
                Ok(())
            }
        }
    }
}
