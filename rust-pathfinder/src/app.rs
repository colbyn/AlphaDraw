pub mod types;
use std::collections::LinkedList;
use std::intrinsics::sqrtf32;
use std::sync::Mutex;
use std::cell::Cell;
use std::collections::VecDeque;
use std::path::PathBuf;
use pathfinder_geometry::rect::RectI;
use pathfinder_geometry::vector::{Vector2I, vec2i};
use rayon::ThreadPoolBuilder;
use io_surface::IOSurfaceRef;
use lazy_static::lazy_static;
// use surfman::{NativeDevice, SystemConnection, SystemDevice, SystemSurface};
use surfman::{SurfaceAccess, SurfaceType};
use euclid::default::Size2D;
use crate::{app, prelude::*};
use types::WindowSize;


///////////////////////////////////////////////////////////////////////////////
// APP WINDOW
///////////////////////////////////////////////////////////////////////////////

// pub struct AppWindow {
//     window: winit::window::Window,
//     connection: surfman::SystemConnection,
//     device: surfman::SystemDevice,
//     metal_device: surfman::NativeDevice,
//     surface: surfman::SystemSurface,
//     event_loop: winit::event_loop::EventLoop<()>,
//     mouse_position: Vector2I,
//     mouse_down: bool,
//     next_user_event_id: Cell<u32>,
// }


///////////////////////////////////////////////////////////////////////////////
// APP INSTANCE
///////////////////////////////////////////////////////////////////////////////


pub struct AppState {
    pub should_exit: bool,
    pub should_redraw: bool,
    pub should_resize: bool,
    pub mouse_down: bool,
    pub focused: bool,
    /// Cursor is within the window frame.
    pub cursor_active: bool,
    pub segments: Vec<Vec<Vector2I>>,
    pub renderer: pf::Renderer<pf::MetalDevice>,
    pub scene_proxy: pf::SceneProxy,
}


impl AppState {
    fn draw(&mut self, app_window: &mut AppWindow) {
        ///////////////////////////////////////////////////////////////////////
        // SETUP
        ///////////////////////////////////////////////////////////////////////
        self.should_redraw = false;
        let (width, height) = {
            let s = app_window.window_size();
            (s.x(), s.y())
        };
        let canvas = pf::Canvas::new(
            Vector2F::new(width as f32, height as f32)
        );
        let canvas_font_context = pf::CanvasFontContext::from_system_source();
        let mut ctx = canvas.get_context_2d(canvas_font_context);
        ///////////////////////////////////////////////////////////////////////
        // RENDERING CONTEXT
        ///////////////////////////////////////////////////////////////////////
        for segment in self.segments.iter() {
            ctx.set_line_width(10.0);
            let style = pf::FillStyle::Color(
                pf::ColorU::new(u8::MAX, 0, 0, u8::MAX)
            );
            // ctx.set_stroke_style(style);
            ctx.set_fill_style(pf::FillStyle::Color(
                pf::ColorU::new(u8::MAX, 0, 0, u8::MAX)
            ));
            let mut last_point = None::<Vector2I>;
            fn distance_between(
                a: Vector2I,
                b: Vector2I,
            ) -> f32 {
                let dx = (
                    (b.x() - a.x()).pow(2)
                ) as f32;
                let dy = (
                    (b.y() - a.y()).pow(2)
                ) as f32;
                unsafe {
                    sqrtf32(dx + dy)
                }
            }
            for (pix, point) in segment.iter().enumerate() {
                ///////////////////////////////////////////////////////////////
                // DEFAULT STYLING
                ///////////////////////////////////////////////////////////////
                ctx.set_fill_style(pf::FillStyle::Color(
                    pf::ColorU::new(u8::MAX, 0, 0, u8::MAX)
                ));
                ctx.set_stroke_style(pf::FillStyle::Color(
                    pf::ColorU::new(u8::MAX, 0, 0, u8::MAX)
                ));
                ///////////////////////////////////////////////////////////////
                // DEBUG POINT
                ///////////////////////////////////////////////////////////////
                {
                    let mut path = pf::Path2D::new();
                    let center = Vector2F::new(
                        point.x() as f32,
                        point.y() as f32,
                    );
                    let tau = 2.0 * std::f32::consts::PI;
                    path.arc(
                        center,
                        5.0,
                        0.0,
                        tau,
                        pf::ArcDirection::CCW
                    );
                    ctx.fill_path(path, pf::FillRule::Winding);
                }
                ///////////////////////////////////////////////////////////////
                // LINE
                ///////////////////////////////////////////////////////////////
                let mut path = pf::Path2D::new();
                if let Some(previous) = last_point {
                    let distance = distance_between(*point, previous);
                    path.move_to(Vector2F::new(
                        previous.x() as f32,
                        previous.y() as f32,
                    ));
                    path.line_to(Vector2F::new(
                        point.x() as f32,
                        point.y() as f32,
                    ));
                    if distance < 30.0 {
                        ctx.set_stroke_style(pf::FillStyle::Color(
                            pf::ColorU::new(
                                0,
                                u8::MAX,
                                0,
                                u8::MAX
                            )
                        ));
                    }
                    ctx.stroke_path(path);
                }
                last_point = Some(*point);
            }
        }
        ///////////////////////////////////////////////////////////////////////
        // SCENE
        ///////////////////////////////////////////////////////////////////////
        self.scene_proxy.replace_scene(
            ctx.into_canvas().into_scene()
        );
        let build_options = pf::BuildOptions {
            subpixel_aa_enabled: true,
            ..pf::BuildOptions::default()
        };
        self.scene_proxy.build_and_render(&mut self.renderer, build_options);
        ///////////////////////////////////////////////////////////////////////
        // FINALIZE
        ///////////////////////////////////////////////////////////////////////
        app_window.present(self.renderer.device_mut());
        self.renderer.dest_framebuffer_size_changed();
    }
    fn resize(&mut self, app_window: &mut AppWindow) {
        self.should_resize = false;
        let new_size = app_window.real_window_size();
        let new_size = euclid::Size2D::new(
            new_size.width as i32,
            new_size.height as i32,
        );
        app_window.device.resize_surface(
            &mut app_window.surface,
            new_size,
        );
        let (mut renderer, mut scene_proxy) = {
            init_renderer(app_window)
        };
        self.renderer = renderer;
        self.scene_proxy = scene_proxy;
    }
    fn begin_new_segment(&mut self) {
        let empty_last = self.segments
            .last()
            .map(|x| x.is_empty())
            .unwrap_or(false);
        if !empty_last {
            self.segments.push(Vec::new());
        }
    }
    fn add_point_to_current_segment(&mut self, point: Vector2I) {
        self.segments
            .last_mut()
            .unwrap()
            .push(point);
    }
    fn handle_window_event(&mut self, event: wit::WindowEvent, app_window: &mut AppWindow) {
        let to_point = |pos: wit::PhysicalPosition<f64>| {
            Vector2I::new(
                pos.x as i32,
                pos.y as i32,
            )
        };
        match event {
            wit::WindowEvent::MouseInput{state: wit::ElementState::Pressed,..} => {
                self.mouse_down = true;
            }
            wit::WindowEvent::MouseInput{state: wit::ElementState::Released,..} => {
                self.mouse_down = false;
                self.begin_new_segment();
            }
            wit::WindowEvent::CursorEntered{..} => {
                self.cursor_active = true;
            }
            wit::WindowEvent::CursorLeft{..} => {
                self.cursor_active = true;
                self.begin_new_segment();
            }
            wit::WindowEvent::CursorMoved{position, ..} => {
                if self.cursor_active && self.mouse_down && self.focused {
                    let point = to_point(position);
                    self.add_point_to_current_segment(point);
                    self.should_redraw = true;
                }
            }
            wit::WindowEvent::Focused(focused) => {
                self.focused = focused;
                if focused == false {
                    self.begin_new_segment();
                }
            }
            wit::WindowEvent::Destroyed => {
                self.should_exit = true;
            }
            wit::WindowEvent::CloseRequested => {
                self.should_exit = true;
            }
            wit::WindowEvent::Resized(..) => {
                self.should_redraw = true;
                self.should_resize = true;
            }
            wit::WindowEvent::ScaleFactorChanged{..} => {
                self.should_redraw = true;
            }
            _ => ()
        }
    }
    fn handle_device_event(&mut self, event: wit::DeviceEvent) {
        match event {
            _ => ()
        }
    }
    pub fn handle_event(
        &mut self,
        event: wit::Event<()>,
        app_window: &mut AppWindow
    ) {
        match event {
            wit::Event::DeviceEvent{event, ..} => {
                self.handle_device_event(event)
            }
            wit::Event::WindowEvent{event, ..} => {
                self.handle_window_event(event, app_window)
            }
            wit::Event::LoopDestroyed => {
                self.should_exit = true;
            }
            wit::Event::RedrawRequested(..) => {
                if self.should_resize {
                    // self.resize(app_window);
                    unimplemented!()
                }
                if self.should_redraw {
                    self.draw(app_window)
                }
            }
            _ => ()
        }
    }
}



///////////////////////////////////////////////////////////////////////////////
// LOW-LEVEL APP WINDOW
///////////////////////////////////////////////////////////////////////////////

pub struct AppWindow {
    window: wit::Window,
    connection: surfman::SystemConnection,
    device: surfman::SystemDevice,
    native_device: surfman::NativeDevice,
    surface: surfman::SystemSurface,
    resource_loader: pf::FilesystemResourceLoader,
}

impl AppWindow {
    pub fn metal_device(&self) -> metal::Device {
        // FIXME(pcwalton): Remove once `surfman` upgrades `metal-rs` version.
        unsafe {
            std::mem::transmute(self.native_device.0.clone())
        }
    }

    pub fn metal_io_surface(&self) -> IOSurfaceRef {
        self.device.native_surface(&self.surface).0
    }

    pub fn size(&self) -> WindowSize {
        let window = &self.window;
        let (monitor, size) = {
            let monitor = window.current_monitor().unwrap();
            (monitor, window.inner_size())
        };
        WindowSize {
            logical_size: vec2i(size.width as i32, size.height as i32),
            backing_scale_factor: monitor.scale_factor() as f32,
        }
    }

    pub fn viewport(&self) -> RectI {
        let WindowSize { logical_size, backing_scale_factor } = self.size();
        let mut size = (logical_size.to_f32() * backing_scale_factor).to_i32();
        let mut x_offset = 0;
        RectI::new(vec2i(x_offset, 0), size)
    }

    pub fn real_window_size(&self) -> wit::PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn window_size(&self) -> Vector2I {
        let WindowSize { logical_size, backing_scale_factor } = self.size();
        let mut size = (logical_size.to_f32() * backing_scale_factor).to_i32();
        size
    }

    pub fn present(&mut self, metal_device: &mut pathfinder_metal::MetalDevice) {
        self.device.present_surface(&mut self.surface).unwrap();
        metal_device.swap_texture(self.device.native_surface(&self.surface).0);
    }
}

///////////////////////////////////////////////////////////////////////////////
// MAIN FUNCTIONS
///////////////////////////////////////////////////////////////////////////////

fn init_renderer(app_window: &mut AppWindow) -> (pf::Renderer<pf::MetalDevice>, pf::SceneProxy) {
    ///////////////////////////////////////////////////////////////////////
    // SETUP PATHFINDER DEVICE
    ///////////////////////////////////////////////////////////////////////
    let viewport = app_window.viewport();
    let window_size = app_window.window_size();
    let metal_device = app_window.metal_device();
    let metal_io_surface = app_window.metal_io_surface();
    let pf_device = unsafe {
        pf::MetalDevice::new(metal_device, metal_io_surface)
    };
    let dest_framebuffer = pf::DestFramebuffer::Default::<pf::MetalDevice> {
        viewport,
        window_size,
    };
    let level = pf::RendererLevel::default_for_device(&pf_device);
    let render_mode = pf::RendererMode {level};
    let render_options = pf::RendererOptions {
        dest: dest_framebuffer,
        background_color: Some(pf::ColorF::white()),
        show_debug_ui: true,
    };
    ///////////////////////////////////////////////////////////////////////
    // RENDERER
    ///////////////////////////////////////////////////////////////////////
    let renderer = pf::Renderer::new(
        pf_device,
        &app_window.resource_loader,
        render_mode,
        render_options
    );
    ///////////////////////////////////////////////////////////////////////
    // SCENE
    ///////////////////////////////////////////////////////////////////////
    let executor = pf::RayonExecutor;
    let scene_proxy = pf::SceneProxy::new(level, executor);
    ///////////////////////////////////////////////////////////////////////
    // DONE
    ///////////////////////////////////////////////////////////////////////
    (renderer, scene_proxy)
}


pub fn start() {
    let low_power_mode = false;
    let event_loop = winit::event_loop::EventLoop::<()>::new();
    let mut app_window = {
        ///////////////////////////////////////////////////////////////////////
        // GPU - METAL SURFACE
        ///////////////////////////////////////////////////////////////////////
        let window = winit::window::WindowBuilder::new()
            .with_title("Canvas")
            .build(&event_loop)
            .unwrap();
        ///////////////////////////////////////////////////////////////////////
        // GPU - METAL SURFACE
        ///////////////////////////////////////////////////////////////////////
        let connection = surfman::SystemConnection::from_winit_window(&window).unwrap();
        let native_widget = connection.create_native_widget_from_winit_window(&window).unwrap();
        let adapter = if low_power_mode {
            connection.create_low_power_adapter().unwrap()
        } else {
            connection.create_hardware_adapter().unwrap()
        };
        let mut device = connection.create_device(&adapter).unwrap();
        let native_device = device.native_device();
        let surface_type = SurfaceType::Widget { native_widget };
        let mut surface = device.create_surface(SurfaceAccess::GPUOnly, surface_type).unwrap();
        let resource_loader = pf::FilesystemResourceLoader::locate();
        AppWindow {window, connection, device, surface, native_device, resource_loader}
    };
    ///////////////////////////////////////////////////////////////////////////
    // INIT RENDERER & SCENE
    ///////////////////////////////////////////////////////////////////////////
    let (mut renderer, mut scene_proxy) = {
        init_renderer(&mut app_window)
    };
    ///////////////////////////////////////////////////////////////////////////
    // APP STATE
    ///////////////////////////////////////////////////////////////////////////
    let mut app_state = AppState {
        mouse_down: false,
        should_redraw: false,
        should_exit: false,
        should_resize: false,
        focused: false,
        cursor_active: false,
        segments: vec![vec![]],
        renderer,
        scene_proxy,
    };
    // app_state.resize(&mut app_window);
    app_state.draw(&mut app_window);
    let tick = {
        move |event: wit::Event<()>, _: &wit::EventLoopWindowTarget<()>, control: &mut wit::ControlFlow| {
            app_state.handle_event(event, &mut app_window);
            if app_state.should_exit {
                *control = wit::ControlFlow::Exit;
                app_window.device.destroy_surface(&mut app_window.surface);
            }
            if app_state.should_redraw || app_state.should_resize {
                app_window.window.request_redraw();
            }
        }
    };
    ///////////////////////////////////////////////////////////////////////////
    // GO!
    ///////////////////////////////////////////////////////////////////////////
    event_loop.run(tick);
    ///////////////////////////////////////////////////////////////////////////
    // DONE
    ///////////////////////////////////////////////////////////////////////////
    println!("DONE");
}



