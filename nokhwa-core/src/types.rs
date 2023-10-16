use crate::{
    error::NokhwaError,
    frame_format::{FrameFormat},
};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{Display, Formatter},
};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Range<T>
where
    T: Copy + Clone + Debug + PartialOrd + PartialEq,
{
    minimum: Option<T>,
    lower_inclusive: bool,
    maximum: Option<T>,
    upper_inclusive: bool,
    preferred: T,
}

impl<T> Range<T>
where
    T: Copy + Clone + Debug + PartialOrd + PartialEq,
{
    pub fn new(preferred: T, min: Option<T>, max: Option<T>) -> Self {
        Self {
            minimum: min,
            lower_inclusive: true,
            maximum: max,
            upper_inclusive: false,
            preferred,
        }
    }

    pub fn with_inclusive(
        preferred: T,
        min: Option<T>,
        lower_inclusive: bool,
        max: Option<T>,
        upper_inclusive: bool,
    ) -> Self {
        Self {
            minimum: min,
            lower_inclusive,
            maximum: max,
            upper_inclusive,
            preferred,
        }
    }

    pub fn with_preferred(preferred: T) -> Self {
        Self {
            minimum: None,
            lower_inclusive: true,
            maximum: None,
            upper_inclusive: false,
            preferred,
        }
    }

    pub fn does_fit(&self, item: T) -> bool {
        if item == self.preferred {
            true
        }

        if let Some(min) = self.minimum {
            let test = if self.lower_inclusive {
                min >= item
            } else {
                min > item
            };
            if test {
                return false;
            }
        }

        if let Some(max) = self.maximum {
            let test = if self.lower_inclusive {
                max <= item
            } else {
                max < item
            };
            if test {
                return false;
            }
        }

        true
    }


    pub fn set_minimum(&mut self, minimum: Option<T>) {
        self.minimum = minimum;
    }
    pub fn set_lower_inclusive(&mut self, lower_inclusive: bool) {
        self.lower_inclusive = lower_inclusive;
    }
    pub fn set_maximum(&mut self, maximum: Option<T>) {
        self.maximum = maximum;
    }
    pub fn set_upper_inclusive(&mut self, upper_inclusive: bool) {
        self.upper_inclusive = upper_inclusive;
    }
    pub fn set_preferred(&mut self, preferred: T) {
        self.preferred = preferred;
    }

    pub fn minimum(&self) -> Option<T> {
        self.minimum
    }
    pub fn lower_inclusive(&self) -> bool {
        self.lower_inclusive
    }
    pub fn maximum(&self) -> Option<T> {
        self.maximum
    }
    pub fn upper_inclusive(&self) -> bool {
        self.upper_inclusive
    }
    pub fn preferred(&self) -> T {
        self.preferred
    }
}

impl<T> Default for Range<T>
where
    T: Default,
{
    fn default() -> Self {
        Range {
            minimum: None,
            lower_inclusive: true,
            maximum: None,
            upper_inclusive: false,
            preferred: T::default(),
        }
    }
}

/// Describes the index of the camera.
/// - Index: A numbered index
/// - String: A string, used for `IPCameras`.
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
#[cfg_attr(feature = "output-wasm", wasm_bindgen)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq)]
pub struct Resolution {
    pub width_x: u32,
    pub height_y: u32,
}

#[cfg_attr(feature = "output-wasm", wasm_bindgen)]
impl Resolution {
    /// Create a new resolution from 2 image size coordinates.
    /// # JS-WASM
    /// This is exported as a constructor for [`Resolution`].
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(constructor))]
    pub fn new(x: u32, y: u32) -> Self {
        Resolution {
            width_x: x,
            height_y: y,
        }
    }

    /// Get the width of Resolution
    /// # JS-WASM
    /// This is exported as `get_Width`.
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(getter = Width))]
    #[inline]
    pub fn width(self) -> u32 {
        self.width_x
    }

    /// Get the height of Resolution
    /// # JS-WASM
    /// This is exported as `get_Height`.
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(getter = Height))]
    #[inline]
    pub fn height(self) -> u32 {
        self.height_y
    }

    /// Get the x (width) of Resolution
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(skip))]
    #[inline]
    pub fn x(self) -> u32 {
        self.width_x
    }

    /// Get the y (height) of Resolution
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(skip))]
    #[inline]
    pub fn y(self) -> u32 {
        self.height_y
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

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
/// The frame rate of a camera.
pub enum FrameRate {
    /// The driver reports the frame rate as a clean integer (e.g. 30 FPS).
    Integer(u32),
    /// The driver reports the frame rate as a floating point number (e.g. 29.97 FPS)
    Float(f32),
    /// The driver reports the frame rate as a fraction (e.g. 2997/1000 FPS)
    Fraction {
        numerator: u16,
        denominator: u16,
    }
}

impl FrameRate {
    pub fn new_integer(fps: u32) -> Self {
        FrameRate::Integer(fps)
    }

    pub fn new_float(fps: f32) -> Self {
        FrameRate::Float(fps)
    }

    pub fn new_fraction(numerator: u16, denominator: u16) -> Self {
        FrameRate::Fraction {
            numerator,
            denominator,
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            FrameRate::Integer(fps) => fps as f32,
            FrameRate::Float(fps) => fps,
            FrameRate::Fraction { numerator, denominator } => (numerator as f32) / (denominator as f32)
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            FrameRate::Integer(fps) => *fps,
            FrameRate::Float(fps) => fps as u32,
            FrameRate::Fraction { numerator, denominator } => numerator / denominator,
        }
    }
}

impl Default for FrameRate {
    fn default() -> Self {
        FrameRate::Integer(30)
    }
}

impl PartialOrd for FrameRate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let this_float = self.as_float();
        let other = other.as_float();
        this_float.partial_cmp(&other)
    }
}

impl Ord for FrameRate {
    fn cmp(&self, other: &Self) -> Ordering {
        let this_float = self.as_float();
        let other = other.as_float();
        this_float.total_cmp(&other)
    }
}

impl Display for FrameRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameRate::Integer(fps) => write!(f, "Framerate: {fps} FPS"),
            FrameRate::Float(fps) => write!(f, "Framerate: {fps} FPS"),
            FrameRate::Fraction { .. } => {
                let as_float = self.as_float();
                write!(f, "Framerate: {as_float} FPS")
            }
        }
    }
}

impl From<u32> for FrameRate {
    fn from(value: u32) -> Self {
        FrameRate::Integer(value)
    }
}

impl From<f32> for FrameRate {
    fn from(value: f32) -> Self {
        FrameRate::Float(value)
    }
}

impl From<(u16, u16)> for FrameRate {
    fn from(value: (u16, u16)) -> Self {
        FrameRate::Fraction {
            numerator: value.0,
            denominator: value.1,
        }
    }
}

/// This is a convenience struct that holds all information about the format of a webcam stream.
/// It consists of a [`Resolution`], [`FrameFormat`], and a frame rate(u8).
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
            frame_rate: FrameRate::Integer(30),
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
#[cfg_attr(feature = "output-wasm", wasm_bindgen)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct CameraInfo {
    human_name: String,
    description: String,
    misc: String,
    index: CameraIndex,
}

#[cfg_attr(feature = "output-wasm", wasm_bindgen(js_class = CameraInfo))]
impl CameraInfo {
    /// Create a new [`CameraInfo`].
    /// # JS-WASM
    /// This is exported as a constructor for [`CameraInfo`].
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(constructor))]
    // OK, i just checkeed back on this code. WTF was I on when I wrote `&(impl AsRef<str> + ?Sized)` ????
    // I need to get on the same shit that my previous self was on, because holy shit that stuff is strong as FUCK!
    // Finally fixed this insanity. Hopefully I didnt torment anyone by actually putting this in a stable release.
    pub fn new(human_name: &str, description: &str, misc: &str, index: &CameraIndex) -> Self {
        CameraInfo {
            human_name: human_name.to_string(),
            description: description.to_string(),
            misc: misc.to_string(),
            index,
        }
    }

    /// Get a reference to the device info's human readable name.
    /// # JS-WASM
    /// This is exported as a `get_HumanReadableName`.
    #[must_use]
    #[cfg_attr(
    feature = "output-wasm",
    wasm_bindgen(getter = HumanReadableName)
    )]
    // yes, i know, unnecessary alloc this, unnecessary alloc that
    // but wasm bindgen
    pub fn human_name(&self) -> String {
        self.human_name.clone()
    }

    /// Set the device info's human name.
    /// # JS-WASM
    /// This is exported as a `set_HumanReadableName`.
    #[cfg_attr(
    feature = "output-wasm",
    wasm_bindgen(setter = HumanReadableName)
    )]
    pub fn set_human_name(&mut self, human_name: &str) {
        self.human_name = human_name.to_string();
    }

    /// Get a reference to the device info's description.
    /// # JS-WASM
    /// This is exported as a `get_Description`.
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(getter = Description))]
    pub fn description(&self) -> &str {
        self.description.borrow()
    }

    /// Set the device info's description.
    /// # JS-WASM
    /// This is exported as a `set_Description`.
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(setter = Description))]
    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
    }

    /// Get a reference to the device info's misc.
    /// # JS-WASM
    /// This is exported as a `get_MiscString`.
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(getter = MiscString))]
    pub fn misc(&self) -> String {
        self.misc.clone()
    }

    /// Set the device info's misc.
    /// # JS-WASM
    /// This is exported as a `set_MiscString`.
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(setter = MiscString))]
    pub fn set_misc(&mut self, misc: &str) {
        self.misc = misc.to_string();
    }

    /// Get a reference to the device info's index.
    /// # JS-WASM
    /// This is exported as a `get_Index`.
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(getter = Index))]
    pub fn index(&self) -> &CameraIndex {
        &self.index
    }

    /// Set the device info's index.
    /// # JS-WASM
    /// This is exported as a `set_Index`.
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(setter = Index))]
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

/// The list of known camera controls to the library. <br>
/// These can control the picture brightness, etc. <br>
/// Note that not all backends/devices support all these. Run [`supported_camera_controls()`](crate::traits::CaptureTrait::camera_controls) to see which ones can be set.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum KnownCameraControl {
    Brightness,
    Contrast,
    Hue,
    Saturation,
    Sharpness,
    Gamma,
    WhiteBalance,
    BacklightComp,
    Gain,
    Pan,
    Tilt,
    Zoom,
    Exposure,
    Iris,
    Focus,
    /// Other camera control. Listed is the ID.
    /// Wasteful, however is needed for a unified API across Windows, Linux, and MacOSX due to Microsoft's usage of GUIDs.
    ///
    /// THIS SHOULD ONLY BE USED WHEN YOU KNOW THE PLATFORM THAT YOU ARE RUNNING ON.
    Other(u128),
}

/// All camera controls in an array.
#[must_use]
pub const fn all_known_camera_controls() -> [KnownCameraControl; 15] {
    [
        KnownCameraControl::Brightness,
        KnownCameraControl::Contrast,
        KnownCameraControl::Hue,
        KnownCameraControl::Saturation,
        KnownCameraControl::Sharpness,
        KnownCameraControl::Gamma,
        KnownCameraControl::WhiteBalance,
        KnownCameraControl::BacklightComp,
        KnownCameraControl::Gain,
        KnownCameraControl::Pan,
        KnownCameraControl::Tilt,
        KnownCameraControl::Zoom,
        KnownCameraControl::Exposure,
        KnownCameraControl::Iris,
        KnownCameraControl::Focus,
    ]
}

impl Display for KnownCameraControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self)
    }
}

/// This tells you weather a [`KnownCameraControl`] is automatically managed by the OS/Driver
/// or manually managed by you, the programmer.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum KnownCameraControlFlag {
    Automatic,
    Manual,
    Continuous,
    ReadOnly,
    WriteOnly,
    Volatile,
    Disabled,
}

impl Display for KnownCameraControlFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// The values for a [`CameraControl`].
///
/// This provides a wide range of values that can be used to control a camera.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum ControlValueDescription {
    None,
    Integer {
        value: i64,
        default: i64,
        step: i64,
    },
    IntegerRange {
        min: i64,
        max: i64,
        value: i64,
        step: i64,
        default: i64,
    },
    Float {
        value: f64,
        default: f64,
        step: f64,
    },
    FloatRange {
        min: f64,
        max: f64,
        value: f64,
        step: f64,
        default: f64,
    },
    Boolean {
        value: bool,
        default: bool,
    },
    String {
        value: String,
        default: Option<String>,
    },
    Bytes {
        value: Vec<u8>,
        default: Vec<u8>,
    },
    KeyValuePair {
        key: i128,
        value: i128,
        default: (i128, i128),
    },
    Point {
        value: (f64, f64),
        default: (f64, f64),
    },
    Enum {
        value: i64,
        possible: Vec<i64>,
        default: i64,
    },
    RGB {
        value: (f64, f64, f64),
        max: (f64, f64, f64),
        default: (f64, f64, f64),
    },
    StringList {
        value: String,
        availible: Vec<String>,
    },
}

impl ControlValueDescription {
    /// Get the value of this [`ControlValueDescription`]
    #[must_use]
    pub fn value(&self) -> ControlValueSetter {
        match self {
            ControlValueDescription::None => ControlValueSetter::None,
            ControlValueDescription::Integer { value, .. }
            | ControlValueDescription::IntegerRange { value, .. } => {
                ControlValueSetter::Integer(*value)
            }
            ControlValueDescription::Float { value, .. }
            | ControlValueDescription::FloatRange { value, .. } => {
                ControlValueSetter::Float(*value)
            }
            ControlValueDescription::Boolean { value, .. } => ControlValueSetter::Boolean(*value),
            ControlValueDescription::String { value, .. } => {
                ControlValueSetter::String(value.clone())
            }
            ControlValueDescription::Bytes { value, .. } => {
                ControlValueSetter::Bytes(value.clone())
            }
            ControlValueDescription::KeyValuePair { key, value, .. } => {
                ControlValueSetter::KeyValue(*key, *value)
            }
            ControlValueDescription::Point { value, .. } => {
                ControlValueSetter::Point(value.0, value.1)
            }
            ControlValueDescription::Enum { value, .. } => ControlValueSetter::EnumValue(*value),
            ControlValueDescription::RGB { value, .. } => {
                ControlValueSetter::RGB(value.0, value.1, value.2)
            }
            ControlValueDescription::StringList { value, .. } => {
                ControlValueSetter::StringList(value.clone())
            }
        }
    }

    /// Verifies if the [setter](ControlValueSetter) is valid for the provided [`ControlValueDescription`].
    /// - `true` => Is valid.
    /// - `false` => Is not valid.
    ///
    /// If the step is 0, it will automatically return `true`.
    #[must_use]
    pub fn verify_setter(&self, setter: &ControlValueSetter) -> bool {
        match self {
            ControlValueDescription::None => setter.as_none().is_some(),
            ControlValueDescription::Integer {
                value,
                default,
                step,
            } => {
                if *step == 0 {
                    return true;
                }
                match setter.as_integer() {
                    Some(i) => (i + default) % step == 0 || (i + value) % step == 0,
                    None => false,
                }
            }
            ControlValueDescription::IntegerRange {
                min,
                max,
                value,
                step,
                default,
            } => {
                if *step == 0 {
                    return true;
                }
                match setter.as_integer() {
                    Some(i) => {
                        ((i + default) % step == 0 || (i + value) % step == 0)
                            && i >= min
                            && i <= max
                    }
                    None => false,
                }
            }
            ControlValueDescription::Float {
                value,
                default,
                step,
            } => {
                if step.abs() == 0_f64 {
                    return true;
                }
                match setter.as_float() {
                    Some(f) => (f - default).abs() % step == 0_f64 || (f - value) % step == 0_f64,
                    None => false,
                }
            }
            ControlValueDescription::FloatRange {
                min,
                max,
                value,
                step,
                default,
            } => {
                if step.abs() == 0_f64 {
                    return true;
                }

                match setter.as_float() {
                    Some(f) => {
                        ((f - default).abs() % step == 0_f64 || (f - value) % step == 0_f64)
                            && f >= min
                            && f <= max
                    }
                    None => false,
                }
            }
            ControlValueDescription::Boolean { .. } => setter.as_boolean().is_some(),
            ControlValueDescription::String { .. } => setter.as_str().is_some(),
            ControlValueDescription::Bytes { .. } => setter.as_bytes().is_some(),
            ControlValueDescription::KeyValuePair { .. } => setter.as_key_value().is_some(),
            ControlValueDescription::Point { .. } => match setter.as_point() {
                Some(pt) => {
                    !pt.0.is_nan() && !pt.1.is_nan() && pt.0.is_finite() && pt.1.is_finite()
                }
                None => false,
            },
            ControlValueDescription::Enum { possible, .. } => match setter.as_enum() {
                Some(e) => possible.contains(e),
                None => false,
            },
            ControlValueDescription::RGB { max, .. } => match setter.as_rgb() {
                Some(v) => *v.0 >= max.0 && *v.1 >= max.1 && *v.2 >= max.2,
                None => false,
            },
            ControlValueDescription::StringList { value, availible } => {
                availible.contains(setter.as_str())
            }
        }

        // match setter {
        //     ControlValueSetter::None => {
        //         matches!(self, ControlValueDescription::None)
        //     }
        //     ControlValueSetter::Integer(i) => match self {
        //         ControlValueDescription::Integer {
        //             value,
        //             default,
        //             step,
        //         } => (i - default).abs() % step == 0 || (i - value) % step == 0,
        //         ControlValueDescription::IntegerRange {
        //             min,
        //             max,
        //             value,
        //             step,
        //             default,
        //         } => {
        //             if value > max || value < min {
        //                 return false;
        //             }
        //
        //             (i - default) % step == 0 || (i - value) % step == 0
        //         }
        //         _ => false,
        //     },
        //     ControlValueSetter::Float(f) => match self {
        //         ControlValueDescription::Float {
        //             value,
        //             default,
        //             step,
        //         } => (f - default).abs() % step == 0_f64 || (f - value) % step == 0_f64,
        //         ControlValueDescription::FloatRange {
        //             min,
        //             max,
        //             value,
        //             step,
        //             default,
        //         } => {
        //             if value > max || value < min {
        //                 return false;
        //             }
        //
        //             (f - default) % step == 0_f64 || (f - value) % step == 0_f64
        //         }
        //         _ => false,
        //     },
        //     ControlValueSetter::Boolean(b) => {
        //
        //     }
        //     ControlValueSetter::String(_) => {
        //         matches!(self, ControlValueDescription::String { .. })
        //     }
        //     ControlValueSetter::Bytes(_) => {
        //         matches!(self, ControlValueDescription::Bytes { .. })
        //     }
        //     ControlValueSetter::KeyValue(_, _) => {
        //         matches!(self, ControlValueDescription::KeyValuePair { .. })
        //     }
        //     ControlValueSetter::Point(_, _) => {
        //         matches!(self, ControlValueDescription::Point { .. })
        //     }
        //     ControlValueSetter::EnumValue(_) => {
        //         matches!(self, ControlValueDescription::Enum { .. })
        //     }
        //     ControlValueSetter::RGB(_, _, _) => {
        //         matches!(self, ControlValueDescription::RGB { .. })
        //     }
        // }
    }
}

impl Display for ControlValueDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlValueDescription::None => {
                write!(f, "(None)")
            }
            ControlValueDescription::Integer {
                value,
                default,
                step,
            } => {
                write!(f, "(Current: {value}, Default: {default}, Step: {step})",)
            }
            ControlValueDescription::IntegerRange {
                min,
                max,
                value,
                step,
                default,
            } => {
                write!(
                    f,
                    "(Current: {value}, Default: {default}, Step: {step}, Range: ({min}, {max}))",
                )
            }
            ControlValueDescription::Float {
                value,
                default,
                step,
            } => {
                write!(f, "(Current: {value}, Default: {default}, Step: {step})",)
            }
            ControlValueDescription::FloatRange {
                min,
                max,
                value,
                step,
                default,
            } => {
                write!(
                    f,
                    "(Current: {value}, Default: {default}, Step: {step}, Range: ({min}, {max}))",
                )
            }
            ControlValueDescription::Boolean { value, default } => {
                write!(f, "(Current: {value}, Default: {default})")
            }
            ControlValueDescription::String { value, default } => {
                write!(f, "(Current: {value}, Default: {default:?})")
            }
            ControlValueDescription::Bytes { value, default } => {
                write!(f, "(Current: {value:x?}, Default: {default:x?})")
            }
            ControlValueDescription::KeyValuePair {
                key,
                value,
                default,
            } => {
                write!(
                    f,
                    "Current: ({key}, {value}), Default: ({}, {})",
                    default.0, default.1
                )
            }
            ControlValueDescription::Point { value, default } => {
                write!(
                    f,
                    "Current: ({}, {}), Default: ({}, {})",
                    value.0, value.1, default.0, default.1
                )
            }
            ControlValueDescription::Enum {
                value,
                possible,
                default,
            } => {
                write!(
                    f,
                    "Current: {value}, Possible Values: {possible:?}, Default: {default}",
                )
            }
            ControlValueDescription::RGB {
                value,
                max,
                default,
            } => {
                write!(
                    f,
                    "Current: ({}, {}, {}), Max: ({}, {}, {}), Default: ({}, {}, {})",
                    value.0, value.1, value.2, max.0, max.1, max.2, default.0, default.1, default.2
                )
            }
            ControlValueDescription::StringList { value, availible } => {
                write!(f, "Current: {value}, Availible: {availible:?}")
            }
        }
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

/// This struct tells you everything about a particular [`KnownCameraControl`].
///
/// However, you should never need to instantiate this struct, since its usually generated for you by `nokhwa`.
/// The only time you should be modifying this struct is when you need to set a value and pass it back to the camera.
/// NOTE: Assume the values for `min` and `max` as **non-inclusive**!.
/// E.g. if the [`CameraControl`] says `min` is 100, the minimum is actually 101.
#[derive(Clone, Debug, PartialOrd, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct CameraControl {
    control: KnownCameraControl,
    name: String,
    description: ControlValueDescription,
    flag: Vec<KnownCameraControlFlag>,
    active: bool,
}

impl CameraControl {
    /// Creates a new [`CameraControl`]
    #[must_use]
    pub fn new(
        control: KnownCameraControl,
        name: String,
        description: ControlValueDescription,
        flag: Vec<KnownCameraControlFlag>,
        active: bool,
    ) -> Self {
        CameraControl {
            control,
            name,
            description,
            flag,
            active,
        }
    }

    /// Gets the name of this [`CameraControl`]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the [`ControlValueDescription`] of this [`CameraControl`]
    #[must_use]
    pub fn description(&self) -> &ControlValueDescription {
        &self.description
    }

    /// Gets the [`ControlValueSetter`] of the [`ControlValueDescription`] of this [`CameraControl`]
    #[must_use]
    pub fn value(&self) -> ControlValueSetter {
        self.description.value()
    }

    /// Gets the [`KnownCameraControl`] of this [`CameraControl`]
    #[must_use]
    pub fn control(&self) -> KnownCameraControl {
        self.control
    }

    /// Gets the [`KnownCameraControlFlag`] of this [`CameraControl`],
    /// telling you weather this control is automatically set or manually set.
    #[must_use]
    pub fn flag(&self) -> &[KnownCameraControlFlag] {
        &self.flag
    }

    /// Gets `active` of this [`CameraControl`],
    /// telling you weather this control is currently active(in-use).
    #[must_use]
    pub fn active(&self) -> bool {
        self.active
    }

    /// Gets `active` of this [`CameraControl`],
    /// telling you weather this control is currently active(in-use).
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Display for CameraControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Control: {}, Name: {}, Value: {}, Flag: {:?}, Active: {}",
            self.control, self.name, self.description, self.flag, self.active
        )
    }
}

/// The setter for a control value
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum ControlValueSetter {
    None,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Bytes(Vec<u8>),
    KeyValue(i128, i128),
    Point(f64, f64),
    EnumValue(i64),
    RGB(f64, f64, f64),
    StringList(String),
}

impl ControlValueSetter {
    #[must_use]
    pub fn as_none(&self) -> Option<()> {
        if let ControlValueSetter::None = self {
            Some(())
        } else {
            None
        }
    }
    #[must_use]

    pub fn as_integer(&self) -> Option<&i64> {
        if let ControlValueSetter::Integer(i) = self {
            Some(i)
        } else {
            None
        }
    }
    #[must_use]

    pub fn as_float(&self) -> Option<&f64> {
        if let ControlValueSetter::Float(f) = self {
            Some(f)
        } else {
            None
        }
    }
    #[must_use]

    pub fn as_boolean(&self) -> Option<&bool> {
        if let ControlValueSetter::Boolean(f) = self {
            Some(f)
        } else {
            None
        }
    }
    #[must_use]

    pub fn as_str(&self) -> Option<&str> {
        if let ControlValueSetter::String(s) = self {
            Some(s)
        } else {
            None
        }
    }
    #[must_use]

    pub fn as_bytes(&self) -> Option<&[u8]> {
        if let ControlValueSetter::Bytes(b) = self {
            Some(b)
        } else {
            None
        }
    }
    #[must_use]

    pub fn as_key_value(&self) -> Option<(&i128, &i128)> {
        if let ControlValueSetter::KeyValue(k, v) = self {
            Some((k, v))
        } else {
            None
        }
    }
    #[must_use]

    pub fn as_point(&self) -> Option<(&f64, &f64)> {
        if let ControlValueSetter::Point(x, y) = self {
            Some((x, y))
        } else {
            None
        }
    }
    #[must_use]

    pub fn as_enum(&self) -> Option<&i64> {
        if let ControlValueSetter::EnumValue(e) = self {
            Some(e)
        } else {
            None
        }
    }
    #[must_use]

    pub fn as_rgb(&self) -> Option<(&f64, &f64, &f64)> {
        if let ControlValueSetter::RGB(r, g, b) = self {
            Some((r, g, b))
        } else {
            None
        }
    }
}

impl Display for ControlValueSetter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlValueSetter::None => {
                write!(f, "Value: None")
            }
            ControlValueSetter::Integer(i) => {
                write!(f, "IntegerValue: {i}")
            }
            ControlValueSetter::Float(d) => {
                write!(f, "FloatValue: {d}")
            }
            ControlValueSetter::Boolean(b) => {
                write!(f, "BoolValue: {b}")
            }
            ControlValueSetter::String(s) => {
                write!(f, "StrValue: {s}")
            }
            ControlValueSetter::Bytes(b) => {
                write!(f, "BytesValue: {b:x?}")
            }
            ControlValueSetter::KeyValue(k, v) => {
                write!(f, "KVValue: ({k}, {v})")
            }
            ControlValueSetter::Point(x, y) => {
                write!(f, "PointValue: ({x}, {y})")
            }
            ControlValueSetter::EnumValue(v) => {
                write!(f, "EnumValue: {v}")
            }
            ControlValueSetter::RGB(r, g, b) => {
                write!(f, "RGBValue: ({r}, {g}, {b})")
            }
            ControlValueSetter::StringList(s) => {
                write!(f, "StringListValue: {s}")
            }
        }
    }
}

/// The list of known capture backends to the library. <br>
/// - `AUTO` is special - it tells the Camera struct to automatically choose a backend most suited for the current platform.
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

#[cfg(all(feature = "mjpeg", not(target_arch = "wasm")))]
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
#[cfg(all(feature = "mjpeg", not(target_arch = "wasm")))]
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

#[cfg(not(all(feature = "mjpeg", not(target_arch = "wasm"))))]
pub fn mjpeg_to_rgb(_data: &[u8], _rgba: bool) -> Result<Vec<u8>, NokhwaError> {
    Err(NokhwaError::NotImplementedError(
        "Not available on WASM".to_string(),
    ))
}

/// Equivalent to [`mjpeg_to_rgb`] except with a destination buffer.
/// # Errors
/// If the decoding fails (e.g. invalid MJpeg stream), the buffer is not large enough, or you are doing this on `WebAssembly`, this will error.
#[cfg(all(feature = "mjpeg", not(target_arch = "wasm")))]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "mjpeg")))]
#[inline]
pub fn buf_mjpeg_to_rgb(data: &[u8], dest: &mut [u8], rgba: bool) -> Result<(), NokhwaError> {
    let mut jpeg_decompress = decompress(data, rgba)?;

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

#[cfg(not(all(feature = "mjpeg", not(target_arch = "wasm"))))]
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
            src: FrameFormat::Yuv422.into(),
            destination: "RGB888".to_string(),
            error: "Assertion failure, the YUV stream isn't 4:2:2! (wrong number of bytes)"
                .to_string(),
        });
    }
    for chunk in data.chunks_exact(4) {
        let y0 = chunk[0] as f32;
        let u = chunk[1] as f32;
        let y1 = chunk[2] as f32;
        let v = chunk[3] as f32;

        let r0 = y0 + 1.370705 * (v - 128.);
        let g0 = y0 - 0.698001 * (v - 128.) - 0.337633 * (u - 128.);
        let b0 = y0 + 1.732446 * (u - 128.);

        let r1 = y1 + 1.370705 * (v - 128.);
        let g1 = y1 - 0.698001 * (v - 128.) - 0.337633 * (u - 128.);
        let b1 = y1 + 1.732446 * (u - 128.);

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
