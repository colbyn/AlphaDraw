use foreign_types::ForeignTypeRef;
use metal::{CAMetalLayer, CoreAnimationLayerRef};
use pathfinder_canvas::{Canvas, CanvasFontContext, Path2D};
use pathfinder_color::ColorF;
use pathfinder_geometry::vector::{vec2f, vec2i, Vector2F, Vector2I};
use pathfinder_geometry::rect::RectF;
use pathfinder_metal::MetalDevice;
use pathfinder_renderer::concurrent::executor::Executor;
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererMode, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer;
use pathfinder_gpu::Device;
use pathfinder_renderer::options::BuildOptions;
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use pathfinder_renderer::gpu::options::RendererLevel;

pub fn view<E: Executor + Send + 'static>(
    renderer: &mut Renderer<MetalDevice>,
    window_size: Vector2I,
    executor: E
) {
    let canvas = Canvas::new(window_size.to_f32());
    let mut canvas = canvas.get_context_2d(CanvasFontContext::from_system_source());
        // Set line width.
    canvas.set_line_width(10.0);

    // Draw walls.
    canvas.stroke_rect(RectF::new(vec2f(75.0, 140.0), vec2f(150.0, 110.0)));

    // Draw door.
    canvas.fill_rect(RectF::new(vec2f(130.0, 190.0), vec2f(40.0, 60.0)));

    // Draw roof.
    let mut path = Path2D::new();
    path.move_to(vec2f(50.0, 140.0));
    path.line_to(vec2f(150.0, 60.0));
    path.line_to(vec2f(250.0, 140.0));
    path.close_path();
    canvas.stroke_path(path);

    // Render the canvas to screen.
    let mut scene = SceneProxy::from_scene(
        canvas.into_canvas().into_scene(),
        renderer.mode().level,
        executor
    );
    scene.build_and_render(renderer, BuildOptions::default());
    // renderer.device().present_drawable(drawable);
}

