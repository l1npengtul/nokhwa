# 0.7.0
- Bumped some dependencies.

# 0.5.0
 - Fixed `msmf`
 - Relicensed to Apache-2.0

# 0.4.0
- Added AVFoundation, MSMF, WASM
- `.get_info()` returns a `&CameraInfo`
- Added Threaded Camera
- Added JSCamera
- Changed `new` to use `CaptureAPIBackend::Auto` by default. Old functionally still possible with `with_backend()`
- Added `query()`, which uses `CaptureAPIBackend::Auto` by default.
- Fixed/Added examples

# 0.3.2
- Bumped `ouroboros` to avoid potential UB
- [INTERNAL] Removed `Box<T>` from many internal struct fields of `UVCCaptureDevice`

# 0.3.1
- Added feature hacks to prevent gstreamer/opencv docs.rs build failure

# 0.3.0
- Added `query_devices()` to query available devices on system
- Added `GStreamer` and `OpenCV` backends
- Added `NetworkCamera`
- Added WGPU Texture and raw buffer write support
- Added `capture` example
- Removed `get_` from all APIs. 
- General documentation fixes
- General bugfixes/performance enhancements


# 0.2.0
First release
- UVC/V4L backends
- `Camera` struct for simplification
- `CaptureBackendTrait` to simplify writing backends
