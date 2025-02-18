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
    buf_bgra_to_rgb, buf_mjpeg_to_rgb, buf_nv12_to_rgb, buf_yuyv422_to_rgb, color_frame_formats,
    frame_formats, mjpeg_to_rgb, nv12_to_rgb, yuyv422_to_rgb, FrameFormat, Resolution,
};
use image::{Luma, LumaA, Pixel, Rgb, Rgba};
use mozjpeg::Format;
use std::fmt::Debug;

/// Trait that has methods to convert raw data from the webcam to a proper raw image.
pub trait FormatDecoder: Clone + Sized + Send + Sync {
    type Output: Pixel<Subpixel = u8>;
    const FORMATS: &'static [FrameFormat];

    /// Allocates and returns a `Vec`
    /// # Errors
    /// If the data is malformed, or the source [`FrameFormat`] is incompatible, this will error.
    fn write_output(
        fcc: FrameFormat,
        resolution: Resolution,
        data: &[u8],
    ) -> Result<Vec<u8>, NokhwaError>;

    /// Writes to a user provided buffer.
    /// # Errors
    /// If the data is malformed, the source [`FrameFormat`] is incompatible, or the user-alloted buffer is not large enough, this will error.
    fn write_output_buffer(
        fcc: FrameFormat,
        resolution: Resolution,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError>;
}

/// A Zero-Size-Type that contains the definition to convert a given image stream to an RGB888 in the [`Buffer`](crate::buffer::Buffer)'s [`.decode_image()`](crate::buffer::Buffer::decode_image)
///
/// ```.ignore
/// use image::{ImageBuffer, Rgb};
/// let image: ImageBuffer<Rgb<u8>, Vec<u8>> = buffer.to_image::<RgbFormat>();
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct RgbFormat;

impl FormatDecoder for RgbFormat {
    type Output = Rgb<u8>;
    const FORMATS: &'static [FrameFormat] = color_frame_formats();

    #[inline]
    fn write_output(
        fcc: FrameFormat,
        resolution: Resolution,
        data: &[u8],
    ) -> Result<Vec<u8>, NokhwaError> {
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
            FrameFormat::RAWRGB => Ok(data.to_vec()),
            FrameFormat::NV12 => nv12_to_rgb(resolution, data, false),
            FrameFormat::BGRA => {
                let mut rgb = vec![0u8; data.len()];
                data.chunks_exact(4).enumerate().for_each(|(idx, px)| {
                    let index = idx * 3;
                    rgb[index] = px[2];
                    rgb[index + 1] = px[1];
                    rgb[index + 2] = px[0];
                });
                Ok(rgb)
            }
        }
    }

    #[inline]
    fn write_output_buffer(
        fcc: FrameFormat,
        resolution: Resolution,
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
            FrameFormat::RAWRGB => {
                dest.copy_from_slice(data);
                Ok(())
            }
            FrameFormat::NV12 => buf_nv12_to_rgb(resolution, data, dest, false),
            FrameFormat::BGRA => buf_bgra_to_rgb(resolution, data, dest),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct BgraFormat;

impl FormatDecoder for BgraFormat {
    /// TODO: Image has to be changed to support BGRA
    type Output = Rgba<u8>;

    const FORMATS: &'static [FrameFormat] = color_frame_formats();

    #[inline]
    fn write_output(
        fcc: FrameFormat,
        resolution: Resolution,
        data: &[u8],
    ) -> Result<Vec<u8>, NokhwaError> {
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
            FrameFormat::RAWRGB => Ok(data
                .chunks_exact(3)
                .flat_map(|x| [x[0], x[1], x[2], 255])
                .collect()),
            FrameFormat::NV12 => nv12_to_rgb(resolution, data, true),
            FrameFormat::BGRA => Ok(data.to_vec()),
        }
    }

    #[inline]
    fn write_output_buffer(
        fcc: FrameFormat,
        resolution: Resolution,
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
            FrameFormat::RAWRGB => {
                data.chunks_exact(3).enumerate().for_each(|(idx, px)| {
                    let index = idx * 4;
                    dest[index] = px[0];
                    dest[index + 1] = px[1];
                    dest[index + 2] = px[2];
                    dest[index + 3] = 255;
                });
                Ok(())
            }
            FrameFormat::NV12 => buf_nv12_to_rgb(resolution, data, dest, true),
            FrameFormat::BGRA => {
                // If dest < data, only copy what fits
                if dest.len() < data.len() {
                    dest.copy_from_slice(data.get(..dest.len()).unwrap());
                } else {
                    dest.copy_from_slice(data);
                }
                Ok(())
            }
        }
    }
}

/// A Zero-Size-Type that contains the definition to convert a given image stream to an RGBA8888 in the [`Buffer`](crate::buffer::Buffer)'s [`.decode_image()`](crate::buffer::Buffer::decode_image)
///
/// ```.ignore
/// use image::{ImageBuffer, Rgba};
/// let image: ImageBuffer<Rgba<u8>, Vec<u8>> = buffer.to_image::<RgbAFormat>();
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct RgbAFormat;

impl FormatDecoder for RgbAFormat {
    type Output = Rgba<u8>;

    const FORMATS: &'static [FrameFormat] = color_frame_formats();

    #[inline]
    fn write_output(
        fcc: FrameFormat,
        resolution: Resolution,
        data: &[u8],
    ) -> Result<Vec<u8>, NokhwaError> {
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
            FrameFormat::RAWRGB => Ok(data
                .chunks_exact(3)
                .flat_map(|x| [x[0], x[1], x[2], 255])
                .collect()),
            FrameFormat::NV12 => nv12_to_rgb(resolution, data, true),
            FrameFormat::BGRA => {
                let mut rgba = vec![0u8; data.len()];
                data.chunks_exact(4).enumerate().for_each(|(idx, px)| {
                    let index = idx * 4;
                    rgba[index] = px[2];
                    rgba[index + 1] = px[1];
                    rgba[index + 2] = px[0];
                    rgba[index + 3] = px[3];
                });
                Ok(rgba)
            }
        }
    }

    #[inline]
    fn write_output_buffer(
        fcc: FrameFormat,
        resolution: Resolution,

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
            FrameFormat::RAWRGB => {
                data.chunks_exact(3).enumerate().for_each(|(idx, px)| {
                    let index = idx * 4;
                    dest[index] = px[0];
                    dest[index + 1] = px[1];
                    dest[index + 2] = px[2];
                    dest[index + 3] = 255;
                });
                Ok(())
            }
            FrameFormat::NV12 => buf_nv12_to_rgb(resolution, data, dest, true),
            FrameFormat::BGRA => {
                if dest.len() != data.len() {
                    return Err(NokhwaError::ProcessFrameError {
                        src: fcc,
                        destination: "BGRA => RGBA".to_string(),
                        error: "Bad buffer length".to_string(),
                    });
                }

                data.chunks_exact(4).enumerate().for_each(|(idx, px)| {
                    let index = idx * 4;
                    dest[index] = px[2];
                    dest[index + 1] = px[1];
                    dest[index + 2] = px[0];
                    dest[index + 3] = px[3];
                });
                Ok(())
            }
        }
    }
}

/// A Zero-Size-Type that contains the definition to convert a given image stream to an Luma8(Grayscale 8-bit) in the [`Buffer`](crate::buffer::Buffer)'s [`.decode_image()`](crate::buffer::Buffer::decode_image)
///
/// ```.ignore
/// use image::{ImageBuffer, Luma};
/// let image: ImageBuffer<Luma<u8>, Vec<u8>> = buffer.to_image::<LumaFormat>();
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct LumaFormat;

impl FormatDecoder for LumaFormat {
    type Output = Luma<u8>;

    const FORMATS: &'static [FrameFormat] = frame_formats();

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    fn write_output(
        fcc: FrameFormat,
        resolution: Resolution,
        data: &[u8],
    ) -> Result<Vec<u8>, NokhwaError> {
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
            FrameFormat::NV12 => Ok(nv12_to_rgb(resolution, data, false)?
                .as_slice()
                .chunks_exact(3)
                .map(|x| {
                    let mut avg = 0;
                    x.iter().for_each(|v| avg += u16::from(*v));
                    (avg / 3) as u8
                })
                .collect()),
            FrameFormat::GRAY => Ok(data.to_vec()),
            FrameFormat::RAWRGB => Ok(data
                .chunks(3)
                .map(|px| ((i32::from(px[0]) + i32::from(px[1]) + i32::from(px[2])) / 3) as u8)
                .collect()),
            FrameFormat::BGRA => Ok(data
                .chunks_exact(4)
                .map(|px| ((i32::from(px[0]) + i32::from(px[1]) + i32::from(px[2])) / 3) as u8)
                .collect()),
        }
    }

    #[inline]
    fn write_output_buffer(
        fcc: FrameFormat,
        _resolution: Resolution,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError> {
        match fcc {
            // TODO: implement!
            FrameFormat::MJPEG | FrameFormat::YUYV | FrameFormat::NV12 => {
                Err(NokhwaError::ProcessFrameError {
                    src: fcc,
                    destination: "Luma => RGB".to_string(),
                    error: "Conversion Error".to_string(),
                })
            }

            FrameFormat::GRAY => {
                data.iter().zip(dest.iter_mut()).for_each(|(pxv, d)| {
                    *d = *pxv;
                });
                Ok(())
            }
            FrameFormat::RAWRGB => Err(NokhwaError::ProcessFrameError {
                src: fcc,
                destination: "RGB => RGB".to_string(),
                error: "Conversion Error".to_string(),
            }),
            FrameFormat::BGRA => {
                data.chunks_exact(4)
                    .zip(dest.iter_mut())
                    .for_each(|(px, d)| {
                        *d = ((i32::from(px[0]) + i32::from(px[1]) + i32::from(px[2])) / 3) as u8;
                    });
                Ok(())
            }
        }
    }
}

/// A Zero-Size-Type that contains the definition to convert a given image stream to an LumaA8(Grayscale 8-bit with 8-bit alpha) in the [`Buffer`](crate::buffer::Buffer)'s [`.decode_image()`](crate::buffer::Buffer::decode_image)
///
/// ```.ignore
/// use image::{ImageBuffer, LumaA};
/// let image: ImageBuffer<LumaA<u8>, Vec<u8>> = buffer.to_image::<LumaAFormat>();
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct LumaAFormat;

impl FormatDecoder for LumaAFormat {
    type Output = LumaA<u8>;

    const FORMATS: &'static [FrameFormat] = frame_formats();

    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn write_output(
        fcc: FrameFormat,
        resolution: Resolution,
        data: &[u8],
    ) -> Result<Vec<u8>, NokhwaError> {
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
            FrameFormat::NV12 => Ok(nv12_to_rgb(resolution, data, false)?
                .as_slice()
                .chunks_exact(3)
                .flat_map(|x| {
                    let mut avg = 0;
                    x.iter().for_each(|v| avg += u16::from(*v));
                    [(avg / 3) as u8, 255]
                })
                .collect()),
            FrameFormat::GRAY => Ok(data.iter().flat_map(|x| [*x, 255]).collect()),
            FrameFormat::RAWRGB => Err(NokhwaError::ProcessFrameError {
                src: fcc,
                destination: "RGB => RGB".to_string(),
                error: "Conversion Error".to_string(),
            }),
            FrameFormat::BGRA => {
                let mut luma_a = vec![0u8; data.len() / 4 * 2];
                data.chunks_exact(4).enumerate().for_each(|(idx, px)| {
                    let index = idx * 2;
                    luma_a[index] =
                        ((i32::from(px[0]) + i32::from(px[1]) + i32::from(px[2])) / 3) as u8;
                    luma_a[index + 1] = px[3];
                });
                Ok(luma_a)
            }
        }
    }

    #[inline]
    fn write_output_buffer(
        fcc: FrameFormat,
        _resolution: Resolution,
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
            FrameFormat::NV12 => Err(NokhwaError::ProcessFrameError {
                src: fcc,
                destination: "NV12 => LumaA".to_string(),
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
            FrameFormat::RAWRGB => Err(NokhwaError::ProcessFrameError {
                src: fcc,
                destination: "RGB => RGB".to_string(),
                error: "Conversion Error".to_string(),
            }),
            FrameFormat::BGRA => {
                if dest.len() != data.len() / 4 * 2 {
                    return Err(NokhwaError::ProcessFrameError {
                        src: fcc,
                        destination: "BGRA => LumaA".to_string(),
                        error: "Conversion Error".to_string(),
                    });
                }

                data.chunks_exact(4)
                    .zip(dest.chunks_exact_mut(2))
                    .for_each(|(px, d)| {
                        d[0] = ((i32::from(px[0]) + i32::from(px[1]) + i32::from(px[2])) / 3) as u8;
                        d[1] = px[3];
                    });
                Ok(())
            }
        }
    }
}

/// let image: ImageBuffer<Rgb<u8>, Vec<u8>> = buffer.to_image::<YuyvFormat>();
/// ```
#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct I420Format;

impl FormatDecoder for I420Format {
    // YUV 4:2:0 planar colors. but we need to change the image crate to use this format
    type Output = Rgb<u8>;
    const FORMATS: &'static [FrameFormat] = color_frame_formats();

    #[inline]
    fn write_output(
        fcc: FrameFormat,
        resolution: Resolution,
        data: &[u8],
    ) -> Result<Vec<u8>, NokhwaError> {
        match fcc {
            FrameFormat::YUYV => {
                let mut i420 =
                    vec![0u8; resolution.width() as usize * resolution.height() as usize * 3 / 2];
                convert_yuyv_to_i420_direct(
                    data,
                    resolution.width() as usize,
                    resolution.height() as usize,
                    &mut i420,
                )?;
                Ok(i420)
            }
            FrameFormat::BGRA => {
                println!("BGRA to I420 2");
                // Transform to RGB first, 3 pixels per pixel
                let mut rgb = vec![0u8; data.len() / 4 * 3];
                data.chunks_exact(4).enumerate().for_each(|(idx, px)| {
                    let index = idx * 3;
                    rgb[index] = px[2];
                    rgb[index + 1] = px[1];
                    rgb[index + 2] = px[0];
                });
                let width = resolution.width() as usize;
                let height = resolution.height() as usize;
                let mut nv12_data = vec![0u8; width * height * 3 / 2];

                rgb_to_i420(&rgb, width, height, &mut nv12_data);
                Ok(nv12_data)
            }
            _ => Err(NokhwaError::GeneralError(format!(
                "Invalid FrameFormat in write_output: {:?}",
                fcc
            ))),
        }
    }

    #[inline]
    fn write_output_buffer(
        fcc: FrameFormat,
        resolution: Resolution,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError> {
        match fcc {
            FrameFormat::YUYV => {
                convert_yuyv_to_i420_direct(
                    data,
                    resolution.width() as usize,
                    resolution.height() as usize,
                    dest,
                )?;
                Ok(())
            }

            FrameFormat::NV12 => {
                nv12_to_i420(
                    data,
                    resolution.width() as usize,
                    resolution.height() as usize,
                    dest,
                );
                Ok(())
            }

            FrameFormat::BGRA => {
                bgra_to_i420(
                    data,
                    resolution.width() as usize,
                    resolution.height() as usize,
                    dest,
                );
                Ok(())
            }

            _ => Err(NokhwaError::GeneralError(format!(
                "Invalid FrameFormat in write_output_buffer: {:?}",
                fcc
            ))),
        }
    }
}

/// Converts an image in YUYV format to I420 (YUV 4:2:0) format.
/// YUYV format is a packed format with two Y samples followed by one U and one V sample.
/// I420 format is a planar format with Y plane followed by U and V planes.
/// The U and V planes are half the width and height of the Y plane.
/// # Arguments
/// - `yuyv`: Input buffer containing the YUYV pixel data.
/// - `width`: Width of the image.
/// - `height`: Height of the image.
/// - `dest`: Output buffer to store the I420 data.
fn convert_yuyv_to_i420_direct(
    yuyv: &[u8],
    width: usize,
    height: usize,
    dest: &mut [u8],
) -> Result<(), NokhwaError> {
    // Ensure the destination buffer is large enough
    if dest.len() < width * height + 2 * (width / 2) * (height / 2) {
        return Err(NokhwaError::GeneralError(
            "Destination buffer is too small".into(),
        ));
    }

    // Split the destination buffer into Y, U, and V planes
    let (y_plane, uv_plane) = dest.split_at_mut(width * height);
    let (u_plane, v_plane) = uv_plane.split_at_mut(uv_plane.len() / 2);

    // Convert YUYV to I420
    for y in 0..height {
        for x in (0..width).step_by(2) {
            let base_index = (y * width + x) * 2;
            let y0 = yuyv[base_index];
            let u = yuyv[base_index + 1];
            let y1 = yuyv[base_index + 2];
            let v = yuyv[base_index + 3];

            y_plane[y * width + x] = y0;
            y_plane[y * width + x + 1] = y1;

            if y % 2 == 0 {
                u_plane[y / 2 * (width / 2) + x / 2] = u;
                v_plane[y / 2 * (width / 2) + x / 2] = v;
            }
        }
    }

    Ok(())
}

/// Converts an image in NV12 format to I420 (YUV 4:2:0) format.
/// NV12 format is a planar format with Y plane followed by interleaved UV plane.
/// I420 format is a planar format with Y plane followed by U and V planes.
/// The U and V planes are half the width and height of the Y plane.
/// # Arguments
/// - `nv12`: Input buffer containing the NV12 pixel data.
/// - `width`: Width of the image.
/// - `height`: Height of the image.
/// - `i420`: Output buffer to store the I420 data.s
fn nv12_to_i420(nv12: &[u8], width: usize, height: usize, i420: &mut [u8]) {
    // assert that the nv12 has the expected size
    assert!(
        width % 2 == 0 && height % 2 == 0,
        "Width and height must be even numbers."
    );

    let y_plane_size = width * height;
    let uv_plane_size = y_plane_size / 2; // Interleaved UV plane size
    let u_plane_size = uv_plane_size / 2;

    let (y_plane, uv_plane) = i420.split_at_mut(y_plane_size);
    let (u_plane, v_plane) = uv_plane.split_at_mut(u_plane_size);

    // Step 1: Copy Y plane
    y_plane.copy_from_slice(&nv12[..y_plane_size]);

    // Step 2: Process interleaved UV data
    let nv12_uv = &nv12[y_plane_size..];

    for row in 0..(height / 2) {
        for col in 0..(width / 2) {
            let nv12_index = row * width + col * 2; // Index in NV12 interleaved UV plane
            let uv_index = row * (width / 2) + col; // Index in U and V planes

            u_plane[uv_index] = nv12_uv[nv12_index]; // U value
            v_plane[uv_index] = nv12_uv[nv12_index + 1]; // V value
        }
    }
}

/// Converts an image in BGRA format to I420 (YUV 4:2:0) format.
///
/// # Arguments
/// - `bgra`: Input buffer containing the BGRA pixel data.
/// - `width`: Width of the image.
/// - `height`: Height of the image.
/// - `i420`: Output buffer to store the I420 data.
///            Must have at least `width * height * 3 / 2` bytes allocated.
fn bgra_to_i420(bgra: &[u8], width: usize, height: usize, i420: &mut [u8]) {
    assert!(
        i420.len() >= width * height * 3 / 2,
        "Insufficient I420 buffer size, got {} expected {}",
        i420.len(),
        width * height * 3 / 2
    );

    let (y_plane, uv_planes) = i420.split_at_mut(width * height);
    let (u_plane, v_plane) = uv_planes.split_at_mut(width * height / 4);

    for y in 0..height {
        for x in 0..width {
            let bgra_index = (y * width + x) * 4;
            let b = bgra[bgra_index] as f32;
            let g = bgra[bgra_index + 1] as f32;
            let r = bgra[bgra_index + 2] as f32;

            // Calculate Y, U, V components
            let y_value = (0.257 * r + 0.504 * g + 0.098 * b + 16.0).round() as u8;
            let u_value = (-0.148 * r - 0.291 * g + 0.439 * b + 128.0).round() as u8;
            let v_value = (0.439 * r - 0.368 * g - 0.071 * b + 128.0).round() as u8;

            y_plane[y * width + x] = y_value;

            if y % 2 == 0 && x % 2 == 0 {
                let uv_index = (y / 2) * (width / 2) + (x / 2);
                u_plane[uv_index] = u_value;
                v_plane[uv_index] = v_value;
            }
        }
    }
}

pub fn rgb_to_i420(rgb: &[u8], width: usize, height: usize, i420: &mut [u8]) {
    // assert_eq!(rgb.len(), width * height * 3, "Invalid RGB buffer size");
    assert!(
        i420.len() >= width * height * 3 / 2,
        "Insufficient I420 buffer size"
    );

    let (y_plane, uv_planes) = i420.split_at_mut(width * height);
    let (u_plane, v_plane) = uv_planes.split_at_mut(width * height / 4);

    for y in 0..height {
        for x in 0..width {
            let rgb_index = (y * width + x) * 3;
            let r = rgb[rgb_index] as f32;
            let g = rgb[rgb_index + 1] as f32;
            let b = rgb[rgb_index + 2] as f32;

            // Calculate Y, U, V components
            let y_value = (0.257 * r + 0.504 * g + 0.098 * b + 16.0).round() as u8;
            let u_value = (-0.148 * r - 0.291 * g + 0.439 * b + 128.0).round() as u8;
            let v_value = (0.439 * r - 0.368 * g - 0.071 * b + 128.0).round() as u8;

            y_plane[y * width + x] = y_value;

            if y % 2 == 0 && x % 2 == 0 {
                let uv_index = (y / 2) * (width / 2) + (x / 2);
                u_plane[uv_index] = u_value;
                v_plane[uv_index] = v_value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RgbFormat;
    use crate::pixel_format::FormatDecoder;

    fn assert_i420_equal_with_epsilon(
        epsilon_y: u8,
        epsilon_u: u8,
        epsilon_v: u8,
        actual: &[u8],
        expected: &[u8],
        width: usize,
        height: usize,
    ) {
        assert_eq!(actual.len(), expected.len());
        let (actual_y, actual_uv) = actual.split_at(width * height);
        let (actual_u, actual_v) = actual_uv.split_at(actual_uv.len() / 2);

        let (expected_y, expected_uv) = expected.split_at(width * height);
        let (expected_u, expected_v) = expected_uv.split_at(expected_uv.len() / 2);

        // Validate Y plane
        for (i, (&actual, &expected)) in actual_y.iter().zip(expected_y.iter()).enumerate() {
            assert!(
                (actual as i32 - expected as i32).abs() <= epsilon_y as i32,
                "Y plane mismatch at index {}: actual = {}, expected = {}",
                i,
                actual,
                expected
            );
        }

        // Validate U plane
        for (i, (&actual, &expected)) in actual_u.iter().zip(expected_u.iter()).enumerate() {
            assert!(
                (actual as i32 - expected as i32).abs() <= epsilon_u as i32,
                "U plane mismatch at index {}: actual = {}, expected = {}",
                i,
                actual,
                expected
            );
        }

        // Validate V plane
        for (i, (&actual, &expected)) in actual_v.iter().zip(expected_v.iter()).enumerate() {
            assert!(
                (actual as i32 - expected as i32).abs() <= epsilon_v as i32,
                "V plane mismatch at index {}: actual = {}, expected = {}",
                i,
                actual,
                expected
            );
        }
    }

    #[test]
    fn yuyv_to_i420() {
        let yuyv = include_bytes!("../tests/assets/chichen_itza.yuyv");
        let expected_i420 = include_bytes!("../tests/assets/chichen_itza.yuyv_i420");
        let width = 1280;
        let height = 680;
        let mut actual: Vec<u8> = vec![0u8; width * height * 3 / 2];
        super::convert_yuyv_to_i420_direct(yuyv, width, height, &mut actual).unwrap();
        // I generated the expected I420 data using ffmpeg, so we allow some error in the conversion
        assert_i420_equal_with_epsilon(0, 5, 5, &actual, expected_i420, width, height);
    }

    #[test]
    fn bgra_to_i420() {
        let bgra = include_bytes!("../tests/assets/chichen_itza.bgra");
        let expected_i420 = include_bytes!("../tests/assets/chichen_itza.bgra_i420");
        let width = 1280;
        let height = 680;
        let mut actual = vec![0u8; width * height * 3 / 2];
        super::bgra_to_i420(bgra, width, height, &mut actual);
        // I generated the expected I420 data using ffmpeg, so we allow some error in the conversion
        assert_i420_equal_with_epsilon(1, 6, 6, &actual, expected_i420, width, height);
    }

    #[test]
    fn nv12_to_i420() {
        let nv12 = include_bytes!("../tests/assets/chichen_itza.nv12");
        let expected_i420 = include_bytes!("../tests/assets/chichen_itza.nv12_i420");
        let width = 1280;
        let height = 680;
        let mut actual = vec![0u8; width * height * 3 / 2];
        super::nv12_to_i420(nv12, width, height, &mut actual);
        // I generated the expected I420 data using ffmpeg, so we allow some error in the conversion
        assert_i420_equal_with_epsilon(0, 0, 0, &actual, expected_i420, width, height);
    }

    #[test]
    fn buf_bgra_to_rgb() {
        let input_bgra = include_bytes!("../tests/assets/chichen_itza.bgra");
        let expected_rgb = include_bytes!("../tests/assets/chichen_itza.rgb");
        let width = 1280;
        let height = 680;
        let mut actual = vec![0u8; input_bgra.len() / 4 * 3];

        RgbFormat::write_output_buffer(
            super::FrameFormat::BGRA,
            super::Resolution::new(width as u32, height as u32),
            input_bgra,
            &mut actual,
        )
        .unwrap();
        assert_eq!(actual.len(), expected_rgb.len());
        for (i, (&actual, &expected)) in actual.iter().zip(expected_rgb.iter()).enumerate() {
            assert_eq!(
                actual, expected,
                "Mismatch at index {} expected {:#X} actual {:#X}",
                i, expected, actual
            );
        }
    }
}
