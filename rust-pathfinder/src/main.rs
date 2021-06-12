#![feature(core_intrinsics)]
#![allow(unused)]
pub mod prelude;
pub mod engine;
pub mod app;
// pub mod framework;
use std::cell::Cell;
use std::collections::VecDeque;
use std::mem;
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;
use euclid::default::Size2D;
// use nfd::Response;
use surfman::{SurfaceAccess, SurfaceType, declare_surfman};
use winit::event::{ElementState, Event as WinitEvent};
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::event_loop::{ControlFlow};
// use winit::event_loop::{MouseButton, VirtualKeyCode, Window as WinitWindow, WindowBuilder, WindowEvent};
use winit::dpi::LogicalSize;
use io_surface::IOSurfaceRef;
use pathfinder_geometry::rect::RectI;
use pathfinder_geometry::vector::{Vector2I, vec2i};
use pathfinder_resources::ResourceLoader;
use pathfinder_resources::fs::FilesystemResourceLoader;
use pathfinder_metal::MetalDevice;
use surfman::{NativeDevice, SystemConnection, SystemDevice, SystemSurface};
use jemallocator;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

declare_surfman!();



fn run() {
    color_backtrace::install();
    pretty_env_logger::init();

    // Read command line options.
    let mut options = crate::engine::Options::default();
    options.command_line_overrides();

    let window = crate::engine::window::WindowImpl::new(&options);
    let window_size = window.size();

    let app = crate::engine::DemoApp::new(window, window_size, options);
    app.start();

    // while !app.should_exit {
    //     let mut events = Vec::<Event>::new();
    //     if !app.dirty {
    //         events.push(app.window.get_event());
    //     }
    //     while let Some(event) = app.window.try_get_event() {
    //         events.push(event);
    //     }

    //     let scene_count = app.prepare_frame(events);

    //     app.draw_scene();
    //     app.begin_compositing();
    //     app.finish_drawing_frame();
    // }
}


fn main() {
    // app::main();
    // run();
    app::start();
}