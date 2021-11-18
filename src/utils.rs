/*
 * Copyright 2021 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
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

/*
 * Copyright 2021 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
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
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};
#[cfg(feature = "output-wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

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
#[cfg(feature = "input-uvc")]
use uvc::StreamFormat;
#[cfg(all(feature = "input-v4l", target_os = "linux"))]
use v4l::{control::Description, Format, FourCC};

/// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC`.
/// - YUYV is a mathematical color space. You can read more [here.](https://en.wikipedia.org/wiki/YCbCr)
/// - MJPEG is a motion-jpeg compressed frame, it allows for high frame rates.
/// # JS-WASM
/// This is exported as `FrameFormat`
#[derive(Copy, Clone, Debug, PartialEq, Hash, PartialOrd, Ord, Eq)]
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
        }
    }
}

/// Describes a Resolution.
/// This struct consists of a Width and a Height value (x,y). <br>
/// Note: the [`Ord`] implementation of this struct is flipped from highest to lowest.
/// # JS-WASM
/// This is exported as `Resolution`
#[cfg_attr(feature = "output-wasm", wasm_bindgen(js_name = Resolution))]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Resolution {
    pub width_x: u32,
    pub height_y: u32,
}

#[cfg_attr(feature = "output-wasm", wasm_bindgen(js_class = Resolution))]
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
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
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
            frame_rate: 15,
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
/// # JS-WASM
/// This is exported as a `CameraInfo`.
#[cfg_attr(feature = "output-wasm", wasm_bindgen(js_name = CameraInfo))]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct CameraInfo {
    human_name: String,
    description: String,
    misc: String,
    index: usize,
}

#[cfg_attr(feature = "output-wasm", wasm_bindgen(js_class = CameraInfo))]
impl CameraInfo {
    /// Create a new [`CameraInfo`].
    /// # JS-WASM
    /// This is exported as a constructor for [`CameraInfo`].
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(constructor))]
    pub fn new(human_name: String, description: String, misc: String, index: usize) -> Self {
        CameraInfo {
            human_name,
            description,
            misc,
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
    pub fn set_human_name(&mut self, human_name: String) {
        self.human_name = human_name;
    }

    /// Get a reference to the device info's description.
    /// # JS-WASM
    /// This is exported as a `get_Description`.
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(getter = Description))]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    /// Set the device info's description.
    /// # JS-WASM
    /// This is exported as a `set_Description`.
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(setter = Description))]
    pub fn set_description(&mut self, description: String) {
        self.description = description;
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
    pub fn set_misc(&mut self, misc: String) {
        self.misc = misc;
    }

    /// Get a reference to the device info's index.
    /// # JS-WASM
    /// This is exported as a `get_Index`.
    #[must_use]
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(getter = Index))]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Set the device info's index.
    /// # JS-WASM
    /// This is exported as a `set_Index`.
    #[cfg_attr(feature = "output-wasm", wasm_bindgen(setter = Index))]
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
            index: dev_desc.index(),
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
            index: descriptor.index as usize,
        }
    }
}

/// The list of known camera controls to the library. <br>
/// These can control the picture brightness, etc. <br>
/// Note that not all backends/devices support all these. Run [`supported_camera_controls()`](crate::CaptureBackendTrait::supported_camera_controls) to see which ones can be set.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum KnownCameraControls {
    Brightness,
    Contrast,
    Hue,
    Saturation,
    Sharpness,
    Gamma,
    ColorEnable,
    WhiteBalance,
    BacklightComp,
    Gain,
    Pan,
    Tilt,
    Roll,
    Zoom,
    Exposure,
    Iris,
    Focus,
}

/// All camera controls in an array.
#[must_use]
pub fn all_known_camera_controls() -> [KnownCameraControls; 17] {
    [
        KnownCameraControls::Brightness,
        KnownCameraControls::Contrast,
        KnownCameraControls::Hue,
        KnownCameraControls::Saturation,
        KnownCameraControls::Sharpness,
        KnownCameraControls::Gamma,
        KnownCameraControls::ColorEnable,
        KnownCameraControls::WhiteBalance,
        KnownCameraControls::BacklightComp,
        KnownCameraControls::Gain,
        KnownCameraControls::Pan,
        KnownCameraControls::Tilt,
        KnownCameraControls::Roll,
        KnownCameraControls::Zoom,
        KnownCameraControls::Exposure,
        KnownCameraControls::Iris,
        KnownCameraControls::Focus,
    ]
}

impl Display for KnownCameraControls {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self)
    }
}

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<MediaFoundationControls> for KnownCameraControls {
    fn from(mf_c: MediaFoundationControls) -> Self {
        match mf_c {
            MediaFoundationControls::Brightness => KnownCameraControls::Brightness,
            MediaFoundationControls::Contrast => KnownCameraControls::Contrast,
            MediaFoundationControls::Hue => KnownCameraControls::Hue,
            MediaFoundationControls::Saturation => KnownCameraControls::Saturation,
            MediaFoundationControls::Sharpness => KnownCameraControls::Sharpness,
            MediaFoundationControls::Gamma => KnownCameraControls::Gamma,
            MediaFoundationControls::ColorEnable => KnownCameraControls::ColorEnable,
            MediaFoundationControls::WhiteBalance => KnownCameraControls::WhiteBalance,
            MediaFoundationControls::BacklightComp => KnownCameraControls::BacklightComp,
            MediaFoundationControls::Gain => KnownCameraControls::Gain,
            MediaFoundationControls::Pan => KnownCameraControls::Pan,
            MediaFoundationControls::Tilt => KnownCameraControls::Tilt,
            MediaFoundationControls::Roll => KnownCameraControls::Roll,
            MediaFoundationControls::Zoom => KnownCameraControls::Zoom,
            MediaFoundationControls::Exposure => KnownCameraControls::Exposure,
            MediaFoundationControls::Iris => KnownCameraControls::Iris,
            MediaFoundationControls::Focus => KnownCameraControls::Focus,
        }
    }
}

#[cfg(any(
    all(feature = "input-msmf", target_os = "windows"),
    all(feature = "docs-only", feature = "docs-nolink", feature = "input-msmf")
))]
impl From<MFControl> for KnownCameraControls {
    fn from(mf_cc: MFControl) -> Self {
        mf_cc.control().into()
    }
}

#[cfg(all(feature = "input-v4l", target_os = "linux"))]
impl std::convert::TryFrom<Description> for KnownCameraControls {
    type Error = NokhwaError;

    fn try_from(value: Description) -> Result<Self, Self::Error> {
        Ok(match value.id {
            9_963_776 => KnownCameraControls::Brightness,
            9_963_777 => KnownCameraControls::Contrast,
            9_963_779 => KnownCameraControls::Hue,
            9_963_778 => KnownCameraControls::Saturation,
            9_963_803 => KnownCameraControls::Sharpness,
            9_963_792 => KnownCameraControls::Gamma,
            9_963_802 => KnownCameraControls::WhiteBalance,
            9_963_804 => KnownCameraControls::BacklightComp,
            9_963_795 => KnownCameraControls::Gain,
            10_094_852 => KnownCameraControls::Pan,
            10_094_853 => KnownCameraControls::Tilt,
            10_094_862 => KnownCameraControls::Zoom,
            10_094_850 => KnownCameraControls::Exposure,
            10_094_866 => KnownCameraControls::Iris,
            10_094_859 => KnownCameraControls::Focus,
            _ => {
                return Err(NokhwaError::NotImplementedError(
                    "Control not implemented!".to_string(),
                ))
            }
        })
    }
}

/// This tells you weather a [`KnownCameraControls`] is automatically managed by the OS/Driver
/// or manually managed by you, the programmer.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum KnownCameraControlFlag {
    Automatic,
    Manual,
}

impl Display for KnownCameraControlFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// This struct tells you everything about a particular [`KnownCameraControls`]. <br>
/// However, you should never need to instantiate this struct, since its usually generated for you by `nokhwa`.
/// The only time you should be modifying this struct is when you need to set a value and pass it back to the camera.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct CameraControl {
    control: KnownCameraControls,
    min: i32,
    max: i32,
    value: i32,
    step: i32,
    default: i32,
    flag: KnownCameraControlFlag,
    active: bool,
}

impl CameraControl {
    /// Creates a new [`CameraControl`]
    /// # Errors
    /// If the `value` is below `min`, above `max`, or is not divisible by `step`, this will error
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        control: KnownCameraControls,
        minimum: i32,
        maximum: i32,
        value: i32,
        step: i32,
        default: i32,
        flag: KnownCameraControlFlag,
        active: bool,
    ) -> Result<Self, NokhwaError> {
        if value >= maximum {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too large".to_string(),
            });
        }
        if value <= minimum {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too low".to_string(),
            });
        }
        if value % step != 0 {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Not aligned with step".to_string(),
            });
        }

        Ok(CameraControl {
            control,
            min: minimum,
            max: maximum,
            value,
            step,
            default,
            flag,
            active,
        })
    }

    /// Gets the [`KnownCameraControls`] of this [`CameraControl`]
    #[must_use]
    pub fn control(&self) -> KnownCameraControls {
        self.control
    }

    /// Gets the minimum value of this [`CameraControl`]
    #[must_use]
    pub fn minimum_value(&self) -> i32 {
        self.min
    }

    /// Gets the maximum value of this [`CameraControl`]
    #[must_use]
    pub fn maximum_value(&self) -> i32 {
        self.max
    }

    /// Gets the current value of this [`CameraControl`]
    #[must_use]
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Sets the value of this [`CameraControl`]
    /// # Errors
    /// If the `value` is below `min`, above `max`, or is not divisible by `step`, this will error
    pub fn set_value(&mut self, value: i32) -> Result<(), NokhwaError> {
        if value >= self.maximum_value() {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too large".to_string(),
            });
        }
        if value <= self.minimum_value() {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too low".to_string(),
            });
        }
        if value % self.step() != 0 {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Not aligned with step".to_string(),
            });
        }

        self.value = value;
        Ok(())
    }

    /// Creates a new [`CameraControl`] but with `value`
    /// # Errors
    /// If the `value` is below `min`, above `max`, or is not divisible by `step`, this will error
    pub fn with_value(self, value: i32) -> Result<Self, NokhwaError> {
        if value >= self.maximum_value() {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too large".to_string(),
            });
        }
        if value <= self.minimum_value() {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too low".to_string(),
            });
        }
        if value % self.step() != 0 {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Not aligned with step".to_string(),
            });
        }

        Ok(CameraControl {
            control: self.control(),
            min: self.minimum_value(),
            max: self.maximum_value(),
            value,
            step: self.step(),
            default: self.default(),
            flag: self.flag(),
            active: true,
        })
    }

    /// Gets the step value of this [`CameraControl`]
    /// Note that `value` must be divisible by `step`
    #[must_use]
    pub fn step(&self) -> i32 {
        self.step
    }

    /// Gets the default value of this [`CameraControl`]
    #[must_use]
    pub fn default(&self) -> i32 {
        self.default
    }

    /// Gets the [`KnownCameraControlFlag`] of this [`CameraControl`],
    /// telling you weather this control is automatically set or manually set.
    #[must_use]
    pub fn flag(&self) -> KnownCameraControlFlag {
        self.flag
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

    /// Returns a list of i32s that are valid to be set.
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn valid_values(&self) -> Vec<i32> {
        (self.minimum_value()..=self.maximum_value())
            .step_by(self.step() as usize)
            .into_iter()
            .collect()
    }
}

impl Display for CameraControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, Range: ({}~{}), Current: {}, Step: {}, Default: {}, Flag: {}, Active: {}",
            self.control,
            self.min,
            self.max,
            self.value,
            self.step,
            self.default,
            self.flag,
            self.active
        )
    }
}

impl PartialOrd for CameraControl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CameraControl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.control().cmp(&other.control())
    }
}

/// The list of known capture backends to the library. <br>
/// - `AUTO` is special - it tells the Camera struct to automatically choose a backend most suited for the current platform.
/// - `AVFoundation` - Uses `AVFoundation` on MacOSX
/// - `V4L2` - `Video4Linux2`, a linux specific backend.
/// - `UVC` - Universal Video Class (please check [libuvc](https://github.com/libuvc/libuvc)). Platform agnostic, although on linux it needs `sudo` permissions or similar to use.
/// - `MediaFoundation` - Microsoft Media Foundation, Windows only,
/// - `OpenCV` - Uses `OpenCV` to capture. Platform agnostic.
/// - `GStreamer` - Uses `GStreamer` RTP to capture. Platform agnostic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaptureAPIBackend {
    Auto,
    AVFoundation,
    Video4Linux,
    UniversalVideoClass,
    MediaFoundation,
    OpenCv,
    GStreamer,
}

impl Display for CaptureAPIBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let self_str = format!("{:?}", self);
        write!(f, "{}", self_str)
    }
}

/// The `OpenCV` backend supports both native cameras and IP Cameras, so this is an enum to differentiate them
/// The `IPCamera`'s string follows the pattern
/// ```.ignore
/// <protocol>://<IP>:<port>/
/// ```
/// but please consult the manufacturer's specification for more details.
/// The index is a standard webcam index.
#[derive(Clone, Debug, PartialEq)]
pub enum CameraIndexType {
    Index(u32),
    IPCamera(String),
}

impl Display for CameraIndexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CameraIndexType::Index(idx) => {
                write!(f, "{}", idx)
            }
            CameraIndexType::IPCamera(ip) => {
                write!(f, "{}", ip)
            }
        }
    }
}

/// Converts a MJPEG stream of [u8] into a Vec<u8> of RGB888. (R,G,B,R,G,B,...)
/// # Errors
/// If `mozjpeg` fails to read scanlines or setup the decompressor, this will error.
/// # Safety
/// This function uses `unsafe`. The caller must ensure that:
/// - The input data is of the right size, does not exceed bounds, and/or the final size matches with the initial size.
#[cfg(feature = "decoding")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "decoding")))]
pub fn mjpeg_to_rgb888(data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
    use mozjpeg::Decompress;

    let mut jpeg_decompress = match Decompress::new_mem(data) {
        Ok(decompress) => match decompress.rgb() {
            Ok(decompressor) => decompressor,
            Err(why) => {
                return Err(NokhwaError::ProcessFrameError {
                    src: FrameFormat::MJPEG,
                    destination: "RGB888".to_string(),
                    error: why.to_string(),
                })
            }
        },
        Err(why) => {
            return Err(NokhwaError::ProcessFrameError {
                src: FrameFormat::MJPEG,
                destination: "RGB888".to_string(),
                error: why.to_string(),
            })
        }
    };
    let decompressed = match jpeg_decompress.read_scanlines::<[u8; 3]>() {
        Some(pixels) => pixels,
        None => {
            return Err(NokhwaError::ProcessFrameError {
                src: FrameFormat::MJPEG,
                destination: "RGB888".to_string(),
                error: "Failed to get read readlines into RGB888 pixels!".to_string(),
            })
        }
    };

    Ok(
        unsafe { std::slice::from_raw_parts(decompressed.as_ptr().cast(), decompressed.len() * 3) }
            .to_vec(),
    )
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
#[cfg(any(not(target_family = "wasm"), feature = "decoding"))]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "decoding")))]
#[inline]
pub fn yuyv422_to_rgb888(data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
    use std::convert::TryFrom;

    let mut rgb_vec: Vec<u8> = vec![];
    if data.len() % 4 == 0 {
        for px_idx in (0..data.len()).step_by(4) {
            let y1 = match data.get(px_idx) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::ProcessFrameError { src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx, why) });
                    }
                },
                None => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx
                        ),
                    });
                }
            };

            let u = match data.get(px_idx + 1) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::ProcessFrameError { src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx+1, why) });
                    }
                },
                None => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx + 1
                        ),
                    });
                }
            };

            let y2 = match data.get(px_idx + 2) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::ProcessFrameError { src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx+2, why) });
                    }
                },
                None => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx + 2
                        ),
                    });
                }
            };

            let v = match data.get(px_idx + 3) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::ProcessFrameError { src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx+3, why) });
                    }
                },
                None => {
                    return Err(NokhwaError::ProcessFrameError {
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
        Err(NokhwaError::ProcessFrameError {
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
#[must_use]
#[cfg(any(not(target_family = "wasm"), feature = "decoding"))]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "decoding")))]
#[inline]
pub fn yuyv444_to_rgb888(y: i32, u: i32, v: i32) -> [u8; 3] {
    let c298 = (y - 16) * 298;
    let d = u - 128;
    let e = v - 128;
    let r = ((c298 + 409 * e + 128) >> 8).clamp(0, 255) as u8;
    let g = ((c298 - 100 * d - 208 * e + 128) >> 8).clamp(0, 255) as u8;
    let b = ((c298 + 516 * d + 128) >> 8).clamp(0, 255) as u8;
    [r, g, b]
}
