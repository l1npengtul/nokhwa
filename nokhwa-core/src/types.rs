use crate::{
    error::NokhwaError,
    frame_format::FrameFormat,
};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow, cmp::Ordering, fmt::{
        Debug,
        Display,
        Formatter
    }, hash::{Hash, Hasher}, ops::{Add, Deref, DerefMut, Sub}
};
use crate::traits::Distance;


/// Describes the index of the camera.
/// - Index: A numbered index
/// - String: A string, used for `IPCameras` or on the Browser as DeviceIDs.
#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum CameraIndex {
    Index(u32),
    String(String),
}

impl CameraIndex {
    /// Turns this value into a number. If it is a string, it will attempt to parse it as a `u32`.
    /// # Errors
    /// Fails if the value is not a number.
    pub fn as_index(&self) -> Result<u32, NokhwaError> {
        match self {
            CameraIndex::Index(i) => Ok(*i),
            CameraIndex::String(s) => s
                .parse::<u32>()
                .map_err(|why| NokhwaError::GeneralError(why.to_string())),
        }
    }

    /// Turns this value into a `String`. If it is a number, it will be automatically converted.
    #[must_use]
    pub fn as_string(&self) -> String {
        match self {
            CameraIndex::Index(i) => i.to_string(),
            CameraIndex::String(s) => s.to_string(),
        }
    }

    /// Returns true if this [`CameraIndex`] contains an [`CameraIndex::Index`]
    #[must_use]
    pub fn is_index(&self) -> bool {
        match self {
            CameraIndex::Index(_) => true,
            CameraIndex::String(_) => false,
        }
    }

    /// Returns true if this [`CameraIndex`] contains an [`CameraIndex::String`]
    #[must_use]
    pub fn is_string(&self) -> bool {
        !self.is_index()
    }
}

impl Display for CameraIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl Default for CameraIndex {
    fn default() -> Self {
        CameraIndex::Index(0)
    }
}

impl TryFrom<CameraIndex> for u32 {
    type Error = NokhwaError;

    fn try_from(value: CameraIndex) -> Result<Self, Self::Error> {
        value.as_index()
    }
}

impl TryFrom<CameraIndex> for usize {
    type Error = NokhwaError;

    fn try_from(value: CameraIndex) -> Result<Self, Self::Error> {
        value.as_index().map(|i| i as usize)
    }
}

/// Describes a Resolution.
/// This struct consists of a Width and a Height value (x,y). <br>
/// Note: the [`Ord`] implementation of this struct is flipped from highest to lowest.
/// # JS-WASM
/// This is exported as `JSResolution`
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq)]
pub struct Resolution {
    pub width_x: u32,
    pub height_y: u32,
}

impl Resolution {
    /// Create a new resolution from 2 image size coordinates.
    #[must_use]
    pub fn new(x: u32, y: u32) -> Self {
        Resolution {
            width_x: x,
            height_y: y,
        }
    }

    /// Get the width of Resolution
    #[must_use]
    #[inline]
    pub fn width(self) -> u32 {
        self.width_x
    }

    /// Get the height of Resolution
    #[must_use]
    #[inline]
    pub fn height(self) -> u32 {
        self.height_y
    }

    /// Get the x (width) of Resolution
    #[must_use]
    #[inline]
    pub fn x(self) -> u32 {
        self.width_x
    }

    /// Get the y (height) of Resolution
    #[must_use]
    #[inline]
    pub fn y(self) -> u32 {
        self.height_y
    }

    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        f64::from(self.width_x) / f64::from(self.height_y)
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

impl Distance<u32> for Resolution {
    fn distance_from(&self, other: &Self) -> u32 {
        let x1 = self.x();
        let x2 = other.x();

        let y1 = self.y();
        let y2 = other.y();

        (x2 - x1).pow(2) + (y2 - y1).pow(2)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct FrameRate(pub f32);

impl FrameRate {
    #[must_use]
    pub fn new(fps: f32) -> Self {
        Self(fps)
    }

    #[must_use]
    pub fn frame_rate(&self) -> f32 {
        self.0
    }
}

impl Deref for FrameRate {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrameRate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Hash for FrameRate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.0.to_bits());
    }
}

impl Default for FrameRate {
    fn default() -> Self {
        FrameRate(30.0)
    }
}

impl Display for FrameRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} FPS", self.0)
    }
}

impl Add for FrameRate {
    type Output = FrameRate;

    fn add(self, rhs: Self) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl Add for &FrameRate {
    type Output = FrameRate;

    fn add(self, rhs: Self) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}


impl Sub for FrameRate {
    type Output = FrameRate;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.0 - rhs.0).into()
    }
}

impl Sub for &FrameRate {
    type Output = FrameRate;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.0 - rhs.0).into()
    }
}

impl From<f32> for FrameRate {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<FrameRate> for f32 {
    fn from(value: FrameRate) -> Self {
        value.0
    }
}

/// This is a convenience struct that holds all information about the format of a webcam stream.
/// It consists of a [`Resolution`], [`FrameFormat`], and a [`FrameRate`].
#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct CameraFormat {
    resolution: Resolution,
    format: FrameFormat,
    frame_rate: FrameRate,
}

impl CameraFormat {
    /// Construct a new [`CameraFormat`]
    #[must_use]
    pub fn new(resolution: Resolution, format: FrameFormat, frame_rate: FrameRate) -> Self {
        CameraFormat {
            resolution,
            format,
            frame_rate,
        }
    }

    /// [`CameraFormat::new()`], but raw.
    #[must_use]
    pub fn new_from(res_x: u32, res_y: u32, format: FrameFormat, fps: FrameRate) -> Self {
        CameraFormat {
            resolution: Resolution {
                width_x: res_x,
                height_y: res_y,
            },
            format,
            frame_rate: fps,
        }
    }

    /// Get the resolution of the current [`CameraFormat`]
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Get the width of the resolution of the current [`CameraFormat`]
    #[must_use]
    pub fn width(&self) -> u32 {
        self.resolution.width()
    }

    /// Get the height of the resolution of the current [`CameraFormat`]
    #[must_use]
    pub fn height(&self) -> u32 {
        self.resolution.height()
    }

    /// Set the [`CameraFormat`]'s resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.resolution = resolution;
    }

    /// Get the frame rate of the current [`CameraFormat`]
    #[must_use]
    pub fn frame_rate(&self) -> FrameRate {
        self.frame_rate
    }

    /// Set the [`CameraFormat`]'s frame rate.
    pub fn set_frame_rate(&mut self, frame_rate: FrameRate) {
        self.frame_rate = frame_rate;
    }

    /// Get the [`CameraFormat`]'s format.
    #[must_use]
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
            format: FrameFormat::MJpeg,
            frame_rate: FrameRate(30.),
        }
    }
}

impl Display for CameraFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}FPS, {} Format",
            self.resolution, self.frame_rate, self.format
        )
    }
}

/// Information about a Camera e.g. its name.
/// `description` amd `misc` may contain information that may differ from backend to backend. Refer to each backend for details.
/// `index` is a camera's index given to it by (usually) the OS usually in the order it is known to the system.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct CameraInfo {
    human_name: String,
    description: String,
    misc: String,
    index: CameraIndex,
}

impl CameraInfo {
    /// Create a new [`CameraInfo`].
    /// # JS-WASM
    /// This is exported as a constructor for [`CameraInfo`].
    #[must_use]
    // OK, i just checkeed back on this code. WTF was I on when I wrote `&(impl AsRef<str> + ?Sized)` ????
    // I need to get on the same shit that my previous self was on, because holy shit that stuff is strong as FUCK!
    // Finally fixed this insanity. Hopefully I didnt torment anyone by actually putting this in a stable release.
    pub fn new(human_name: &str, description: &str, misc: &str, index: &CameraIndex) -> Self {
        CameraInfo {
            human_name: human_name.to_string(),
            description: description.to_string(),
            misc: misc.to_string(),
            index: index.clone(),
        }
    }

    /// Get a reference to the device info's human readable name.
    /// # JS-WASM
    /// This is exported as a `get_HumanReadableName`.
    #[must_use]
    // yes, i know, unnecessary alloc this, unnecessary alloc that
    // but wasm bindgen
    pub fn human_name(&self) -> String {
        self.human_name.clone()
    }

    /// Set the device info's human name.
    /// # JS-WASM
    /// This is exported as a `set_HumanReadableName`.
    pub fn set_human_name(&mut self, human_name: &str) {
        self.human_name = human_name.to_string();
    }

    /// Get a reference to the device info's description.
    /// # JS-WASM
    /// This is exported as a `get_Description`.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.borrow()
    }

    /// Set the device info's description.
    /// # JS-WASM
    /// This is exported as a `set_Description`.
    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
    }

    /// Get a reference to the device info's misc.
    /// # JS-WASM
    /// This is exported as a `get_MiscString`.
    #[must_use]
    pub fn misc(&self) -> String {
        self.misc.clone()
    }

    /// Set the device info's misc.
    /// # JS-WASM
    /// This is exported as a `set_MiscString`.
    pub fn set_misc(&mut self, misc: &str) {
        self.misc = misc.to_string();
    }

    /// Get a reference to the device info's index.
    /// # JS-WASM
    /// This is exported as a `get_Index`.
    #[must_use]
    pub fn index(&self) -> &CameraIndex {
        &self.index
    }

    /// Set the device info's index.
    /// # JS-WASM
    /// This is exported as a `set_Index`.
    pub fn set_index(&mut self, index: CameraIndex) {
        self.index = index;
    }

    // /// Gets the device info's index as an `u32`.
    // /// # Errors
    // /// If the index is not parsable as a `u32`, this will error.
    // /// # JS-WASM
    // /// This is exported as `get_Index_Int`
    // #[cfg_attr(feature = "output-wasm", wasm_bindgen(getter = Index_Int))]
    // pub fn index_num(&self) -> Result<u32, NokhwaError> {
    //     match &self.index {
    //         CameraIndex::Index(i) => Ok(*i),
    //         CameraIndex::String(s) => match s.parse::<u32>() {
    //             Ok(p) => Ok(p),
    //             Err(why) => Err(NokhwaError::GetPropertyError {
    //                 property: "index-int".to_string(),
    //                 error: why.to_string(),
    //             }),
    //         },
    //     }
    // }
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

// fn step_chk(val: i64, default: i64, step: i64) -> Result<(), NokhwaError> {
//     if (val - default) % step != 0 {
//         return Err(NokhwaError::StructureError {
//             structure: "Value".to_string(),
//             error: "Doesnt fit step".to_string(),
//         });
//     }
//     Ok(())
// }

/// The list of known capture backends to the library. <br>
/// - `Auto` - Use automatic selection.
/// - `AVFoundation` - Uses `AVFoundation` on `MacOSX`
/// - `Video4Linux` - `Video4Linux2`, a linux specific backend.
/// - `UniversalVideoClass` -  ***DEPRECATED*** Universal Video Class (please check [libuvc](https://github.com/libuvc/libuvc)). Platform agnostic, although on linux it needs `sudo` permissions or similar to use.
/// - `MediaFoundation` - Microsoft Media Foundation, Windows only,
/// - `OpenCv` - Uses `OpenCV` to capture. Platform agnostic.
/// - `GStreamer` - ***DEPRECATED*** Uses `GStreamer` RTP to capture. Platform agnostic.
/// - `Browser` - Uses browser APIs to capture from a webcam.
pub enum SelectableBackend {
    Auto,
    Custom(&'static str),
    AVFoundation,
    Video4Linux,
    UniversalVideoClass,
    MediaFoundation,
    OpenCv,
    GStreamer,
    Browser,
}

/// The list of known capture backends to the library. <br>
/// - `AVFoundation` - Uses `AVFoundation` on `MacOSX`
/// - `Video4Linux` - `Video4Linux2`, a linux specific backend.
/// - `UniversalVideoClass` -  ***DEPRECATED*** Universal Video Class (please check [libuvc](https://github.com/libuvc/libuvc)). Platform agnostic, although on linux it needs `sudo` permissions or similar to use.
/// - `MediaFoundation` - Microsoft Media Foundation, Windows only,
/// - `OpenCv` - Uses `OpenCV` to capture. Platform agnostic.
/// - `GStreamer` - ***DEPRECATED*** Uses `GStreamer` RTP to capture. Platform agnostic.
/// - `Browser` - Uses browser APIs to capture from a webcam.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum ApiBackend {
    Custom(&'static str),
    AVFoundation,
    Video4Linux,
    UniversalVideoClass,
    MediaFoundation,
    OpenCv,
    GStreamer,
    Browser,
}

impl Display for ApiBackend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

// /// A webcam index that supports both strings and integers. Most backends take an int, but `IPCamera`s take a URL (string).
// #[derive(Clone, Debug, Hash, PartialEq, PartialOrd)]
// pub enum CameraIndex {
//     Index(u32),
//     String(String),
// }

// impl CameraIndex {
//     /// Gets the device info's index as an `u32`.
//     /// # Errors
//     /// If the index is not parsable as a `u32`, this will error.
//     pub fn as_index(&self) -> Result<u32, NokhwaError> {
//         match self {
//             CameraIndex::Index(i) => Ok(*i),
//             CameraIndex::String(s) => match s.parse::<u32>() {
//                 Ok(p) => Ok(p),
//                 Err(why) => Err(NokhwaError::GetPropertyError {
//                     property: "index-int".to_string(),
//                     error: why.to_string(),
//                 }),
//             },
//         }
//     }
// }

// impl Display for CameraIndex {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             CameraIndex::Index(idx) => {
//                 write!(f, "{}", idx)
//             }
//             CameraIndex::String(ip) => {
//                 write!(f, "{}", ip)
//             }
//         }
//     }
// }

// impl From<u32> for CameraIndex {
//     fn from(v: u32) -> Self {
//         CameraIndex::Index(v)
//     }
// }

// /// Trait for strings that can be converted to [`CameraIndex`]es.
// pub trait ValidString: AsRef<str> {}
//
// impl ValidString for String {}
// impl<'a> ValidString for &'a String {}
// impl<'a> ValidString for &'a mut String {}
// impl<'a> ValidString for Cow<'a, str> {}
// impl<'a> ValidString for &'a Cow<'a, str> {}
// impl<'a> ValidString for &'a mut Cow<'a, str> {}
// impl<'a> ValidString for &'a str {}
// impl<'a> ValidString for &'a mut str {}

// impl<T> From<T> for CameraIndex
// where
//     T: ValidString,
// {
//     fn from(v: T) -> Self {
//         CameraIndex::String(v.as_ref().to_string())
//     }
// }

#[cfg(all(feature = "conversions", not(target_arch = "wasm32")))]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "mjpeg")))]
#[inline]
fn decompress<'a>(
    data: &'a [u8],
    rgba: bool,
) -> Result<mozjpeg::decompress::DecompressStarted<'a>, NokhwaError> {
    use mozjpeg::Decompress;

    match Decompress::new_mem(data) {
        Ok(decompress) => {
            let decompressor_res = if rgba {
                decompress.rgba()
            } else {
                decompress.rgb()
            };
            match decompressor_res {
                Ok(decompressor) => Ok(decompressor),
                Err(why) => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::MJpeg,
                        destination: "RGB888".to_string(),
                        error: why.to_string(),
                    })
                }
            }
        }
        Err(why) => {
            return Err(NokhwaError::ProcessFrameError {
                src: FrameFormat::MJpeg,
                destination: "RGB888".to_string(),
                error: why.to_string(),
            })
        }
    }
}

/// Converts a MJpeg stream of `&[u8]` into a `Vec<u8>` of RGB888. (R,G,B,R,G,B,...)
/// # Errors
/// If `mozjpeg` fails to read scanlines or setup the decompressor, this will error.
/// # Safety
/// This function uses `unsafe`. The caller must ensure that:
/// - The input data is of the right size, does not exceed bounds, and/or the final size matches with the initial size.
#[cfg(all(feature = "conversions", not(target_arch = "wasm32")))]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "mjpeg")))]
#[inline]
pub fn mjpeg_to_rgb(data: &[u8], rgba: bool) -> Result<Vec<u8>, NokhwaError> {
    let mut jpeg_decompress = decompress(data, rgba)?;

    let scanlines_res: Option<Vec<u8>> = jpeg_decompress.read_scanlines_flat();
    // assert!(jpeg_decompress.finish_decompress());
    if !jpeg_decompress.finish_decompress() {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::MJpeg,
            destination: "RGB888".to_string(),
            error: "JPEG Decompressor did not finish.".to_string(),
        });
    }

    match scanlines_res {
        Some(pixels) => Ok(pixels),
        None => Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::MJpeg,
            destination: "RGB888".to_string(),
            error: "Failed to get read readlines into RGB888 pixels!".to_string(),
        }),
    }
}


/// Equivalent to [`mjpeg_to_rgb`] except with a destination buffer.
/// # Errors
/// If the decoding fails (e.g. invalid MJpeg stream), the buffer is not large enough, or you are doing this on `WebAssembly`, this will error.
#[cfg(not(all(feature = "conversions", not(target_arch = "wasm32"))))]
pub fn mjpeg_to_rgb(_data: &[u8], _rgba: bool) -> Result<Vec<u8>, NokhwaError> {
    Err(NokhwaError::NotImplementedError(
        "Not available on WASM".to_string(),
    ))
}

/// Equivalent to [`mjpeg_to_rgb`] except with a destination buffer.
/// # Errors
/// If the decoding fails (e.g. invalid MJpeg stream), the buffer is not large enough, or you are doing this on `WebAssembly`, this will error.
#[cfg(not(all(feature = "conversions", not(target_arch = "wasm32"))))]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "mjpeg")))]
#[inline]
pub fn buf_mjpeg_to_rgb(data: &[u8], dest: &mut [u8], rgba: bool) -> Result<(), NokhwaError> {
    let mut jpeg_decompress = mozjpeg::decompress(data, rgba)?;

    // assert_eq!(dest.len(), jpeg_decompress.min_flat_buffer_size());
    if dest.len() != jpeg_decompress.min_flat_buffer_size() {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::MJpeg,
            destination: "RGB888".to_string(),
            error: "Bad decoded buffer size".to_string(),
        });
    }

    jpeg_decompress.read_scanlines_flat_into(dest);
    // assert!(jpeg_decompress.finish_decompress());
    if !jpeg_decompress.finish_decompress() {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::MJpeg,
            destination: "RGB888".to_string(),
            error: "JPEG Decompressor did not finish.".to_string(),
        });
    }
    Ok(())
}

// TODO: deprecate?
/// Equivalent to [`mjpeg_to_rgb`] except with a destination buffer.
/// # Errors
/// If the decoding fails (e.g. invalid MJpeg stream), the buffer is not large enough, or you are doing this on `WebAssembly`, this will error.
#[cfg(all(feature = "conversions", not(target_arch = "wasm32")))]
pub fn buf_mjpeg_to_rgb(_data: &[u8], _dest: &mut [u8], _rgba: bool) -> Result<(), NokhwaError> {
    Err(NokhwaError::NotImplementedError(
        "Not available on WASM".to_string(),
    ))
}

/// Returns the predicted size of the destination Yuv422422 buffer.
#[inline]
pub fn yuyv422_predicted_size(size: usize, rgba: bool) -> usize {
    let pixel_size = if rgba { 4 } else { 3 };
    // yuyv yields 2 3-byte pixels per yuyv chunk
    (size / 4) * (2 * pixel_size)
}

#[inline]
pub fn yuyv422_to_rgb(data: &[u8], rgba: bool) -> Result<Vec<u8>, NokhwaError> {
    let capacity = yuyv422_predicted_size(data.len(), rgba);
    let mut rgb = vec![0; capacity];
    buf_yuyv422_to_rgb(data, &mut rgb, rgba)?;
    Ok(rgb)
}

/// Same as [`yuyv422_to_rgb`] but with a destination buffer instead of a return `Vec<u8>`
/// # Errors
/// If the stream is invalid Yuv422, or the destination buffer is not large enough, this will error.
#[inline]
pub fn buf_yuyv422_to_rgb(data: &[u8], dest: &mut [u8], rgba: bool) -> Result<(), NokhwaError> {
    let mut buf: Vec<u8> = Vec::new();
    if data.len() % 4 != 0 {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::Yuy2_422,
            destination: "RGB888".to_string(),
            error: "Assertion failure, the YUV stream isn't 4:2:2! (wrong number of bytes)"
                .to_string(),
        });
    }
    for chunk in data.chunks_exact(4) {
        let y0 = f32::from(chunk[0]);
        let u = f32::from(chunk[1]);
        let y1 = f32::from(chunk[2]);
        let v = f32::from(chunk[3]);

        let r0 = y0 + 1.370_705 * (v - 128.);
        let g0 = y0 - 0.698_001 * (v - 128.) - 0.337_633 * (u - 128.);
        let b0 = y0 + 1.732_446 * (u - 128.);

        let r1 = y1 + 1.370_705 * (v - 128.);
        let g1 = y1 - 0.698_001 * (v - 128.) - 0.337_633 * (u - 128.);
        let b1 = y1 + 1.732_446 * (u - 128.);

        if rgba {
            buf.extend_from_slice(&[
                r0 as u8, g0 as u8, b0 as u8, 255, r1 as u8, g1 as u8, b1 as u8, 255,
            ]);
        } else {
            buf.extend_from_slice(&[r0 as u8, g0 as u8, b0 as u8, r1 as u8, g1 as u8, b1 as u8]);
        }
    }
    dest.copy_from_slice(&buf);
    Ok(())
}

// equation from https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
/// Convert `YCbCr` 4:4:4 to a RGB888. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
#[allow(clippy::many_single_char_names)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[must_use]
#[inline]
pub fn yuyv444_to_rgb(y: i32, u: i32, v: i32) -> [u8; 3] {
    let c298 = (y - 16) * 298;
    let d = u - 128;
    let e = v - 128;
    let r = ((c298 + 409 * e + 128) >> 8).clamp(0, 255) as u8;
    let g = ((c298 - 100 * d - 208 * e + 128) >> 8).clamp(0, 255) as u8;
    let b = ((c298 + 516 * d + 128) >> 8).clamp(0, 255) as u8;
    [r, g, b]
}

// equation from https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
/// Convert `YCbCr` 4:4:4 to a RGBA8888. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
///
/// Equivalent to [`yuyv444_to_rgb`] but with an alpha channel attached.
#[allow(clippy::many_single_char_names)]
#[must_use]
#[inline]
pub fn yuyv444_to_rgba(y: i32, u: i32, v: i32) -> [u8; 4] {
    let [r, g, b] = yuyv444_to_rgb(y, u, v);
    [r, g, b, 255]
}

/// Converts a Yuv422 4:2:0 bi-planar (NV12) datastream to a RGB888 Stream. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
/// # Errors
/// This may error when the data stream size is wrong.
#[inline]
pub fn nv12_to_rgb(
    resolution: Resolution,
    data: &[u8],
    rgba: bool,
) -> Result<Vec<u8>, NokhwaError> {
    let pxsize = if rgba { 4 } else { 3 };
    let mut dest = vec![0; (pxsize * resolution.width() * resolution.height()) as usize];
    buf_nv12_to_rgb(resolution, data, &mut dest, rgba)?;
    Ok(dest)
}

// this depresses me
// like, everytime i open this codebase all the life is sucked out of me
// i hate it
/// Converts a Yuv422 4:2:0 bi-planar (NV12) datastream to a RGB888 Stream and outputs it into a destination buffer. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
/// # Errors
/// This may error when the data stream size is wrong.
#[allow(clippy::similar_names)]
#[inline]
pub fn buf_nv12_to_rgb(
    resolution: Resolution,
    data: &[u8],
    out: &mut [u8],
    rgba: bool,
) -> Result<(), NokhwaError> {
    if resolution.width() % 2 != 0 || resolution.height() % 2 != 0 {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::Nv12,
            destination: "RGB".to_string(),
            error: "bad resolution".to_string(),
        });
    }

    if data.len() != ((resolution.width() * resolution.height() * 3) / 2) as usize {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::Nv12,
            destination: "RGB".to_string(),
            error: "bad input buffer size".to_string(),
        });
    }

    let pxsize = if rgba { 4 } else { 3 };

    if out.len() != (pxsize * resolution.width() * resolution.height()) as usize {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::Nv12,
            destination: "RGB".to_string(),
            error: "bad output buffer size".to_string(),
        });
    }

    let rgba_size = if rgba { 4 } else { 3 };

    let y_section = (resolution.width() * resolution.height()) as usize;

    let width_usize = resolution.width() as usize;
    // let height_usize = resolution.height() as usize;

    for (hidx, horizontal_row) in data[0..y_section].chunks_exact(width_usize).enumerate() {
        for (cidx, column) in horizontal_row.chunks_exact(2).enumerate() {
            let u = data[(y_section) + ((hidx / 2) * width_usize) + (cidx * 2)];
            let v = data[(y_section) + ((hidx / 2) * width_usize) + (cidx * 2) + 1];

            let y0 = column[0];
            let y1 = column[1];
            let base_index = (hidx * width_usize * rgba_size) + cidx * rgba_size * 2;

            if rgba {
                let px0 = yuyv444_to_rgba(y0 as i32, u as i32, v as i32);
                let px1 = yuyv444_to_rgba(y1 as i32, u as i32, v as i32);

                out[base_index] = px0[0];
                out[base_index + 1] = px0[1];
                out[base_index + 2] = px0[2];
                out[base_index + 3] = px0[3];
                out[base_index + 4] = px1[0];
                out[base_index + 5] = px1[1];
                out[base_index + 6] = px1[2];
                out[base_index + 7] = px1[3];
            } else {
                let px0 = yuyv444_to_rgb(y0 as i32, u as i32, v as i32);
                let px1 = yuyv444_to_rgb(y1 as i32, u as i32, v as i32);

                out[base_index] = px0[0];
                out[base_index + 1] = px0[1];
                out[base_index + 2] = px0[2];
                out[base_index + 3] = px1[0];
                out[base_index + 4] = px1[1];
                out[base_index + 5] = px1[2];
            }
        }
    }

    Ok(())
}
