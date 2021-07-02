# Examples 

## capture
Capture is a command line application designed to test features of backends to see if they are implemented correctly. 
For the UVC backend, you may need to run the app as admin. 

### Capture - Usage
`<>` indicates an optional parameter. `[]` indicates a mandatory one.
- `-q <BACKEND>`/`--query <BACKEND>`: Queries the system for available devices. If `<BACKEND>` is set, it will query using that backend.
- `-c <DEVICE>`/`--capture <DEVICE>`: Captures using device. If `<DEVICE>` is set, it will using that device index or IP.
- `-s`/`--query-device`: Show device queries from `compatible_fourcc` and `compatible_list_by_resolution`. Requires -c to be passed to work.
- `-w [WIDTH:U32]`/`--width [WIDTH:U32]`: Set width of capture to `[WIDTH:U32]`. Does nothing if -c flag is not set. Value Has to be a `u32`
- `-h [HEIGHT:U32]`/`--height [HEIGHT:U32]`: Set height of capture to `[HEIGHT:U32]`. Does nothing if -c flag is not set. Value Has to be a `u32`
- `-rate [FPS:U32]`/`--framerate [FPS:U32]`: Set FPS of capture to `[FPS:U32]`. Does nothing if -c flag is not set. Value Has to be a `u32`
- `-4cc [FORMAT]`/`--format [FORMAT]`: Set format of capture to `[FORMAT]`. Does nothing if -c flag is not set. Possible values are MJPG and YUYV. Will be ignored if not one of those two.
- `-b [BACKEND]`/`--backend [BACKEND]`: Set the capture backend to `[BACKEND]`. Pass AUTO for automatic backend, UVC to use UVC, V4L to use Video4Linux, GST to use Gstreamer, OPENCV to use OpenCV.
- `-d`/`--display`: Enable glium display. Note: This is currently bugged as it shows an upside down feed. It also does not respond to `x` button press from window. (FIXME)

Example Usage: 
```
./capture -q V4L -c 0 -s -w 1920 -h 1080 --framerate 30 --format MJPG -b V4L -d
```
Query system using V4L backend, capture device with index 0 at 1920x1080 @ 30 FPS on MJPG format, using backend V4L, then display to glium window. 
