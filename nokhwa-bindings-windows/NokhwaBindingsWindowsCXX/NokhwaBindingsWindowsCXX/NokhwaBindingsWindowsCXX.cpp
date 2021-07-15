
#include <Windows.h>
#include <mfapi.h>
#include <mfidl.h>
#include <mfreadwrite.h>
#include <dshow.h>
#include <atlcomcli.h>
#include "NokhwaBindingsWindowsCXX.h"
#pragma comment(lib, "strmiids.lib")
#pragma comment(lib,"Mfplat.lib")
#pragma comment(lib,"Mf.lib")
#pragma comment(lib,"Mfreadwrite.lib")
#pragma comment(lib,"mfuuid.lib")
#pragma comment(lib,"shlwapi.lib")


HRESULT InitMF() {
	HRESULT hr = CoInitializeEx(0, COINIT_MULTITHREADED);
	if (FAILED(hr)) {
		return hr;
	}

	hr = MFStartup(MF_VERSION, MFSTARTUP_NOSOCKET | COINIT_DISABLE_OLE1DDE);
	if (FAILED(hr)) {
		return hr;
	}

	return hr;
}

NOKHWARESULT NokhwaInitMF() {
	HRESULT hr = InitMF();
	if (FAILED(hr)) {
		goto on_error;
	}

	return NOKHWARESULT::OK;

on_error:
	return NOKHWARESULT::ERR_CANNOT_INIT_MF;
}

void NokhwaShutdownMF() {
	MFShutdown();
	CoUninitialize();
}

NOKHWARESULT NokhwaQuerySystemDevices(NokhwaCaptureDeviceDefinition* devices, size_t* len) {
	IMFAttributes* attributes = NULL;
	IMFActivate** imf_devices = NULL;
	UINT32 count = 0;

	HRESULT hr = MFCreateAttributes(&attributes, 1);
	if (FAILED(hr)) {
		goto on_error;
	}

	hr = attributes->SetGUID(
		MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
		MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID
	);

	if (FAILED(hr)) {
		goto on_error;
	}

	hr = MFEnumDeviceSources(attributes, &imf_devices, &count);
	if (FAILED(hr)) {
		goto on_error;
	}

	for (UINT32 idx = 0; idx < count; idx++) {

		LPWSTR name;
		UINT32 len_name;
		hr = imf_devices[idx]->GetAllocatedString(MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME, &name, &len_name);
		if (FAILED(hr)) {
			CoTaskMemFree(name);
			goto on_error;
		}

		LPWSTR sym_lnk;
		UINT32 len_sym_lnk;
		hr = imf_devices[idx]->GetAllocatedString(MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK, &sym_lnk, &len_sym_lnk);
		if (FAILED(hr)) {
			CoTaskMemFree(sym_lnk);
			goto on_error;
		}

		if (name && sym_lnk) {
			NokhwaCaptureDeviceDefinition device = NokhwaCaptureDeviceDefinition{
				(size_t)idx,
				(wchar_t*)name,
				(uint32_t)len_name,
				(wchar_t*)sym_lnk,
				(uint32_t)len_sym_lnk,
			};
			devices[idx] = device;
		}
		else {
			goto on_error;
		}

	}

	len = (size_t*)count;

	goto clean_up;
	return NOKHWARESULT::OK;

clean_up:
	if (attributes) {
		attributes->Release();
		attributes = NULL;
	}
	SafeRelease(imf_devices);
on_error:
	goto clean_up;
	return NOKHWARESULT::ERR_CANNOT_QUERY_SYSTEM;
}

NOKHWARESULT NokhwaCaptureDevice::CreateSuitableIMFMediaType(IMFMediaType** media_type) {
	IMFMediaType* media_type_set = NULL;

	HRESULT hr = MFCreateMediaType(&media_type_set);
	if (FAILED(hr)) {
		goto on_error;
	}

	hr = media_type_set->SetGUID(MF_MT_MAJOR_TYPE, MFMediaType_Video);
	if (FAILED(hr)) {
		goto on_error;
	}

	switch (this->device_format.FourCC) {
	case NokhwaFourCC::MJPG:
		hr = media_type_set->SetGUID(MF_MT_SUBTYPE, MFVideoFormat_MJPG);
		break;
	case NokhwaFourCC::YUY2:
		hr = media_type_set->SetGUID(MF_MT_SUBTYPE, MFVideoFormat_YUY2);
		break;
	}
	if (FAILED(hr)) {
		goto on_error;
	}

	hr = MFSetAttributeSize(media_type_set, MF_MT_FRAME_SIZE, this->device_format.Resolution.width, this->device_format.Resolution.height);
	if (FAILED(hr)) {
		goto on_error;
	}

	hr = MFSetAttributeRatio(media_type_set, MF_MT_FRAME_RATE, this->device_format.Framerate, 1);
	if (FAILED(hr)) {
		goto on_error;
	}

	hr = this->source_reader->SetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM, NULL, media_type_set);
	if (FAILED(hr)) {
		goto on_error;
	}

	media_type = &media_type_set;
	return NOKHWARESULT::OK;

clean_up:
	if (media_type_set) {
		media_type_set->Release();
		media_type_set = NULL;
	}
on_error:
	goto clean_up;
	return NOKHWARESULT::ERR_CANNOT_SET_CAMERA_SETTING;
}

NokhwaCaptureDevice::NokhwaCaptureDevice(size_t* index) {
	this->index = *index;
}

NokhwaCaptureDevice::~NokhwaCaptureDevice() {
	// delete device info
	delete[] this->device_info.name;
	delete[] this->device_info.symlink;
	delete& this->device_info.index;
	delete& this->device_info.name_len;
	delete& this->device_info.symlink_len;
	delete& this->device_info;

	// delete device format
	delete& this->device_format;

	delete& this->is_open;
	delete& this->index;

	if (this->media_source) {
		this->media_source->Release();
		this->media_source = NULL;
	}

	if (this->source_reader) {
		this->source_reader->Release();
		this->source_reader = NULL;
	}
}

NOKHWARESULT NokhwaCaptureDevice::Init(const NokhwaDeviceFMT* optional_fmt) {
	// make sure we initialized MF
	HRESULT hr = InitMF();
	if (FAILED(hr)) {
		return NOKHWARESULT::ERR_CANNOT_INIT_MF;
	}

	NokhwaDeviceFMT devfmt = NokhwaDeviceFMT{
		NokhwaDeviceResolution {
			640,
			480
		},
		NokhwaFourCC::MJPG,
		15,
	};

	if (optional_fmt != NULL) {
		devfmt = *optional_fmt;
	}

	NokhwaCaptureDeviceDefinition* device_list = NULL;
	size_t device_len = 0;

	NOKHWARESULT nr = NokhwaQuerySystemDevices(device_list, &device_len);
	if (nr != NOKHWARESULT::OK) {
		delete[] device_list;
		goto clean_up;
		return nr;
	}

	if (this->index > device_len) {
		return NOKHWARESULT::ERR_CANNNOT_OPEN_DEVICE;
	}

	{
		IMFAttributes* attributes = NULL;
		HRESULT hr = MFCreateAttributes(&attributes, 2);
		if (FAILED(hr)) {
			if (attributes) {
				attributes->Release();
				attributes = NULL;
			}
			goto on_error;
		}

		hr = attributes->SetGUID(MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID);
		if (FAILED(hr)) {
			if (attributes) {
				attributes->Release();
				attributes = NULL;
			}
			goto on_error;

		}

		hr = attributes->SetString(MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK, device_list[this->index].symlink);
		if (FAILED(hr)) {
			if (attributes) {
				attributes->Release();
				attributes = NULL;
			}
			goto on_error;

		}

		hr = MFCreateDeviceSource(attributes, &this->media_source);
		if (FAILED(hr)) {
			if (attributes) {
				attributes->Release();
				attributes = NULL;
			}
			goto on_error;
		}

		if (attributes) {
			attributes->Release();
			attributes = NULL;
		}
	}

	hr = MFCreateSourceReaderFromMediaSource(this->media_source, NULL, &this->source_reader);
	if (FAILED(hr)) {
		goto on_error;
	}

	this->device_info = device_list[this->index];

	return NOKHWARESULT::OK;

clean_up:
	delete& devfmt;
	delete[] device_list;
	delete& device_len;
on_error:
	goto clean_up;
	return NOKHWARESULT::ERR_CANNNOT_OPEN_DEVICE;
}

NokhwaCaptureDeviceDefinition NokhwaCaptureDevice::DeviceInfo() {
	this->device_info;
}


NokhwaDeviceFMT NokhwaCaptureDevice::GetDeviceFMT() {
	this->device_format;
}

NOKHWARESULT NokhwaCaptureDevice::SetDeviceFMT(const NokhwaDeviceFMT* format) {
	NokhwaDeviceFMT previous = this->device_format;
	this->device_format = *format;

	IMFMediaType* media_type = NULL;

	HRESULT hr = S_OK;

	NOKHWARESULT nr = this->CreateSuitableIMFMediaType(&media_type);
	if (nr != NOKHWARESULT::OK) {
		goto on_error;
	}

	hr = this->source_reader->SetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM, NULL, media_type);
	if (FAILED(hr)) {
		goto on_error;
	}

	return NOKHWARESULT::OK;

clean_up:
	if (media_type) {
		media_type->Release();
		media_type = NULL;
	}
	delete& previous;
on_error:
	this->device_format = previous;
	goto clean_up;
	return NOKHWARESULT::ERR_CANNOT_SET_CAMERA_SETTING;
}

NOKHWARESULT NokhwaCaptureDevice::GetSupportedNativeFormats(NokhwaDeviceFMT* supported_formats, size_t len) {
	HRESULT hr = S_OK;
	size_t index = 0;
	IMFMediaType* media_type = NULL;
	GUID fourcc { 0 };
	while (hr == S_OK) {
		UINT32 height = 0;
		UINT32 width = 0;

		UINT32 frame_rate_1 = 0;
		UINT32 denominator_1 = 0;

		UINT32 frame_rate_min = 0;
		UINT32 denominator_min = 0;

		UINT32 frame_rate_max = 0;
		UINT32 denominator_max = 0;

		hr = this->source_reader->GetNativeMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM, index, &media_type);
		if (FAILED(hr)) {
			break;
		}

		hr = media_type->GetGUID(MF_MT_SUBTYPE, &fourcc);
		if (FAILED(hr)) {
			break;
		}
		hr = MFGetAttributeSize(media_type, MF_MT_FRAME_SIZE, &width, &height);
		if (FAILED(hr)) {
			break;
		}
		hr = MFGetAttributeRatio(media_type, MF_MT_FRAME_RATE, &frame_rate_1, &denominator_1);
		if (FAILED(hr)) {
			break;
		}
		hr = MFGetAttributeRatio(media_type, MF_MT_FRAME_RATE_RANGE_MIN, &frame_rate_min, &denominator_min);
		if (FAILED(hr)) {
			break;
		}
		hr = MFGetAttributeRatio(media_type, MF_MT_FRAME_RATE_RANGE_MAX, &frame_rate_max, &denominator_max);
		if (FAILED(hr)) {
			break;
		}

		if (fourcc == MFVideoFormat_YUY2) {
			if (denominator_min == 1) {
				NokhwaDeviceFMT device_format = NokhwaDeviceFMT{
					NokhwaDeviceResolution {
						(uint32_t)height,
						(uint32_t)width,
					},
					NokhwaFourCC::YUY2,
					(uint32_t)frame_rate_min,
				};
				supported_formats[index] = device_format;
			}
			if (denominator_1 == 1 && frame_rate_1 != frame_rate_min) {
				NokhwaDeviceFMT device_format = NokhwaDeviceFMT{
					NokhwaDeviceResolution {
						(uint32_t)height,
						(uint32_t)width,
					},
					NokhwaFourCC::YUY2,
					(uint32_t)frame_rate_1,
				};
				supported_formats[index+1] = device_format;
			}
			if (denominator_max == 1 && frame_rate_max != frame_rate_1 && frame_rate_max != frame_rate_min) {
				NokhwaDeviceFMT device_format = NokhwaDeviceFMT{
					NokhwaDeviceResolution {
						(uint32_t)height,
						(uint32_t)width,
					},
					NokhwaFourCC::YUY2,
					(uint32_t)frame_rate_max,
				};
				supported_formats[index+2] = device_format;
			}
		}
		else if (fourcc == MFVideoFormat_MJPG) {
			if (denominator_min == 1) {
				NokhwaDeviceFMT device_format = NokhwaDeviceFMT{
					NokhwaDeviceResolution {
						(uint32_t)height,
						(uint32_t)width,
					},
					NokhwaFourCC::MJPG,
					(uint32_t)frame_rate_min,
				};
				supported_formats[index] = device_format;
			}
			if (denominator_1 == 1 && frame_rate_1 != frame_rate_min) {
				NokhwaDeviceFMT device_format = NokhwaDeviceFMT{
					NokhwaDeviceResolution {
						(uint32_t)height,
						(uint32_t)width,
					},
					NokhwaFourCC::MJPG,
					(uint32_t)frame_rate_1,
				};
				supported_formats[index+1] = device_format;
			}
			if (denominator_max == 1 && frame_rate_max != frame_rate_1 && frame_rate_max != frame_rate_min) {
				NokhwaDeviceFMT device_format = NokhwaDeviceFMT{
					NokhwaDeviceResolution {
						(uint32_t)height,
						(uint32_t)width,
					},
					NokhwaFourCC::MJPG,
					(uint32_t)frame_rate_max,
				};
				supported_formats[index+2] = device_format;
			}
		}
	}

	index++;

	if (media_type) {
		media_type->Release();
		media_type = NULL;
	}

	return NOKHWARESULT::ERR_CANNOT_READ_DEVICE_NATIVE_TYPES;
}

NOKHWARESULT NokhwaCaptureDevice::GetCameraControl(const NokhwaCameraControls* control, NokhwaControlParameters* value) {
	IAMCameraControl* camera_control = NULL;
	IAMVideoProcAmp* camera_procv = NULL;
	long min, max, step, def, ctrl, current;


	HRESULT hr = this->media_source->QueryInterface(IID_PPV_ARGS(&camera_control));
	if (FAILED(hr)) {
		goto on_error;
	}
	hr = this->media_source->QueryInterface(IID_PPV_ARGS(&camera_procv));
	if (FAILED(hr)) {
		goto on_error;
	}
	if (camera_control && camera_procv) {

		NokhwaControlParameters parameters = { 0,0,0,0,0,0 };

		switch (*control) {
			case NokhwaCameraControls::BRIGHTNESS:
				hr = camera_procv->GetRange(VideoProcAmp_Brightness, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_procv->Get(VideoProcAmp_Brightness, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::CONTRAST:
				hr = camera_procv->GetRange(VideoProcAmp_Contrast, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_procv->Get(VideoProcAmp_Contrast, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::HUE:
				hr = camera_procv->GetRange(VideoProcAmp_Hue, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_procv->Get(VideoProcAmp_Hue, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::SATURATION:
				hr = camera_procv->GetRange(VideoProcAmp_Saturation, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_procv->Get(VideoProcAmp_Saturation, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::SHARPNESS:
				hr = camera_procv->GetRange(VideoProcAmp_Sharpness, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_procv->Get(VideoProcAmp_Sharpness, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::GAMMA:
				hr = camera_procv->GetRange(VideoProcAmp_Gamma, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_procv->Get(VideoProcAmp_Gamma, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::WHITE_BALANCE:
				hr = camera_procv->GetRange(VideoProcAmp_WhiteBalance, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_procv->Get(VideoProcAmp_WhiteBalance, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::BACKLIGHT_COMP:
				hr = camera_procv->GetRange(VideoProcAmp_BacklightCompensation, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_procv->Get(VideoProcAmp_BacklightCompensation, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::GAIN:
				hr = camera_procv->GetRange(VideoProcAmp_Gain, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_procv->Get(VideoProcAmp_Gain, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::PAN:
				hr = camera_control->GetRange(CameraControl_Pan, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_control->Get(CameraControl_Pan, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::TILT:
				hr = camera_control->GetRange(CameraControl_Tilt, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_control->Get(CameraControl_Tilt, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::ROLL:
				hr = camera_control->GetRange(CameraControl_Roll, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_control->Get(CameraControl_Roll, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::ZOOM:
				hr = camera_control->GetRange(CameraControl_Zoom, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_control->Get(CameraControl_Zoom, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::EXPOSURE:
				hr = camera_control->GetRange(CameraControl_Exposure, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_control->Get(CameraControl_Exposure, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::IRIS:
				hr = camera_control->GetRange(CameraControl_Iris, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_control->Get(CameraControl_Iris, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
			case NokhwaCameraControls::FOCUS:
				hr = camera_control->GetRange(CameraControl_Focus, &min, &max, &step, &def, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				hr = camera_control->Get(CameraControl_Focus, &current, &ctrl);
				if (FAILED(hr)) {
					goto on_error;
				}
				break;
		}

		NokhwaControlParameters nk_ctrl = NokhwaControlParameters{
			(uint32_t)*control,
			(int32_t)min,
			(int32_t)max,
			(int32_t)step,
			(int32_t)current, // CURRENT
			(int32_t)def,
			(int32_t)ctrl,
		};
		value = &nk_ctrl;

		goto clean_up;
		return NOKHWARESULT::OK;
	}

	goto clean_up;
	return NOKHWARESULT::ERR_CANNOT_READ_CTRLS;

clean_up:
	if (camera_control) {
		camera_control->Release();
		camera_control = NULL;
	}
	if (camera_procv) {
		camera_procv->Release();
		camera_procv = NULL;
	}
on_error:
	goto clean_up;
	return NOKHWARESULT::ERR_CANNOT_READ_CTRLS;
}

NOKHWARESULT NokhwaCaptureDevice::SetCameraControl(const NokhwaCameraControls* control, const int32_t* value, const int32_t* flag) {
	IAMCameraControl* camera_control = NULL;
	IAMVideoProcAmp* camera_procv = NULL;
	long min, max, step, def, ctrl, current;

	NOKHWARESULT nr = NOKHWARESULT::OK;
	HRESULT hr = this->media_source->QueryInterface(IID_PPV_ARGS(&camera_control));
	if (FAILED(hr)) {
		goto on_error;
	}
	hr = this->media_source->QueryInterface(IID_PPV_ARGS(&camera_procv));
	if (FAILED(hr)) {
		goto on_error;
	}

	if (camera_control && camera_procv) {
		NokhwaControlParameters base_param = NokhwaControlParameters{ };
		nr = this->GetCameraControl(control, &base_param);
		if (nr != NOKHWARESULT::OK) {
			goto clean_up;
			return nr;
		}
		

		switch (*control) {
		case NokhwaCameraControls::BRIGHTNESS:
			hr = camera_procv->Set(VideoProcAmp_Brightness, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::CONTRAST:
			hr = camera_procv->Set(VideoProcAmp_Contrast, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::HUE:
			hr = camera_procv->Set(VideoProcAmp_Hue, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::SATURATION:
			hr = camera_procv->Set(VideoProcAmp_Saturation, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::SHARPNESS:
			hr = camera_procv->Set(VideoProcAmp_Sharpness, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::GAMMA:
			hr = camera_procv->Set(VideoProcAmp_Gamma, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::WHITE_BALANCE:
			hr = camera_procv->Set(VideoProcAmp_WhiteBalance, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::BACKLIGHT_COMP:
			hr = camera_procv->Set(VideoProcAmp_BacklightCompensation, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::GAIN:
			hr = camera_procv->Set(VideoProcAmp_Gain, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::PAN:
			hr = camera_control->Set(CameraControl_Pan, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::TILT:
			hr = camera_control->Set(CameraControl_Tilt , (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::ROLL:
			hr = camera_control->Set(CameraControl_Roll, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::ZOOM:
			hr = camera_control->Set(CameraControl_Zoom, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::EXPOSURE:
			hr = camera_control->Set(CameraControl_Exposure, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::IRIS:
			hr = camera_control->Set(CameraControl_Iris, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		case NokhwaCameraControls::FOCUS:
			hr = camera_control->Set(CameraControl_Focus, (long)*value, (long)*flag);
			if (FAILED(hr)) {
				goto on_error;
			}
			break;
		}

		goto clean_up;
		return NOKHWARESULT::OK;
	}

	goto clean_up;
	return NOKHWARESULT::ERR_CANNOT_READ_CTRLS;

clean_up:
	if (camera_control) {
		camera_control->Release();
		camera_control = NULL;
	}
	if (camera_procv) {
		camera_procv->Release();
		camera_procv = NULL;
	}
on_error:
	goto clean_up;
	return NOKHWARESULT::ERR_CANNOT_SET_CTRLS;
}


bool NokhwaCaptureDevice::IsStreamOpen() {
	return this->is_open;
}

NOKHWARESULT NokhwaCaptureDevice::OpenStream() {
	if (!this->media_source || !this->source_reader) {
		return Init(&this->device_format);
	}

	NOKHWARESULT nr = this->SetDeviceFMT(&this->device_format);
	if (nr != NOKHWARESULT::OK) {
		return nr;
	}
	else {
		this->is_open = true;
		return NOKHWARESULT::OK;
	}
}

NOKHWARESULT NokhwaCaptureDevice::RawFrameRead(byte* data, uint64_t* len) {
	if (!this->is_open) {
		return NOKHWARESULT::ERR_STREAM_NOT_INIT;
	}

	IMFSample* sample = NULL;
	DWORD flags;
	IMFMediaBuffer* buffer = NULL;
	DWORD length;
	uint64_t temp;

	HRESULT hr = this->source_reader->ReadSample(MF_SOURCE_READER_FIRST_VIDEO_STREAM, 0, NULL, &flags, NULL, &sample);
	if (FAILED(hr) || flags == MF_SOURCE_READERF_ERROR) {
		this->CloseStream();

		if (sample) {
			sample->Release();
			sample = NULL;
		}

		return NOKHWARESULT::ERR_STREAM_ERR;
	}

	hr = sample->ConvertToContiguousBuffer(&buffer);
	if (FAILED(hr)) {
		goto on_error;
	}

	hr = buffer->Lock(&data, NULL, &length);
	if (FAILED(hr)) {
		goto on_error;
	}

	hr = buffer->Unlock();
	if (FAILED(hr)) {
		goto on_error;
	}

	temp = (uint64_t)length;
	len = &temp;

	goto clean_up;
	return NOKHWARESULT::OK;

clean_up:
	if (sample) {
		sample->Release();
		sample = NULL;
	}
	
	if (buffer) {
		buffer->Release();
		buffer = NULL;
	}

	delete& length;
	delete& flags;
	delete& temp;
on_error:
	goto clean_up;
	return NOKHWARESULT::ERR_CANNOT_READ_FRAME;
}

void NokhwaCaptureDevice::CloseStream() {
	if (this->is_open) {
		if (this->media_source) {
			this->media_source->Shutdown(); // Shutdown always returns S_OK.
			this->media_source->Release();
			this->media_source = NULL;
		}

		if (this->source_reader) {
			this->source_reader->Release();
			this->source_reader = NULL;
		}
	}
}

template <class T> void SafeRelease(T** ppT)
{
	if (*ppT)
	{
		(*ppT)->Release();
		*ppT = NULL;
	}
}
