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

use crate::NokhwaError;
#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
))]
use nokhwa_bindings_macos::avfoundation::{
    AVCaptureDeviceDescriptor, AVFourCC, AVVideoResolution, CaptureDeviceFormatDescriptor,
};
#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
use nokhwa_bindings_windows::{
    MFCameraFormat, MFControl, MFFrameFormat, MFResolution, MediaFoundationControls,
    MediaFoundationDeviceDescriptor,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    fmt::{Display, Formatter},
};
#[cfg(feature = "input-uvc")]
use uvc::StreamFormat;
#[cfg(all(feature = "input-v4l", target_os = "linux"))]
use v4l::{control::Description, Format, FourCC};
#[cfg(feature = "output-wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Tells the init function what camera format to pick.
/// - `HighestResolution(Option<u32>)`: Pick the highest [`Resolution`] for the given framerate (the `Option<u32>`). If its `None`, it will pick the highest possible [`Resolution`]
/// - `HighestFrameRate(Option<Resolution>)`: Pick the highest frame rate for the given [`Resolution`] (the `Option<Resolution>`). If it is `None`, it will pick the highest possinle framerate.
/// - `Exact`: Pick the exact [`CameraFormat`] provided.
/// - `None`: Pick a random [`CameraFormat`]
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "output-wasm", wasm_bindgen)]
pub enum BestEffort {
    HighestResolution(Option<u32>),
    HighestFrameRate(Option<Resolution>),
    Exact(CameraFormat),
    None,
}

/// Describes the index of the camera.
/// - Index: A numbered index
/// - String: A string, used for IPCameras.
#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl AsRef<str> for CameraIndex {
    fn as_ref(&self) -> &str {
        self.to_string().as_str()
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

/// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC`.
/// - YUYV is a mathematical color space. You can read more [here.](https://en.wikipedia.org/wiki/YCbCr)
/// - MJPEG is a motion-jpeg compressed frame, it allows for high frame rates.
/// - GRAY is a grayscale image format, usually for specialized cameras such as IR Cameras.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FrameFormat {
    MJPEG,
    YUYV,
    GRAY,
}

impl Display for FrameFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameFormat::MJPEG => {
                write!(f, "MJPEG")
            }
            FrameFormat::YUYV => {
                write!(f, "YUYV")
            }
            FrameFormat::GRAY => {
                write!(f, "GRAY8")
            }
        }
    }
}

#[cfg(feature = "input-uvc")]
impl From<FrameFormat> for uvc::FrameFormat {
    fn from(ff: FrameFormat) -> Self {
        match ff {
            FrameFormat::MJPEG => uvc::FrameFormat::MJPEG,
            FrameFormat::YUYV => uvc::FrameFormat::YUYV,
        }
    }
}

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<MFFrameFormat> for FrameFormat {
    fn from(mf_ff: MFFrameFormat) -> Self {
        match mf_ff {
            MFFrameFormat::MJPEG => FrameFormat::MJPEG,
            MFFrameFormat::YUYV => FrameFormat::YUYV,
            MFFrameFormat::GRAY8 => FrameFormat::GRAY,
        }
    }
}

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<FrameFormat> for MFFrameFormat {
    fn from(ff: FrameFormat) -> Self {
        match ff {
            FrameFormat::MJPEG => MFFrameFormat::MJPEG,
            FrameFormat::YUYV => MFFrameFormat::YUYV,
            FrameFormat::GRAY => MFFrameFormat::GRAY8, //FIXME
        }
    }
}

#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
))]
impl From<AVFourCC> for FrameFormat {
    fn from(av_fcc: AVFourCC) -> Self {
        match av_fcc {
            AVFourCC::YUV2 => FrameFormat::YUYV,
            AVFourCC::MJPEG => FrameFormat::MJPEG,
            AVFourCC::GRAY8 => FrameFormat::GRAY,
        }
    }
}

#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
))]
impl From<FrameFormat> for AVFourCC {
    fn from(ff: FrameFormat) -> Self {
        match ff {
            FrameFormat::MJPEG => AVFourCC::MJPEG,
            FrameFormat::YUYV => AVFourCC::YUV2,
            FrameFormat::GRAY => AVFourCC::GRAY8,
        }
    }
}

#[must_use]
pub const fn frame_formats() -> [FrameFormat; 3] {
    [FrameFormat::MJPEG, FrameFormat::YUYV, FrameFormat::GRAY]
}

/// Describes a Resolution.
/// This struct consists of a Width and a Height value (x,y). <br>
/// Note: the [`Ord`] implementation of this struct is flipped from highest to lowest.
/// # JS-WASM
/// This is exported as `JSResolution`
#[cfg_attr(feature = "output-wasm", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    pub fn width(self) -> u32 {
        self.width_x
    }

    /// Get the height of Resolution
    /// # JS-WASM
    /// This is exported as `get_Height`.
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(getter = Height))]
    pub fn height(self) -> u32 {
        self.height_y
    }

    /// Get the x (width) of Resolution
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(skip))]
    pub fn x(self) -> u32 {
        self.width_x
    }

    /// Get the y (height) of Resolution
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(skip))]
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

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<MFResolution> for Resolution {
    fn from(mf_res: MFResolution) -> Self {
        Resolution {
            width_x: mf_res.width_x,
            height_y: mf_res.height_y,
        }
    }
}

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<Resolution> for MFResolution {
    fn from(res: Resolution) -> Self {
        MFResolution {
            width_x: res.width(),
            height_y: res.height(),
        }
    }
}

#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
))]
#[allow(clippy::cast_sign_loss)]
impl From<AVVideoResolution> for Resolution {
    fn from(res: AVVideoResolution) -> Self {
        Resolution {
            width_x: res.width as u32,
            height_y: res.height as u32,
        }
    }
}

/// This is a convenience struct that holds all information about the format of a webcam stream.
/// It consists of a [`Resolution`], [`FrameFormat`], and a frame rate(u8).
#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CameraFormat {
    resolution: Resolution,
    format: FrameFormat,
    frame_rate: u32,
}

impl CameraFormat {
    /// Construct a new [`CameraFormat`]
    #[must_use]
    pub fn new(resolution: Resolution, format: FrameFormat, frame_rate: u32) -> Self {
        CameraFormat {
            resolution,
            format,
            frame_rate,
        }
    }

    /// [`CameraFormat::new()`], but raw.
    #[must_use]
    pub fn new_from(res_x: u32, res_y: u32, format: FrameFormat, fps: u32) -> Self {
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
    pub fn frame_rate(&self) -> u32 {
        self.frame_rate
    }

    /// Set the [`CameraFormat`]'s frame rate.
    pub fn set_frame_rate(&mut self, frame_rate: u32) {
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

#[cfg(feature = "input-uvc")]
impl From<CameraFormat> for StreamFormat {
    fn from(cf: CameraFormat) -> Self {
        StreamFormat {
            width: cf.width(),
            height: cf.height(),
            fps: cf.frame_rate(),
            format: cf.format().into(),
        }
    }
}

impl Default for CameraFormat {
    fn default() -> Self {
        CameraFormat {
            resolution: Resolution::new(640, 480),
            format: FrameFormat::MJPEG,
            frame_rate: 30,
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

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<MFCameraFormat> for CameraFormat {
    fn from(mf_cam_fmt: MFCameraFormat) -> Self {
        CameraFormat {
            resolution: mf_cam_fmt.resolution().into(),
            format: mf_cam_fmt.format().into(),
            frame_rate: mf_cam_fmt.frame_rate(),
        }
    }
}

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<CameraFormat> for MFCameraFormat {
    fn from(cf: CameraFormat) -> Self {
        MFCameraFormat::new(cf.resolution.into(), cf.format.into(), cf.frame_rate)
    }
}

#[cfg(all(feature = "input-v4l", target_os = "linux"))]
impl From<CameraFormat> for Format {
    fn from(cam_fmt: CameraFormat) -> Self {
        let pxfmt = match cam_fmt.format() {
            FrameFormat::MJPEG => FourCC::new(b"MJPG"),
            FrameFormat::YUYV => FourCC::new(b"YUYV"),
            FrameFormat::GRAY => FourCC::new(b"GREY"),
        };

        Format::new(cam_fmt.width(), cam_fmt.height(), pxfmt)
    }
}

#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
))]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_lossless)]
impl From<CameraFormat> for CaptureDeviceFormatDescriptor {
    fn from(cf: CameraFormat) -> Self {
        CaptureDeviceFormatDescriptor {
            resolution: AVVideoResolution {
                width: cf.width() as i32,
                height: cf.height() as i32,
            },
            fps: cf.frame_rate(),
            fourcc: cf.format().into(),
        }
    }
}

/// Information about a Camera e.g. its name.
/// `description` amd `misc` may contain information that may differ from backend to backend. Refer to each backend for details.
/// `index` is a camera's index given to it by (usually) the OS usually in the order it is known to the system.
#[derive(Clone, Debug, Hash, PartialEq, PartialOrd)]
#[cfg_attr(feature = "output-wasm", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    pub fn new(human_name: &str, description: &str, misc: &str, index: CameraIndex) -> Self {
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

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<MediaFoundationDeviceDescriptor<'_>> for CameraInfo {
    fn from(dev_desc: MediaFoundationDeviceDescriptor<'_>) -> Self {
        CameraInfo {
            human_name: dev_desc.name_as_string(),
            description: "Media Foundation Device".to_string(),
            misc: dev_desc.link_as_string(),
            index: dev_desc.index() as u32,
        }
    }
}

#[cfg(any(
    all(
        feature = "input-avfoundation",
        any(target_os = "macos", target_os = "ios")
    ),
    all(
        feature = "docs-only",
        feature = "docs-nolink",
        feature = "input-avfoundation"
    )
))]
#[allow(clippy::cast_possible_truncation)]
impl From<AVCaptureDeviceDescriptor> for CameraInfo {
    fn from(descriptor: AVCaptureDeviceDescriptor) -> Self {
        CameraInfo {
            human_name: descriptor.name,
            description: descriptor.description,
            misc: descriptor.misc,
            index: descriptor.index as u32,
        }
    }
}

/// The list of known camera controls to the library. <br>
/// These can control the picture brightness, etc. <br>
/// Note that not all backends/devices support all these. Run [`supported_camera_controls()`](crate::CaptureBackendTrait::camera_controls) to see which ones can be set.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<MediaFoundationControls> for KnownCameraControl {
    fn from(mf_c: MediaFoundationControls) -> Self {
        match mf_c {
            MediaFoundationControls::Brightness => KnownCameraControl::Brightness,
            MediaFoundationControls::Contrast => KnownCameraControl::Contrast,
            MediaFoundationControls::Hue => KnownCameraControl::Hue,
            MediaFoundationControls::Saturation => KnownCameraControl::Saturation,
            MediaFoundationControls::Sharpness => KnownCameraControl::Sharpness,
            MediaFoundationControls::Gamma => KnownCameraControl::Gamma,
            MediaFoundationControls::WhiteBalance => KnownCameraControl::WhiteBalance,
            MediaFoundationControls::BacklightComp => KnownCameraControl::BacklightComp,
            MediaFoundationControls::Gain => KnownCameraControl::Gain,
            MediaFoundationControls::Pan => KnownCameraControl::Pan,
            MediaFoundationControls::Tilt => KnownCameraControl::Tilt,
            MediaFoundationControls::Zoom => KnownCameraControl::Zoom,
            MediaFoundationControls::Exposure => KnownCameraControl::Exposure,
            MediaFoundationControls::Iris => KnownCameraControl::Iris,
            MediaFoundationControls::Focus => KnownCameraControl::Focus,
            MediaFoundationControls::ColorEnable => KnownCameraControl::Other(0),
            MediaFoundationControls::Roll => KnownCameraControl::Other(1),
        }
    }
}

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<MFControl> for KnownCameraControl {
    fn from(mf_cc: MFControl) -> Self {
        mf_cc.control().into()
    }
}

#[cfg(all(feature = "input-v4l", target_os = "linux"))]
impl From<Description> for KnownCameraControl {
    fn from(value: Description) -> KnownCameraControl {
        match value.id {
            9_963_776 => KnownCameraControl::Brightness,
            9_963_777 => KnownCameraControl::Contrast,
            9_963_779 => KnownCameraControl::Hue,
            9_963_778 => KnownCameraControl::Saturation,
            9_963_803 => KnownCameraControl::Sharpness,
            9_963_792 => KnownCameraControl::Gamma,
            9_963_802 => KnownCameraControl::WhiteBalance,
            9_963_804 => KnownCameraControl::BacklightComp,
            9_963_795 => KnownCameraControl::Gain,
            10_094_852 => KnownCameraControl::Pan,
            10_094_853 => KnownCameraControl::Tilt,
            10_094_862 => KnownCameraControl::Zoom,
            10_094_850 => KnownCameraControl::Exposure,
            10_094_866 => KnownCameraControl::Iris,
            10_094_859 => KnownCameraControl::Focus,
            id => KnownCameraControl::Other(id as u128),
        }
    }
}

/// This tells you weather a [`KnownCameraControl`] is automatically managed by the OS/Driver
/// or manually managed by you, the programmer.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum KnownCameraControlFlag {
    Automatic,
    Manual,
    ReadOnly,
    WriteOnly,
    Volatile,
    Disabled,
    Inactive,
}

impl Display for KnownCameraControlFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// TODO: use in CameraControl
/// The values for a [`CameraControl`].
///
/// This provides a wide range of values that can be used to control a camera.
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
        }
    }

    /// Verifies if the [setter](crate::utils::ControlValueSetter) is valid for the provided [`ControlValueDescription`].
    /// - `true` => Is valid.
    /// - `false` => Is not valid.
    #[must_use]
    pub fn verify_setter(&self, setter: &ControlValueSetter) -> bool {
        match setter {
            ControlValueSetter::None => {
                matches!(self, ControlValueDescription::None)
            }
            ControlValueSetter::Integer(i) => match self {
                ControlValueDescription::Integer {
                    value,
                    default,
                    step,
                } => (i - default).abs() % step == 0 || (i - value) % step == 0,
                ControlValueDescription::IntegerRange {
                    min,
                    max,
                    value,
                    step,
                    default,
                } => {
                    if value > max || value < min {
                        return false;
                    }

                    (i - default) % step == 0 || (i - value) % step == 0
                }
                _ => false,
            },
            ControlValueSetter::Float(f) => match self {
                ControlValueDescription::Float {
                    value,
                    default,
                    step,
                } => (f - default).abs() % step == 0_f64 || (f - value) % step == 0_f64,
                ControlValueDescription::FloatRange {
                    min,
                    max,
                    value,
                    step,
                    default,
                } => {
                    if value > max || value < min {
                        return false;
                    }

                    (f - default) % step == 0_f64 || (f - value) % step == 0_f64
                }
                _ => false,
            },
            ControlValueSetter::Boolean(_) => {
                matches!(self, ControlValueDescription::Boolean { .. })
            }
            ControlValueSetter::String(_) => {
                matches!(self, ControlValueDescription::String { .. })
            }
            ControlValueSetter::Bytes(_) => {
                matches!(self, ControlValueDescription::Bytes { .. })
            }
        }
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
                write!(
                    f,
                    "(Current: {}, Default: {}, Step: {})",
                    value, default, step
                )
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
                    "(Current: {}, Default: {}, Step: {}, Range: ({}, {}))",
                    value, default, step, min, max
                )
            }
            ControlValueDescription::Float {
                value,
                default,
                step,
            } => {
                write!(
                    f,
                    "(Current: {}, Default: {}, Step: {})",
                    value, default, step
                )
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
                    "(Current: {}, Default: {}, Step: {}, Range: ({}, {}))",
                    value, default, step, min, max
                )
            }
            ControlValueDescription::Boolean { value, default } => {
                write!(f, "(Current: {}, Default: {})", value, default)
            }
            ControlValueDescription::String { value, default } => {
                write!(f, "(Current: {}, Default: {:?})", value, default)
            }
            ControlValueDescription::Bytes { value, default } => {
                write!(f, "(Current: {:x?}, Default: {:x?})", value, default)
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CameraControl {
    control: KnownCameraControl,
    name: String,
    description: ControlValueDescription,
    value: ControlValueSetter,
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
        let value = description.value();
        CameraControl {
            control,
            name,
            description,
            value,
            flag,
            active,
        }
    }

    /// Gets the [`KnownCameraControl`] of this [`CameraControl`]
    #[must_use]
    pub fn control(&self) -> KnownCameraControl {
        self.control
    }

    /// Gets the current value description of this [`CameraControl`]
    #[must_use]
    pub fn value(&self) -> &ControlValueDescription {
        &self.description
    }

    /// Sets the value of this [`CameraControl`]
    /// # Errors
    /// If the `value` is below `min`, above `max`, or is not divisible by `step`, this will error
    pub fn set_value(&mut self, value: ControlValueSetter) -> Result<(), NokhwaError> {
        if !self.description.verify_setter(&value) {
            return Err(NokhwaError::SetPropertyError {
                property: format!("ControlValueDescription: {}", self.description),
                value: format!("ControlValueSetter: {}", self.value),
                error: "Invalid Value.".to_string(),
            });
        }

        self.value = value;
        Ok(())
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ControlValueSetter {
    None,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Bytes(Vec<u8>),
}

impl Display for ControlValueSetter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlValueSetter::None => {
                write!(f, "Value: None")
            }
            ControlValueSetter::Integer(i) => {
                write!(f, "Value: {}", i)
            }
            ControlValueSetter::Float(d) => {
                write!(f, "Value: {}", d)
            }
            ControlValueSetter::Boolean(b) => {
                write!(f, "Value: {}", b)
            }
            ControlValueSetter::String(s) => {
                write!(f, "Value: {}", s)
            }
            ControlValueSetter::Bytes(b) => {
                write!(f, "Value: {:x?}", b)
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
/// - `Network` - Uses `OpenCV` to capture from an IP.
/// - `Browser` - Uses browser APIs to capture from a webcam.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ApiBackend {
    Auto,
    AVFoundation,
    Video4Linux,
    #[deprecated]
    UniversalVideoClass,
    MediaFoundation,
    OpenCv,
    #[deprecated]
    GStreamer,
    Network,
    Browser,
}

impl Display for ApiBackend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let self_str = format!("{:?}", self);
        write!(f, "{}", self_str)
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

/// Converts a MJPEG stream of [u8] into a Vec<u8> of RGB888. (R,G,B,R,G,B,...)
/// # Errors
/// If `mozjpeg` fails to read scanlines or setup the decompressor, this will error.
/// # Safety
/// This function uses `unsafe`. The caller must ensure that:
/// - The input data is of the right size, does not exceed bounds, and/or the final size matches with the initial size.
#[cfg(all(feature = "decoding", not(target_arch = "wasm")))]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "decoding")))]
pub fn mjpeg_to_rgb(data: &[u8], rgba: bool) -> Result<Vec<u8>, NokhwaError> {
    use mozjpeg::Decompress;

    let mut jpeg_decompress = match Decompress::new_mem(data) {
        Ok(decompress) => {
            let decompressor_res = if rgba {
                decompress.rgba()
            } else {
                decompress.rgb()
            };
            match decompressor_res {
                Ok(decompressor) => decompressor,
                Err(why) => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::MJPEG,
                        destination: "RGB888".to_string(),
                        error: why.to_string(),
                    })
                }
            }
        }
        Err(why) => {
            return Err(NokhwaError::ProcessFrameError {
                src: FrameFormat::MJPEG,
                destination: "RGB888".to_string(),
                error: why.to_string(),
            })
        }
    };

    let scanlines_res: Option<Vec<u8>> = jpeg_decompress.read_scanlines_flat();
    // assert!(jpeg_decompress.finish_decompress());
    if !jpeg_decompress.finish_decompress() {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::MJPEG,
            destination: "RGB888".to_string(),
            error: "JPEG Decompressor did not finish.".to_string(),
        });
    }

    match scanlines_res {
        Some(pixels) => Ok(pixels),
        None => Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::MJPEG,
            destination: "RGB888".to_string(),
            error: "Failed to get read readlines into RGB888 pixels!".to_string(),
        }),
    }
}

#[cfg(not(all(feature = "decoding", not(target_arch = "wasm"))))]
pub fn mjpeg_to_rgb(data: &[u8], rgba: bool) -> Result<Vec<u8>, NokhwaError> {
    Err(NokhwaError::NotImplementedError(
        "Not available on WASM".to_string(),
    ))
}

/// Equivalent to [`mjpeg_to_rgb`] except with a destination buffer.
/// # Errors
/// If the decoding fails (e.g. invalid MJPEG stream), the buffer is not large enough, or you are doing this on `WebAssembly`, this will error.
#[cfg(all(feature = "decoding", not(target_arch = "wasm")))]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "decoding")))]
pub fn buf_mjpeg_to_rgb(data: &[u8], dest: &mut [u8], rgba: bool) -> Result<(), NokhwaError> {
    use mozjpeg::Decompress;

    let mut jpeg_decompress = match Decompress::new_mem(data) {
        Ok(decompress) => {
            let decompressor_res = if rgba {
                decompress.rgba()
            } else {
                decompress.rgb()
            };
            match decompressor_res {
                Ok(decompressor) => decompressor,
                Err(why) => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::MJPEG,
                        destination: "RGB888".to_string(),
                        error: why.to_string(),
                    })
                }
            }
        }
        Err(why) => {
            return Err(NokhwaError::ProcessFrameError {
                src: FrameFormat::MJPEG,
                destination: "RGB888".to_string(),
                error: why.to_string(),
            })
        }
    };

    // assert_eq!(dest.len(), jpeg_decompress.min_flat_buffer_size());
    if dest.len() != jpeg_decompress.min_flat_buffer_size() {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::MJPEG,
            destination: "RGB888".to_string(),
            error: "Bad decoded buffer size".to_string(),
        });
    }

    jpeg_decompress.read_scanlines_flat_into(dest);
    // assert!(jpeg_decompress.finish_decompress());
    if !jpeg_decompress.finish_decompress() {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::MJPEG,
            destination: "RGB888".to_string(),
            error: "JPEG Decompressor did not finish.".to_string(),
        });
    }
    Ok(())
}

#[cfg(not(all(feature = "decoding", not(target_arch = "wasm"))))]
pub fn buf_mjpeg_to_rgb(data: &[u8], dest: &mut [u8], rgba: bool) -> Result<(), NokhwaError> {
    Err(NokhwaError::NotImplementedError(
        "Not available on WASM".to_string(),
    ))
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
pub fn yuyv422_to_rgb(data: &[u8], rgba: bool) -> Result<Vec<u8>, NokhwaError> {
    if data.len() % 4 != 0 {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::YUYV,
            destination: "RGB888".to_string(),
            error: "Assertion failure, the YUV stream isn't 4:2:2! (wrong number of bytes)"
                .to_string(),
        });
    }

    let pixel_size = if rgba { 4 } else { 3 };
    // yuyv yields 2 3-byte pixels per yuyv chunk
    let rgb_buf_size = (data.len() / 4) * (2 * pixel_size);

    let mut dest = vec![0; rgb_buf_size];
    buf_yuyv422_to_rgb(data, &mut dest, rgba)?;

    Ok(dest)
}

/// Same as [`yuyv422_to_rgb`] but with a destination buffer instead of a return `Vec<u8>`
/// # Errors
/// If the stream is invalid YUYV, or the destination buffer is not large enough, this will error.
pub fn buf_yuyv422_to_rgb(data: &[u8], dest: &mut [u8], rgba: bool) -> Result<(), NokhwaError> {
    if data.len() % 4 != 0 {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::YUYV,
            destination: "RGB888".to_string(),
            error: "Assertion failure, the YUV stream isn't 4:2:2! (wrong number of bytes)"
                .to_string(),
        });
    }

    let pixel_size = if rgba { 4 } else { 3 };
    // yuyv yields 2 3-byte pixels per yuyv chunk
    let rgb_buf_size = (data.len() / 4) * (2 * pixel_size);

    if dest.len() != rgb_buf_size {
        return Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::YUYV,
            destination: "RGB888".to_string(),
            error: format!("Assertion failure, the destination RGB buffer is of the wrong size! [expected: {rgb_buf_size}, actual: {}]", dest.len()),
        });
    }

    let iter = data.chunks_exact(4);

    if rgba {
        let mut iter = iter
            .flat_map(|yuyv| {
                let y1 = i32::from(yuyv[0]);
                let u = i32::from(yuyv[1]);
                let y2 = i32::from(yuyv[2]);
                let v = i32::from(yuyv[3]);
                let pixel1 = yuyv444_to_rgba(y1, u, v);
                let pixel2 = yuyv444_to_rgba(y2, u, v);
                [pixel1, pixel2]
            })
            .flatten();
        for i in dest.iter_mut().take(rgb_buf_size) {
            *i = match iter.next() {
                Some(v) => v,
                None => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::YUYV,
                        destination: "RGBA8888".to_string(),
                        error: "Ran out of RGBA YUYV values! (this should not happen, please file an issue: l1npengtul/nokhwa)".to_string()
                    })
                }
            }
        }
    } else {
        let mut iter = iter
            .flat_map(|yuyv| {
                let y1 = i32::from(yuyv[0]);
                let u = i32::from(yuyv[1]);
                let y2 = i32::from(yuyv[2]);
                let v = i32::from(yuyv[3]);
                let pixel1 = yuyv444_to_rgb(y1, u, v);
                let pixel2 = yuyv444_to_rgb(y2, u, v);
                [pixel1, pixel2]
            })
            .flatten();

        for i in dest.iter_mut().take(rgb_buf_size) {
            *i = match iter.next() {
                Some(v) => v,
                None => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: "Ran out of RGB YUYV values! (this should not happen, please file an issue: l1npengtul/nokhwa)".to_string()
                    })
                }
            }
        }
    }

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
    let r = ((c298 + 409 * e + 128) >> 8) as u8;
    let g = ((c298 - 100 * d - 208 * e + 128) >> 8) as u8;
    let b = ((c298 + 516 * d + 128) >> 8) as u8;
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
