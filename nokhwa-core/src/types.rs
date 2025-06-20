use crate::ranges::RangeItem;
use crate::utils::Distance;
use crate::{error::NokhwaError, frame_format::FrameFormat};
use num_rational::Rational32;
use num_traits::FromPrimitive;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::num::NonZeroI32;
use std::ops::{Div, Rem};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
    hash::Hash,
    ops::Sub,
};

/// Describes the index of the camera.
/// - Index: A numbered index
/// - String: A string, used for `IPCameras` or on the Browser as `DeviceIDs`.
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
    width_x: u32,
    height_y: u32,
}

impl Resolution {
    /// Create a new resolution from 2 image size coordinates.
    #[must_use]
    // TODO: make this height and width.
    pub const fn new(x: u32, y: u32) -> Self {
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

impl Div for Resolution {
    type Output = Resolution;

    fn div(self, rhs: Self) -> Self::Output {
        let x_div = self.x().div(rhs.x());
        let y_div = self.y().div(rhs.y());
        Resolution::new(x_div, y_div)
    }
}

impl Sub for Resolution {
    type Output = Resolution;

    fn sub(self, rhs: Self) -> Self::Output {
        let x_sub = self.x().sub(rhs.x());
        let y_sub = self.y().sub(rhs.y());
        Resolution::new(x_sub, y_sub)
    }
}

impl Rem for Resolution {
    type Output = Resolution;

    fn rem(self, rhs: Self) -> Self::Output {
        let x_rem = self.x().rem(rhs.x());
        let y_rem = self.y().rem(rhs.y());
        Resolution::new(x_rem, y_rem)
    }
}

impl RangeItem for Resolution {
    const ZERO: Self = Resolution::new(0, 0);
    const MIN: Self = Resolution::new(u32::MIN, u32::MIN);
    const MAX: Self = Resolution::new(u32::MAX, u32::MAX);
}

/// Framerate of a camera, backed by a num-rational Ratio type.
///
/// Note that while constructing negative is allowed, the absolute value
/// will be passed to the driver.
///
/// # Panics
/// If denominator is 0, any attempt to use the [`FrameRate`] will **panic.**
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct FrameRate {
    rational: Rational32,
}

impl FrameRate {
    #[must_use] pub const fn new(numerator: i32, denominator: NonZeroI32) -> Self {
        Self {
            rational: Rational32::new_raw(numerator, denominator.get()),
        }
    }

    #[must_use] pub const fn from_fps(fps: i32) -> Self {
        Self {
            rational: Rational32::new_raw(fps, 1),
        }
    }

    #[must_use] pub fn numerator(&self) -> i32 {
        *self.rational.numer()
    }

    #[must_use] pub fn denominator(&self) -> i32 {
        *self.rational.denom()
    }

    #[must_use] pub fn as_raw(&self) -> &Rational32 {
        &self.rational
    }

    #[must_use] pub fn approximate_float(&self) -> Option<f32> {
        let numerator_float = f32::from_i32(self.numerator())?;
        let denominator_float = f32::from_i32(self.denominator())?;

        Some(numerator_float / denominator_float)
    }
}

impl Default for FrameRate {
    fn default() -> Self {
        FrameRate::new(30, NonZeroI32::new(1).unwrap())
    }
}

impl Display for FrameRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{} FPS", self.numerator(), self.denominator())
    }
}

impl Div for FrameRate {
    type Output = FrameRate;

    fn div(self, rhs: Self) -> Self::Output {
        self.rational.div(rhs.rational).into()
    }
}

impl Sub for FrameRate {
    type Output = FrameRate;

    fn sub(self, rhs: Self) -> Self::Output {
        self.rational.sub(rhs.rational).into()
    }
}

impl Sub for &FrameRate {
    type Output = FrameRate;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.rational - rhs.rational).into()
    }
}

impl Rem for FrameRate {
    type Output = FrameRate;

    fn rem(self, rhs: Self) -> Self::Output {
        self.rational.rem(rhs.rational).into()
    }
}

impl RangeItem for FrameRate {
    const ZERO: Self = FrameRate::from_fps(0);
    const MIN: Self = FrameRate::from_fps(0);
    const MAX: Self = FrameRate::from_fps(i32::MAX);
}

impl From<Rational32> for FrameRate {
    fn from(value: Rational32) -> Self {
        FrameRate { rational: value }
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
    pub const fn new(resolution: Resolution, format: FrameFormat, frame_rate: FrameRate) -> Self {
        CameraFormat {
            resolution,
            format,
            frame_rate,
        }
    }

    /// [`CameraFormat::new()`], but raw.
    #[must_use]
    pub const fn new_from(res_x: u32, res_y: u32, format: FrameFormat, fps: FrameRate) -> Self {
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
    pub fn resolution(&self) -> &Resolution {
        &self.resolution
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
    pub fn frame_rate(&self) -> &FrameRate {
        &self.frame_rate
    }

    /// Set the [`CameraFormat`]'s frame rate.
    pub fn set_frame_rate(&mut self, frame_rate: FrameRate) {
        self.frame_rate = frame_rate;
    }

    /// Get the [`CameraFormat`]'s format.
    #[must_use]
    pub fn format(&self) -> &FrameFormat {
        &self.format
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
            frame_rate: FrameRate::default(),
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
pub struct CameraInformation {
    human_name: String,
    description: String,
    misc: String,
    index: CameraIndex,
}

impl CameraInformation {
    /// Create a new [`CameraInformation`].
    #[must_use]
    // OK, i just checkeed back on this code. WTF was I on when I wrote `&(impl AsRef<str> + ?Sized)` ????
    // I need to get on the same shit that my previous self was on, because holy shit that stuff is strong as FUCK!
    // Finally fixed this insanity. Hopefully I didnt torment anyone by actually putting this in a stable release.
    pub fn new(human_name: String, description: String, misc: String, index: CameraIndex) -> Self {
        CameraInformation {
            human_name,
            description,
            misc,
            index,
        }
    }

    /// Get a reference to the device info's human readable name.
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

impl Display for CameraInformation {
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

// /// The list of known capture backends to the library. <br>
// /// - `Auto` - Use automatic selection.
// /// - `AVFoundation` - Uses `AVFoundation` on `MacOSX`
// /// - `Video4Linux` - `Video4Linux2`, a linux specific backend.
// /// - `UniversalVideoClass` -  ***DEPRECATED*** Universal Video Class (please check [libuvc](https://github.com/libuvc/libuvc)). Platform agnostic, although on linux it needs `sudo` permissions or similar to use.
// /// - `MediaFoundation` - Microsoft Media Foundation, Windows only,
// /// - `OpenCv` - Uses `OpenCV` to capture. Platform agnostic.
// /// - `GStreamer` - ***DEPRECATED*** Uses `GStreamer` RTP to capture. Platform agnostic.
// /// - `Browser` - Uses browser APIs to capture from a webcam.
// pub enum SelectableBackend {
//     Auto,
//     Custom(&'static str),
//     AVFoundation,
//     Video4Linux,
//     UniversalVideoClass,
//     MediaFoundation,
//     OpenCv,
//     GStreamer,
//     Browser,
// }
//
// /// The list of known capture backends to the library. <br>
// /// - `AVFoundation` - Uses `AVFoundation` on `MacOSX`
// /// - `Video4Linux` - `Video4Linux2`, a linux specific backend.
// /// - `UniversalVideoClass` -  ***DEPRECATED*** Universal Video Class (please check [libuvc](https://github.com/libuvc/libuvc)). Platform agnostic, although on linux it needs `sudo` permissions or similar to use.
// /// - `MediaFoundation` - Microsoft Media Foundation, Windows only,
// /// - `OpenCv` - Uses `OpenCV` to capture. Platform agnostic.
// /// - `GStreamer` - ***DEPRECATED*** Uses `GStreamer` RTP to capture. Platform agnostic.
// /// - `Browser` - Uses browser APIs to capture from a webcam.
// #[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
// #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
// pub enum ApiBackend {
//     Custom(&'static str),
//     AVFoundation,
//     Video4Linux,
//     UniversalVideoClass,
//     MediaFoundation,
//     OpenCv,
//     GStreamer,
//     Browser,
// }
//
// impl Display for ApiBackend {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{self:?}")
//     }
// }
