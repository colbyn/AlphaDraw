#![allow(unused)]
#![feature(concat_idents)]
pub mod data;
use std::borrow::Borrow;
use std::convert::AsRef;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use js_sys::Math::sqrt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, EventTarget, HtmlCanvasElement, MouseEvent};

use data::*;



///////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_str(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_js_value(s: &JsValue);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => {
        unsafe {
            log_str(&format_args!($($t)*).to_string())
        }
    }
}

impl AsRef<web_sys::EventTarget> for Canvas {
    fn as_ref(&self) -> &web_sys::EventTarget {
        &self.canvas
    }
}

macro_rules! cloned {
    ($ident:ident, $func:expr) => {{
        let $ident = $ident.clone();
        $func
    }};
}

fn cleanup_points(dpi: f64, original_points: Vec<Point>) -> Vec<Point> {
    use cubic_spline::TryFrom;
    let distance_between = |a: Point, b: Point| {
        let dx = b.x() - a.x();
        let dy = b.y() - a.y();
        f64::sqrt(dx.powi(2) + dy.powi(2))
    };
    let mut points: Vec<Point> = Vec::new();
    for point in original_points.iter() {
        if let Some(last) = points.last() {
            let distance = distance_between(*point, *last);
            if distance < 2.0 * dpi {
                continue;
            }
        }
        points.push(*point);
    }
    return points
    // let points = points
    //     .into_iter()
    //     .map(|p| {
    //         let [x, y] = p.into();
    //         cubic_spline::Point::new(x, y)
    //     })
    //     .collect::<Vec<_>>();
    // if points.len() < 2 {
    //     return original_points
    // }

    // let points = cubic_spline::Points::try_from(&points).unwrap();
    // let opts = cubic_spline::SplineOpts::new()
    //     .tension(0.6)
    //     .num_of_segments(3);
    // let result = points.calc_spline(&opts).unwrap();
    
    
    // result
    //     .into_inner()
    //     .into_iter()
    //     .map(|p| {
    //         let x = p.x;
    //         let y = p.y;
    //         Point::new(x, y)
    //     })
    //     .collect()
}

///////////////////////////////////////////////////////////////////////////////
// HELPER TYPES
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, Default)]
pub struct BoundingBox {
    top: f64,
    left: f64,
    width: f64,
    height: f64,
}



///////////////////////////////////////////////////////////////////////////////
// ROOT CANVAS
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Canvas {
    canvas: HtmlCanvasElement,
    ctx: Rc<CanvasRenderingContext2d>,
    // CANVAS STATE
    pressed: bool,
    needs_redraw: bool,
    // CANVAS METADATA
    dpi: f64,
    rect: BoundingBox,
    // CANVAS PAYLOAD
    segments: Segments,
}

impl Canvas {
    ///////////////////////////////////////////////////////////////////////////
    // CREATE SELF
    ///////////////////////////////////////////////////////////////////////////
    fn new() -> Result<Self, JsValue> {
        ///////////////////////////////////////////////////////////////////////
        // SETUP
        ///////////////////////////////////////////////////////////////////////
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .create_element_ns(Some("http://www.w3.org/1999/xhtml"), "canvas")?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        // let context = Rc::new(context);
        ///////////////////////////////////////////////////////////////////////
        // OTHER METADATA
        ///////////////////////////////////////////////////////////////////////
        let dpi = window.device_pixel_ratio();
        let rect: web_sys::DomRect = canvas.get_bounding_client_rect();
        let width = rect.width() * dpi;
        let height = rect.height() * dpi;
        ///////////////////////////////////////////////////////////////////////
        // DONE
        ///////////////////////////////////////////////////////////////////////
        Ok(Canvas{
            canvas,
            ctx: Rc::new(context),
            pressed: Default::default(),
            needs_redraw: Default::default(),
            dpi: Default::default(),
            rect: Default::default(),
            segments: Default::default(),
        })
    }
    ///////////////////////////////////////////////////////////////////////////
    // INTERNAL HELPERS
    ///////////////////////////////////////////////////////////////////////////
    fn setup(&mut self) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document.body().unwrap().append_child(&self.canvas).unwrap();
        self.update_resolution();
    }
    fn update_resolution(&mut self) {
        let window = web_sys::window().unwrap();
        let dpi = window.device_pixel_ratio();
        let rect: web_sys::DomRect = self.canvas.get_bounding_client_rect();
        let width = rect.width() * dpi;
        let height = rect.height() * dpi;
        // UPDATE DOM
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        // UPDATE SELF
        self.dpi = dpi;
        self.rect = BoundingBox {
            top: rect.top(),
            left: rect.left(),
            width,
            height,
        };
        self.needs_redraw = true;
    }
    fn get_current_pos(&self, event: &web_sys::MouseEvent) -> Point {
        // unsafe {
        //     log_js_value(event);
        // };
        let event_type = event.type_();
        let is_touchstart = &event_type == "touchstart";
        let is_touchcancel = &event_type == "touchcancel";
        let is_touchend = &event_type == "touchend";
        let is_touchmove = &event_type == "touchmove";
        let is_mobile_event = {
            is_touchstart ||
            is_touchcancel ||
            is_touchend ||
            is_touchmove
        };
        let window = web_sys::window().unwrap();
        let rect = self.rect;
        let dpi = self.dpi;
        let (x, y) = {
            if is_mobile_event {
                (event.page_x(), event.page_y())
            } else {
                (event.client_x(), event.client_y())
            }
        };
        let x = {
            (x as f64 - rect.left) * dpi
        };
        let y = {
            (y as f64 - rect.top) * dpi
        };
        Point::new(x, y)
    }
    ///////////////////////////////////////////////////////////////////////////
    // DOM EVENT HANDLERS
    ///////////////////////////////////////////////////////////////////////////
    fn on_mouse_down(&mut self, event: web_sys::MouseEvent) {
        self.pressed = true;
        let point = self.get_current_pos(&event);
        self.segments.begin_new_segment(point);
        self.needs_redraw = true;
    }
    fn on_mouse_up(&mut self, event: web_sys::MouseEvent) {
        self.pressed = false;
        let point = self.get_current_pos(&event);
        self.segments.add_point(point);
        self.needs_redraw = true;
    }
    fn on_mouse_move(&mut self, event: web_sys::MouseEvent) {
        if self.pressed {
            let point = self.get_current_pos(&event);
            self.segments.add_point(point);
            self.needs_redraw = true;
        }
    }
    ///////////////////////////////////////////////////////////////////////////
    // RUNTIME
    ///////////////////////////////////////////////////////////////////////////
    fn redraw(&mut self) {
        let dpi = self.dpi;
        let ctx = self.ctx.clone();
        let default_color = "hsl(0deg 0% 69% / 77%)";
        let green_color = "rgb(4 251 49 / 63%)";
        let red_color = "rgb(251 4 4 / 63%)";
        let blue_color = "rgb(4 193 251 / 63%)";
        let begin_path = || {
            ctx.save();
            ctx.begin_path();
            ctx.set_line_cap("round");
            ctx.set_line_width(10.0);
            ctx.set_stroke_style(&JsValue::from_str(default_color));
        };
        let end_path = || {
            ctx.stroke();
            ctx.restore();
        };
        let move_to = |point: Point| {
            let [x, y] = point.into();
            ctx.move_to(x, y)
        };
        let line_to = |point: Point| {
            let [x, y] = point.into();
            ctx.line_to(x, y)
        };
        let draw_point = |point: Point| {
            let [x, y] = point.into();
            let radius = 10.0;
            let tau = 2.0 * std::f64::consts::PI;
            ctx.arc(x, y, radius, 0.0, tau);
        };
        let line_to_with = |point: Point, color: &str| {
            let [x, y] = point.into();
            ctx.set_stroke_style(&JsValue::from_str(
                "rgba(255, 0, 0, 0.5)"
            ));
            ctx.line_to(x, y)
        };
        let distance_between = |a: Point, b: Point| {
            let dx = b.x() - a.x();
            let dy = b.y() - a.y();
            f64::sqrt(dx.powi(2) + dy.powi(2))
        };
        // INIT POINTS

        let segment_length = self.segments.0.len();
        let segment_len = self.segments.0.len();
        for (seg_ix, segment) in self.segments.0.iter().enumerate() {
            let is_last_segment = seg_ix == segment_len - 1;
            let mut points = segment.points
                .iter()
                .map(|segment| segment.point)
                .collect::<Vec<_>>();
            let mut last_point = None::<Point>;
            'next_point: for (ix, segment_point) in points.into_iter().enumerate() {
                let point = segment_point;
                if ix == 0 {
                    move_to(point);
                    last_point = Some(point);
                    continue 'next_point;
                }
                begin_path();
                if let Some(previous) = last_point {
                    let distance = distance_between(point, previous);
                    move_to(previous);
                }
                line_to(point);
                last_point = Some(point);
                end_path();
            }
        }
    }
    fn tick(&mut self) {
        if self.needs_redraw {
            self.ctx.clear_rect(0.0, 0.0, self.rect.width, self.rect.height);
            self.redraw();
            self.needs_redraw = false;
        }
    }
}


thread_local! {
    static GLOBAL_BINARY_CONTROLLER: RefCell<Canvas> = {
        let mut canvas = Canvas::new().unwrap();
        RefCell::new(canvas)
    };
}

macro_rules! add_event_handler {
    ($target:expr, $event_name:expr, fn($event:ident : $ty:ty) $x:tt) => {{
        fn callback($event:$ty) {$x}
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(_)>);
        {
            $target.add_event_listener_with_callback(
                $event_name,
                closure.as_ref().unchecked_ref()
            ).unwrap()
        }
        closure.forget();
    }};
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    ///////////////////////////////////////////////////////////////////////////
    // SYSTEM INIT
    ///////////////////////////////////////////////////////////////////////////
    console_error_panic_hook::set_once();
    ///////////////////////////////////////////////////////////////////////////
    // SETUP CANVAS
    ///////////////////////////////////////////////////////////////////////////
    fn register_event_handlers(canvas: &Canvas) {
        ///////////////////////////////////////////////////////////////////////
        // DESKTOP EVENTS
        ///////////////////////////////////////////////////////////////////////
        add_event_handler!(
            &canvas.canvas,
            "mousedown",
            fn(event: MouseEvent) {
                GLOBAL_BINARY_CONTROLLER.with(|canvas| {
                    canvas.borrow_mut().on_mouse_down(event)
                });
            }
        );
        add_event_handler!(
            &canvas.canvas,
            "mouseup",
            fn(event: MouseEvent) {
                GLOBAL_BINARY_CONTROLLER.with(|canvas| {
                    canvas.borrow_mut().on_mouse_up(event)
                });
            }
        );
        add_event_handler!(
            &canvas.canvas,
            "mousemove",
            fn(event: MouseEvent) {
                GLOBAL_BINARY_CONTROLLER.with(|canvas| {
                    canvas.borrow_mut().on_mouse_move(event)
                });
            }
        );
        ///////////////////////////////////////////////////////////////////////
        // MOBILE EVENTS
        ///////////////////////////////////////////////////////////////////////
        add_event_handler!(
            &canvas.canvas,
            "touchstart",
            fn(event: MouseEvent) {
                GLOBAL_BINARY_CONTROLLER.with(|canvas| {
                    canvas.borrow_mut().on_mouse_down(event)
                });
            }
        );
        add_event_handler!(
            &canvas.canvas,
            "touchcancel",
            fn(event: MouseEvent) {
                GLOBAL_BINARY_CONTROLLER.with(|canvas| {
                    canvas.borrow_mut().on_mouse_up(event)
                });
            }
        );
        add_event_handler!(
            &canvas.canvas,
            "touchend",
            fn(event: MouseEvent) {
                GLOBAL_BINARY_CONTROLLER.with(|canvas| {
                    canvas.borrow_mut().on_mouse_up(event)
                });
            }
        );
        add_event_handler!(
            &canvas.canvas,
            "touchmove",
            fn(event: MouseEvent) {
                GLOBAL_BINARY_CONTROLLER.with(|canvas| {
                    canvas.borrow_mut().on_mouse_move(event)
                });
            }
        );
        ///////////////////////////////////////////////////////////////////////
        // WINDOW EVENTS
        ///////////////////////////////////////////////////////////////////////
        add_event_handler!(
            &web_sys::window().unwrap(),
            "resize",
            fn(event: MouseEvent) {
                GLOBAL_BINARY_CONTROLLER.with(|canvas| {
                    canvas.borrow_mut().update_resolution()
                });
            }
        );
    }
    GLOBAL_BINARY_CONTROLLER.with(|canvas| {
        register_event_handlers(&canvas.borrow());
    });

    Ok(())
}

#[wasm_bindgen]
pub fn init_system() {
    GLOBAL_BINARY_CONTROLLER.with(|canvas| {
        canvas.borrow_mut().setup();
        console_log!("init system");
    });
}


#[wasm_bindgen]
pub fn tick() {
    GLOBAL_BINARY_CONTROLLER.with(|canvas| {
        canvas.borrow_mut().tick();
    });
}


