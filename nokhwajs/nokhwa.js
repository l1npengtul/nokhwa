
let wasm;

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_22(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha79fab5af65c7d0b(arg0, arg1, addHeapObject(arg2));
}

/**
* Requests Webcam permissions from the browser using [`MediaDevices::get_user_media()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaDevices.html#method.get_user_media) [MDN](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getUserMedia)
* # Errors
* This will error if there is no valid web context or the web API is not supported
* # JS-WASM
* In exported JS bindings, the name of the function is `requestPermissions`. It may throw an exception.
* @returns {any}
*/
export function requestPermissions() {
    var ret = wasm.requestPermissions();
    return takeObject(ret);
}

/**
* Queries Cameras using [`MediaDevices::enumerate_devices()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaDevices.html#method.enumerate_devices) [MDN](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/enumerateDevices)
* # Errors
* This will error if there is no valid web context or the web API is not supported
* # JS-WASM
* This is exported as `queryCameras`. It may throw an exception.
* @returns {any}
*/
export function queryCameras() {
    var ret = wasm.queryCameras();
    return takeObject(ret);
}

/**
* Queries the browser's supported constraints using [`navigator.mediaDevices.getSupportedConstraints()`](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getSupportedConstraints)
* # Errors
* This will error if there is no valid web context or the web API is not supported
* # JS-WASM
* This is exported as `queryConstraints` and returns an array of strings.
* @returns {Array<any>}
*/
export function queryConstraints() {
    var ret = wasm.queryConstraints();
    return takeObject(ret);
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

let cachegetFloat64Memory0 = null;
function getFloat64Memory0() {
    if (cachegetFloat64Memory0 === null || cachegetFloat64Memory0.buffer !== wasm.memory.buffer) {
        cachegetFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachegetFloat64Memory0;
}

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
function __wbg_adapter_215(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures__invoke2_mut__hf03b20e2f7b10743(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

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
export const CameraSupportedCapabilities = Object.freeze({ DeviceID:0,"0":"DeviceID",GroupID:1,"1":"GroupID",AspectRatio:2,"2":"AspectRatio",FacingMode:3,"3":"FacingMode",FrameRate:4,"4":"FrameRate",Height:5,"5":"Height",Width:6,"6":"Width",ResizeMode:7,"7":"ResizeMode", });
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
export const CameraFacingMode = Object.freeze({ Any:0,"0":"Any",Environment:1,"1":"Environment",User:2,"2":"User",Left:3,"3":"Left",Right:4,"4":"Right", });
/**
* Whether the browser can crop and/or scale to match the requested resolution.
* - `Any`: Make no particular choice.
* - `None`: Do not crop and/or scale.
* - `CropAndScale`: Crop and/or scale to match the requested resolution.
* See More: [`resizeMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#resizemode)
* # JS-WASM
* This is exported as `CameraResizeMode`.
*/
export const CameraResizeMode = Object.freeze({ Any:0,"0":"Any",None:1,"1":"None",CropAndScale:2,"2":"CropAndScale", });
/**
* Constraints to create a [`JSCamera`]
*
* If you want more options, see [`JSCameraConstraintsBuilder`]
* # JS-WASM
* This is exported as `CameraConstraints`.
*/
export class CameraConstraints {

    static __wrap(ptr) {
        const obj = Object.create(CameraConstraints.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_cameraconstraints_free(ptr);
    }
}
/**
* A builder that builds a [`JSCameraConstraints`] that is used to construct a [`JSCamera`].
* See More: [`Constraints MDN`](https://developer.mozilla.org/en-US/docs/Web/API/Media_Streams_API/Constraints), [`Properties of Media Tracks MDN`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints)
* # JS-WASM
* This is exported as `CameraConstraintsBuilder`.
*/
export class CameraConstraintsBuilder {

    static __wrap(ptr) {
        const obj = Object.create(CameraConstraintsBuilder.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_cameraconstraintsbuilder_free(ptr);
    }
}
/**
* Information about a Camera e.g. its name.
* `description` amd `misc` may contain information that may differ from backend to backend. Refer to each backend for details.
* `index` is a camera's index given to it by (usually) the OS usually in the order it is known to the system.
* # JS-WASM
* This is exported as a `CameraInfo`.
*/
export class CameraInfo {

    static __wrap(ptr) {
        const obj = Object.create(CameraInfo.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_camerainfo_free(ptr);
    }
    /**
    * Create a new [`CameraInfo`].
    * # JS-WASM
    * This is exported as a constructor for [`CameraInfo`].
    * @param {string} human_name
    * @param {string} description
    * @param {string} misc
    * @param {number} index
    */
    constructor(human_name, description, misc, index) {
        var ptr0 = passStringToWasm0(human_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        var ptr1 = passStringToWasm0(description, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        var ptr2 = passStringToWasm0(misc, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len2 = WASM_VECTOR_LEN;
        var ret = wasm.camerainfo_new(ptr0, len0, ptr1, len1, ptr2, len2, index);
        return CameraInfo.__wrap(ret);
    }
    /**
    * Get a reference to the device info's human readable name.
    * # JS-WASM
    * This is exported as a `get_HumanReadableName`.
    * @returns {string}
    */
    get HumanReadableName() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.camerainfo_HumanReadableName(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * Set the device info's human name.
    * # JS-WASM
    * This is exported as a `set_HumanReadableName`.
    * @param {string} human_name
    */
    set HumanReadableName(human_name) {
        var ptr0 = passStringToWasm0(human_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.camerainfo_set_HumanReadableName(this.ptr, ptr0, len0);
    }
    /**
    * Get a reference to the device info's description.
    * # JS-WASM
    * This is exported as a `get_Description`.
    * @returns {string}
    */
    get Description() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.camerainfo_Description(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * Set the device info's description.
    * # JS-WASM
    * This is exported as a `set_Description`.
    * @param {string} description
    */
    set Description(description) {
        var ptr0 = passStringToWasm0(description, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.camerainfo_set_Description(this.ptr, ptr0, len0);
    }
    /**
    * Get a reference to the device info's misc.
    * # JS-WASM
    * This is exported as a `get_MiscString`.
    * @returns {string}
    */
    get MiscString() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.camerainfo_MiscString(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * Set the device info's misc.
    * # JS-WASM
    * This is exported as a `set_MiscString`.
    * @param {string} misc
    */
    set MiscString(misc) {
        var ptr0 = passStringToWasm0(misc, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.camerainfo_set_MiscString(this.ptr, ptr0, len0);
    }
    /**
    * Get a reference to the device info's index.
    * # JS-WASM
    * This is exported as a `get_Index`.
    * @returns {number}
    */
    get Index() {
        var ret = wasm.camerainfo_Index(this.ptr);
        return ret >>> 0;
    }
    /**
    * Set the device info's index.
    * # JS-WASM
    * This is exported as a `set_Index`.
    * @param {number} index
    */
    set Index(index) {
        wasm.camerainfo_set_Index(this.ptr, index);
    }
}

export class JSCamera {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jscamera_free(ptr);
    }
    /**
    * Creates a new [`JSCamera`] using [`JSCameraConstraints`].
    *
    * # Errors
    * This may error if permission is not granted, or the constraints are invalid.
    * # JS-WASM
    * This is the constructor for `NOKCamera`. It returns a promise and may throw an error.
    * @param {CameraConstraints} constraints
    */
    constructor(constraints) {
        _assertClass(constraints, CameraConstraints);
        var ptr0 = constraints.ptr;
        constraints.ptr = 0;
        var ret = wasm.jscamera_js_new(ptr0);
        return takeObject(ret);
    }
    /**
    * Gets the internal [`JSCameraConstraints`].
    * Most likely, you will edit this value by taking ownership of it, then feed it back into [`set_constraints`](crate::js_camera::JSCamera::set_constraints).
    * # JS-WASM
    * This is exported as `get_Constraints`.
    * @returns {CameraConstraints}
    */
    get Constraints() {
        var ret = wasm.jscamera_Constraints(this.ptr);
        return CameraConstraints.__wrap(ret);
    }
    /**
    * Sets the [`JSCameraConstraints`]. This calls [`apply_constraints`](crate::js_camera::JSCamera::apply_constraints) internally.
    *
    * # Errors
    * See [`apply_constraints`](crate::js_camera::JSCamera::apply_constraints).
    * # JS-WASM
    * This is exported as `set_Constraints`. It may throw an error.
    * @param {CameraConstraints} constraints
    */
    set Constraints(constraints) {
        _assertClass(constraints, CameraConstraints);
        var ptr0 = constraints.ptr;
        constraints.ptr = 0;
        wasm.jscamera_set_Constraints(this.ptr, ptr0);
    }
    /**
    * Gets the internal [`Resolution`].
    *
    * Note: This value is only updated after you call [`measure_resolution`](crate::js_camera::JSCamera::measure_resolution)
    * # JS-WASM
    * This is exported as `get_Resolution`.
    * @returns {Resolution}
    */
    get Resolution() {
        var ret = wasm.jscamera_Resolution(this.ptr);
        return Resolution.__wrap(ret);
    }
    /**
    * Measures the [`Resolution`] of the internal stream. You usually do not need to call this.
    *
    * # Errors
    * If the camera fails to attach to the created `<video>`, this will error.
    *
    * # JS-WASM
    * This is exported as `measureResolution`. It may throw an error.
    */
    measureResolution() {
        wasm.jscamera_measureResolution(this.ptr);
    }
    /**
    * Applies any modified constraints.
    * # Errors
    * This function may return an error on failing to measure the resolution. Please check [`measure_resolution()`](crate::js_camera::JSCamera::measure_resolution) for details.
    * # JS-WASM
    * This is exported as `applyConstraints`. It may throw an error.
    */
    applyConstraints() {
        wasm.jscamera_applyConstraints(this.ptr);
    }
    /**
    * Gets the internal [`MediaStream`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStream.html) [`MDN`](https://developer.mozilla.org/en-US/docs/Web/API/MediaStream)
    * # JS-WASM
    * This is exported as `MediaStream`.
    * @returns {MediaStream}
    */
    get MediaStream() {
        var ret = wasm.jscamera_MediaStream(this.ptr);
        return takeObject(ret);
    }
    /**
    * Captures an [`ImageData`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.ImageData.html) [`MDN`](https://developer.mozilla.org/en-US/docs/Web/API/ImageData) by drawing the image to a non-existent canvas.
    *
    * # Errors
    * If drawing to the canvas fails this will error.
    * # JS-WASM
    * This is exported as `captureImageData`. It may throw an error.
    * @returns {ImageData}
    */
    captureImageData() {
        var ret = wasm.jscamera_captureImageData(this.ptr);
        return takeObject(ret);
    }
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
    captureImageURI(mime_type, image_quality) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            var ptr0 = passStringToWasm0(mime_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.jscamera_captureImageURI(retptr, this.ptr, ptr0, len0, image_quality);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * Creates an off-screen canvas and a `<video>` element (if not already attached) and returns a raw `Cow<[u8]>` RGBA frame.
    * # Errors
    * If a cast fails, the camera fails to attach, the currently attached node is invalid, or writing/reading from the canvas fails, this will error.
    * # JS-WASM
    * This is exported as `captureFrameRawData`. This may throw an error.
    * @returns {Uint8Array}
    */
    captureFrameRawData() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.jscamera_captureFrameRawData(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
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
    attachToElement(element, generate_new) {
        var ptr0 = passStringToWasm0(element, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.jscamera_attachToElement(this.ptr, ptr0, len0, generate_new);
    }
    /**
    * Detaches the camera from the `<video>` node.
    * # Errors
    * If the casting fails (the stored node is not a `<video>`) this will error.
    * # JS-WASM
    * This is exported as `detachCamera`. This may throw an error.
    */
    detachCamera() {
        wasm.jscamera_detachCamera(this.ptr);
    }
    /**
    * Stops all streams and detaches the camera.
    * # Errors
    * There may be an error while detaching the camera. Please see [`detach()`](crate::js_camera::JSCamera::detach) for more details.
    */
    stopAll() {
        wasm.jscamera_stopAll(this.ptr);
    }
}

export class JSCameraConstraints {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jscameraconstraints_free(ptr);
    }
    /**
    * Gets the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html)
    * # JS-WASM
    * This is exported as `get_MediaStreamConstraints`.
    * @returns {any}
    */
    get MediaStreamConstraints() {
        var ret = wasm.jscameraconstraints_MediaStreamConstraints(this.ptr);
        return takeObject(ret);
    }
    /**
    * Gets the minimum [`Resolution`].
    * # JS-WASM
    * This is exported as `get_MinResolution`.
    * @returns {Resolution | undefined}
    */
    get MinResolution() {
        var ret = wasm.jscameraconstraints_MinResolution(this.ptr);
        return ret === 0 ? undefined : Resolution.__wrap(ret);
    }
    /**
    * Gets the minimum [`Resolution`].
    * # JS-WASM
    * This is exported as `set_MinResolution`.
    * @param {Resolution} min_resolution
    */
    set MinResolution(min_resolution) {
        _assertClass(min_resolution, Resolution);
        var ptr0 = min_resolution.ptr;
        min_resolution.ptr = 0;
        wasm.jscameraconstraints_set_MinResolution(this.ptr, ptr0);
    }
    /**
    * Gets the internal [`Resolution`]
    * # JS-WASM
    * This is exported as `get_Resolution`.
    * @returns {Resolution}
    */
    get Resolution() {
        var ret = wasm.jscameraconstraints_Resolution(this.ptr);
        return Resolution.__wrap(ret);
    }
    /**
    * Sets the internal [`Resolution`]
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_Resolution`.
    * @param {Resolution} preferred_resolution
    */
    set Resolution(preferred_resolution) {
        _assertClass(preferred_resolution, Resolution);
        var ptr0 = preferred_resolution.ptr;
        preferred_resolution.ptr = 0;
        wasm.jscameraconstraints_set_Resolution(this.ptr, ptr0);
    }
    /**
    * Gets the maximum [`Resolution`].
    * # JS-WASM
    * This is exported as `get_MaxResolution`.
    * @returns {Resolution | undefined}
    */
    get MaxResolution() {
        var ret = wasm.jscameraconstraints_MaxResolution(this.ptr);
        return ret === 0 ? undefined : Resolution.__wrap(ret);
    }
    /**
    * Gets the maximum [`Resolution`].
    * # JS-WASM
    * This is exported as `set_MaxResolution`.
    * @param {Resolution} max_resolution
    */
    set MaxResolution(max_resolution) {
        _assertClass(max_resolution, Resolution);
        var ptr0 = max_resolution.ptr;
        max_resolution.ptr = 0;
        wasm.jscameraconstraints_set_MaxResolution(this.ptr, ptr0);
    }
    /**
    * Gets the internal resolution exact.
    * # JS-WASM
    * This is exported as `get_ResolutionExact`.
    * @returns {boolean}
    */
    get ResolutionExact() {
        var ret = wasm.jscameraconstraints_ResolutionExact(this.ptr);
        return ret !== 0;
    }
    /**
    * Sets the internal resolution exact.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_ResolutionExact`.
    * @param {boolean} resolution_exact
    */
    set ResolutionExact(resolution_exact) {
        wasm.jscameraconstraints_set_ResolutionExact(this.ptr, resolution_exact);
    }
    /**
    * Gets the minimum aspect ratio of the [`JSCameraConstraints`].
    * # JS-WASM
    * This is exported as `get_MinAspectRatio`.
    * @returns {number | undefined}
    */
    get MinAspectRatio() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.jscameraconstraints_MinAspectRatio(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getFloat64Memory0()[retptr / 8 + 1];
            return r0 === 0 ? undefined : r1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Sets the minimum aspect ratio of the [`JSCameraConstraints`].
    * # JS-WASM
    * This is exported as `set_MinAspectRatio`.
    * @param {number} min_aspect_ratio
    */
    set MinAspectRatio(min_aspect_ratio) {
        wasm.jscameraconstraints_set_MinAspectRatio(this.ptr, min_aspect_ratio);
    }
    /**
    * Gets the internal aspect ratio.
    * # JS-WASM
    * This is exported as `get_AspectRatio`.
    * @returns {number}
    */
    get AspectRatio() {
        var ret = wasm.jscameraconstraints_AspectRatio(this.ptr);
        return ret;
    }
    /**
    * Sets the internal aspect ratio.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_AspectRatio`.
    * @param {number} aspect_ratio
    */
    set AspectRatio(aspect_ratio) {
        wasm.jscameraconstraints_set_AspectRatio(this.ptr, aspect_ratio);
    }
    /**
    * Gets the maximum aspect ratio.
    * # JS-WASM
    * This is exported as `get_MaxAspectRatio`.
    * @returns {number | undefined}
    */
    get MaxAspectRatio() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.jscameraconstraints_MaxAspectRatio(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getFloat64Memory0()[retptr / 8 + 1];
            return r0 === 0 ? undefined : r1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Sets the maximum internal aspect ratio.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_MaxAspectRatio`.
    * @param {number} max_aspect_ratio
    */
    set MaxAspectRatio(max_aspect_ratio) {
        wasm.jscameraconstraints_set_MaxAspectRatio(this.ptr, max_aspect_ratio);
    }
    /**
    * Gets the internal aspect ratio exact.
    * # JS-WASM
    * This is exported as `get_AspectRatioExact`.
    * @returns {boolean}
    */
    get AspectRatioExact() {
        var ret = wasm.jscameraconstraints_AspectRatioExact(this.ptr);
        return ret !== 0;
    }
    /**
    * Sets the internal aspect ratio exact.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_AspectRatioExact`.
    * @param {boolean} aspect_ratio_exact
    */
    set AspectRatioExact(aspect_ratio_exact) {
        wasm.jscameraconstraints_set_AspectRatioExact(this.ptr, aspect_ratio_exact);
    }
    /**
    * Gets the internal [`JSCameraFacingMode`].
    * # JS-WASM
    * This is exported as `get_FacingMode`.
    * @returns {number}
    */
    get FacingMode() {
        var ret = wasm.jscameraconstraints_FacingMode(this.ptr);
        return ret >>> 0;
    }
    /**
    * Sets the internal [`JSCameraFacingMode`]
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_FacingMode`.
    * @param {number} facing_mode
    */
    set FacingMode(facing_mode) {
        wasm.jscameraconstraints_set_FacingMode(this.ptr, facing_mode);
    }
    /**
    * Gets the internal facing mode exact.
    * # JS-WASM
    * This is exported as `get_FacingModeExact`.
    * @returns {boolean}
    */
    get FacingModeExact() {
        var ret = wasm.jscameraconstraints_FacingModeExact(this.ptr);
        return ret !== 0;
    }
    /**
    * Sets the internal facing mode exact
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_FacingModeExact`.
    * @param {boolean} facing_mode_exact
    */
    set FacingModeExact(facing_mode_exact) {
        wasm.jscameraconstraints_set_FacingModeExact(this.ptr, facing_mode_exact);
    }
    /**
    * Gets the minimum internal frame rate.
    * # JS-WASM
    * This is exported as `get_MinFrameRate`.
    * @returns {number | undefined}
    */
    get MinFrameRate() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.jscameraconstraints_MinFrameRate(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return r0 === 0 ? undefined : r1 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Sets the minimum internal frame rate
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_MinFrameRate`.
    * @param {number} min_frame_rate
    */
    set MinFrameRate(min_frame_rate) {
        wasm.jscameraconstraints_set_MinFrameRate(this.ptr, min_frame_rate);
    }
    /**
    * Gets the internal frame rate.
    * # JS-WASM
    * This is exported as `get_FrameRate`.
    * @returns {number}
    */
    get FrameRate() {
        var ret = wasm.jscameraconstraints_FrameRate(this.ptr);
        return ret >>> 0;
    }
    /**
    * Sets the internal frame rate
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_FrameRate`.
    * @param {number} frame_rate
    */
    set FrameRate(frame_rate) {
        wasm.jscameraconstraints_set_FrameRate(this.ptr, frame_rate);
    }
    /**
    * Gets the maximum internal frame rate.
    * # JS-WASM
    * This is exported as `get_MaxFrameRate`.
    * @returns {number | undefined}
    */
    get MaxFrameRate() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.jscameraconstraints_MaxFrameRate(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return r0 === 0 ? undefined : r1 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * Sets the maximum internal frame rate
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_MaxFrameRate`.
    * @param {number} max_frame_rate
    */
    set MaxFrameRate(max_frame_rate) {
        wasm.jscameraconstraints_set_MaxFrameRate(this.ptr, max_frame_rate);
    }
    /**
    * Gets the internal frame rate exact.
    * # JS-WASM
    * This is exported as `get_FrameRateExact`.
    * @returns {boolean}
    */
    get FrameRateExact() {
        var ret = wasm.jscameraconstraints_FrameRateExact(this.ptr);
        return ret !== 0;
    }
    /**
    * Sets the internal frame rate exact.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_FrameRateExact`.
    * @param {boolean} frame_rate_exact
    */
    set FrameRateExact(frame_rate_exact) {
        wasm.jscameraconstraints_set_FrameRateExact(this.ptr, frame_rate_exact);
    }
    /**
    * Gets the internal [`JSCameraResizeMode`].
    * # JS-WASM
    * This is exported as `get_ResizeMode`.
    * @returns {number}
    */
    get ResizeMode() {
        var ret = wasm.jscameraconstraints_ResizeMode(this.ptr);
        return ret >>> 0;
    }
    /**
    * Sets the internal [`JSCameraResizeMode`]
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_ResizeMode`.
    * @param {number} resize_mode
    */
    set ResizeMode(resize_mode) {
        wasm.jscameraconstraints_set_ResizeMode(this.ptr, resize_mode);
    }
    /**
    * Gets the internal resize mode exact.
    * # JS-WASM
    * This is exported as `get_ResizeModeExact`.
    * @returns {boolean}
    */
    get ResizeModeExact() {
        var ret = wasm.jscameraconstraints_ResizeModeExact(this.ptr);
        return ret !== 0;
    }
    /**
    * Sets the internal resize mode exact.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_ResizeModeExact`.
    * @param {boolean} resize_mode_exact
    */
    set ResizeModeExact(resize_mode_exact) {
        wasm.jscameraconstraints_set_ResizeModeExact(this.ptr, resize_mode_exact);
    }
    /**
    * Gets the internal device id.
    * # JS-WASM
    * This is exported as `get_DeviceId`.
    * @returns {string}
    */
    get DeviceId() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.jscameraconstraints_DeviceId(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * Sets the internal device ID.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_DeviceId`.
    * @param {string} device_id
    */
    set DeviceId(device_id) {
        var ptr0 = passStringToWasm0(device_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.jscameraconstraints_set_DeviceId(this.ptr, ptr0, len0);
    }
    /**
    * Gets the internal device id exact.
    * # JS-WASM
    * This is exported as `get_DeviceIdExact`.
    * @returns {boolean}
    */
    get DeviceIdExact() {
        var ret = wasm.jscameraconstraints_DeviceIdExact(this.ptr);
        return ret !== 0;
    }
    /**
    * Sets the internal device ID exact.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_DeviceIdExact`.
    * @param {boolean} device_id_exact
    */
    set DeviceIdExact(device_id_exact) {
        wasm.jscameraconstraints_set_DeviceIdExact(this.ptr, device_id_exact);
    }
    /**
    * Gets the internal group id.
    * # JS-WASM
    * This is exported as `get_GroupId`.
    * @returns {string}
    */
    get GroupId() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.jscameraconstraints_GroupId(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * Sets the internal group ID.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_GroupId`.
    * @param {string} group_id
    */
    set GroupId(group_id) {
        var ptr0 = passStringToWasm0(group_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.jscameraconstraints_set_GroupId(this.ptr, ptr0, len0);
    }
    /**
    * Gets the internal group id exact.
    * # JS-WASM
    * This is exported as `get_GroupIdExact`.
    * @returns {boolean}
    */
    get GroupIdExact() {
        var ret = wasm.jscameraconstraints_GroupIdExact(this.ptr);
        return ret !== 0;
    }
    /**
    * Sets the internal group ID exact.
    * Note that this doesn't affect the internal [`MediaStreamConstraints`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStreamConstraints.html) until you call
    * [`apply_constraints()`](crate::js_camera::JSCameraConstraints::apply_constraints)
    * # JS-WASM
    * This is exported as `set_GroupIdExact`.
    * @param {boolean} group_id_exact
    */
    set GroupIdExact(group_id_exact) {
        wasm.jscameraconstraints_set_GroupIdExact(this.ptr, group_id_exact);
    }
    /**
    * Applies any modified constraints.
    * # JS-WASM
    * This is exported as `applyConstraints`.
    */
    applyConstraints() {
        wasm.jscameraconstraints_applyConstraints(this.ptr);
    }
}

export class JSCameraConstraintsBuilder {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jscameraconstraintsbuilder_free(ptr);
    }
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
    constructor() {
        var ret = wasm.jscameraconstraintsbuilder_new();
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the minimum resolution for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width) and [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height).
    * # JS-WASM
    * This is exported as `set_MinResolution`.
    * @param {Resolution} min_resolution
    * @returns {CameraConstraintsBuilder}
    */
    set MinResolution(min_resolution) {
        const ptr = this.__destroy_into_raw();
        _assertClass(min_resolution, Resolution);
        var ptr0 = min_resolution.ptr;
        min_resolution.ptr = 0;
        var ret = wasm.jscameraconstraintsbuilder_set_MaxResolution(ptr, ptr0);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the preferred resolution for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width) and [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height).
    * # JS-WASM
    * This is exported as `set_Resolution`.
    * @param {Resolution} new_resolution
    * @returns {CameraConstraintsBuilder}
    */
    set Resolution(new_resolution) {
        const ptr = this.__destroy_into_raw();
        _assertClass(new_resolution, Resolution);
        var ptr0 = new_resolution.ptr;
        new_resolution.ptr = 0;
        var ret = wasm.jscameraconstraintsbuilder_set_Resolution(ptr, ptr0);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the maximum resolution for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width) and [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height).
    * # JS-WASM
    * This is exported as `set_MaxResolution`.
    * @param {Resolution} max_resolution
    * @returns {CameraConstraintsBuilder}
    */
    set MaxResolution(max_resolution) {
        const ptr = this.__destroy_into_raw();
        _assertClass(max_resolution, Resolution);
        var ptr0 = max_resolution.ptr;
        max_resolution.ptr = 0;
        var ret = wasm.jscameraconstraintsbuilder_set_MaxResolution(ptr, ptr0);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets whether the resolution fields ([`width`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/width), [`height`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/height)/[`resolution`](crate::js_camera::JSCameraConstraintsBuilder::resolution))
    * should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    * Note that this will make the builder ignore [`min_resolution`](crate::js_camera::JSCameraConstraintsBuilder::min_resolution) and [`max_resolution`](crate::js_camera::JSCameraConstraintsBuilder::max_resolution).
    * # JS-WASM
    * This is exported as `set_ResolutionExact`.
    * @param {boolean} value
    * @returns {CameraConstraintsBuilder}
    */
    set ResolutionExact(value) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_ResolutionExact(ptr, value);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the minimum aspect ratio of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`aspectRatio`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/aspectRatio).
    * # JS-WASM
    * This is exported as `set_MinAspectRatio`.
    * @param {number} ratio
    * @returns {CameraConstraintsBuilder}
    */
    set MinAspectRatio(ratio) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_MinAspectRatio(ptr, ratio);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the aspect ratio of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`aspectRatio`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/aspectRatio).
    * # JS-WASM
    * This is exported as `set_AspectRatio`.
    * @param {number} ratio
    * @returns {CameraConstraintsBuilder}
    */
    set AspectRatio(ratio) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_AspectRatio(ptr, ratio);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the maximum aspect ratio of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`aspectRatio`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/aspectRatio).
    * # JS-WASM
    * This is exported as `set_MaxAspectRatio`.
    * @param {number} ratio
    * @returns {CameraConstraintsBuilder}
    */
    set MaxAspectRatio(ratio) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_MaxAspectRatio(ptr, ratio);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets whether the [`aspect_ratio`](crate::js_camera::JSCameraConstraintsBuilder::aspect_ratio) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    * Note that this will make the builder ignore [`min_aspect_ratio`](crate::js_camera::JSCameraConstraintsBuilder::min_aspect_ratio) and [`max_aspect_ratio`](crate::js_camera::JSCameraConstraintsBuilder::max_aspect_ratio).
    * # JS-WASM
    * This is exported as `set_AspectRatioExact`.
    * @param {boolean} value
    * @returns {CameraConstraintsBuilder}
    */
    set AspectRatioExact(value) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_AspectRatioExact(ptr, value);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the facing mode of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`facingMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/facingMode).
    * # JS-WASM
    * This is exported as `set_FacingMode`.
    * @param {number} facing_mode
    * @returns {CameraConstraintsBuilder}
    */
    set FacingMode(facing_mode) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_FacingMode(ptr, facing_mode);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets whether the [`facing_mode`](crate::js_camera::JSCameraConstraintsBuilder::facing_mode) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    * # JS-WASM
    * This is exported as `set_FacingModeExact`.
    * @param {boolean} value
    * @returns {CameraConstraintsBuilder}
    */
    set FacingModeExact(value) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_FacingModeExact(ptr, value);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the minimum frame rate of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`frameRate`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/frameRate).
    * # JS-WASM
    * This is exported as `set_MinFrameRate`.
    * @param {number} fps
    * @returns {CameraConstraintsBuilder}
    */
    set MinFrameRate(fps) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_MinFrameRate(ptr, fps);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the frame rate of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`frameRate`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/frameRate).
    * # JS-WASM
    * This is exported as `set_FrameRate`.
    * @param {number} fps
    * @returns {CameraConstraintsBuilder}
    */
    set FrameRate(fps) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_FrameRate(ptr, fps);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the maximum frame rate of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`frameRate`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/frameRate).
    * # JS-WASM
    * This is exported as `set_MaxFrameRate`.
    * @param {number} fps
    * @returns {CameraConstraintsBuilder}
    */
    set MaxFrameRate(fps) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_MaxFrameRate(ptr, fps);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets whether the [`frame_rate`](crate::js_camera::JSCameraConstraintsBuilder::frame_rate) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    * Note that this will make the builder ignore [`min_frame_rate`](crate::js_camera::JSCameraConstraintsBuilder::min_frame_rate) and [`max_frame_rate`](crate::js_camera::JSCameraConstraintsBuilder::max_frame_rate).
    * # JS-WASM
    * This is exported as `set_FrameRateExact`.
    * @param {boolean} value
    * @returns {CameraConstraintsBuilder}
    */
    set FrameRateExact(value) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_FrameRateExact(ptr, value);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the resize mode of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`resizeMode`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#resizemode).
    * # JS-WASM
    * This is exported as `set_ResizeMode`.
    * @param {number} resize_mode
    * @returns {CameraConstraintsBuilder}
    */
    set ResizeMode(resize_mode) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_ResizeMode(ptr, resize_mode);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets whether the [`resize_mode`](crate::js_camera::JSCameraConstraintsBuilder::resize_mode) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    * # JS-WASM
    * This is exported as `set_ResizeModeExact`.
    * @param {boolean} value
    * @returns {CameraConstraintsBuilder}
    */
    set ResizeModeExact(value) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_ResizeModeExact(ptr, value);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the device ID of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`deviceId`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/deviceId).
    * # JS-WASM
    * This is exported as `set_DeviceId`.
    * @param {string} id
    * @returns {CameraConstraintsBuilder}
    */
    set DeviceId(id) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        var ret = wasm.jscameraconstraintsbuilder_set_DeviceId(ptr, ptr0, len0);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets whether the [`device_id`](crate::js_camera::JSCameraConstraintsBuilder::device_id) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    * # JS-WASM
    * This is exported as `set_DeviceIdExact`.
    * @param {boolean} value
    * @returns {CameraConstraintsBuilder}
    */
    set DeviceIdExact(value) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_DeviceIdExact(ptr, value);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets the group ID of the resulting constraint for the [`JSCameraConstraintsBuilder`].
    *
    * Sets [`groupId`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints/groupId).
    * # JS-WASM
    * This is exported as `set_GroupId`.
    * @param {string} id
    * @returns {CameraConstraintsBuilder}
    */
    set GroupId(id) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        var ret = wasm.jscameraconstraintsbuilder_set_GroupId(ptr, ptr0, len0);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Sets whether the [`group_id`](crate::js_camera::JSCameraConstraintsBuilder::group_id) field should use [`exact`](https://developer.mozilla.org/en-US/docs/Web/API/MediaTrackConstraints#constraints).
    * # JS-WASM
    * This is exported as `set_GroupIdExact`.
    * @param {boolean} value
    * @returns {CameraConstraintsBuilder}
    */
    set GroupIdExact(value) {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_set_GroupIdExact(ptr, value);
        return CameraConstraintsBuilder.__wrap(ret);
    }
    /**
    * Builds the [`JSCameraConstraints`]. Wrapper for [`build`](crate::js_camera::JSCameraConstraintsBuilder::build)
    *
    * Fields that use exact are marked `exact`, otherwise are marked with `ideal`. If min-max are involved, they will use `min` and `max` accordingly.
    * # JS-WASM
    * This is exported as `buildCameraConstraints`.
    * @returns {CameraConstraints}
    */
    buildCameraConstraints() {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.jscameraconstraintsbuilder_buildCameraConstraints(ptr);
        return CameraConstraints.__wrap(ret);
    }
}
/**
* A wrapper around a [`MediaStream`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaStream.html)
* # JS-WASM
* This is exported as `NOKCamera`.
*/
export class NOKCamera {

    static __wrap(ptr) {
        const obj = Object.create(NOKCamera.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_nokcamera_free(ptr);
    }
}
/**
* Describes a Resolution.
* This struct consists of a Width and a Height value (x,y). <br>
* Note: the [`Ord`] implementation of this struct is flipped from highest to lowest.
* # JS-WASM
* This is exported as `Resolution`
*/
export class Resolution {

    static __wrap(ptr) {
        const obj = Object.create(Resolution.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_resolution_free(ptr);
    }
    /**
    * @returns {number}
    */
    get width_x() {
        var ret = wasm.__wbg_get_resolution_width_x(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set width_x(arg0) {
        wasm.__wbg_set_resolution_width_x(this.ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get height_y() {
        var ret = wasm.__wbg_get_resolution_height_y(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set height_y(arg0) {
        wasm.__wbg_set_resolution_height_y(this.ptr, arg0);
    }
    /**
    * Create a new resolution from 2 image size coordinates.
    * # JS-WASM
    * This is exported as a constructor for [`Resolution`].
    * @param {number} x
    * @param {number} y
    */
    constructor(x, y) {
        var ret = wasm.resolution_new(x, y);
        return Resolution.__wrap(ret);
    }
    /**
    * Get the width of Resolution
    * # JS-WASM
    * This is exported as `get_Width`.
    * @returns {number}
    */
    get Width() {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.resolution_Width(ptr);
        return ret >>> 0;
    }
    /**
    * Get the height of Resolution
    * # JS-WASM
    * This is exported as `get_Height`.
    * @returns {number}
    */
    get Height() {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.resolution_Height(ptr);
        return ret >>> 0;
    }
    /**
    * Get the x (width) of Resolution
    * @returns {number}
    */
    x() {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.resolution_Width(ptr);
        return ret >>> 0;
    }
    /**
    * Get the y (height) of Resolution
    * @returns {number}
    */
    y() {
        const ptr = this.__destroy_into_raw();
        var ret = wasm.resolution_Height(ptr);
        return ret >>> 0;
    }
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = new URL('nokhwa_bg.wasm', import.meta.url);
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        var ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_nokcamera_new = function(arg0) {
        var ret = NOKCamera.__wrap(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        var ret = arg0;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        var ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = takeObject(arg0).original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        var ret = false;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Window_11e25482011fc506 = function(arg0) {
        var ret = getObject(arg0) instanceof Window;
        return ret;
    };
    imports.wbg.__wbg_document_5aff8cd83ef968f5 = function(arg0) {
        var ret = getObject(arg0).document;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_navigator_5c90643c2a2b6cda = function(arg0) {
        var ret = getObject(arg0).navigator;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_createElement_ac65a6ce60c4812c = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getElementById_b180ea4ada06a837 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_stop_f96817735e68ad3d = function(arg0) {
        getObject(arg0).stop();
    };
    imports.wbg.__wbg_instanceof_MediaDeviceInfo_cdf31c28bb9459ae = function(arg0) {
        var ret = getObject(arg0) instanceof MediaDeviceInfo;
        return ret;
    };
    imports.wbg.__wbg_deviceId_52909f91167fa503 = function(arg0, arg1) {
        var ret = getObject(arg1).deviceId;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_kind_7f46301e232da76a = function(arg0) {
        var ret = getObject(arg0).kind;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_groupId_f6b7aa74f6b78e09 = function(arg0, arg1) {
        var ret = getObject(arg1).groupId;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_enumerateDevices_fe5818b3c5c78822 = function() { return handleError(function (arg0) {
        var ret = getObject(arg0).enumerateDevices();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getSupportedConstraints_fa3e91cb9dd6da4c = function(arg0) {
        var ret = getObject(arg0).getSupportedConstraints();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getUserMedia_c7f57dc542dcd8c6 = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).getUserMedia(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_HtmlCanvasElement_fd3cbbe3906d7792 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLCanvasElement;
        return ret;
    };
    imports.wbg.__wbg_setwidth_f3c88eb520ba8d47 = function(arg0, arg1) {
        getObject(arg0).width = arg1 >>> 0;
    };
    imports.wbg.__wbg_setheight_5a1abba41e35c42a = function(arg0, arg1) {
        getObject(arg0).height = arg1 >>> 0;
    };
    imports.wbg.__wbg_getContext_813df131fcbd6e91 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_toDataURL_fa0138af5cf03aa2 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        var ret = getObject(arg1).toDataURL(getStringFromWasm0(arg2, arg3), getObject(arg4));
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    }, arguments) };
    imports.wbg.__wbg_data_315524ada7b563f4 = function(arg0, arg1) {
        var ret = getObject(arg1).data;
        var ptr0 = passArray8ToWasm0(ret, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_setAttribute_27ca65e30a1c3c4a = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_log_9a99fb1af846153b = function(arg0) {
        console.log(getObject(arg0));
    };
    imports.wbg.__wbg_instanceof_CanvasRenderingContext2d_779e79c4121aa91b = function(arg0) {
        var ret = getObject(arg0) instanceof CanvasRenderingContext2D;
        return ret;
    };
    imports.wbg.__wbg_drawImage_13e48c4e7b9bcf28 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).drawImage(getObject(arg1), arg2, arg3, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_getImageData_b5842f1d6ce40388 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        var ret = getObject(arg0).getImageData(arg1, arg2, arg3, arg4);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_setsrcObject_340bcd93145a797b = function(arg0, arg1) {
        getObject(arg0).srcObject = getObject(arg1);
    };
    imports.wbg.__wbg_mediaDevices_b3973ebf40387065 = function() { return handleError(function (arg0) {
        var ret = getObject(arg0).mediaDevices;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_HtmlVideoElement_acfaf2202e3e0d29 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLVideoElement;
        return ret;
    };
    imports.wbg.__wbg_setwidth_4006f4b33b15224b = function(arg0, arg1) {
        getObject(arg0).width = arg1 >>> 0;
    };
    imports.wbg.__wbg_videoWidth_6bb879ceeecadf27 = function(arg0) {
        var ret = getObject(arg0).videoWidth;
        return ret;
    };
    imports.wbg.__wbg_videoHeight_384ef46e0b174f86 = function(arg0) {
        var ret = getObject(arg0).videoHeight;
        return ret;
    };
    imports.wbg.__wbg_clone_907d18181dd9fdec = function(arg0) {
        var ret = getObject(arg0).clone();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getTracks_453d59960c6b0998 = function(arg0) {
        var ret = getObject(arg0).getTracks();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_appendChild_6ed236bb79c198df = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).appendChild(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_get_73c087db0a496c21 = function(arg0, arg1) {
        var ret = getObject(arg0)[arg1 >>> 0];
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_length_c5fa152b8c3f311f = function(arg0) {
        var ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_new_ec75d0d5815be736 = function() {
        var ret = new Array();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newnoargs_1a11e7e8c906996c = function(arg0, arg1) {
        var ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_193281ce8fd4b1c8 = function() {
        var ret = new Map();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_call_e91f71ddf1f45cff = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_4b48f9f8159fea77 = function() {
        var ret = new Object();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_from_28631399e1e647cb = function(arg0) {
        var ret = Array.from(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_push_0daae9343162dbe7 = function(arg0, arg1) {
        var ret = getObject(arg0).push(getObject(arg1));
        return ret;
    };
    imports.wbg.__wbg_call_e3c72355d091d5d4 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_set_b9dad32fc360b408 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_assign_39a180d12d813399 = function(arg0, arg1) {
        var ret = Object.assign(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_keys_9f3a5511f779059c = function(arg0) {
        var ret = Object.keys(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_119f8177d8717c43 = function(arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = (arg0, arg1) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_215(a, state0.b, arg0, arg1);
                } finally {
                    state0.a = a;
                }
            };
            var ret = new Promise(cb0);
            return addHeapObject(ret);
        } finally {
            state0.a = state0.b = 0;
        }
    };
    imports.wbg.__wbg_resolve_7161ec6fd5b1cd29 = function(arg0) {
        var ret = Promise.resolve(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_then_6d5072fec3fdb237 = function(arg0, arg1) {
        var ret = getObject(arg0).then(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_then_4f3c7f6f3d36634a = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_self_b4546ea7b590539e = function() { return handleError(function () {
        var ret = self.self;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_window_c279fea81f426a68 = function() { return handleError(function () {
        var ret = window.window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_globalThis_038a6ea0ff17789f = function() { return handleError(function () {
        var ret = globalThis.globalThis;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_global_4f93ce884bcee597 = function() { return handleError(function () {
        var ret = global.global;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        var ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbg_set_d29a397c9cc5d746 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        var ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        var ret = debugString(getObject(arg1));
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_rethrow = function(arg0) {
        throw takeObject(arg0);
    };
    imports.wbg.__wbindgen_closure_wrapper737 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 65, __wbg_adapter_22);
        return addHeapObject(ret);
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }



    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;

    return wasm;
}

export default init;

