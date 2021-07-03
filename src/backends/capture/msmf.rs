use nokhwa_bindings_windows::Windows::Win32::{
    Foundation::PWSTR,
    Media::MediaFoundation::{
        IMFActivate, IMFAttributes, MFCreateAttributes, MFEnumDeviceSources, MFStartup,
        MFSTARTUP_NOSOCKET, MF_API_VERSION, MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
        MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
    },
    System::Com::CoTaskMemFree,
};
use std::ffi::{c_void, OsStr, OsString};

pub struct MediaFoundationCaptureDevice {}

impl MediaFoundationCaptureDevice {
    
}
