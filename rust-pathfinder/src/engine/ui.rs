// pathfinder/demo/src/ui.rs
//
// Copyright © 2019 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::engine::window::WindowImpl;
use crate::engine::{BackgroundColor, Options};
use pathfinder_color::ColorU;
use pathfinder_geometry::rect::RectI;
use pathfinder_geometry::vector::{Vector2I, vec2i};
use pathfinder_gpu::allocator::GPUMemoryAllocator;
use pathfinder_gpu::{Device, TextureFormat};
use pathfinder_renderer::gpu::debug::DebugUIPresenter;
use pathfinder_resources::ResourceLoader;
use pathfinder_ui::{BUTTON_HEIGHT, BUTTON_TEXT_OFFSET, BUTTON_WIDTH, FONT_ASCENT, PADDING};
use pathfinder_ui::{TEXT_COLOR, TOOLTIP_HEIGHT, WINDOW_COLOR};
use std::f32::consts::PI;
use std::path::PathBuf;

const SLIDER_WIDTH: i32 = 360;
const SLIDER_HEIGHT: i32 = 48;
const SLIDER_TRACK_HEIGHT: i32 = 24;
const SLIDER_KNOB_WIDTH: i32 = 12;
const SLIDER_KNOB_HEIGHT: i32 = 48;

const EFFECTS_PANEL_WIDTH: i32 = 550;
const EFFECTS_PANEL_HEIGHT: i32 = BUTTON_HEIGHT * 3 + PADDING * 4;

const BACKGROUND_PANEL_WIDTH: i32 = 250;
const BACKGROUND_PANEL_HEIGHT: i32 = BUTTON_HEIGHT * 3;

const SCREENSHOT_PANEL_WIDTH: i32 = 275;
const SCREENSHOT_PANEL_HEIGHT: i32 = BUTTON_HEIGHT * 2;

const ROTATE_PANEL_WIDTH: i32 = SLIDER_WIDTH + PADDING * 2;
const ROTATE_PANEL_HEIGHT: i32 = PADDING * 2 + SLIDER_HEIGHT;

const LIGHT_BG_COLOR:       ColorU = ColorU { r: 248, g: 248, b: 248, a: 255, };
const DARK_BG_COLOR:        ColorU = ColorU { r: 32,  g: 32,  b: 32,  a: 255, };
const TRANSPARENT_BG_COLOR: ColorU = ColorU { r: 0,   g: 0,   b: 0,   a: 0,   };

static EFFECTS_PNG_NAME: &'static str = "demo-effects";
static OPEN_PNG_NAME: &'static str = "demo-open";
static ROTATE_PNG_NAME: &'static str = "demo-rotate";
static ZOOM_IN_PNG_NAME: &'static str = "demo-zoom-in";
static ZOOM_ACTUAL_SIZE_PNG_NAME: &'static str = "demo-zoom-actual-size";
static ZOOM_OUT_PNG_NAME: &'static str = "demo-zoom-out";
static BACKGROUND_PNG_NAME: &'static str = "demo-background";
static SCREENSHOT_PNG_NAME: &'static str = "demo-screenshot";

pub struct DemoUIModel {
    pub background_color: BackgroundColor,
    pub gamma_correction_effect_enabled: bool,
    pub stem_darkening_effect_enabled: bool,
    pub subpixel_aa_effect_enabled: bool,
    pub rotation: i32,
    pub message: String,
}

impl DemoUIModel {
    pub fn new(options: &Options) -> DemoUIModel {
        DemoUIModel {
            background_color: options.background_color,
            gamma_correction_effect_enabled: false,
            stem_darkening_effect_enabled: false,
            subpixel_aa_effect_enabled: false,
            rotation: SLIDER_WIDTH / 2,
            message: String::new(),
        }
    }

    fn rotation(&self) -> f32 {
        (self.rotation as f32 / SLIDER_WIDTH as f32 * 2.0 - 1.0) * PI
    }

    // Only relevant if in monochrome mode.
    pub fn foreground_color(&self) -> ColorU {
        match self.background_color {
            BackgroundColor::Light | BackgroundColor::Transparent => ColorU::black(),
            BackgroundColor::Dark => ColorU::white(),
        }
    }

    pub fn background_color(&self) -> ColorU {
        match self.background_color {
            BackgroundColor::Light => LIGHT_BG_COLOR,
            BackgroundColor::Dark => DARK_BG_COLOR,
            BackgroundColor::Transparent => TRANSPARENT_BG_COLOR,
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum UIAction {
    None,
    ModelChanged,
    EffectsChanged,
    TakeScreenshot(ScreenshotInfo),
    ZoomIn,
    ZoomActualSize,
    ZoomOut,
    Rotate(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenshotInfo {
    pub kind: ScreenshotType,
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScreenshotType {
    PNG = 0,
    SVG = 1,
}

impl ScreenshotType {
    fn extension(&self) -> &'static str {
        match *self {
            ScreenshotType::PNG => "png",
            ScreenshotType::SVG => "svg",
        }
    }

    fn as_str(&self) -> &'static str {
        match *self {
            ScreenshotType::PNG => "PNG",
            ScreenshotType::SVG => "SVG",
        }
    }
}
