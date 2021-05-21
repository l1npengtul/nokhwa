use std::{cmp::Ordering, fmt::Display};

/// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC`

/// YUYV is a mathmatical color space. You can read more [here.](https://en.wikipedia.org/wiki/YCbCr)

/// MJPEG is a motion-jpeg compressed frame, it allows for high frame rates.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum FrameFormat {
    MJPEG,
    YUYV,
}

/// Describes a Resolution.
/// This struct consists of a Width and a Height value (x,y).
/// Note that the [`Ord`] implementation of this struct is flipped from highest to lowest.
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

/// This is a conveinence struct that holds all information about the format of a webcam stream.
/// It consists of a Resolution, `FrameFormat`, and a framerate.
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
    pub fn resoltuion(&self) -> Resolution {
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

/// The list of known capture backends to the library.
///
/// AUTO is special - it tells the Camera struct to automatically choose a backend most suited for the current platform.
///
/// V4L2 - `Video4Linux2`, a linux specific backend.
///
/// UVC - Universal Video Class (please check [libuvc](https://github.com/libuvc/libuvc)). Platform agnostic, although on linux it needs `sudo` permissions or similar to use.
///
/// MSMF - Microsoft Media Foundation, Winsows only (replacement for `DirectShow`)
///
/// `OpenCV` - Uses `OpenCV` to capture. Platform agnostic.
///
/// FFMPEG - Uses FFMPEG (libavdevice) to capture. Platform agnostic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaptureAPIBackend {
    AUTO,
    V4L2,
    UVC,
    MSMF,
    OPENCV,
    FFMPEG,
}
