/* tslint:disable */
/* eslint-disable */
/**
* Requests Webcam permissions from the browser using [`MediaDevices::get_user_media()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaDevices.html#method.get_user_media) [MDN](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getUserMedia)
* # Errors
* This will error if there is no valid web context or the web API is not supported
* # JS-WASM
* In exported JS bindings, the name of the function is `requestPermissions`. It may throw an exception.
* @returns {any}
*/
export function requestPermissions(): any;
/**
* Queries Cameras using [`MediaDevices::enumerate_devices()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaDevices.html#method.enumerate_devices) [MDN](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/enumerateDevices)
* # Errors
* This will error if there is no valid web context or the web API is not supported
* # JS-WASM
* This is exported as `queryCameras`. It may throw an exception.
* @returns {any}
*/
export function queryCameras(): any;
/**
* Queries the browser's supported constraints using [`navigator.mediaDevices.getSupportedConstraints()`](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getSupportedConstraints)
* # Errors
* This will error if there is no valid web context or the web API is not supported
* # JS-WASM
* This is exported as `queryConstraints` and returns an array of strings.
* @returns {Array<any>}
*/
export function queryConstraints(): Array<any>;
/**
* The enum describing the possible constraints for video in the browser.
* - `DeviceID`: The ID of the device
* - `GroupID`: The ID of the group that the device is in
* - `AspectRatio`: The Aspect Ratio of the final stream
* - `FacingMode`: What direction the camera is facing. This is more common on mobile. See [`JSCameraFacingMode`]
* - `FrameRate`: The Frame Rate of the final stream
* - `Height`: The height of the final stream in pixels
* - `Width`: The width of the final stream in pixels
* - `ResizeMode`: Whether the client can crop and/or scale the stream to match the resolution (width, height). See [`JSCameraResizeMode`]
* See More: [`MediaTrackConstraints`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints) [`Capabilities, constraints, and settings`](https://developer.mozilla.org/en-US/docs/Web/API/Media_Streams_API/Constraints)
* # JS-WASM
* This is exported as `CameraSupportedCapabilities`.
*/
export enum CameraSupportedCapabilities {
  DeviceID,
  GroupID,
  AspectRatio,
  FacingMode,
  FrameRate,
  Height,
  Width,
  ResizeMode,
}
/**
* The Facing Mode of the camera
* - Any: Make no particular choice.
* - Environment: The camera that shows the user's environment, such as the back camera of a smartphone
* - User: The camera that shows the user, such as the front camera of a smartphone
* - Left: The camera that shows the user but to their left, such as a camera that shows a user but to their left shoulder
* - Right: The camera that shows the user but to their right, such as a camera that shows a user but to their right shoulder
* See More: [`facingMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/facingMode)
* # JS-WASM
* This is exported as `CameraFacingMode`.
*/
export enum CameraFacingMode {
  Any,
  Environment,
  User,
  Left,
  Right,
}
/**
* Whether the browser can crop and/or scale to match the requested resolution.
* - `Any`: Make no particular choice.
* - `None`: Do not crop and/or scale.
* - `CropAndScale`: Crop and/or scale to match the requested resolution.
* See More: [`resizeMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#resizemode)
* # JS-WASM
* This is exported as `CameraResizeMode`.
*/
export enum CameraResizeMode {
  Any,
  None,
  CropAndScale,
}
/**
* Constraints to create a [`JSCamera`]
*
* If you want more options, see [`JSCameraConstraintsBuilder`]
* # JS-WASM
* This is exported as `CameraConstraints`.
*/
export class CameraConstraints {
  free(): void;
/**
* Applies any modified constraints.
* # JS-WASM
* This is exported as `applyConstraints`.
*/
  applyConstraints(): void;
/**
* Gets the internal aspect ratio.
* # JS-WASM
* This is exported as `get_AspectRatio`.
* @returns {number}
*/
  AspectRatio: number;
/**
* Gets the internal aspect ratio exact.
* # JS-WASM
* This is exported as `get_AspectRatioExact`.
* @returns {boolean}
*/
  AspectRatioExact: boolean;
/**
* Gets the internal device id.
* # JS-WASM
* This is exported as `get_DeviceId`.
* @returns {string}
*/
  DeviceId: string;
/**
* Gets the internal device id exact.
* # JS-WASM
* This is exported as `get_DeviceIdExact`.
* @returns {boolean}
*/
  DeviceIdExact: boolean;
/**
* Gets the internal [`JSCameraFacingMode`].
* # JS-WASM
* This is exported as `get_FacingMode`.
* @returns {number}
*/
  FacingMode: number;
/**
* Gets the internal facing mode exact.
* # JS-WASM
* This is exported as `get_FacingModeExact`.
* @returns {boolean}
*/
  FacingModeExact: boolean;
/**
* Gets the internal frame rate.
* # JS-WASM
* This is exported as `get_FrameRate`.
* @returns {number}
*/
  FrameRate: number;
/**
* Gets the internal frame rate exact.
* # JS-WASM
* This is exported as `get_FrameRateExact`.
* @returns {boolean}
*/
  FrameRateExact: boolean;
/**
* Gets the internal group id.
* # JS-WASM
* This is exported as `get_GroupId`.
* @returns {string}
*/
  GroupId: string;
/**
* Gets the internal group id exact.
* # JS-WASM
* This is exported as `get_GroupIdExact`.
* @returns {boolean}
*/
  GroupIdExact: boolean;
/**
* Gets the maximum aspect ratio.
* # JS-WASM
* This is exported as `get_MaxAspectRatio`.
* @returns {number | undefined}
*/
  MaxAspectRatio: number;
/**
* Gets the maximum internal frame rate.
* # JS-WASM
* This is exported as `get_MaxFrameRate`.
* @returns {number | undefined}
*/
  MaxFrameRate: number;
/**
* Gets the maximum [`Resolution`].
* # JS-WASM
* This is exported as `get_MaxResolution`.
* @returns {Resolution | undefined}
*/
  MaxResolution: Resolution;
/**
* Gets the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html)
* # JS-WASM
* This is exported as `get_MediaStreamConstraints`.
* @returns {any}
*/
  readonly MediaStreamConstraints: any;
/**
* Gets the minimum aspect ratio of the [`JSCameraConstraints`].
* # JS-WASM
* This is exported as `get_MinAspectRatio`.
* @returns {number | undefined}
*/
  MinAspectRatio: number;
/**
* Gets the minimum internal frame rate.
* # JS-WASM
* This is exported as `get_MinFrameRate`.
* @returns {number | undefined}
*/
  MinFrameRate: number;
/**
* Gets the minimum [`Resolution`].
* # JS-WASM
* This is exported as `get_MinResolution`.
* @returns {Resolution | undefined}
*/
  MinResolution: Resolution;
/**
* Gets the internal [`JSCameraResizeMode`].
* # JS-WASM
* This is exported as `get_ResizeMode`.
* @returns {number}
*/
  ResizeMode: number;
/**
* Gets the internal resize mode exact.
* # JS-WASM
* This is exported as `get_ResizeModeExact`.
* @returns {boolean}
*/
  ResizeModeExact: boolean;
/**
* Gets the internal [`Resolution`]
* # JS-WASM
* This is exported as `get_Resolution`.
* @returns {Resolution}
*/
  Resolution: Resolution;
/**
* Gets the internal resolution exact.
* # JS-WASM
* This is exported as `get_ResolutionExact`.
* @returns {boolean}
*/
  ResolutionExact: boolean;
}
/**
* A builder that builds a [`JSCameraConstraints`] that is used to construct a [`JSCamera`].
* See More: [`Constraints MDN`](https://developer.mozilla.org/en-US/docs/Web/API/Media_Streams_API/Constraints), [`Properties of Media Tracks MDN`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints)
* # JS-WASM
* This is exported as `CameraConstraintsBuilder`.
*/
export class CameraConstraintsBuilder {
  free(): void;
/**
* Constructs a default [`JSCameraConstraintsBuilder`].
* The constructed default [`JSCameraConstraintsBuilder`] has these settings:
* - 480x234 min, 640x360 ideal, 1920x1080 max
* - 10 FPS min, 15 FPS ideal, 30 FPS max
* - 1.0 aspect ratio min, 1.77777777778 aspect ratio ideal, 2.0 aspect ratio max
* - No `exact`s
* # JS-WASM
* This is exported as a constructor.
*/
  constructor();
/**
* Sets the minimum resolution for the [`JSCameraConstraintsBuilder`].
*
* Sets [`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width) and [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height).
* # JS-WASM
* This is exported as `set_MinResolution`.
* @param {Resolution} min_resolution
* @returns {CameraConstraintsBuilder}
*/
  MinResolution(min_resolution: Resolution): CameraConstraintsBuilder;
/**
* Sets the preferred resolution for the [`JSCameraConstraintsBuilder`].
*
* Sets [`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width) and [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height).
* # JS-WASM
* This is exported as `set_Resolution`.
* @param {Resolution} new_resolution
* @returns {CameraConstraintsBuilder}
*/
  Resolution(new_resolution: Resolution): CameraConstraintsBuilder;
/**
* Sets the maximum resolution for the [`JSCameraConstraintsBuilder`].
*
* Sets [`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width) and [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height).
* # JS-WASM
* This is exported as `set_MaxResolution`.
* @param {Resolution} max_resolution
* @returns {CameraConstraintsBuilder}
*/
  MaxResolution(max_resolution: Resolution): CameraConstraintsBuilder;
/**
* Sets whether the resolution fields ([`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width), [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height)/[`resolution`](crate::js_camera::JSCameraConstraintsBuilder::resolution))
* should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
* Note that this will make the builder ignore [`min_resolution`](crate::js_camera::JSCameraConstraintsBuilder::min_resolution) and [`max_resolution`](crate::js_camera::JSCameraConstraintsBuilder::max_resolution).
* # JS-WASM
* This is exported as `set_ResolutionExact`.
* @param {boolean} value
* @returns {CameraConstraintsBuilder}
*/
  ResolutionExact(value: boolean): CameraConstraintsBuilder;
/**
* Sets the minimum aspect ratio of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`aspectRatio`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/aspectRatio).
* # JS-WASM
* This is exported as `set_MinAspectRatio`.
* @param {number} ratio
* @returns {CameraConstraintsBuilder}
*/
  MinAspectRatio(ratio: number): CameraConstraintsBuilder;
/**
* Sets the aspect ratio of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`aspectRatio`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/aspectRatio).
* # JS-WASM
* This is exported as `set_AspectRatio`.
* @param {number} ratio
* @returns {CameraConstraintsBuilder}
*/
  AspectRatio(ratio: number): CameraConstraintsBuilder;
/**
* Sets the maximum aspect ratio of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`aspectRatio`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/aspectRatio).
* # JS-WASM
* This is exported as `set_MaxAspectRatio`.
* @param {number} ratio
* @returns {CameraConstraintsBuilder}
*/
  MaxAspectRatio(ratio: number): CameraConstraintsBuilder;
/**
* Sets whether the [`aspect_ratio`](crate::js_camera::JSCameraConstraintsBuilder::aspect_ratio) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
* Note that this will make the builder ignore [`min_aspect_ratio`](crate::js_camera::JSCameraConstraintsBuilder::min_aspect_ratio) and [`max_aspect_ratio`](crate::js_camera::JSCameraConstraintsBuilder::max_aspect_ratio).
* # JS-WASM
* This is exported as `set_AspectRatioExact`.
* @param {boolean} value
* @returns {CameraConstraintsBuilder}
*/
  AspectRatioExact(value: boolean): CameraConstraintsBuilder;
/**
* Sets the facing mode of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`facingMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/facingMode).
* # JS-WASM
* This is exported as `set_FacingMode`.
* @param {number} facing_mode
* @returns {CameraConstraintsBuilder}
*/
  FacingMode(facing_mode: number): CameraConstraintsBuilder;
/**
* Sets whether the [`facing_mode`](crate::js_camera::JSCameraConstraintsBuilder::facing_mode) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
* # JS-WASM
* This is exported as `set_FacingModeExact`.
* @param {boolean} value
* @returns {CameraConstraintsBuilder}
*/
  FacingModeExact(value: boolean): CameraConstraintsBuilder;
/**
* Sets the minimum frame rate of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`frameRate`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/frameRate).
* # JS-WASM
* This is exported as `set_MinFrameRate`.
* @param {number} fps
* @returns {CameraConstraintsBuilder}
*/
  MinFrameRate(fps: number): CameraConstraintsBuilder;
/**
* Sets the frame rate of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`frameRate`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/frameRate).
* # JS-WASM
* This is exported as `set_FrameRate`.
* @param {number} fps
* @returns {CameraConstraintsBuilder}
*/
  FrameRate(fps: number): CameraConstraintsBuilder;
/**
* Sets the maximum frame rate of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`frameRate`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/frameRate).
* # JS-WASM
* This is exported as `set_MaxFrameRate`.
* @param {number} fps
* @returns {CameraConstraintsBuilder}
*/
  MaxFrameRate(fps: number): CameraConstraintsBuilder;
/**
* Sets whether the [`frame_rate`](crate::js_camera::JSCameraConstraintsBuilder::frame_rate) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
* Note that this will make the builder ignore [`min_frame_rate`](crate::js_camera::JSCameraConstraintsBuilder::min_frame_rate) and [`max_frame_rate`](crate::js_camera::JSCameraConstraintsBuilder::max_frame_rate).
* # JS-WASM
* This is exported as `set_FrameRateExact`.
* @param {boolean} value
* @returns {CameraConstraintsBuilder}
*/
  FrameRateExact(value: boolean): CameraConstraintsBuilder;
/**
* Sets the resize mode of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`resizeMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#resizemode).
* # JS-WASM
* This is exported as `set_ResizeMode`.
* @param {number} resize_mode
* @returns {CameraConstraintsBuilder}
*/
  ResizeMode(resize_mode: number): CameraConstraintsBuilder;
/**
* Sets whether the [`resize_mode`](crate::js_camera::JSCameraConstraintsBuilder::resize_mode) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
* # JS-WASM
* This is exported as `set_ResizeModeExact`.
* @param {boolean} value
* @returns {CameraConstraintsBuilder}
*/
  ResizeModeExact(value: boolean): CameraConstraintsBuilder;
/**
* Sets the device ID of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`deviceId`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/deviceId).
* # JS-WASM
* This is exported as `set_DeviceId`.
* @param {string} id
* @returns {CameraConstraintsBuilder}
*/
  DeviceId(id: string): CameraConstraintsBuilder;
/**
* Sets whether the [`device_id`](crate::js_camera::JSCameraConstraintsBuilder::device_id) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
* # JS-WASM
* This is exported as `set_DeviceIdExact`.
* @param {boolean} value
* @returns {CameraConstraintsBuilder}
*/
  DeviceIdExact(value: boolean): CameraConstraintsBuilder;
/**
* Sets the group ID of the resulting constraint for the [`JSCameraConstraintsBuilder`].
*
* Sets [`groupId`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/groupId).
* # JS-WASM
* This is exported as `set_GroupId`.
* @param {string} id
* @returns {CameraConstraintsBuilder}
*/
  GroupId(id: string): CameraConstraintsBuilder;
/**
* Sets whether the [`group_id`](crate::js_camera::JSCameraConstraintsBuilder::group_id) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
* # JS-WASM
* This is exported as `set_GroupIdExact`.
* @param {boolean} value
* @returns {CameraConstraintsBuilder}
*/
  GroupIdExact(value: boolean): CameraConstraintsBuilder;
/**
* Builds the [`JSCameraConstraints`]. Wrapper for [`build`](crate::js_camera::JSCameraConstraintsBuilder::build)
*
* Fields that use exact are marked `exact`, otherwise are marked with `ideal`. If min-max are involved, they will use `min` and `max` accordingly.
* # JS-WASM
* This is exported as `buildCameraConstraints`.
* @returns {CameraConstraints}
*/
  buildCameraConstraints(): CameraConstraints;
}
/**
* Information about a Camera e.g. its name.
* `description` amd `misc` may contain information that may differ from backend to backend. Refer to each backend for details.
* `index` is a camera's index given to it by (usually) the OS usually in the order it is known to the system.
* # JS-WASM
* This is exported as a `CameraInfo`.
*/
export class CameraInfo {
  free(): void;
/**
* Create a new [`CameraInfo`].
* # JS-WASM
* This is exported as a constructor for [`CameraInfo`].
* @param {string} human_name
* @param {string} description
* @param {string} misc
* @param {number} index
*/
  constructor(human_name: string, description: string, misc: string, index: number);
/**
* Get a reference to the device info's description.
* # JS-WASM
* This is exported as a `get_Description`.
* @returns {string}
*/
  Description: string;
/**
* Get a reference to the device info's human readable name.
* # JS-WASM
* This is exported as a `get_HumanReadableName`.
* @returns {string}
*/
  HumanReadableName: string;
/**
* Get a reference to the device info's index.
* # JS-WASM
* This is exported as a `get_Index`.
* @returns {number}
*/
  Index: number;
/**
* Get a reference to the device info's misc.
* # JS-WASM
* This is exported as a `get_MiscString`.
* @returns {string}
*/
  MiscString: string;
}
/**
* A wrapper around a [`MediaStream`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStream.html)
* # JS-WASM
* This is exported as `NOKCamera`.
*/
export class NokhwaCamera {
  free(): void;
/**
* Creates a new [`JSCamera`] using [`JSCameraConstraints`].
*
* # Errors
* This may error if permission is not granted, or the constraints are invalid.
* # JS-WASM
* This is the constructor for `NOKCamera`. It returns a promise and may throw an error.
* @param {CameraConstraints} constraints
*/
  constructor(constraints: CameraConstraints);
/**
* Measures the [`Resolution`] of the internal stream. You usually do not need to call this.
*
* # Errors
* If the camera fails to attach to the created `<video>`, this will error.
*
* # JS-WASM
* This is exported as `measureResolution`. It may throw an error.
*/
  measureResolution(): void;
/**
* Applies any modified constraints.
* # Errors
* This function may return an error on failing to measure the resolution. Please check [`measure_resolution()`](crate::js_camera::JSCamera::measure_resolution) for details.
* # JS-WASM
* This is exported as `applyConstraints`. It may throw an error.
*/
  applyConstraints(): void;
/**
* Captures an [`ImageData`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.ImageData.html) [`MDN`](https://developer.mozilla.org/en-US/docs/Web/API/ImageData) by drawing the image to a non-existent canvas.
*
* # Errors
* If drawing to the canvas fails this will error.
* # JS-WASM
* This is exported as `captureImageData`. It may throw an error.
* @returns {ImageData}
*/
  captureImageData(): ImageData;
/**
* Captures an [`ImageData`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.ImageData.html) [`MDN`](https://developer.mozilla.org/en-US/docs/Web/API/ImageData) and then returns its `URL` as a string.
* - `mime_type`: The mime type of the resulting URI. It is `image/png` by default (lossless) but can be set to `image/jpeg` or `image/webp` (lossy). Anything else is ignored.
* - `image_quality`: A number between `0` and `1` indicating the resulting image quality in case you are using a lossy image mime type. The default value is 0.92, and all other values are ignored.
*
* # Errors
* If drawing to the canvas fails or URI generation is not supported or fails this will error.
* # JS-WASM
* This is exported as `captureImageURI`. It may throw an error
* @param {string} mime_type
* @param {number} image_quality
* @returns {string}
*/
  captureImageURI(mime_type: string, image_quality: number): string;
/**
* Creates an off-screen canvas and a `<video>` element (if not already attached) and returns a raw `Cow<[u8]>` RGBA frame.
* # Errors
* If a cast fails, the camera fails to attach, the currently attached node is invalid, or writing/reading from the canvas fails, this will error.
* # JS-WASM
* This is exported as `captureFrameRawData`. This may throw an error.
* @returns {Uint8Array}
*/
  captureFrameRawData(): Uint8Array;
/**
* Attaches camera to a `element`(by-id).
* - `generate_new`: Whether to add a video element to provided element to attach to. Set this to `false` if the `element` ID you are passing is already a `<video>` element.
* # Errors
* If the camera fails to attach, fails to generate the video element, or a cast fails, this will error.
* # JS-WASM
* This is exported as `attachToElement`. It may throw an error.
* @param {string} element
* @param {boolean} generate_new
*/
  attachToElement(element: string, generate_new: boolean): void;
/**
* Detaches the camera from the `<video>` node.
* # Errors
* If the casting fails (the stored node is not a `<video>`) this will error.
* # JS-WASM
* This is exported as `detachCamera`. This may throw an error.
*/
  detachCamera(): void;
/**
* Stops all streams and detaches the camera.
* # Errors
* There may be an error while detaching the camera. Please see [`detach()`](crate::js_camera::JSCamera::detach) for more details.
*/
  stopAll(): void;
/**
* Gets the internal [`JSCameraConstraints`].
* Most likely, you will edit this value by taking ownership of it, then feed it back into [`set_constraints`](crate::js_camera::JSCamera::set_constraints).
* # JS-WASM
* This is exported as `get_Constraints`.
* @returns {CameraConstraints}
*/
  Constraints: CameraConstraints;
/**
* Gets the internal [`MediaStream`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStream.html) [`MDN`](https://developer.mozilla.org/en-US/docs/Web/API/MediaStream)
* # JS-WASM
* This is exported as `MediaStream`.
* @returns {MediaStream}
*/
  readonly MediaStream: MediaStream;
/**
* Gets the internal [`Resolution`].
*
* Note: This value is only updated after you call [`measure_resolution`](crate::js_camera::JSCamera::measure_resolution)
* # JS-WASM
* This is exported as `get_Resolution`.
* @returns {Resolution}
*/
  readonly Resolution: Resolution;
}
/**
* Describes a Resolution.
* This struct consists of a Width and a Height value (x,y). <br>
* Note: the [`Ord`] implementation of this struct is flipped from highest to lowest.
* # JS-WASM
* This is exported as `Resolution`
*/
export class Resolution {
  free(): void;
/**
* Create a new resolution from 2 image size coordinates.
* # JS-WASM
* This is exported as a constructor for [`Resolution`].
* @param {number} x
* @param {number} y
*/
  constructor(x: number, y: number);
/**
* Get the x (width) of Resolution
* @returns {number}
*/
  x(): number;
/**
* Get the y (height) of Resolution
* @returns {number}
*/
  y(): number;
/**
* Get the height of Resolution
* # JS-WASM
* This is exported as `get_Height`.
* @returns {number}
*/
  readonly Height: number;
/**
* Get the width of Resolution
* # JS-WASM
* This is exported as `get_Width`.
* @returns {number}
*/
  readonly Width: number;
/**
* @returns {number}
*/
  height_y: number;
/**
* @returns {number}
*/
  width_x: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly requestPermissions: () => number;
  readonly queryCameras: () => number;
  readonly queryConstraints: () => number;
  readonly __wbg_cameraconstraintsbuilder_free: (a: number) => void;
  readonly cameraconstraintsbuilder_new: () => number;
  readonly cameraconstraintsbuilder_Resolution: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_MaxResolution: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_ResolutionExact: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_MinAspectRatio: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_AspectRatio: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_MaxAspectRatio: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_AspectRatioExact: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_FacingMode: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_FacingModeExact: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_MinFrameRate: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_FrameRate: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_MaxFrameRate: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_FrameRateExact: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_ResizeMode: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_ResizeModeExact: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_DeviceId: (a: number, b: number, c: number) => number;
  readonly cameraconstraintsbuilder_DeviceIdExact: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_GroupId: (a: number, b: number, c: number) => number;
  readonly cameraconstraintsbuilder_GroupIdExact: (a: number, b: number) => number;
  readonly cameraconstraintsbuilder_buildCameraConstraints: (a: number) => number;
  readonly __wbg_cameraconstraints_free: (a: number) => void;
  readonly cameraconstraints_media_constraints: (a: number) => number;
  readonly cameraconstraints_min_resolution: (a: number) => number;
  readonly cameraconstraints_set_min_resolution: (a: number, b: number) => void;
  readonly cameraconstraints_resolution: (a: number) => number;
  readonly cameraconstraints_set_resolution: (a: number, b: number) => void;
  readonly cameraconstraints_max_resolution: (a: number) => number;
  readonly cameraconstraints_set_max_resolution: (a: number, b: number) => void;
  readonly cameraconstraints_resolution_exact: (a: number) => number;
  readonly cameraconstraints_set_resolution_exact: (a: number, b: number) => void;
  readonly cameraconstraints_min_aspect_ratio: (a: number, b: number) => void;
  readonly cameraconstraints_set_min_aspect_ratio: (a: number, b: number) => void;
  readonly cameraconstraints_aspect_ratio: (a: number) => number;
  readonly cameraconstraints_set_aspect_ratio: (a: number, b: number) => void;
  readonly cameraconstraints_max_aspect_ratio: (a: number, b: number) => void;
  readonly cameraconstraints_set_max_aspect_ratio: (a: number, b: number) => void;
  readonly cameraconstraints_aspect_ratio_exact: (a: number) => number;
  readonly cameraconstraints_set_aspect_ratio_exact: (a: number, b: number) => void;
  readonly cameraconstraints_facing_mode: (a: number) => number;
  readonly cameraconstraints_set_facing_mode: (a: number, b: number) => void;
  readonly cameraconstraints_facing_mode_exact: (a: number) => number;
  readonly cameraconstraints_set_facing_mode_exact: (a: number, b: number) => void;
  readonly cameraconstraints_min_frame_rate: (a: number, b: number) => void;
  readonly cameraconstraints_set_min_frame_rate: (a: number, b: number) => void;
  readonly cameraconstraints_frame_rate: (a: number) => number;
  readonly cameraconstraints_set_frame_rate: (a: number, b: number) => void;
  readonly cameraconstraints_max_frame_rate: (a: number, b: number) => void;
  readonly cameraconstraints_set_max_frame_rate: (a: number, b: number) => void;
  readonly cameraconstraints_frame_rate_exact: (a: number) => number;
  readonly cameraconstraints_set_frame_rate_exact: (a: number, b: number) => void;
  readonly cameraconstraints_resize_mode: (a: number) => number;
  readonly cameraconstraints_set_resize_mode: (a: number, b: number) => void;
  readonly cameraconstraints_resize_mode_exact: (a: number) => number;
  readonly cameraconstraints_set_resize_mode_exact: (a: number, b: number) => void;
  readonly cameraconstraints_device_id: (a: number, b: number) => void;
  readonly cameraconstraints_set_device_id: (a: number, b: number, c: number) => void;
  readonly cameraconstraints_device_id_exact: (a: number) => number;
  readonly cameraconstraints_set_device_id_exact: (a: number, b: number) => void;
  readonly cameraconstraints_group_id: (a: number, b: number) => void;
  readonly cameraconstraints_set_group_id: (a: number, b: number, c: number) => void;
  readonly cameraconstraints_group_id_exact: (a: number) => number;
  readonly cameraconstraints_set_group_id_exact: (a: number, b: number) => void;
  readonly cameraconstraints_applyConstraints: (a: number) => void;
  readonly __wbg_nokhwacamera_free: (a: number) => void;
  readonly nokhwacamera_js_new: (a: number) => number;
  readonly nokhwacamera_constraints: (a: number) => number;
  readonly nokhwacamera_js_set_constraints: (a: number, b: number) => void;
  readonly nokhwacamera_resolution: (a: number) => number;
  readonly nokhwacamera_measureResolution: (a: number) => void;
  readonly nokhwacamera_applyConstraints: (a: number) => void;
  readonly nokhwacamera_media_stream: (a: number) => number;
  readonly nokhwacamera_captureImageData: (a: number) => number;
  readonly nokhwacamera_captureImageURI: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly nokhwacamera_captureFrameRawData: (a: number, b: number) => void;
  readonly nokhwacamera_attachToElement: (a: number, b: number, c: number, d: number) => void;
  readonly nokhwacamera_detachCamera: (a: number) => void;
  readonly nokhwacamera_stopAll: (a: number) => void;
  readonly cameraconstraintsbuilder_MinResolution: (a: number, b: number) => number;
  readonly __wbg_resolution_free: (a: number) => void;
  readonly __wbg_get_resolution_width_x: (a: number) => number;
  readonly __wbg_set_resolution_width_x: (a: number, b: number) => void;
  readonly __wbg_get_resolution_height_y: (a: number) => number;
  readonly __wbg_set_resolution_height_y: (a: number, b: number) => void;
  readonly resolution_width: (a: number) => number;
  readonly resolution_height: (a: number) => number;
  readonly __wbg_camerainfo_free: (a: number) => void;
  readonly camerainfo_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
  readonly camerainfo_human_name: (a: number, b: number) => void;
  readonly camerainfo_set_human_name: (a: number, b: number, c: number) => void;
  readonly camerainfo_description: (a: number, b: number) => void;
  readonly camerainfo_set_description: (a: number, b: number, c: number) => void;
  readonly camerainfo_misc: (a: number, b: number) => void;
  readonly camerainfo_set_misc: (a: number, b: number, c: number) => void;
  readonly camerainfo_index: (a: number) => number;
  readonly camerainfo_set_index: (a: number, b: number) => void;
  readonly resolution_new: (a: number, b: number) => number;
  readonly resolution_x: (a: number) => number;
  readonly resolution_y: (a: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha79fab5af65c7d0b: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__hf03b20e2f7b10743: (a: number, b: number, c: number, d: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
