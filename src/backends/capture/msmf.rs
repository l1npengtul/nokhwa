use crate::{CameraFormat, CameraInfo, NokhwaError};
use nokhwa_bindings_windows::{
    Windows::{
        Win32::{
            Media::{
                MediaFoundation::{
                    MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                    IMFActivate,
                    IMFAttributes,
                    MFCreateAttributes,
                    MFEnumDeviceSources,
                    MFStartup,
                    MFSTARTUP_NOSOCKET,
                    MF_API_VERSION,
                    MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                    MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                    MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                }
            },
            Foundation::PWSTR,
            System::Com::CoTaskMemFree,
        }
    }
};
use std::ffi::{c_void, OsStr, OsString};

pub struct MediaFoundationCaptureDevice {}

impl MediaFoundationCaptureDevice {
    pub fn new(index: usize, cam_fmt: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        unsafe {
            MFStartup(MF_API_VERSION, MFSTARTUP_NOSOCKET);
        }

        let mut attributes_opt: Option<IMFAttributes> = None;
        unsafe { MFCreateAttributes(&mut attributes_opt, 1) };

        let attributes = match attributes_opt {
            Some(attr) => {
                if let Err(why) = unsafe {
                    attr.SetGUID(
                        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                    )
                } {
                    return Err(NokhwaError::GeneralError(format!(
                        "Failed to set GUID for MSMF: {}",
                        why.to_string()
                    )));
                }
                attr
            }
            None => {
                return Err(NokhwaError::GeneralError(
                    "Failed to set Attributes for MSMF: {}"(),
                ))
            }
        };

        let mut devices_opt: Option<IMFActivate> = None;
        let mut devices_ptr: *mut Option<IMFActivate> = &mut devices_opt;
        let devices_ptr_d: *mut *mut Option<IMFActivate> = &mut devices_ptr; // WHYYYYYYYYYYYYYYY

        let mut camera_info_vec = vec![];
        let mut count = 0;

        if let Err(why) = unsafe { MFEnumDeviceSources(attributes, devices_ptr_d, &mut count) } {
            return Err(NokhwaError::GeneralError(format!(
                "Failed to query devices: {}",
                why.to_string()
            )));
        }

        if count as usize > index {
            return Err(NokhwaError::CouldntOpenDevice(format!(
                "No device at index {}",
                index
            )));
        }

        match &mut devices_opt {
            Some(devices) => {
                for device_index in 0..count {
                    // name
                    let mut raw_name = PWSTR::default();
                    let mut size_name = 0_u32;
                    unsafe {
                        devices[device_index].GetAllocatedString(
                            &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                            &mut raw_name,
                            &mut size_name,
                        )
                    }

                    let device_name = {
                        let os_str = unsafe {
                            OsString::from_wide_ptr(&raw_name as *const u16, size_name as usize)
                        };
                        os_str.to_string_lossy().to_string()
                    };

                    // desc
                    let mut raw_catagory = PWSTR::default();
                    let mut size_catagory = 0_u32;
                    unsafe {
                        devices.GetAllocatedString(
                            &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                            &mut raw_catagory,
                            &mut size_catagory,
                        )
                    }

                    let device_catagory = {
                        let os_str = unsafe {
                            OsString::from_wide_ptr(
                                &raw_catagory as *const u16,
                                size_catagory as usize,
                            )
                        };
                        os_str.to_string_lossy().to_string()
                    };

                    // symbolic link
                    let mut raw_lnk = PWSTR::default();
                    let mut size_lnk = 0_u32;
                    unsafe {
                        devices.GetAllocatedString(
                            &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                            &mut raw_lnk,
                            &mut size_lnk,
                        )
                    }

                    let device_lnk = {
                        let os_str = unsafe {
                            OsString::from_wide_ptr(&raw_lnk as *const u16, size_lnk as usize)
                        };
                        os_str.to_string_lossy().to_string()
                    };

                    camera_info_vec.push(CameraInfo::new(
                        device_name,
                        device_catagory,
                        device_lnk,
                        device_index as usize,
                    ));

                    unsafe { CoTaskMemFree(&mut (raw_name as c_void)) }
                }
            }
            None => {}
        }

        Err(NokhwaError::GeneralError("".to_string()))
    }
}
