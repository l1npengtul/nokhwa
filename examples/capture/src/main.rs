use clap::{App, Arg};
use glium::{
    implement_vertex, index::PrimitiveType, program, texture::RawImage2d, uniform, Display,
    IndexBuffer, Surface, Texture2d, VertexBuffer,
};
use glutin::{event_loop::EventLoop, window::WindowBuilder, ContextBuilder};
use nokhwa::{query_devices, Camera, CaptureAPIBackend, FrameFormat, NetworkCamera};
use std::time::Instant;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

fn main() {
    let matches = App::new("nokhwa-example")
        .version("0.1.0")
        .author("l1npengtul <l1npengtul@protonmail.com> and the Nokhwa Contributers")
        .about("Example program using Nokhwa")
        .arg(Arg::with_name("query")
            .short("q")
            .long("query")
            .value_name("BACKEND")
            // TODO: Update as new backends are added!
            .help("Query the system? Pass AUTO for automatic backend, UVC to query using UVC, V4L to query using Video4Linux, GST to query using Gstreamer.. Will post the list of availible devices.")
            .default_value("AUTO")
            .takes_value(true))
        .arg(Arg::with_name("capture")
            .short("c")
            .long("capture")
            .value_name("LOCATION")
            .help("Capture from device? Pass the device index or string. Defaults to 0. If the input is not a number, it will be assumed an IPCamera")
            .default_value("0")
            .takes_value(true))
        .arg(Arg::with_name("query-device")
            .short("s")
            .long("query-device")
            .help("Show device queries from `compatible_fourcc` and `compatible_list_by_resolution`. Requires -c to be passed to work.")
            .takes_value(false))
        .arg(Arg::with_name("width")
            .short("w")
            .long("width")
            .value_name("WIDTH")
            .help("Set width of capture. Does nothing if -c flag is not set.")
            .default_value("640")
            .takes_value(true))
        .arg(Arg::with_name("height")
            .short("h")
            .long("height")
            .value_name("HEIGHT")
            .help("Set height of capture. Does nothing if -c flag is not set.")
            .default_value("480")
            .takes_value(true))
        .arg(Arg::with_name("framerate")
            .short("rate")
            .long("framerate")
            .value_name("FRAMES_PER_SECOND")
            .help("Set FPS of capture. Does nothing if -c flag is not set.")
            .default_value("15")
            .takes_value(true))
        .arg(Arg::with_name("format")
            .short("4cc")
            .long("format")
            .value_name("FORMAT")
            .help("Set format of capture. Does nothing if -c flag is not set. Possible values are MJPG and YUYV. Will be ignored if not either. Ignored by GStreamer backend.")
            .default_value("MJPG")
            .takes_value(true))
        .arg(Arg::with_name("capture-backend")
            .short("b")
            .long("backend")
            .value_name("BACKEND")
            .help("Set the capture backend. Pass AUTO for automatic backend, UVC to query using UVC, V4L to query using Video4Linux, GST to query using Gstreamer, OPENCV to use OpenCV.")
            .default_value("AUTO")
            .takes_value(true))
        .arg(Arg::with_name("display")
            .short("d")
            .long("display")
            .help("Pass to open a window and display.")
            .takes_value(false)).get_matches();

    // Query example
    if matches.is_present("query") {
        let backend_value = matches.value_of("query").unwrap();
        let mut use_backend = CaptureAPIBackend::Auto;
        // AUTO
        if backend_value == "AUTO" {
            use_backend = CaptureAPIBackend::Auto;
        } else if backend_value == "UVC" {
            use_backend = CaptureAPIBackend::UniversalVideoClass;
        } else if backend_value == "GST" {
            use_backend = CaptureAPIBackend::GStreamer;
        } else if backend_value == "V4L" {
            use_backend = CaptureAPIBackend::Video4Linux;
        }

        match query_devices(use_backend) {
            Ok(devs) => {
                for (idx, camera) in devs.iter().enumerate() {
                    println!("Device at index {}: {}", idx, camera)
                }
            }
            Err(why) => {
                println!("Failed to query: {}", why.to_string())
            }
        }
    }

    if matches.is_present("capture") {
        let backend_value = {
            match matches.value_of("capture-backend").unwrap() {
                "UVC" => CaptureAPIBackend::UniversalVideoClass,
                "GST" => CaptureAPIBackend::GStreamer,
                "V4L" => CaptureAPIBackend::Video4Linux,
                "OPENCV" => CaptureAPIBackend::OpenCv,
                _ => CaptureAPIBackend::Auto,
            }
        };
        let width = matches
            .value_of("width")
            .unwrap()
            .trim()
            .parse::<u32>()
            .expect("Width must be a u32!");
        let height = matches
            .value_of("height")
            .unwrap()
            .trim()
            .parse::<u32>()
            .expect("Height must be a u32!");
        let fps = matches
            .value_of("framerate")
            .unwrap()
            .trim()
            .parse::<u32>()
            .expect("Framerate must be a u32!");
        let format = {
            match matches.value_of("format").unwrap() {
                "YUYV" => FrameFormat::YUYV,
                _ => FrameFormat::MJPEG,
            }
        };

        let matches_clone = matches.clone();

        let (send, recv) = flume::unbounded();
        // spawn a thread for capture
        std::thread::spawn(move || {
            // Index camera
            if let Ok(index) = matches_clone
                .value_of("capture")
                .unwrap()
                .trim()
                .parse::<usize>()
            {
                let mut camera =
                    Camera::new_with(index, width, height, fps, format, backend_value).unwrap();

                if matches_clone.is_present("query-device") {
                    match camera.compatible_fourcc() {
                        Ok(fcc) => {
                            for ff in fcc {
                                match camera.compatible_list_by_resolution(ff) {
                                    Ok(compat) => {
                                        println!("For FourCC {}", ff);
                                        for (res, fps) in compat {
                                            println!("{}x{}: {:?}", res.width(), res.height(), fps);
                                        }
                                    }
                                    Err(why) => {
                                        println!("Failed to get compatible resolution/FPS list for FrameFormat {}: {}", ff, why.to_string())
                                    }
                                }
                            }
                        }
                        Err(why) => {
                            println!("Failed to get compatible FourCC: {}", why.to_string())
                        }
                    }
                }

                // open stream
                camera.open_stream().unwrap();
                loop {
                    let frame = camera.frame().unwrap();
                    println!(
                        "Captured frame {}x{} @ {}FPS size {}",
                        frame.width(),
                        frame.height(),
                        fps,
                        frame.len()
                    );
                    send.send(frame).unwrap()
                }
            }
            // IP Camera
            else {
                let ip_camera =
                    NetworkCamera::new(matches_clone.value_of("capture").unwrap().to_string())
                        .expect("Invalid IP!");
                ip_camera.open_stream().unwrap();
                loop {
                    let frame = ip_camera.frame().unwrap();
                    println!(
                        "Captured frame {}x{} @ {}FPS size {}",
                        frame.width(),
                        frame.height(),
                        fps,
                        frame.len()
                    );
                    send.send(frame).unwrap();
                }
            }
        });

        // run glium
        if matches.is_present("display") {
            let gl_event_loop = EventLoop::new();
            let window_builder = WindowBuilder::new();
            let context_builder = ContextBuilder::new().with_vsync(true);
            let gl_display = Display::new(window_builder, context_builder, &gl_event_loop).unwrap();

            implement_vertex!(Vertex, position, tex_coords);

            let vert_buffer = VertexBuffer::new(
                &gl_display,
                &[
                    Vertex {
                        position: [-1.0, -1.0],
                        tex_coords: [0.0, 0.0],
                    },
                    Vertex {
                        position: [-1.0, 1.0],
                        tex_coords: [0.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, 1.0],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, -1.0],
                        tex_coords: [1.0, 0.0],
                    },
                ],
            )
            .unwrap();

            let idx_buf = IndexBuffer::new(
                &gl_display,
                PrimitiveType::TriangleStrip,
                &[1 as u16, 2, 0, 3],
            )
            .unwrap();

            let program = program!(&gl_display,
                140 => {
                    vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec2 tex_coords;
                out vec2 v_tex_coords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

                    fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                out vec4 f_color;
                void main() {
                    f_color = texture(tex, v_tex_coords);
                }
            "
                },
            )
            .unwrap();

            // run the event loop

            gl_event_loop.run(move |event, _window, ctrl| {
                let before_capture = Instant::now();
                let frame = recv.recv().unwrap();
                let after_capture = Instant::now();

                let width = &frame.width();
                let height = &frame.height();

                let raw_data = RawImage2d::from_raw_rgb(frame.into_raw(), (*width, *height));
                let gl_texture = Texture2d::new(&gl_display, raw_data).unwrap();

                let uniforms = uniform! {
                    matrix: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32]
                    ],
                    tex: &gl_texture
                };

                let mut target = gl_display.draw();
                target.clear_color(0.0, 0.0, 0.0, 0.0);
                target
                    .draw(
                        &vert_buffer,
                        &idx_buf,
                        &program,
                        &uniforms,
                        &Default::default(),
                    )
                    .unwrap();
                target.finish().unwrap();

                match event {
                    glutin::event::Event::WindowEvent { event, .. } => match event {
                        glutin::event::WindowEvent::CloseRequested => {
                            *ctrl = glutin::event_loop::ControlFlow::Exit;
                        }
                        _ => {}
                    },
                    _ => {}
                }

                println!(
                    "Took {}ms to capture",
                    after_capture.duration_since(before_capture).as_millis()
                )
            })
        }
        // dont
        else {
            loop {
                if let Ok(frame) = recv.recv() {
                    println!(
                        "Frame width {} height {} size {}",
                        frame.width(),
                        frame.height(),
                        frame.len()
                    );
                } else {
                    println!("Thread terminated, closing!");
                    break;
                }
            }
        }
    }
}
