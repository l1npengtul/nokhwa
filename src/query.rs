/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{CameraInfo, CaptureAPIBackend, NokhwaError};

// TODO: Update as this goes
/// Query the system for a list of available devices. Please refer to the API Backends that support `Query`) <br>
/// Currently, these are V4L, UVC, and GST. <br>
/// Usually the order goes Native -> UVC -> Gstreamer.
/// # Errors
/// If you use an unsupported API (check the README or crate root for more info), incompatible backend for current platform, incompatible platform, or insufficient permissions, etc
/// this will error.
#[allow(clippy::module_name_repetitions)]
pub fn query_devices(api: CaptureAPIBackend) -> Result<Vec<CameraInfo>, NokhwaError> {
    match api {
        CaptureAPIBackend::Auto => {
            // determine platform
            match std::env::consts::OS {
                "linux" => {
                    if cfg!(feature = "input-v4l") {
                        query_devices(CaptureAPIBackend::Video4Linux)
                    } else if cfg!(feature = "input-uvc") {
                        query_devices(CaptureAPIBackend::UniversalVideoClass)
                    } else if cfg!(feature = "input-gstreamer") {
                        query_devices(CaptureAPIBackend::GStreamer)
                    } else {
                        Err(NokhwaError::UnsupportedOperationError(
                            CaptureAPIBackend::Auto,
                        ))
                    }
                }
                "windows" => {
                    if cfg!(feature = "input-msmf") {
                        query_devices(CaptureAPIBackend::MediaFoundation)
                    } else if cfg!(feature = "input-uvc") {
                        query_devices(CaptureAPIBackend::UniversalVideoClass)
                    } else if cfg!(feature = "input-gstreamer") {
                        query_devices(CaptureAPIBackend::GStreamer)
                    } else {
                        Err(NokhwaError::UnsupportedOperationError(
                            CaptureAPIBackend::Auto,
                        ))
                    }
                }
                "macos" => {
                    if cfg!(feature = "input-avfoundation") {
                        query_devices(CaptureAPIBackend::AVFoundation)
                    } else if cfg!(feature = "input-uvc") {
                        query_devices(CaptureAPIBackend::UniversalVideoClass)
                    } else if cfg!(feature = "input-gstreamer") {
                        query_devices(CaptureAPIBackend::GStreamer)
                    } else {
                        Err(NokhwaError::UnsupportedOperationError(
                            CaptureAPIBackend::Auto,
                        ))
                    }
                }
                _ => Err(NokhwaError::UnsupportedOperationError(
                    CaptureAPIBackend::Auto,
                )),
            }
        }
        CaptureAPIBackend::Video4Linux => query_v4l(),
        CaptureAPIBackend::UniversalVideoClass => query_uvc(),
        CaptureAPIBackend::MediaFoundation => query_msmf(),
        CaptureAPIBackend::GStreamer => query_gstreamer(),
        _ => Err(NokhwaError::UnsupportedOperationError(api)),
    }
}

// TODO: More

#[cfg(feature = "input-v4l")]
#[allow(clippy::unnecessary_wraps)]
fn query_v4l() -> Result<Vec<CameraInfo>, NokhwaError> {
    Ok({
        let camera_info: Vec<CameraInfo> = v4l::context::enum_devices()
            .iter()
            .map(|node| {
                CameraInfo::new(
                    node.name()
                        .unwrap_or(format!("{}", node.path().to_string_lossy())),
                    format!("Video4Linux Device @ {}", node.path().to_string_lossy()),
                    "".to_string(),
                    node.index(),
                )
            })
            .collect();
        camera_info
    })
}

#[cfg(not(feature = "input-v4l"))]
fn query_v4l() -> Result<Vec<CameraInfo>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::Video4Linux,
    ))
}

#[cfg(feature = "input-uvc")]
fn query_uvc() -> Result<Vec<CameraInfo>, NokhwaError> {
    use uvc::Device;
    let context = match uvc::Context::new() {
        Ok(ctx) => ctx,
        Err(why) => {
            return Err(NokhwaError::GeneralError(format!(
                "UVC Context failure: {}",
                why.to_string()
            )))
        }
    };

    let usb_devices = usb_enumeration::enumerate(None, None);
    let uvc_devices = match context.devices() {
        Ok(devs) => {
            let device_vec: Vec<Device> = devs.collect();
            device_vec
        }
        Err(why) => {
            return Err(NokhwaError::GeneralError(format!(
                "UVC Context Devicelist failure: {}",
                why.to_string()
            )))
        }
    };

    let mut camera_info_vec = vec![];
    let mut counter = 0_usize;

    // Optimize this O(n*m) algorithm
    for usb_dev in &usb_devices {
        for uvc_dev in &uvc_devices {
            if let Ok(desc) = uvc_dev.description() {
                if desc.product_id == usb_dev.product_id && desc.vendor_id == usb_dev.vendor_id {
                    let name = usb_dev
                        .description
                        .as_ref()
                        .unwrap_or(&format!(
                            "{}:{} {} {}",
                            desc.vendor_id,
                            desc.product_id,
                            desc.manufacturer.unwrap_or_else(|| "Generic".to_string()),
                            desc.product.unwrap_or_else(|| "Camera".to_string())
                        ))
                        .clone();

                    camera_info_vec.push(CameraInfo::new(
                        name.clone(),
                        usb_dev
                            .description
                            .as_ref()
                            .unwrap_or(&"".to_string())
                            .clone(),
                        format!(
                            "{}:{} {}",
                            desc.vendor_id,
                            desc.product_id,
                            desc.serial_number.unwrap_or_else(|| "".to_string())
                        ),
                        counter,
                    ));
                    counter += 1;
                }
            }
        }
    }
    Ok(camera_info_vec)
}

#[cfg(not(feature = "input-uvc"))]
fn query_uvc() -> Result<Vec<CameraInfo>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::UniversalVideoClass,
    ))
}

#[cfg(feature = "input-gst")]
fn query_gstreamer() -> Result<Vec<CameraInfo>, NokhwaError> {
    use gstreamer::{
        prelude::{DeviceExt, DeviceMonitorExt, DeviceMonitorExtManual},
        Caps, DeviceMonitor,
    };
    use std::str::FromStr;
    if let Err(why) = gstreamer::init() {
        return Err(NokhwaError::GeneralError(format!(
            "Failed to init gstreamer: {}",
            why
        )));
    }
    let device_monitor = DeviceMonitor::new();
    let video_caps = match Caps::from_str("video/x-raw") {
        Ok(cap) => cap,
        Err(why) => {
            return Err(NokhwaError::GeneralError(format!(
                "Failed to generate caps: {}",
                why.to_string()
            )))
        }
    };
    let _video_filter_id = match device_monitor.add_filter(Some("Video/Source"), Some(&video_caps))
    {
        Some(id) => id,
        None => {
            return Err(NokhwaError::StructureError {
                structure: "Video Filter ID Video/Source".to_string(),
                error: "Null".to_string(),
            })
        }
    };
    if let Err(why) = device_monitor.start() {
        return Err(NokhwaError::GeneralError(format!(
            "Failed to start device monitor: {}",
            why.to_string()
        )));
    }
    let mut counter = 0;
    let devices: Vec<CameraInfo> = device_monitor
        .devices()
        .iter_mut()
        .map(|gst_dev| {
            let name = DeviceExt::display_name(gst_dev);
            let class = DeviceExt::device_class(gst_dev);
            counter += 1;
            CameraInfo::new(
                name.to_string(),
                class.to_string(),
                "".to_string(),
                counter - 1,
            )
        })
        .collect();
    device_monitor.stop();
    Ok(devices)
}

#[cfg(not(feature = "input-gst"))]
fn query_gstreamer() -> Result<Vec<CameraInfo>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::GStreamer,
    ))
}

// please refer to https://docs.microsoft.com/en-us/windows/win32/medfound/enumerating-video-capture-devices
#[cfg(feature = "input-msmf")]
fn query_msmf() -> Result<Vec<CameraInfo>, NokhwaError> {
    use nokhwa_bindings_windows::{BindingError, MediaFoundationDeviceDescriptor};

    let list: Vec<CameraInfo> = match nokhwa_bindings_windows::wmf::query_msmf() {
        Ok(l) => l
            .into_iter()
            .map(|mf_desc| {
                let camera_info: CameraInfo = mf_desc.into();
                camera_info
            })
            .collect(),
        Err(why) => return Err(why.into()),
    };
    Ok(list)
}

#[cfg(not(feature = "input-msmf"))]
fn query_msmf() -> Result<Vec<CameraInfo>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::MediaFoundation,
    ))
}
