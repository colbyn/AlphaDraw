// pathfinder/demo/common/src/window.rs
//
// Copyright © 2019 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A minimal cross-platform windowing layer.
use std::cell::Cell;
use std::collections::VecDeque;
use std::path::PathBuf;
use pathfinder_geometry::rect::RectI;
use pathfinder_geometry::transform3d::{Perspective, Transform4F};
use pathfinder_geometry::vector::{Vector2I, vec2i};
use pathfinder_resources::ResourceLoader;
use pathfinder_resources::fs::FilesystemResourceLoader;
use rayon::ThreadPoolBuilder;
use std::sync::Mutex;
use io_surface::IOSurfaceRef;
// use metal::Device as MetalDevice;
// use pathfinder_metal::MetalDevice;
use lazy_static::lazy_static;
use surfman::{NativeDevice, SystemConnection, SystemDevice, SystemSurface};
use surfman::{SurfaceAccess, SurfaceType};
use euclid::default::Size2D;

lazy_static! {
    static ref EVENT_QUEUE: Mutex<Option<EventQueue>> = Mutex::new(None);
}

const DEFAULT_WINDOW_WIDTH: u32 = 1067;
const DEFAULT_WINDOW_HEIGHT: u32 = 800;

#[derive(Clone)]
enum CustomEvent {
    User { message_type: u32, message_data: u32 },
    OpenData(PathBuf),
}

struct EventQueue {
    event_loop_proxy: winit::event_loop::EventLoopProxy<CustomEvent>,
    pending_custom_events: VecDeque<CustomEvent>,
}

// pub trait Window {
//     fn metal_device(&self) -> metal::Device;
//     fn metal_io_surface(&self) -> IOSurfaceRef;
//     fn present(&mut self, device: &mut pathfinder_metal::MetalDevice);

//     fn make_current(&mut self);
//     fn viewport(&self) -> RectI;
//     fn resource_loader(&self) -> &dyn ResourceLoader;
//     fn create_user_event_id(&self) -> u32;
//     fn push_user_event(message_type: u32, message_data: u32);
//     fn present_open_svg_dialog(&mut self);
//     fn run_save_dialog(&self, extension: &str) -> Result<PathBuf, ()>;

//     fn adjust_thread_pool_settings(&self, builder: ThreadPoolBuilder) -> ThreadPoolBuilder {
//         builder
//     }
// }

pub enum Event {
    Quit,
    WindowResized(WindowSize),
    KeyDown(Keycode),
    KeyUp(Keycode),
    MouseDown(Vector2I),
    MouseMoved(Vector2I),
    MouseDragged {
        delta: Vector2I,
        global: Vector2I,
    },
    Look {
        pitch: f32,
        yaw: f32,
    },
    SetEyeTransforms(Vec<OcularTransform>),
    OpenData(DataPath),
    User {
        message_type: u32,
        message_data: u32,
    },
}

impl Event {
    fn init_from_winit_event(
        winit_event: winit::event::Event<CustomEvent>,
        window: &winit::window::Window,
        mouse_position: &mut Vector2I,
        mouse_down: &mut bool
    ) -> Option<Event> {
        let orig_mouse_position = *mouse_position;
        match winit_event {
            // winit::event::Event::Awakened => {
            //     let mut event_queue = EVENT_QUEUE.lock().unwrap();
            //     let event_queue = event_queue.as_mut().unwrap();
            //     match event_queue.pending_custom_events
            //                     .pop_front()
            //                     .expect("`Awakened` with no pending custom event!") {
            //         CustomEvent::OpenData(data_path) => Some(Event::OpenData(DataPath::Path(data_path))),
            //         CustomEvent::User { message_data, message_type } => {
            //             Some(Event::User { message_data, message_type })
            //         }
            //     }
            // }
            winit::event::Event::WindowEvent { event: window_event, .. } => {
                match window_event {
                    // winit::WindowEvent::MouseWheel{delta, ..} => {
                    //     match delta {
                    //         e @ winit::MouseScrollDelta::LineDelta(..) => {
                    //             println!("{:?}", e);
                    //         }
                    //         e @ winit::MouseScrollDelta::PixelDelta(..) => {
                    //             println!("{:?}", e);
                    //         }
                    //     }
                    //     None
                    // }
                    winit::event::WindowEvent::MouseInput {
                        state: winit::event::ElementState::Pressed,
                        button: winit::event::MouseButton::Left,
                        ..
                    } => {
                        *mouse_down = true;
                        Some(Event::MouseDown(*mouse_position))
                    }
                    winit::event::WindowEvent::MouseInput {
                        state: winit::event::ElementState::Released,
                        button: winit::event::MouseButton::Left,
                        ..
                    } => {
                        *mouse_down = false;
                        None
                    }
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        *mouse_position = vec2i(position.x as i32, position.y as i32);
                        if *mouse_down {
                            Some(Event::MouseDragged {
                                delta: *mouse_position,
                                global: orig_mouse_position,
                            })
                        } else {
                            Some(Event::MouseMoved(*mouse_position))
                        }
                    }
                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        input.virtual_keycode.and_then(|virtual_keycode| {
                            match virtual_keycode {
                                winit::event::VirtualKeyCode::Escape => Some(Keycode::Escape),
                                winit::event::VirtualKeyCode::Tab => Some(Keycode::Tab),
                                virtual_keycode => {
                                    let vk = virtual_keycode as u32;
                                    let vk_a = winit::event::VirtualKeyCode::A as u32;
                                    let vk_z = winit::event::VirtualKeyCode::Z as u32;
                                    if vk >= vk_a && vk <= vk_z {
                                        let character = ((vk - vk_a) + 'A' as u32) as u8;
                                        Some(Keycode::Alphanumeric(character))
                                    } else {
                                        None
                                    }
                                }
                            }
                        }).map(|keycode| {
                            match input.state {
                                winit::event::ElementState::Pressed => Event::KeyDown(keycode),
                                winit::event::ElementState::Released => Event::KeyUp(keycode),
                            }
                        })
                    }
                    winit::event::WindowEvent::CloseRequested => Some(Event::Quit),
                    winit::event::WindowEvent::Resized(new_size) => {
                        let logical_size = vec2i(new_size.width as i32, new_size.height as i32);
                        let current_window = window.current_monitor().unwrap();
                        let backing_scale_factor = current_window.scale_factor() as f32;
                        // let backing_scale_factor = window.current_monitor().unwrap().get_hidpi_factor() as f32;
                        Some(Event::WindowResized(WindowSize {
                            logical_size,
                            backing_scale_factor,
                        }))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Keycode {
    Alphanumeric(u8),
    Escape,
    Tab,
}

#[derive(Clone, Copy, Debug)]
pub struct WindowSize {
    pub logical_size: Vector2I,
    pub backing_scale_factor: f32,
}

impl WindowSize {
    #[inline]
    pub fn device_size(&self) -> Vector2I {
        (self.logical_size.to_f32() * self.backing_scale_factor).to_i32()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OcularTransform {
    // The perspective which converts from camera coordinates to display coordinates
    pub perspective: Perspective,

    // The view transform which converts from world coordinates to camera coordinates
    pub modelview_to_eye: Transform4F,
}

#[derive(Clone)]
pub enum DataPath {
    Default,
    Resource(String),
    Path(PathBuf),
}


pub struct WindowImpl {
    window: winit::window::Window,
    connection: SystemConnection,
    device: SystemDevice,
    metal_device: NativeDevice,
    surface: SystemSurface,
    event_loop: winit::event_loop::EventLoop<CustomEvent>,
    pending_events: VecDeque<Event>,
    mouse_position: Vector2I,
    mouse_down: bool,
    next_user_event_id: Cell<u32>,
    resource_loader: FilesystemResourceLoader,
}



impl WindowImpl {
    pub fn metal_device(&self) -> metal::Device {
        // FIXME(pcwalton): Remove once `surfman` upgrades `metal-rs` version.
        unsafe {
            std::mem::transmute(self.metal_device.0.clone())
        }
    }

    pub fn metal_io_surface(&self) -> IOSurfaceRef {
        self.device.native_surface(&self.surface).0
    }

    pub fn viewport(&self) -> RectI {
        let WindowSize { logical_size, backing_scale_factor } = self.size();
        let mut size = (logical_size.to_f32() * backing_scale_factor).to_i32();
        let mut x_offset = 0;
        // if let View::Stereo(index) = view {
        //     size.set_x(size.x() / 2);
        //     x_offset = size.x() * (index as i32);
        // }
        RectI::new(vec2i(x_offset, 0), size)
    }

    pub fn window_size(&self) -> Vector2I {
        let WindowSize { logical_size, backing_scale_factor } = self.size();
        let mut size = (logical_size.to_f32() * backing_scale_factor).to_i32();
        size
    }

    pub fn present(&mut self, metal_device: &mut pathfinder_metal::MetalDevice) {
        self.device.present_surface(&mut self.surface).expect("Failed to present surface!");
        metal_device.swap_texture(self.device.native_surface(&self.surface).0);
    }

    pub fn resource_loader(&self) -> &dyn ResourceLoader {
        &self.resource_loader
    }

    // pub fn present_open_svg_dialog(&mut self) {
    //     if let Ok(nfd::Response::Okay(path)) = nfd::open_file_dialog(Some("svg,pdf"), None) {
    //         let mut event_queue = EVENT_QUEUE.lock().unwrap();
    //         let event_queue = event_queue.as_mut().unwrap();
    //         event_queue.pending_custom_events.push_back(CustomEvent::OpenData(PathBuf::from(path)));
    //         // drop(event_queue.event_loop_proxy.wakeup());
    //         unimplemented!()
    //     }
    // }

    // pub fn run_save_dialog(&self, extension: &str) -> Result<PathBuf, ()> {
    //     match nfd::open_save_dialog(Some(extension), None) {
    //         Ok(nfd::Response::Okay(file)) => Ok(PathBuf::from(file)),
    //         _ => Err(()),
    //     }
    // }

    pub fn create_user_event_id(&self) -> u32 {
        let id = self.next_user_event_id.get();
        self.next_user_event_id.set(id + 1);
        id
    }

    pub fn push_user_event(message_type: u32, message_data: u32) {
        let mut event_queue = EVENT_QUEUE.lock().unwrap();
        let event_queue = event_queue.as_mut().unwrap();
        event_queue.pending_custom_events.push_back(CustomEvent::User {
            message_type,
            message_data,
        });
        // drop(event_queue.event_loop_proxy.wakeup());
        unimplemented!()
    }
}

impl WindowImpl {
    pub fn new(options: &crate::engine::Options) -> WindowImpl {
        let event_loop = winit::event_loop::EventLoop::<CustomEvent>::with_user_event();
        let window_size = Size2D::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);
        let logical_size = winit::dpi::LogicalSize::new(window_size.width as f64, window_size.height as f64);
        let window = winit::window::WindowBuilder::new()
            .with_title("Pathfinder Demo")
            .with_inner_size(logical_size)
            .build(&event_loop)
            .unwrap();
        
        // window.show();

        let connection = SystemConnection::from_winit_window(&window).unwrap();
        let native_widget = connection.create_native_widget_from_winit_window(&window).unwrap();

        let adapter = if options.high_performance_gpu {
            connection.create_hardware_adapter().unwrap()
        } else {
            connection.create_low_power_adapter().unwrap()
        };

        let mut device = connection.create_device(&adapter).unwrap();
        let native_device = device.native_device();

        let surface_type = SurfaceType::Widget { native_widget };
        let surface = device.create_surface(SurfaceAccess::GPUOnly, surface_type).unwrap();

        let resource_loader = FilesystemResourceLoader::locate();

        *EVENT_QUEUE.lock().unwrap() = Some(EventQueue {
            event_loop_proxy: event_loop.create_proxy(),
            pending_custom_events: VecDeque::new(),
        });

        WindowImpl {
            window,
            event_loop,
            connection,
            device,
            metal_device: native_device,
            surface,
            next_user_event_id: Cell::new(0),
            pending_events: VecDeque::new(),
            mouse_position: vec2i(0, 0),
            mouse_down: false,
            resource_loader,
        }
    }

    fn window(&self) -> &winit::window::Window { &self.window }

    pub fn size(&self) -> WindowSize {
        let window = self.window();
        let (monitor, size) = {
            let monitor = window.current_monitor().unwrap();
            (monitor, window.inner_size())
        };

        WindowSize {
            logical_size: vec2i(size.width as i32, size.height as i32),
            backing_scale_factor: monitor.scale_factor() as f32,
        }
    }

    // pub fn get_event(&mut self) -> Event {
    //     if self.pending_events.is_empty() {
    //         let window = &self.window;
    //         let mouse_position = &mut self.mouse_position;
    //         let mouse_down = &mut self.mouse_down;
    //         let pending_events = &mut self.pending_events;
    //         self.event_loop.run(|winit_event, _, control| {
    //             //println!("blocking {:?}", winit_event);
    //             // match Event::init_from_winit_event(
    //             //     winit_event,
    //             //     window,
    //             //     mouse_position,
    //             //     mouse_down
    //             // ) {
    //             //     Some(event) => {
    //             //         //println!("handled");
    //             //         pending_events.push_back(event);
    //             //         *control = winit::event_loop::ControlFlow::Exit;
    //             //     }
    //             //     None => {
    //             //         *control = winit::event_loop::ControlFlow::Poll;
    //             //     }
    //             // }
    //         });
    //     }

    //     self.pending_events.pop_front().expect("Where's the event?")
    // }

    pub fn try_get_event(&mut self) -> Option<Event> {
        // if self.pending_events.is_empty() {
        //     let window = &self.window;
        //     let mouse_position = &mut self.mouse_position;
        //     let mouse_down = &mut self.mouse_down;
        //     let pending_events = &mut self.pending_events;
        //     self.event_loop.poll_events(|winit_event| {
        //         //println!("nonblocking {:?}", winit_event);
        //         if let Some(event) = Event::init_from_winit_event(
        //             winit_event,
        //             window,
        //             mouse_position,
        //             mouse_down
        //         ) {
        //             //println!("handled");
        //             pending_events.push_back(event);
        //         }
        //     });
        // }
        // self.pending_events.pop_front()
        unimplemented!()
    }
}
