// pathfinder/demo/common/src/lib.rs
//
// Copyright © 2019 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
pub mod window;
pub mod view;
mod camera;
mod concurrent;
mod device;
mod renderer;
mod ui;
use clap::{App, Arg};
use pathfinder_content::effects::DEFRINGING_KERNEL_CORE_GRAPHICS;
use pathfinder_content::effects::PatternFilter;
use pathfinder_content::effects::STEM_DARKENING_FACTORS;
use pathfinder_content::outline::Outline;
use pathfinder_content::pattern::Pattern;
use pathfinder_content::render_target::RenderTargetId;
use pathfinder_export::{Export, FileFormat};
use pathfinder_geometry::rect::{RectF, RectI};
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::transform3d::Transform4F;
use pathfinder_geometry::vector::{Vector2F, Vector2I, Vector4F, vec2f, vec2i};
use pathfinder_gpu::Device;
use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererLevel};
use pathfinder_renderer::gpu::options::{RendererMode, RendererOptions};
use pathfinder_renderer::gpu::renderer::{DebugUIPresenterInfo, Renderer};
use pathfinder_renderer::options::{BuildOptions, RenderTransform};
use pathfinder_renderer::paint::Paint;
use pathfinder_renderer::scene::{DrawPath, RenderTarget, Scene};
use pathfinder_resources::ResourceLoader;
use pathfinder_svg::SVGScene;
use pathfinder_ui::{MousePosition, UIEvent};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use usvg::{Options as UsvgOptions, Tree as SvgTree};
use pathfinder_metal::MetalDevice as DeviceImpl;
use crate::engine::camera::Camera;
use crate::engine::concurrent::DemoExecutor;
use crate::engine::device::{GroundProgram, GroundVertexArray};
use crate::engine::ui::{DemoUIModel, ScreenshotInfo, ScreenshotType, UIAction};
use crate::engine::window::{Event, Keycode, DataPath, WindowImpl, WindowSize};



///////////////////////////////////////////////////////////////////////////////
// MISC CONSTANTS
///////////////////////////////////////////////////////////////////////////////

static DEFAULT_SVG_VIRTUAL_PATH: &'static str = "svg/Ghostscript_Tiger.svg";

const MOUSELOOK_ROTATION_SPEED: f32 = 0.007;
const CAMERA_VELOCITY: f32 = 0.02;

// How much the scene is scaled when a scale gesture is performed.
const CAMERA_SCALE_SPEED_2D: f32 = 6.0;
// How much the scene is scaled when a zoom button is clicked.
const CAMERA_ZOOM_AMOUNT_2D: f32 = 0.1;

// Half of the eye separation distance.
const DEFAULT_EYE_OFFSET: f32 = 0.025;

const APPROX_FONT_SIZE: f32 = 16.0;

const MESSAGE_TIMEOUT_SECS: u64 = 5;


///////////////////////////////////////////////////////////////////////////////
// APP
///////////////////////////////////////////////////////////////////////////////


pub struct DemoApp {
    pub should_exit: bool,
    pub options: Options,

    window_size: WindowSize,

    svg_model: SvgDataModel,
    scene_metadata: SceneMetadata,
    render_transform: Option<RenderTransform>,

    camera: Camera,
    frame_counter: u32,
    pending_screenshot_info: Option<ScreenshotInfo>,
    mouselook_enabled: bool,
    pub dirty: bool,
    expire_message_event_id: u32,
    message_epoch: u32,
    last_mouse_position: Vector2I,

    current_frame: Option<Frame>,

    ui_model: DemoUIModel,

    scene_proxy: SceneProxy,
    renderer: Renderer<DeviceImpl>,

    scene_framebuffer: Option<<DeviceImpl as Device>::Framebuffer>,

    points: Vec<Vector2I>,
}

impl DemoApp {
    pub fn new(window: WindowImpl, window_size: WindowSize, options: Options) -> DemoApp {
        let expire_message_event_id = window.create_user_event_id();

        let device = unsafe {
            DeviceImpl::new(window.metal_device(), window.metal_io_surface())
        };

        let resources = window.resource_loader();

        // Set up the executor.
        let executor = DemoExecutor::new(options.jobs);

        let mut ui_model = DemoUIModel::new(&options);
        let level = match options.renderer_level {
            Some(level) => level,
            None => RendererLevel::default_for_device(&device),
        };
        let viewport = window.viewport();
        let dest_framebuffer = DestFramebuffer::Default {
            viewport,
            window_size: window_size.device_size(),
        };
        let render_mode = RendererMode { level };
        let render_options = RendererOptions {
            dest: dest_framebuffer,
            background_color: None,
            show_debug_ui: true,
        };

        let filter = build_filter(&ui_model);

        let viewport = window.viewport();
        let mut svg_model = load_scene(
            &window_size,
            &[],
            resources
        );

        let (mut scene, message) = svg_model.render(viewport.size(), filter);
        let renderer = Renderer::new(device, resources, render_mode, render_options);
        let scene_metadata = SceneMetadata::new_clipping_view_box(
            &mut scene,
            viewport.size()
        );
        let camera = Camera::new(
            scene_metadata.view_box,
            viewport.size()
        );
        let scene_proxy = SceneProxy::from_scene(scene, level, executor);

        let mut message_epoch = 0;
        emit_message(
            &mut ui_model,
            &mut message_epoch,
            expire_message_event_id,
            message,
        );

        // let ui_presenter = DemoUIPresenter::new();

        DemoApp {
            should_exit: false,
            options,

            window_size,

            svg_model,
            scene_metadata,
            render_transform: None,

            camera,
            frame_counter: 0,
            pending_screenshot_info: None,
            mouselook_enabled: false,
            dirty: true,
            expire_message_event_id,
            message_epoch,
            last_mouse_position: Vector2I::default(),

            current_frame: None,

            // ui_presenter,
            ui_model,

            scene_proxy,
            renderer,

            scene_framebuffer: None,

            points: Default::default(),
        }
    }

    pub fn start(self) {
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

    pub fn prepare_frame(&mut self, events: Vec<Event>, window: &WindowImpl) -> u32 {
        // Clear dirty flag.
        self.dirty = false;

        // Handle events.
        let ui_events = self.handle_events(events, window);

        // Update the scene.
        self.build_scene();

        // Save the frame.
        //
        // FIXME(pcwalton): This is super ugly.
        let transform = self.render_transform.clone().unwrap();
        self.current_frame = Some(Frame::new(transform, ui_events));

        // Prepare to render the frame.
        self.prepare_frame_rendering(window)
    }

    fn build_scene(&mut self) {
        self.render_transform = match self.camera {
            Camera::TwoD(transform) => Some(RenderTransform::Transform2D(transform)),
        };

        let build_options = BuildOptions {
            transform: self.render_transform.clone().unwrap(),
            dilation: if self.ui_model.stem_darkening_effect_enabled {
                let font_size = APPROX_FONT_SIZE * self.window_size.backing_scale_factor;
                vec2f(STEM_DARKENING_FACTORS[0], STEM_DARKENING_FACTORS[1]) * font_size
            } else {
                Vector2F::zero()
            },
            subpixel_aa_enabled: self.ui_model.subpixel_aa_effect_enabled,
        };

        self.scene_proxy.build(build_options);
    }

    ///////////////////////////////////////////////////////////////////////////
    // MAIN EVENT HANDLER
    ///////////////////////////////////////////////////////////////////////////
    fn process_event(
        &mut self,
        event_sink: &mut Vec<UIEvent>,
        event: Event,
        window: &WindowImpl
    ) {
        // RENAME TODO: What is this?
        let check_user_event = |app: &DemoApp, event_id: u32, expected_epoch: u32| {
            event_id == app.expire_message_event_id && expected_epoch as u32 == app.message_epoch
        };
        let mut handle_zoom = {
            |app: &mut DemoApp, d_dist: f32, position: Vector2I| {
                let Camera::TwoD(ref mut transform) = app.camera;
                let backing_scale_factor = app.window_size.backing_scale_factor;
                    let position = position.to_f32() * backing_scale_factor;
                    let scale_delta = 1.0 + d_dist * CAMERA_SCALE_SPEED_2D;
                    *transform = transform
                        .translate(-position)
                        .scale(scale_delta)
                        .translate(position);
            }
        };
        let process_mouse_position = {
            |app: &mut DemoApp, new_position: Vector2I| -> MousePosition {
                let value =
                    new_position.to_f32() *
                    app.window_size.backing_scale_factor;
                let absolute = value.to_i32();
                let relative = absolute - app.last_mouse_position;
                app.last_mouse_position = absolute;
                MousePosition { absolute, relative }
            }
        };
        let render_scene = |app: &mut DemoApp| {
            let viewport = window.viewport();
            let filter = build_filter(&app.ui_model);
            app.svg_model = load_scene(
                &app.window_size,
                &app.points,
                window.resource_loader(),
            );
            let (mut scene, message) = app.svg_model.render(viewport.size(), filter);
            app.ui_model.message = message;
            let viewport_size = window.viewport().size();
            app.scene_metadata = SceneMetadata::new_clipping_view_box(&mut scene, viewport_size);
            app.camera = Camera::new(
                app.scene_metadata.view_box,
                viewport_size
            );
            app.scene_proxy.replace_scene(scene);
            app.dirty = true;
        };
        match event {
            Event::Quit { .. } | Event::KeyDown(Keycode::Escape) => {
                self.should_exit = true;
                self.dirty = true;
            }
            Event::WindowResized(new_size) => {
                self.window_size = new_size;
                let viewport = window.viewport();
                self.scene_proxy.set_view_box(RectF::new(Vector2F::zero(), viewport.size().to_f32()));
                self.renderer.options_mut().dest =
                    DestFramebuffer::full_window(self.window_size.device_size());
                self.renderer.dest_framebuffer_size_changed();
                self.dirty = true;
            }
            Event::MouseDown(new_position) => {
                let mouse_position = process_mouse_position(self, new_position);
                event_sink.push(UIEvent::MouseDown(mouse_position));
            }
            Event::MouseMoved(new_position) if self.mouselook_enabled => {
                let mouse_position = process_mouse_position(self, new_position);
            }
            Event::MouseDragged{delta, global} => {
                let global_mouse_pos = global;
                let mouse_position = process_mouse_position(self, delta);
                // event_sink.push(UIEvent::MouseDragged(mouse_position));
                self.dirty = true;
                self.points.push(global_mouse_pos);
                render_scene(self)
            }
            // Event::Zoom(d_dist, position) => {
            //     handle_zoom(self, d_dist, position)
            // }
            Event::KeyDown(Keycode::Tab) => {
                self.options.ui = match self.options.ui {
                    UIVisibility::None => UIVisibility::Stats,
                    UIVisibility::Stats => UIVisibility::All,
                    UIVisibility::All => UIVisibility::None,
                }
            }

            Event::OpenData(..) => {
                render_scene(self)
            }

            Event::User {
                message_type: event_id,
                message_data: expected_epoch,
            } if check_user_event(self, event_id, expected_epoch) => {
                self.ui_model.message = String::new();
                self.dirty = true;
            }
            _ => (),
        }
    }

    fn handle_events(
        &mut self,
        events: Vec<Event>,
        window: &WindowImpl
    ) -> Vec<UIEvent> {
        let mut ui_events = vec![];
        self.dirty = false;

        // RENAME TODO: What is this?
        let check_user_event = |app: &DemoApp, event_id: u32, expected_epoch: u32| {
            event_id == app.expire_message_event_id && expected_epoch as u32 == app.message_epoch
        };

        for event in events {
            self.process_event(&mut ui_events, event, window);
        }

        ui_events
    }

    pub fn finish_drawing_frame(&mut self, window: &mut WindowImpl) {
        let frame = self.current_frame.take().unwrap();
        for ui_event in &frame.ui_events {
            self.dirty = true;
            self.renderer
                .debug_ui_presenter_mut()
                .debug_ui_presenter
                .ui_presenter
                .event_queue
                .push(*ui_event);
        }

        let mouse_pos = self.last_mouse_position.to_f32() * self.window_size.backing_scale_factor;
        self.renderer.debug_ui_presenter_mut().debug_ui_presenter.ui_presenter.mouse_position = mouse_pos;
            

        let mut ui_action = UIAction::None;
        if self.options.ui == UIVisibility::All {
            let DebugUIPresenterInfo {
                    device,
                    allocator,
                    debug_ui_presenter
            } = self.renderer.debug_ui_presenter_mut();
        }

        self.handle_ui_events(frame, &mut ui_action);

        self.renderer.device().end_commands();

        window.present(self.renderer.device_mut());
        self.frame_counter += 1;
    }

    fn handle_ui_events(&mut self, mut frame: Frame, ui_action: &mut UIAction) {
        frame.ui_events = self.renderer
            .debug_ui_presenter_mut()
            .debug_ui_presenter
            .ui_presenter
            .event_queue
            .drain();
        // self.handle_ui_action(ui_action);
        for ui_event in frame.ui_events {
            match ui_event {
                UIEvent::MouseDragged(position) => {
                    let Camera::TwoD(ref mut transform) = self.camera;
                    *transform = transform.translate(position.relative.to_f32());
                }
                _ => {}
            }
        }
    }

    fn handle_ui_action(&mut self, ui_action: &mut UIAction, window: &WindowImpl) {
        match ui_action {
            UIAction::None => {}
            UIAction::ModelChanged => {
                self.dirty = true
            },
            UIAction::EffectsChanged => {
                let viewport_size = window.viewport().size();
                let filter = build_filter(&self.ui_model);
                let (mut scene, _) = self.svg_model.render(viewport_size, filter);
                self.scene_metadata =
                    SceneMetadata::new_clipping_view_box(&mut scene, viewport_size);
                self.scene_proxy.replace_scene(scene);
                self.dirty = true;
            }
            UIAction::TakeScreenshot(ref info) => {
                self.pending_screenshot_info = Some((*info).clone());
                self.dirty = true;
            }
            UIAction::ZoomIn => {
                let Camera::TwoD(ref mut transform) = self.camera;
                let scale = 1.0 + CAMERA_ZOOM_AMOUNT_2D;
                let center = center_of_window(&self.window_size);
                *transform = transform.translate(-center).scale(scale).translate(center);
                self.dirty = true;
            }
            UIAction::ZoomOut => {
                let Camera::TwoD(ref mut transform) = self.camera;
                let scale = 1.0 - CAMERA_ZOOM_AMOUNT_2D;
                let center = center_of_window(&self.window_size);
                *transform = transform.translate(-center).scale(scale).translate(center);
                self.dirty = true;
            }
            UIAction::ZoomActualSize => {
                let Camera::TwoD(ref mut transform) = self.camera;
                *transform = Transform2F::default();
                self.dirty = true;
            }
            UIAction::Rotate(theta) => {
                let Camera::TwoD(ref mut transform) = self.camera;
                let old_rotation = transform.rotation();
                let center = center_of_window(&self.window_size);
                *transform = transform
                    .translate(-center)
                    .rotate(*theta - old_rotation)
                    .translate(center);
            }
        }
    }
}


///////////////////////////////////////////////////////////////////////////////
// OPTIONS
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Options {
    pub jobs: Option<usize>,
    pub input_path: DataPath,
    pub ui: UIVisibility,
    pub background_color: BackgroundColor,
    pub high_performance_gpu: bool,
    pub renderer_level: Option<RendererLevel>,
    hidden_field_for_future_proofing: (),
}

impl Default for Options {
    fn default() -> Self {
        Options {
            jobs: None,
            input_path: DataPath::Default,
            ui: UIVisibility::All,
            background_color: BackgroundColor::Light,
            high_performance_gpu: false,
            renderer_level: None,
            hidden_field_for_future_proofing: (),
        }
    }
}

impl Options {
    pub fn command_line_overrides(&mut self) {
        let matches = App::new("demo")
            .arg(
                Arg::with_name("jobs")
                    .short("j")
                    .long("jobs")
                    .value_name("THREADS")
                    .takes_value(true)
                    .help("Number of threads to use"),
            )
            .arg(
                Arg::with_name("3d")
                    .short("3")
                    .long("3d")
                    .help("Run in 3D")
                    .conflicts_with("vr"),
            )
            .arg(
                Arg::with_name("vr")
                    .short("V")
                    .long("vr")
                    .help("Run in VR")
                    .conflicts_with("3d"),
            )
            .arg(
                Arg::with_name("ui")
                    .short("u")
                    .long("ui")
                    .takes_value(true)
                    .possible_values(&["none", "stats", "all"])
                    .help("How much UI to show"),
            )
            .arg(
                Arg::with_name("background")
                    .short("b")
                    .long("background")
                    .takes_value(true)
                    .possible_values(&["light", "dark", "transparent"])
                    .help("The background color to use"),
            )
            .arg(
                Arg::with_name("high-performance-gpu")
                    .short("g")
                    .long("high-performance-gpu")
                    .help("Use the high-performance (discrete) GPU, if available")
            )
            .arg(
                Arg::with_name("level")
                    .long("level")
                    .short("l")
                    .help("Set the renderer feature level as a Direct3D version equivalent")
                    .takes_value(true)
                    .possible_values(&["9", "11"])
            )
            .arg(
                Arg::with_name("INPUT")
                    .help("Path to the SVG file to render")
                    .index(1),
            )
            .get_matches();

        if let Some(jobs) = matches.value_of("jobs") {
            self.jobs = jobs.parse().ok();
        }

        if let Some(ui) = matches.value_of("ui") {
            self.ui = match ui {
                "none" => UIVisibility::None,
                "stats" => UIVisibility::Stats,
                _ => UIVisibility::All,
            };
        }

        if let Some(background_color) = matches.value_of("background") {
            self.background_color = match background_color {
                "light" => BackgroundColor::Light,
                "dark" => BackgroundColor::Dark,
                _ => BackgroundColor::Transparent,
            };
        }

        if matches.is_present("high-performance-gpu") {
            self.high_performance_gpu = true;
        }

        if let Some(renderer_level) = matches.value_of("level") {
            if renderer_level == "11" {
                self.renderer_level = Some(RendererLevel::D3D11);
            } else if renderer_level == "9" {
                self.renderer_level = Some(RendererLevel::D3D9);
            }
        }

        if let Some(path) = matches.value_of("INPUT") {
            self.input_path = DataPath::Path(PathBuf::from(path));
        };
    }
}


///////////////////////////////////////////////////////////////////////////////
// CONTENT RELATED
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq)]
pub enum UIVisibility {
    None,
    Stats,
    All,
}

fn load_scene(
    &window_size: &WindowSize,
    points: &[Vector2I],
    resource_loader: &dyn ResourceLoader,
) -> SvgDataModel {
    let size = window_size.device_size();
    if points.is_empty() {
        let data = format!(
            "
            <svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\">
            <path d=\"M 10 10 H 90 V 90 H 10 Z\" stroke=\"white\" fill=\"white\"/>
            </svg>
            ",
            width=size.x(),
            height=size.y(),
        );
        let tree = SvgTree::from_str(&data, &UsvgOptions::default()).unwrap();
        return SvgDataModel::Svg(tree)
    }
    println!("POINTS: {:?}", points.len());
    let path_data = points
        .into_iter()
        .map(|p| {
            format!("L {} {}", p.x(), p.y())
        })
        .collect::<Vec<_>>()
        .join(" ");
    let path_data = format!(
        "M 0 0 {}",
        path_data
    );
    let data = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\">
        <path d=\"{path_data}\" stroke=\"#000\" stroke-width=\"1px\"/>
        </svg>
        ",
        path_data=path_data,
        width=size.x(),
        height=size.y(),
    );
    let tree = SvgTree::from_str(data.as_str(), &UsvgOptions::default()).unwrap();
    SvgDataModel::Svg(tree)
}

// FIXME(pcwalton): Rework how transforms work in the demo. The transform affects the final
// composite steps, breaking this approach.
fn build_svg_tree(tree: &SvgTree, viewport_size: Vector2I, filter: Option<PatternFilter>) -> SVGScene {
    let mut scene = Scene::new();
    let filter_info = filter.map(|filter| {
        let scale = match filter {
            PatternFilter::Text { defringing_kernel: Some(_), .. } => vec2i(3, 1),
            _ => vec2i(1, 1),
        };
        let name = "Text".to_owned();
        let render_target_size = viewport_size * scale;
        let render_target = RenderTarget::new(render_target_size, name);
        let render_target_id = scene.push_render_target(render_target);
        FilterInfo { filter, render_target_id, render_target_size }
    });

    let mut built_svg = SVGScene::from_tree_and_scene(&tree, scene);
    if let Some(FilterInfo {
        filter,
        render_target_id,
        render_target_size 
    }) = filter_info {
        let mut pattern = Pattern::from_render_target(render_target_id, render_target_size);
        pattern.set_filter(Some(filter));
        let paint_id = built_svg.scene.push_paint(&Paint::from_pattern(pattern));

        let outline = Outline::from_rect(RectI::new(vec2i(0, 0), viewport_size).to_f32());
        let path = DrawPath::new(outline, paint_id);

        built_svg.scene.pop_render_target();
        built_svg.scene.push_draw_path(path);
    }

    return built_svg;

    struct FilterInfo {
        filter: PatternFilter,
        render_target_id: RenderTargetId,
        render_target_size: Vector2I,
    }
}

///////////////////////////////////////////////////////////////////////////////
// WINDOW HELPERS
///////////////////////////////////////////////////////////////////////////////

fn center_of_window(window_size: &WindowSize) -> Vector2F {
    window_size.device_size().to_f32() * 0.5
}

fn get_svg_building_message(built_svg: &SVGScene) -> String {
    if built_svg.result_flags.is_empty() {
        return String::new();
    }
    format!(
        "Warning: These features in the SVG are unsupported: {}.",
        built_svg.result_flags
    )
}

fn emit_message(
    ui_model: &mut DemoUIModel,
    message_epoch: &mut u32,
    expire_message_event_id: u32,
    message: String
) {
    if message.is_empty() {
        return;
    }

    ui_model.message = message;
    let expected_epoch = *message_epoch + 1;
    *message_epoch = expected_epoch;
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(MESSAGE_TIMEOUT_SECS));
        WindowImpl::push_user_event(expire_message_event_id, expected_epoch);
    });
}

///////////////////////////////////////////////////////////////////////////////
// APP EVENTS
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum AppEvent {
    MouseDown {
        absolute: Vector2I,
        relative: Vector2I,
    },
    MouseDrag {
        absolute: Vector2I,
        relative: Vector2I,
    },
    Scroll(winit::event::MouseScrollDelta),
}

///////////////////////////////////////////////////////////////////////////////
// FRAME STATE
///////////////////////////////////////////////////////////////////////////////

struct Frame {
    transform: RenderTransform,
    ui_events: Vec<UIEvent>,
}

impl Frame {
    fn new(transform: RenderTransform, ui_events: Vec<UIEvent>) -> Frame {
        Frame { transform, ui_events }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SVG SCENE TREE
///////////////////////////////////////////////////////////////////////////////

enum SvgDataModel {
    Svg(SvgTree)
}

impl SvgDataModel {
    fn render(
        &mut self,
        viewport_size: Vector2I,
        filter: Option<PatternFilter>
    ) -> (Scene, String) {
        match *self {
            SvgDataModel::Svg(ref tree) => {
                let built_svg = build_svg_tree(&tree, viewport_size, filter);
                let message = get_svg_building_message(&built_svg);
                (built_svg.scene, message)
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// MISC
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BackgroundColor {
    Light = 0,
    Dark = 1,
    Transparent = 2,
}

struct SceneMetadata {
    view_box: RectF,
}

impl SceneMetadata {
    // FIXME(pcwalton): The fact that this mutates the scene is really ugly!
    // Can we simplify this?
    fn new_clipping_view_box(scene: &mut Scene, viewport_size: Vector2I) -> SceneMetadata {
        let view_box = scene.view_box();
        scene.set_view_box(RectF::new(Vector2F::zero(), viewport_size.to_f32()));
        SceneMetadata { view_box }
    }
}

fn build_filter(ui_model: &DemoUIModel) -> Option<PatternFilter> {
    if !ui_model.gamma_correction_effect_enabled && !ui_model.subpixel_aa_effect_enabled {
        return None;
    }

    Some(PatternFilter::Text {
        fg_color: ui_model.foreground_color().to_f32(),
        bg_color: ui_model.background_color().to_f32(),
        gamma_correction: ui_model.gamma_correction_effect_enabled,
        defringing_kernel: if ui_model.subpixel_aa_effect_enabled {
            // TODO(pcwalton): Select FreeType defringing kernel as necessary.
            Some(DEFRINGING_KERNEL_CORE_GRAPHICS)
        } else {
            None
        }
    })
}
