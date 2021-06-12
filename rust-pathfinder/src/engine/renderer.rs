// pathfinder/demo/common/src/renderer.rs
//
// Copyright © 2019 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Rendering functionality for the demo.

use crate::engine::camera::Camera;
use crate::engine::window::WindowImpl;
use crate::engine::{BackgroundColor, DemoApp, UIVisibility};
use image::ColorType;
use pathfinder_color::{ColorF, ColorU};
use pathfinder_gpu::{ClearOps, DepthFunc, DepthState, Device, Primitive, RenderOptions};
use pathfinder_gpu::{RenderState, RenderTarget, TextureData, TextureFormat, UniformData};
use pathfinder_geometry::rect::RectI;
use pathfinder_geometry::transform3d::Transform4F;
use pathfinder_geometry::vector::{Vector2I, Vector4F};
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererOptions};
use pathfinder_renderer::options::RenderTransform;
use std::mem;
use std::path::PathBuf;

const GROUND_SOLID_COLOR: ColorU = ColorU {
    r: 80,
    g: 80,
    b: 80,
    a: 255,
};

const GROUND_LINE_COLOR: ColorU = ColorU {
    r: 127,
    g: 127,
    b: 127,
    a: 255,
};

const GRIDLINE_COUNT: i32 = 10;

impl DemoApp {
    pub fn prepare_frame_rendering(&mut self, window: &WindowImpl) -> u32 {
        // MAKE THE CONTEXT CURRENT.

        // CLEAR TO THE APPROPRIATE COLOR.
        // let mode = self.camera.mode();
        let clear_color = Some(self.ui_model.background_color().to_f32());

        // Set up framebuffers.
        let window_size = window.window_size();
        let scene_count = {
            *self.renderer.options_mut() = RendererOptions {
                dest: DestFramebuffer::Default {
                    viewport: window.viewport(),
                    window_size,
                },
                background_color: clear_color,
                show_debug_ui: self.options.ui != UIVisibility::None,
            };
            1
        };

        scene_count
    }

    pub fn draw_scene(&mut self, window: &WindowImpl) {
        self.renderer.device().begin_commands();
        // window.make_current();
        self.renderer.device().end_commands();
        self.render_vector_scene();
    }

    pub fn begin_compositing(&mut self) {
        self.renderer.device().begin_commands();
    }

    #[allow(deprecated)]
    fn render_vector_scene(&mut self) {
        self.renderer.disable_depth();
        // ISSUE RENDER COMMANDS!
        self.scene_proxy.render(&mut self.renderer);
    }
}
