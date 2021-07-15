#pragma once

#include <cstdint>
#include <Windows.h>
#include <mfapi.h>
#include <mfidl.h>
#include <mfreadwrite.h>
#include <dshow.h>
#pragma comment(lib, "strmiids.lib")
#pragma comment(lib,"Mfplat.lib")
#pragma comment(lib,"Mf.lib")
#pragma comment(lib,"Mfreadwrite.lib")
#pragma comment(lib,"mfuuid.lib")
#pragma comment(lib,"shlwapi.lib")

enum class NOKHWARESULT {
	OK,
	ERR_CANNOT_INIT_MF,
	ERR_CANNOT_QUERY_SYSTEM,
	ERR_CANNOT_READ_DEVICE_NAME,
	ERR_CANNOT_READ_DEVICE_LNK,
	ERR_CANNOT_READ_DEVICE_NATIVE_TYPES,
	ERR_CANNOT_READ_FRAME,
	ERR_CANNOT_READ_CTRLS,
	ERR_CANNNOT_OPEN_DEVICE,
	ERR_CANNOT_SET_CAMERA_SETTING,
	ERR_CANNNOT_OPEN_CAMERA_STREAM,
	ERR_INVALID_FN_FOR_OPTS,
	ERR_CTRL_NOT_SUPPORTED,
	ERR_CANNOT_SET_CTRLS,
	ERR_STREAM_NOT_INIT,
	ERR_STREAM_ERR,
};

enum class NokhwaCameraControls {
	BRIGHTNESS = 0,
	CONTRAST = 1,
	HUE = 2,
	SATURATION = 3,
	SHARPNESS = 4,
	GAMMA = 5,
	WHITE_BALANCE = 6,
	BACKLIGHT_COMP = 7,
	GAIN = 8,
	PAN = 9,
	TILT = 10,
	ROLL = 11,
	ZOOM = 12,
	EXPOSURE = 13,
	IRIS = 14,
	FOCUS = 15,
};

struct NokhwaControlParameters {
	uint32_t paramater;
	int32_t min;
	int32_t max;
	int32_t step;
	int32_t current;
	int32_t n_default;
	int32_t flag;
};

struct NokhwaAllCameraControls {
	NokhwaControlParameters brightness;
	NokhwaControlParameters contrast;
	NokhwaControlParameters hue;
	NokhwaControlParameters saturation;
	NokhwaControlParameters sharpness;
	NokhwaControlParameters gamma;
	NokhwaControlParameters white_balance;
	NokhwaControlParameters backlight_compensation;
	NokhwaControlParameters gain;
	NokhwaControlParameters pan;
	NokhwaControlParameters tilt;
	NokhwaControlParameters roll;
	NokhwaControlParameters zoom;
	NokhwaControlParameters exposure;
	NokhwaControlParameters focus;
};

struct NokhwaCaptureDeviceDefinition {
	size_t index;
	wchar_t* name;
	uint32_t name_len;
	wchar_t* symlink;
	uint32_t symlink_len;
};

struct NokhwaDeviceResolution {
	uint32_t width;
	uint32_t height;
};

enum NokhwaFourCC {
	YUY2,
	MJPG,
};

struct NokhwaDeviceFMT {
	NokhwaDeviceResolution Resolution;
	NokhwaFourCC FourCC;
	uint32_t Framerate;
};


class NokhwaCaptureDevice {
private:
	bool is_open = false;
	size_t index = 0;
	NokhwaCaptureDeviceDefinition device_info = {};
	NokhwaDeviceFMT device_format = {};
	IMFMediaSource* media_source  = NULL;
	IMFSourceReader* source_reader = NULL;

	NOKHWARESULT CreateSuitableIMFMediaType(IMFMediaType** media_type);
public:
	NokhwaCaptureDevice(size_t* index);
	~NokhwaCaptureDevice();

	// INIT MUST BE CALLED
	NOKHWARESULT Init(const NokhwaDeviceFMT* optional_fmt);

	NokhwaCaptureDeviceDefinition DeviceInfo();
	NokhwaDeviceFMT GetDeviceFMT();
	NOKHWARESULT SetDeviceFMT(const NokhwaDeviceFMT* format);

	NOKHWARESULT GetSupportedNativeFormats(NokhwaDeviceFMT* supported_formats, size_t len);
	NOKHWARESULT GetCameraControl(const NokhwaCameraControls* control, NokhwaControlParameters* value);
	NOKHWARESULT SetCameraControl(const NokhwaCameraControls* control, const int32_t* value, const int32_t* flag);


	bool         IsStreamOpen();
	NOKHWARESULT OpenStream();

	NOKHWARESULT RawFrameRead(byte* data, uint64_t* len);

	void CloseStream();
};

NOKHWARESULT NokhwaInitMF();
NOKHWARESULT NokhwaQuerySystemDevices(NokhwaCaptureDeviceDefinition* devices, size_t* len);
void NokhwaShutdownMF();