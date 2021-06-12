pub use std::path::PathBuf;
pub use std::time::Duration;
pub use pathfinder_geometry::rect::{RectF, RectI};
pub use pathfinder_geometry::vector::{Vector2F, Vector2I, Vector4F};

/// short for `winit` (that doesn’t conflict with it).
pub mod wit {
    pub use winit::event::{Event, WindowEvent, DeviceEvent, DeviceId, MouseScrollDelta};
    pub use winit::event::ElementState;
    pub use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopClosed};
    pub use winit::event_loop::EventLoopWindowTarget;
    pub use winit::window::Window;
    pub use winit::window::WindowBuilder;
    pub use winit::window::WindowAttributes;
    pub use winit::window::WindowId;
    pub use winit::dpi::{Position, LogicalPosition, LogicalSize};
    pub use winit::dpi::{PhysicalPosition, PhysicalSize};
    pub use winit::dpi::Pixel;
    pub use winit::window::Theme;
}

pub mod pf {
    pub use pathfinder_content::effects::DEFRINGING_KERNEL_CORE_GRAPHICS;
    pub use pathfinder_content::effects::PatternFilter;
    pub use pathfinder_content::effects::STEM_DARKENING_FACTORS;
    pub use pathfinder_content::outline::Outline;
    pub use pathfinder_content::pattern::Pattern;
    pub use pathfinder_content::render_target::RenderTargetId;
    pub use pathfinder_export::{Export, FileFormat};
    pub use pathfinder_geometry::rect::{RectF, RectI};
    pub use pathfinder_geometry::transform2d::Transform2F;
    pub use pathfinder_geometry::transform3d::Transform4F;
    pub use pathfinder_geometry::vector::{Vector2F, Vector2I, Vector4F, vec2f, vec2i};
    pub use pathfinder_gpu::Device;
    pub use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
    pub use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererLevel};
    pub use pathfinder_renderer::gpu::options::{RendererMode, RendererOptions};
    pub use pathfinder_renderer::gpu::renderer::{DebugUIPresenterInfo, Renderer};
    pub use pathfinder_renderer::options::{BuildOptions, RenderTransform};
    pub use pathfinder_renderer::paint::Paint;
    pub use pathfinder_renderer::scene::{DrawPath, RenderTarget, Scene};
    pub use pathfinder_resources::ResourceLoader;
    pub use pathfinder_svg::SVGScene;
    pub use pathfinder_ui::{MousePosition, UIEvent};
    pub use pathfinder_metal::MetalDevice;
    pub use pathfinder_resources::fs::FilesystemResourceLoader;
    pub use pathfinder_color::{ColorF, ColorU};
    pub use pathfinder_canvas::{Canvas, CanvasRenderingContext2D, CanvasFontContext};
    pub use pathfinder_canvas::Path2D;
    pub use pathfinder_canvas::FillRule;
    pub use pathfinder_canvas::FillStyle;
    pub use pathfinder_renderer::concurrent::rayon::RayonExecutor;
    pub use pathfinder_canvas::ArcDirection;
}