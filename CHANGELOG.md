# 0.3.2
- Updated Ouroboros to 0.10.0 to fix possible UB
- Remove `Box<T>` from many places in the UVC Backend

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
