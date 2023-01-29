// A WebAssembly globe renderer.

// The data module is code generated during the build.
mod data;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::DomMatrix;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, Window};

const CANVAS_WIDTH: u32 = 800;
const CANVAS_HEIGHT: u32 = 800;

const SPHERE_FILL_STYLE: &str = "rgba(159, 159, 255, 1.0)";
const COAST_FRONT_STROKE_STYLE: &str = "rgba(0, 0, 127, 1.0)";
const COAST_BACK_STROKE_STYLE: &str = "rgba(0, 0, 0, 0.25)";
const COAST_FRONT_LINE_WIDTH: f64 = 0.005;
const COAST_BACK_LINE_WIDTH: f64 = 0.0025;

#[derive(Clone, Debug, Default, PartialEq)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Debug, Default)]
struct ControlData {
    pressed: bool,
    position: Position,
    position_prev: Position,
    rotation: f64,
}

fn window() -> Window {
    web_sys::window().expect("should have window")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register request animation frame callback");
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let document = window().document().expect("should have document");

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(CANVAS_WIDTH);
    canvas.set_height(CANVAS_HEIGHT);
    document.body().unwrap().append_child(&canvas)?;

    let context = canvas
        .get_context("2d")?
        .expect("should have 2d context")
        .dyn_into::<CanvasRenderingContext2d>()?;

    // Position calculations for plotting, etc... are performed for a unit sphere
    // centred at the origin; values are scaled and translated to fit on the canvas
    context.set_transform(
        // horizontal scale
        std::cmp::min(CANVAS_WIDTH, CANVAS_HEIGHT) as f64 / 2.0,
        0.0,
        0.0,
        // vertical scale, flipped
        std::cmp::min(CANVAS_WIDTH, CANVAS_HEIGHT) as f64 / -2.0,
        // horizontal translation
        std::cmp::min(CANVAS_WIDTH, CANVAS_HEIGHT) as f64 / 2.0,
        // vertical translation
        std::cmp::min(CANVAS_WIDTH, CANVAS_HEIGHT) as f64 / 2.0,
    )?;
    let context_transform = context.get_transform()?;
    context.set_line_join("round");

    let control_data = std::rc::Rc::new(std::cell::RefCell::new(ControlData::default()));
    draw(&context, control_data.borrow().rotation)?;

    {
        let control_data = control_data.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mut control_data = control_data.borrow_mut();
            if event.button() == 0 {
                control_data.pressed = true;
                control_data.position = Position {
                    x: event.offset_x() as f64,
                    y: event.offset_y() as f64,
                };
                control_data.position_prev = control_data.position.clone();
            }
        });
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let control_data = control_data.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mut control_data = control_data.borrow_mut();
            if control_data.pressed {
                control_data.position = Position {
                    x: event.offset_x() as f64,
                    y: event.offset_y() as f64,
                };
            }
        });
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let control_data = control_data.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mut control_data = control_data.borrow_mut();
            if control_data.pressed {
                control_data.position = Position {
                    x: event.offset_x() as f64,
                    y: event.offset_y() as f64,
                };
            }
            control_data.pressed = false;
        });
        document.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Tranform from canvas coordinates to unit circle
    // coordinates by reversing the context transform
    let canvas_to_unit_coords = |x: f64, y: f64, reverse_transform: &DomMatrix| {
        (
            (x - reverse_transform.e()) / reverse_transform.a(),
            (y - reverse_transform.f()) / reverse_transform.d(),
        )
    };

    // Calculate the (positive) third coordinate value on
    // a unit sphere given the other two coordinate values
    let third_coord_val = |first: f64, second: f64| (1.0 - first * first - second * second).sqrt();

    let f = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        let mut control_data = control_data.borrow_mut();
        if control_data.position != control_data.position_prev {
            let (y, z) = canvas_to_unit_coords(
                control_data.position.x,
                control_data.position.y,
                &context_transform,
            );
            let x = third_coord_val(y, z);
            if !x.is_nan() {
                let (y_prev, z_prev) = canvas_to_unit_coords(
                    control_data.position_prev.x,
                    control_data.position_prev.y,
                    &context_transform,
                );
                let x_prev = third_coord_val(y_prev, z_prev);
                if !x_prev.is_nan() {
                    let (_, phi) = cartesian_to_unit_spherical(x, y, z);
                    let (_, phi_prev) = cartesian_to_unit_spherical(x_prev, y_prev, z_prev);

                    control_data.position_prev = control_data.position.clone();
                    control_data.rotation += phi - phi_prev;

                    draw(&context, control_data.rotation).unwrap();
                }
            }
        }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));
    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

/// Draw data onto the canvas.
fn draw(context: &CanvasRenderingContext2d, rotation: f64) -> Result<(), JsValue> {
    context.clear_rect(-1.0, -1.0, 2.0, 2.0);

    context.set_fill_style(&JsValue::from_str(SPHERE_FILL_STYLE));
    context.begin_path();
    context.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU)?;
    context.fill();

    for polyline in data::COASTLINE_POINTS {
        let mut prev_point = None;
        for point in *polyline {
            let (lon, lat) = point;
            let (x, y, z) = unit_spherical_to_cartesian(90.0 - lat, lon + rotation);
            if let Some((x_prev, y_prev, z_prev)) = prev_point {
                if x_prev < 0.0 || x < 0.0 {
                    context.set_line_width(COAST_BACK_LINE_WIDTH);
                    context.set_stroke_style(&JsValue::from_str(COAST_BACK_STROKE_STYLE));
                } else {
                    context.set_line_width(COAST_FRONT_LINE_WIDTH);
                    context.set_stroke_style(&JsValue::from_str(COAST_FRONT_STROKE_STYLE));
                }
                context.begin_path();
                context.move_to(y_prev, z_prev);
                context.line_to(y, z);
                context.stroke()
            }
            prev_point = Some((x, y, z));
        }
        context.stroke();
    }

    Ok(())
}

/// Convert unit radius spherical coordinates (degrees) to Cartesian coordinates.
fn unit_spherical_to_cartesian(theta: f64, phi: f64) -> (f64, f64, f64) {
    let (sin_theta, cos_theta) = theta.to_radians().sin_cos();
    let (sin_phi, cos_phi) = phi.to_radians().sin_cos();
    (sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
}

/// Convert Cartesian coordinates to unit radius spherical coordinates (degrees).
fn cartesian_to_unit_spherical(x: f64, y: f64, z: f64) -> (f64, f64) {
    (
        z.acos().to_degrees(),
        y.signum() * (x / (x * x + y * y).sqrt()).acos().to_degrees(),
    )
}
