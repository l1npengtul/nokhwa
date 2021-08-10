/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// Note: for WASM bindings you need to bind them yourself.
use crate::{CameraInfo, NokhwaError, Resolution};
use js_sys::{Array, Function, JsString, Promise};
use std::{
    convert::TryFrom,
    fmt::{Debug, Display, Formatter},
    ops::Deref,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::*;

// why no code completion
// big sadger

// intellij 2021.2 review: i like structure window, 4 pengs / 5 pengs

const GET_CONSTRAINT_LIST_JS_CODE_STR: &'static str = r#"
let constraints_list = navigator.mediaDevices.getSupportedConstraints();
let constraint_string_arr = [];

for (let constraint in supportedConstraints) {
    if (constraints_list.hasOwnProperty(constraint)) {
        constraint_string_arr.push(constraint.to_string());
    }
}

return constraint_string_arr;
"#;

/// Requests Webcam permissions from the browser using [`MediaDevices::get_user_media()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaDevices.html#method.get_user_media) [MDN](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getUserMedia)
pub async fn request_permission() -> Result<JsFuture, NokhwaError> {
    let window: Window = match window() {
        Some(win) => win,
        None => {
            return Err(NokhwaError::StructureError {
                structure: "web_sys Window".to_string(),
                error: "None".to_string(),
            })
        }
    };
    let navigator = window.navigator();
    let media_devices = match navigator.media_devices() {
        Ok(media) => media,
        Err(why) => {
            return Err(NokhwaError::StructureError {
                structure: "MediaDevices".to_string(),
                error: format!("{:?}", why),
            })
        }
    };

    match media_devices.get_user_media() {
        Ok(promise) => {
            let promise: Promise = promise;
            Ok(JsFuture::from(promise))
        }
        Err(why) => {
            return Err(NokhwaError::StructureError {
                structure: "UserMediaPermission".to_string(),
                error: format!("{:?}", why),
            })
        }
    }
}

/// Queries Cameras using [`MediaDevices::enumerate_devices()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaDevices.html#method.enumerate_devices) [MDN](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/enumerateDevices)
pub async fn query_js_cameras() -> Result<Vec<CameraInfo>, NokhwaError> {
    let window: Window = match window() {
        Some(win) => win,
        None => {
            return Err(NokhwaError::StructureError {
                structure: "web_sys Window".to_string(),
                error: "None".to_string(),
            })
        }
    };
    let navigator = window.navigator();
    let media_devices = match navigator.media_devices() {
        Ok(media) => media,
        Err(why) => {
            return Err(NokhwaError::StructureError {
                structure: "MediaDevices".to_string(),
                error: format!("{:?}", why),
            })
        }
    };

    match media_devices.enumerate_devices() {
        Ok(prom) => {
            let prom: Promise = prom;
            let future = JsFuture::from(prom);
            match future.await {
                Ok(v) => {
                    let array: Array = Array::from(&v);
                    let mut device_list = vec![];
                    for idx_device in 0_u32..array.length() {
                        if MediaDeviceInfo::instanceof(&array.get(idx_device)) {
                            let media_device_info =
                                MediaDeviceInfo::unchecked_from_js(array.get(idx_device));
                            if media_device_info.kind() == MediaDeviceKind::Videoinput {
                                device_list.push(CameraInfo::new(
                                    media_device_info.label(),
                                    format!("{:?}", media_device_info.kind()),
                                    format!(
                                        "{}:{}",
                                        media_device_info.group_id(),
                                        media_device_info.device_id()
                                    ),
                                    idx_device as usize,
                                ))
                            }
                        }
                    }
                    Ok(device_list)
                }
                Err(why) => Err(NokhwaError::StructureError {
                    structure: "EnumerateDevicesFuture".to_string(),
                    error: format!("{:?}", why),
                }),
            }
        }
        Err(why) => Err(NokhwaError::StructureError {
            structure: "EnumerateDevices".to_string(),
            error: format!("{:?}", why),
        }),
    }
}

/// Queries the browser's supported constraints using [`navigator.mediaDevices.getSupportedConstraints()`](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getSupportedConstraints)
pub fn query_supported_constraints() -> Result<Vec<JSCameraSupportedCapabilities>, NokhwaError> {
    let js_supported_fn = Function::new_no_args(GET_CONSTRAINT_LIST_JS_CODE_STR);
    match js_supported_fn.call0(&JsValue::NULL) {
        Ok(value) => {
            let value: JsValue = value;
            let supported_cap_array: Array = Array::from(&value);

            let mut capability_list = vec![];
            for idx_supported in 0_u32..supported_cap_array.length() {
                let supported = match supported_cap_array.get(idx_supported).dyn_ref::<JsString>() {
                    Some(v) => {
                        let v: &JsValue = v.as_ref();
                        let s: String = match v.as_string() {
                            Some(str) => str,
                            None => {
                                return Err(NokhwaError::StructureError {
                                    structure: "Query Supported Constraints String None"
                                        .to_string(),
                                    error: "None".to_string(),
                                })
                            }
                        };
                        s
                    }
                    None => {
                        continue;
                    }
                };

                let capability = match JSCameraSupportedCapabilities::try_from(supported) {
                    Ok(cap) => cap,
                    Err(_) => {
                        continue;
                    }
                };
                capability_list.push(capability);
            }
            Ok(capability_list)
        }
        Err(why) => {
            return Err(NokhwaError::StructureError {
                structure: "JSCameraSupportedCapabilities List Dict Function".to_string(),
                error: why.as_string().unwrap_or("".to_string()),
            })
        }
    }
}

/// The enum describing the possible constraints for video in the browser.
/// - DeviceID: The ID of the device
/// - GroupID: The ID of the group that the device is in
/// - AspectRatio: The Aspect Ratio of the final stream
/// - FacingMode: What direction the camera is facing. This is more common on mobile. See [`JSCameraFacingMode`]
/// - FrameRate: The Frame Rate of the final stream
/// - Height: The height of the final stream in pixels
/// - Width: The width of the final stream in pixels
/// - ResizeMode: Whether the client can crop and/or scale the stream to match the resolution (width, height). See [`JSCameraResizeMode`]
/// See More: [`MediaTrackConstraints`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints) [`Capabilities, constraints, and settings`](https://developer.mozilla.org/en-US/docs/Web/API/Media_Streams_API/Constraints)
#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum JSCameraSupportedCapabilities {
    DeviceID,
    GroupID,
    AspectRatio,
    FacingMode,
    FrameRate,
    Height,
    Width,
    ResizeMode,
}

impl Display for JSCameraSupportedCapabilities {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cap = match self {
            JSCameraSupportedCapabilities::DeviceID => "deviceId",
            JSCameraSupportedCapabilities::GroupID => "groupId",
            JSCameraSupportedCapabilities::AspectRatio => "aspectRatio",
            JSCameraSupportedCapabilities::FacingMode => "facingMode",
            JSCameraSupportedCapabilities::FrameRate => "frameRate",
            JSCameraSupportedCapabilities::Height => "height",
            JSCameraSupportedCapabilities::Width => "width",
            JSCameraSupportedCapabilities::ResizeMode => "resizeMode",
        };

        write!(f, "{}", cap)
    }
}

impl Debug for JSCameraSupportedCapabilities {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = self.to_string();
        write!(f, "{}", str)
    }
}

impl Into<String> for JSCameraSupportedCapabilities {
    fn into(self) -> String {
        self.to_string()
    }
}

impl TryFrom<String> for JSCameraSupportedCapabilities {
    type Error = NokhwaError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.as_str();
        Ok(match value {
            "deviceId" => JSCameraSupportedCapabilities::DeviceID,
            "groupId" => JSCameraSupportedCapabilities::GroupID,
            "aspectRatio" => JSCameraSupportedCapabilities::AspectRatio,
            "facingMode" => JSCameraSupportedCapabilities::FacingMode,
            "frameRate" => JSCameraSupportedCapabilities::FrameRate,
            "height" => JSCameraSupportedCapabilities::Height,
            "width" => JSCameraSupportedCapabilities::Width,
            "resizeMode" => JSCameraSupportedCapabilities::ResizeMode,
            _ => {
                return Err(NokhwaError::StructureError {
                    structure: "JSCameraSupportedCapabilities".to_string(),
                    error: "No Match Str".to_string(),
                })
            }
        })
    }
}

/// The Facing Mode of the camera
/// - Any: Make no particular choice.
/// - Environment: The camera that shows the user's environment, such as the back camera of a smartphone
/// - User: The camera that shows the user, such as the front camera of a smartphone
/// - Left: The camera that shows the user but to their left, such as a camera that shows a user but to their left shoulder
/// - Right: The camera that shows the user but to their right, such as a camera that shows a user but to their right shoulder
/// See More: [`facingMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/facingMode)
#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum JSCameraFacingMode {
    Any,
    Environment,
    User,
    Left,
    Right,
}

impl Display for JSCameraFacingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cap = match self {
            JSCameraFacingMode::Environment => "environment",
            JSCameraFacingMode::User => "user",
            JSCameraFacingMode::Left => "left",
            JSCameraFacingMode::Right => "right",
            JSCameraFacingMode::Any => "any",
        };
        write!(f, "{}", cap)
    }
}

impl Debug for JSCameraFacingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = self.to_string();
        write!(f, "{}", str)
    }
}

impl Into<String> for JSCameraFacingMode {
    fn into(self) -> String {
        self.to_string()
    }
}

/// Whether the browser can crop and/or scale to match the requested resolution.
/// - Any: Make no particular choice.
/// - None: Do not crop and/or scale.
/// - CropAndScale: Crop and/or scale to match the requested resolution.
/// See More: [`resizeMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#resizemode)
#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum JSCameraResizeMode {
    Any,
    None,
    CropAndScale,
}

impl Display for JSCameraResizeMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cap = match self {
            JSCameraResizeMode::None => "none",
            JSCameraResizeMode::CropAndScale => "crop-and-scale",
            JSCameraResizeMode::Any => "",
        };

        write!(f, "{}", cap)
    }
}

impl Debug for JSCameraResizeMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = self.to_string();
        write!(f, "{}", str)
    }
}

impl Into<String> for JSCameraResizeMode {
    fn into(self) -> String {
        self.to_string()
    }
}

/// A builder that builds a [`JSCameraConstraints`] that is used to construct a [`JSCamera`].
/// See More: [`Constraints MDN`](https://developer.mozilla.org/en-US/docs/Web/API/Media_Streams_API/Constraints), [`Properties of Media Tracks MDN`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints)
#[derive(Clone, Debug)]
pub struct JSCameraConstraintsBuilder {
    preferred_resolution: Resolution,
    resolution_exact: bool,
    aspect_ratio: f64,
    aspect_ratio_exact: bool,
    facing_mode: JSCameraFacingMode,
    facing_mode_exact: bool,
    frame_rate: u32,
    frame_rate_exact: bool,
    resize_mode: JSCameraResizeMode,
    resize_mode_exact: bool,
    device_id: String,
    device_id_exact: bool,
    group_id: String,
    group_id_exact: bool,
}

impl JSCameraConstraintsBuilder {
    /// Constructs a default [`JSCameraConstraintsBuilder`].
    pub fn new() -> Self {
        JSCameraConstraintsBuilder::default()
    }

    /// Sets the preferred resolution for the [`JSCameraConstraintsBuilder`].
    ///
    /// Sets [`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width) and [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height).
    pub fn resolution(mut self, new_resolution: Resolution) -> JSCameraConstraintsBuilder {
        self.preferred_resolution = new_resolution;
        self
    }

    /// Sets whether the resolution fields ([`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width), [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height)/[`resolution`](crate::js_camera::JSCameraConstraintsBuilder::resolution))
    /// should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    pub fn resolution_exact(mut self, value: bool) -> JSCameraConstraintsBuilder {
        self.resolution_exact = value;
        self
    }

    /// Sets the aspect ratio of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    ///
    /// Sets [`aspectRatio`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/aspectRatio).
    pub fn aspect_ratio(mut self, ratio: f64) -> JSCameraConstraintsBuilder {
        self.aspect_ratio = ratio;
        self
    }

    /// Sets whether the [`aspect_ratio`](crate::js_camera::JSCameraConstraintsBuilder::aspect_ratio) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    pub fn aspect_ratio_exact(mut self, value: bool) -> JSCameraConstraintsBuilder {
        self.aspect_ratio_exact = value;
        self
    }

    /// Sets the facing mode of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    ///
    /// Sets [`facingMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/facingMode).
    pub fn facing_mode(mut self, facing_mode: JSCameraFacingMode) -> JSCameraConstraintsBuilder {
        self.facing_mode = facing_mode;
        self
    }

    /// Sets whether the [`facing_mode`](crate::js_camera::JSCameraConstraintsBuilder::facing_mode) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    pub fn facing_mode_exact(mut self, value: bool) -> JSCameraConstraintsBuilder {
        self.facing_mode_exact = value;
        self
    }

    /// Sets the frame rate of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    ///
    /// Sets [`frameRate`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/frameRate).
    pub fn frame_rate(mut self, fps: u32) -> JSCameraConstraintsBuilder {
        self.frame_rate = fps;
        self
    }

    /// Sets whether the [`frame_rate`](crate::js_camera::JSCameraConstraintsBuilder::frame_rate) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    pub fn frame_rate_exact(mut self, value: bool) -> JSCameraConstraintsBuilder {
        self.frame_rate_exact = value;
        self
    }

    /// Sets the resize mode of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    ///
    /// Sets [`resizeMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#resizemode).
    pub fn resize_mode(mut self, resize_mode: JSCameraResizeMode) -> JSCameraConstraintsBuilder {
        self.resize_mode = resize_mode;
        self
    }

    /// Sets whether the [`resize_mode`](crate::js_camera::JSCameraConstraintsBuilder::resize_mode) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    pub fn resize_mode_exact(mut self, value: bool) -> JSCameraConstraintsBuilder {
        self.resize_mode_exact = value;
        self
    }

    /// Sets the device ID of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    ///
    /// Sets [`deviceId`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/deviceId).
    pub fn device_id<S: ToString>(mut self, id: S) -> JSCameraConstraintsBuilder {
        self.device_id = id.to_string();
        self
    }

    /// Sets whether the [`device_id`](crate::js_camera::JSCameraConstraintsBuilder::device_id) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    pub fn device_id_exact(mut self, value: bool) -> JSCameraConstraintsBuilder {
        self.device_id_exact = value;
        self
    }

    /// Sets the group ID of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    ///
    /// Sets [`groupId`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/groupId).
    pub fn group_id<S: ToString>(mut self, id: S) -> JSCameraConstraintsBuilder {
        self.group_id = id.to_string();
        self
    }

    /// Sets whether the [`group_id`](crate::js_camera::JSCameraConstraintsBuilder::group_id) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    pub fn group_id_exact(mut self, value: bool) -> JSCameraConstraintsBuilder {
        self.group_id_exact = value;
        self
    }

    /// Builds the [`JSCameraConstraints`]
    ///
    /// # Security
    /// WARNING: This function uses [`Function`](https://docs.rs/js-sys/0.3.52/js_sys/struct.Function.html) and if the [`device_id`](crate::js_camera::JSCameraConstraintsBuilder::device_id) or [`groupId`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/groupId)
    /// fields are invalid/contain malicious JS, it will run without restraint. Please take care as to make sure the [`device_id`](crate::js_camera::JSCameraConstraintsBuilder::device_id) and the [`groupId`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/groupId)
    /// fields are not malicious! (This usually boils down to not letting users input data directly)
    ///
    /// # Errors
    /// This function may return an error on an invalid string in [`device_id`](crate::js_camera::JSCameraConstraintsBuilder::device_id) or [`groupId`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/groupId) or if the
    /// Javascript Function fails to run.
    pub fn build(self) -> Result<JSCameraConstraints, NokhwaError> {
        let null_resolution = Resolution::default();
        let null_string = String::new();

        let width_string = {
            if self.resolution_exact {
                if self.preferred_resolution != null_resolution {
                    format!("width: {{ exact: {} }}", self.preferred_resolution.width_x)
                } else {
                    format!("")
                }
            } else {
                if self.preferred_resolution.width_x != 0 {
                    format!("width: {{ ideal: {} }}", self.preferred_resolution.width_x)
                } else {
                    format!("")
                }
            }
        };

        let height_string = {
            if self.aspect_ratio_exact {
                if self.preferred_resolution != null_resolution {
                    format!(
                        "height: {{ exact: {} }}",
                        self.preferred_resolution.height_y
                    )
                } else {
                    format!("")
                }
            } else {
                if self.preferred_resolution != null_resolution {
                    format!(
                        "height: {{ ideal: {} }}",
                        self.preferred_resolution.height_y
                    )
                } else {
                    format!("")
                }
            }
        };

        let aspect_ratio_string = {
            if self.aspect_ratio_exact {
                if self.aspect_ratio != 0_f64 {
                    format!("aspectRatio: {{ exact: {} }}", self.aspect_ratio)
                } else {
                    format!("")
                }
            } else {
                if self.aspect_ratio != 0_f64 {
                    format!("aspectRatio: {{ ideal: {} }}", self.aspect_ratio)
                } else {
                    format!("")
                }
            }
        };

        let facing_mode_string = {
            if self.facing_mode_exact {
                if self.facing_mode != JSCameraFacingMode::Any {
                    format!("facingMode: {{ exact: {} }}", self.facing_mode)
                } else {
                    format!("")
                }
            } else {
                if self.facing_mode != JSCameraFacingMode::Any {
                    format!("facingMode: {{ ideal: {} }}", self.facing_mode)
                } else {
                    format!("")
                }
            }
        };

        let frame_rate_string = {
            if self.frame_rate_exact {
                if self.frame_rate != 0 {
                    format!("frameRate: {{ exact: {} }}", self.frame_rate)
                } else {
                    format!("")
                }
            } else {
                if self.frame_rate != 0 {
                    format!("frameRate: {{ ideal: {} }}", self.frame_rate)
                } else {
                    format!("")
                }
            }
        };

        let resize_mode_string = {
            if self.resize_mode_exact {
                if self.resize_mode != JSCameraResizeMode::Any {
                    format!("resizeMode: {{ exact: {} }}", self.resize_mode)
                } else {
                    format!("")
                }
            } else {
                if self.resize_mode != JSCameraResizeMode::Any {
                    format!("resizeMode: {{ ideal: {} }}", self.resize_mode)
                } else {
                    format!("")
                }
            }
        };

        let device_id_string = {
            if self.device_id_exact {
                if self.device_id != null_string {
                    format!("deviceId: {{ exact: {} }}", self.device_id)
                } else {
                    format!("")
                }
            } else {
                if self.device_id != null_string {
                    format!("deviceId: {{ ideal: {} }}", self.device_id)
                } else {
                    format!("")
                }
            }
        };

        let group_id_string = {
            if self.group_id_exact {
                if self.group_id != null_string {
                    format!("groupId: {{ exact: {} }}", self.group_id)
                } else {
                    format!("")
                }
            } else {
                if self.group_id != null_string {
                    format!("groupId: {{ ideal: {} }}", self.group_id)
                } else {
                    format!("")
                }
            }
        };

        let mut arguments = vec![
            width_string,
            height_string,
            aspect_ratio_string,
            facing_mode_string,
            frame_rate_string,
            resize_mode_string,
            device_id_string,
            group_id_string,
        ];
        arguments.sort();
        arguments.dedup();

        let mut arguments_condensed = String::new();
        for argument in arguments {
            if argument != null_string {
                arguments_condensed = format!("{},{}\n", arguments_condensed, argument);
            }
        }
        if arguments_condensed == null_string {
            arguments_condensed = "true".to_string();
        }

        let constraints_fn = Function::new_no_args(&format!(
            r#"
let constraints = {{
    audio: false,
    video: {{
        {}
    }}
}};

return constraints;
"#,
            arguments_condensed
        ));
        match constraints_fn.call0(&JsValue::NULL) {
            Ok(constraints) => {
                let constraints: JsValue = constraints;
                let media_stream_constraints = MediaStreamConstraints::from(constraints);
                Ok(JSCameraConstraints {
                    media_constraints: media_stream_constraints,
                })
            }
            Err(why) => Err(NokhwaError::StructureError {
                structure: "MediaStreamConstraintsJSBuild".to_string(),
                error: format!("{:?}", why),
            }),
        }
    }
}

impl Default for JSCameraConstraintsBuilder {
    fn default() -> Self {
        JSCameraConstraintsBuilder {
            preferred_resolution: Resolution::default(),
            resolution_exact: false,
            aspect_ratio: 0.0,
            aspect_ratio_exact: false,
            facing_mode: JSCameraFacingMode::Any,
            facing_mode_exact: false,
            frame_rate: 0,
            frame_rate_exact: false,
            resize_mode: JSCameraResizeMode::Any,
            resize_mode_exact: false,
            device_id: "".to_string(),
            device_id_exact: false,
            group_id: "".to_string(),
            group_id_exact: false,
        }
    }
}

/// Constraints to create a [`JSCamera`]
///
/// If you want more options, see [`JSCameraConstraintsBuilder`]
#[derive(Clone, Debug)]
pub struct JSCameraConstraints {
    pub(crate) media_constraints: MediaStreamConstraints,
}

/// Generates a default [`JSCameraConstraints`] with
/// ```js
/// let constraints = {{
///     audio: false,
///     video: true;
///
/// return constraints;
/// ```
///
/// # Errors
/// This function may fail to run if the Javascript Function fails to run.
impl JSCameraConstraints {
    pub fn default() -> Result<Self, NokhwaError> {
        let constraints_fn = Function::new_no_args(
            r#"
let constraints = {{
    audio: false,
    video: true;

return constraints;
"#,
        );

        match constraints_fn.call0(&JsValue::NULL) {
            Ok(constraints) => {
                let constraints: JsValue = constraints;
                let media_stream_constraints = MediaStreamConstraints::from(constraints);
                Ok(JSCameraConstraints {
                    media_constraints: media_stream_constraints,
                })
            }
            Err(why) => Err(NokhwaError::StructureError {
                structure: "MediaStreamConstraintsJSBuildDefault".to_string(),
                error: format!("{:?}", why),
            }),
        }
    }
}

impl Deref for JSCameraConstraints {
    type Target = MediaStreamConstraints;

    fn deref(&self) -> &Self::Target {
        &self.media_constraints
    }
}

/// A wrapper around a [`MediaStream`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStream.html)
pub struct JSCamera {
    media_stream: MediaStream,
}

impl JSCamera {
    /// Creates a new [`JSCamera`] using [`JSCameraConstraints`].
    ///
    /// # Errors
    /// This may error if permission is not granted, or the constraints are invalid.
    pub async fn new(constraints: JSCameraConstraints) -> Result<Self, NokhwaError> {
        let window: Window = match window() {
            Some(win) => win,
            None => {
                return Err(NokhwaError::StructureError {
                    structure: "web_sys Window".to_string(),
                    error: "None".to_string(),
                })
            }
        };
        let navigator = window.navigator();
        let media_devices = match navigator.media_devices() {
            Ok(media) => media,
            Err(why) => {
                return Err(NokhwaError::StructureError {
                    structure: "MediaDevices".to_string(),
                    error: format!("{:?}", why),
                })
            }
        };

        let stream: MediaStream = match media_devices.get_user_media_with_constraints(&*constraints)
        {
            Ok(promise) => {
                let future = JsFuture::from(promise);
                match future.await {
                    Ok(stream) => {
                        let media_stream: MediaStream = MediaStream::from(stream);
                        media_stream
                    }
                    Err(why) => {
                        return Err(NokhwaError::StructureError {
                            structure: "MediaDevicesGetUserMediaJsFuture".to_string(),
                            error: format!("{:?}", why),
                        })
                    }
                }
            }
            Err(why) => {
                return Err(NokhwaError::StructureError {
                    structure: "MediaDevicesGetUserMedia".to_string(),
                    error: format!("{:?}", why),
                })
            }
        };

        Ok(JSCamera {
            media_stream: stream,
        })
    }
}

impl Deref for JSCamera {
    type Target = MediaStream;

    fn deref(&self) -> &Self::Target {
        &self.media_stream
    }
}

impl Drop for JSCamera {
    fn drop(&mut self) {
        todo!()
    }
}
