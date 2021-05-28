use crate::NokhwaError;
use mozjpeg::Decompress;
use std::fmt::Formatter;
use std::{cmp::Ordering, convert::TryFrom, fmt::Display, slice::from_raw_parts};

/// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC` <br>
/// YUYV is a mathematical color space. You can read more [here.](https://en.wikipedia.org/wiki/YCbCr) <br>
/// MJPEG is a motion-jpeg compressed frame, it allows for high frame rates.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum FrameFormat {
    MJPEG,
    YUYV,
}
impl Display for FrameFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameFormat::MJPEG => {
                write!(f, "MJPEG")
            }
            FrameFormat::YUYV => {
                write!(f, "YUYV")
            }
        }
    }
}

/// Describes a Resolution.
/// This struct consists of a Width and a Height value (x,y). <br>
/// Note: the [`Ord`] implementation of this struct is flipped from highest to lowest.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Resolution {
    pub width_x: u32,
    pub height_y: u32,
}

impl Resolution {
    /// Create a new resolution from 2 image size coordinates.
    pub fn new(x: u32, y: u32) -> Self {
        Resolution {
            width_x: x,
            height_y: y,
        }
    }

    /// Get the width of Resolution
    pub fn width(self) -> u32 {
        self.width_x
    }

    /// Get the height of Resolution
    pub fn height(self) -> u32 {
        self.height_y
    }

    /// Get the x (width) of Resolution
    pub fn x(self) -> u32 {
        self.width_x
    }

    /// Get the y (height) of Resolution
    pub fn y(self) -> u32 {
        self.height_y
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.x(), self.y())
    }
}

impl PartialOrd for Resolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Resolution {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.x().cmp(&other.x()) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.y().cmp(&other.y()),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

/// This is a convenience struct that holds all information about the format of a webcam stream.
/// It consists of a [`Resolution`], [`FrameFormat`], and a framerate(u8).
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub struct CameraFormat {
    resolution: Resolution,
    format: FrameFormat,
    framerate: u32,
}

impl CameraFormat {
    /// Construct a new [`CameraFormat`]
    pub fn new(resolution: Resolution, format: FrameFormat, framerate: u32) -> Self {
        CameraFormat {
            resolution,
            format,
            framerate,
        }
    }

    /// [`CameraFormat::new()`], but raw.
    pub fn new_from(res_x: u32, res_y: u32, format: FrameFormat, fps: u32) -> Self {
        CameraFormat {
            resolution: Resolution {
                width_x: res_x,
                height_y: res_y,
            },
            format,
            framerate: fps,
        }
    }

    /// Get the resolution of the current [`CameraFormat`]
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Get the width of the resolution of the current [`CameraFormat`]
    pub fn width(&self) -> u32 {
        self.resolution.width()
    }

    /// Get the height of the resolution of the current [`CameraFormat`]
    pub fn height(&self) -> u32 {
        self.resolution.height()
    }

    /// Set the [`CameraFormat`]'s resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.resolution = resolution;
    }

    /// Get the framerate of the current [`CameraFormat`]
    pub fn framerate(&self) -> u32 {
        self.framerate
    }

    /// Set the [`CameraFormat`]'s framerate.
    pub fn set_framerate(&mut self, framerate: u32) {
        self.framerate = framerate;
    }

    /// Get the [`CameraFormat`]'s format.
    pub fn format(&self) -> FrameFormat {
        self.format
    }

    /// Set the [`CameraFormat`]'s format.
    pub fn set_format(&mut self, format: FrameFormat) {
        self.format = format;
    }
}

impl Default for CameraFormat {
    fn default() -> Self {
        CameraFormat {
            resolution: Resolution::new(640, 480),
            format: FrameFormat::MJPEG,
            framerate: 15,
        }
    }
}

impl Display for CameraFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}FPS, {} Format",
            self.resolution, self.framerate, self.format
        )
    }
}

/// Information about a Camera e.g. its name.
/// `description` amd `misc` may contain backend-specific information.
/// `index` is a camera's index given to it by (usually) the OS usually in the order it is known to the system.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct CameraInfo {
    human_name: String,
    description: String,
    misc: String,
    index: usize,
}

impl CameraInfo {
    /// Create a new [`CameraInfo`].
    pub fn new(human_name: String, description: String, misc: String, index: usize) -> Self {
        CameraInfo {
            human_name,
            description,
            misc,
            index,
        }
    }

    /// Get a reference to the device info's human name.
    pub fn human_name(&self) -> &String {
        &self.human_name
    }

    /// Set the device info's human name.
    pub fn set_human_name(&mut self, human_name: String) {
        self.human_name = human_name;
    }

    /// Get a reference to the device info's description.
    pub fn description(&self) -> &String {
        &self.description
    }

    /// Set the device info's description.
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// Get a reference to the device info's misc.
    pub fn misc(&self) -> &String {
        &self.misc
    }

    /// Set the device info's misc.
    pub fn set_misc(&mut self, misc: String) {
        self.misc = misc;
    }

    /// Get a reference to the device info's index.
    pub fn index(&self) -> &usize {
        &self.index
    }

    /// Set the device info's index.
    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

impl PartialOrd for CameraInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CameraInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl Display for CameraInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, Description: {}, Extra: {}, Index: {}",
            self.human_name, self.description, self.misc, self.index
        )
    }
}

/// The list of known capture backends to the library. <br>
/// - AUTO is special - it tells the Camera struct to automatically choose a backend most suited for the current platform.
/// - V4L2 - `Video4Linux2`, a linux specific backend.
/// - UVC - Universal Video Class (please check [libuvc](https://github.com/libuvc/libuvc)). Platform agnostic, although on linux it needs `sudo` permissions or similar to use.
/// - Windows - Directshow, Windows only
/// - `OpenCV` - Uses `OpenCV` to capture. Platform agnostic.
/// - FFMPEG - Uses FFMPEG (libavdevice) to capture. Platform agnostic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaptureAPIBackend {
    AUTO,
    V4L2,
    UVC,
    WINDOWS,
    OPENCV,
    FFMPEG,
}

impl Display for CaptureAPIBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let self_str = format!("{:?}", self);
        write!(f, "{}", self_str)
    }
}

/// Converts a MJPEG stream of [u8] into a Vec<u8> of RGB888. (R,G,B,R,G,B,...)
/// # Errors
/// If `mozjpeg` fails to read scanlines or setup the decompressor, this will error.
/// # Safety
/// This function uses `unsafe`. The caller must ensure that:
/// - The input data is of the right size, does not exceed bounds, and/or the final size matches with the initial size.
pub fn mjpeg_to_rgb888(data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
    let mut mozjpeg_decomp = match Decompress::new_mem(data) {
        Ok(decomp) => match decomp.rgb() {
            Ok(decompresser) => decompresser,
            Err(why) => {
                return Err(NokhwaError::CouldntDecompressFrame {
                    src: FrameFormat::MJPEG,
                    destination: "RGB888".to_string(),
                    error: why.to_string(),
                })
            }
        },
        Err(why) => {
            return Err(NokhwaError::CouldntDecompressFrame {
                src: FrameFormat::MJPEG,
                destination: "RGB888".to_string(),
                error: why.to_string(),
            })
        }
    };

    let decompressed = match mozjpeg_decomp.read_scanlines::<[u8; 3]>() {
        Some(pixels) => unsafe {
            &*(from_raw_parts(pixels.as_ptr(), pixels.len() * 3) as *const [[u8; 3]]
                as *const [u8])
        },
        None => {
            return Err(NokhwaError::CouldntDecompressFrame {
                src: FrameFormat::MJPEG,
                destination: "RGB888".to_string(),
                error: "Failed to get read readlines into RGB888 pixels!".to_string(),
            })
        }
    };

    Ok(decompressed.to_vec())
}

// For those maintaining this, I recommend you read: https://docs.microsoft.com/en-us/windows/win32/medfound/recommended-8-bit-yuv-formats-for-video-rendering#yuy2
// https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
// and this too: https://stackoverflow.com/questions/16107165/convert-from-yuv-420-to-imagebgr-byte
// The YUY2(YUYV) format is a 16 bit format. We read 4 bytes at a time to get 6 bytes of RGB888.
// First, the YUY2 is converted to YCbCr 4:4:4 (4:2:2 -> 4:4:4)
// then it is converted to 6 bytes (2 pixels) of RGB888
/// Converts a YUYV 4:2:2 datastream to a RGB888 Stream. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
/// # Errors
/// This may error when the data stream size is not divisible by 4, a i32 -> u8 conversion fails, or it fails to read from a certain index.
pub fn yuyv422_to_rgb888(data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
    let mut rgb_vec: Vec<u8> = vec![];
    if data.len() % 4 == 0 {
        for px_idx in (0..data.len()).step_by(4) {
            let u = match data.get(px_idx) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::CouldntDecompressFrame{ src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx, why.to_string()) });
                    }
                },
                None => {
                    return Err(NokhwaError::CouldntDecompressFrame {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx
                        ),
                    });
                }
            };

            let y1 = match data.get(px_idx + 1) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::CouldntDecompressFrame{ src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx+1, why.to_string()) });
                    }
                },
                None => {
                    return Err(NokhwaError::CouldntDecompressFrame {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx + 1
                        ),
                    });
                }
            };

            let v = match data.get(px_idx + 2) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::CouldntDecompressFrame{ src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx+2, why.to_string()) });
                    }
                },
                None => {
                    return Err(NokhwaError::CouldntDecompressFrame {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx + 2
                        ),
                    });
                }
            };

            let y2 = match data.get(px_idx + 3) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::CouldntDecompressFrame{ src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx+3, why.to_string()) });
                    }
                },
                None => {
                    return Err(NokhwaError::CouldntDecompressFrame {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx + 3
                        ),
                    });
                }
            };

            let pixel1 = yuyv444_to_rgb888(y1, u, v);
            let pixel2 = yuyv444_to_rgb888(y2, u, v);
            rgb_vec.append(&mut pixel1.to_vec());
            rgb_vec.append(&mut pixel2.to_vec());
        }
        Ok(rgb_vec)
    } else {
        Err(NokhwaError::CouldntDecompressFrame {
            src: FrameFormat::YUYV,
            destination: "RGB888".to_string(),
            error: "Assertion failure, the YUV stream isn't 4:2:2! (wrong number of bytes)"
                .to_string(),
        })
    }
}

// equation from https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
/// Convert `YCbCr` 4:4:4 to a RGB888. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
#[allow(clippy::many_single_char_names)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
pub fn yuyv444_to_rgb888(y: i32, u: i32, v: i32) -> [u8; 3] {
    let c298 = (y - 16) * 298;
    let d = u - 128;
    let e = v - 128;
    let r = ((c298 + 409 * e + 128) >> 8).clamp(0, 255) as u8;
    let g = ((c298 - 100 * d - 208 * e + 128) >> 8).clamp(0, 255) as u8;
    let b = ((c298 + 516 * d + 128) >> 8).clamp(0, 255) as u8;
    [r, g, b]
}
