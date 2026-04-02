mod data; // generated during build

use wasm_bindgen::prelude::*;

const CANVAS_WIDTH: u32 = 800;
const CANVAS_HEIGHT: u32 = 800;

const BACKGROUND: &str = "black";
const SPHERE_FILL_STYLE: &str = "rgba(0, 0, 255, 0.25)";
const COASTLINE_FRONT_STROKE_STYLE: &str = "rgb(63, 127, 63)";
const COASTLINE_BACK_STROKE_STYLE: &str = "rgba(63, 127, 63, 0.5)";
const COASTLINE_FRONT_LINE_WIDTH: f64 = 0.005;
const COASTLINE_BACK_LINE_WIDTH: f64 = 0.0025;

const INITIAL_LONGITUDE: f64 = 103.0;

#[derive(Debug, Default, Clone, PartialEq)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Debug, Default)]
struct ControlData {
    pressed: bool,
    position: Position,
    prev_position: Position,
}

fn window() -> web_sys::Window {
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
    let body = document.body().expect("should have body");

    body.style().set_property("background", BACKGROUND)?;

    // create canvas
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    canvas.set_width(CANVAS_WIDTH);
    canvas.set_height(CANVAS_HEIGHT);
    canvas.style().set_property("touch-action", "pan-y")?; // over browser (i.e. "auto") touch behaviour
    body.append_child(&canvas)?;

    let control_data = std::rc::Rc::new(std::cell::RefCell::new(ControlData::default()));

    // handle pointer down
    {
        let control_data = control_data.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::PointerEvent| {
            let mut control_data = control_data.borrow_mut();
            control_data.pressed = true;
            control_data.position = Position {
                x: event.offset_x() as f64,
                y: event.offset_y() as f64,
            };
            control_data.prev_position = control_data.position.clone();
        });
        canvas.add_event_listener_with_callback("pointerdown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // handle pointer up
    {
        let control_data = control_data.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::PointerEvent| {
            let mut control_data = control_data.borrow_mut();
            control_data.pressed = false;
            control_data.position = Position {
                x: event.offset_x() as f64,
                y: event.offset_y() as f64,
            };
        });
        document.add_event_listener_with_callback("pointerup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // handle pointer move
    {
        let control_data = control_data.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::PointerEvent| {
            let mut control_data = control_data.borrow_mut();
            if control_data.pressed {
                control_data.position = Position {
                    x: event.offset_x() as f64,
                    y: event.offset_y() as f64,
                };
                event.prevent_default();
            }
        });
        canvas.add_event_listener_with_callback("pointermove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    let context = canvas
        .get_context("2d")?
        .expect("should have 2d context")
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    // set canvas context transform to fit unit circle to canvas
    {
        let size = CANVAS_WIDTH.min(CANVAS_HEIGHT) as f64;
        context.set_transform(
            size / 2.0, // horizontal scaling
            0.0,
            0.0,
            -size / 2.0, // vertical scaling; upward increasing
            size / 2.0,  // horizontal translation
            size / 2.0,  // vertical translation
        )?;
    }

    // closure to convert canvas coordinates to unit circle coordinates
    let transform = context.get_transform()?;
    let canvas_to_unit_coords = move |x: f64, y: f64| {
        let point = web_sys::DomPointInit::new();
        point.set_x(x);
        point.set_y(y);
        let point = transform.inverse().transform_point_with_point(&point);
        (
            (1.0 - point.x() * point.x() - point.y() * point.y()).sqrt(),
            point.x(),
            point.y(),
        )
    };

    context.set_line_join("round");

    let mut rotation = -INITIAL_LONGITUDE.rem_euclid(360.0);
    draw(&context, rotation)?;

    let f = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        let mut control_data = control_data.borrow_mut();
        if control_data.position != control_data.prev_position {
            let (x, y, z) = canvas_to_unit_coords(control_data.position.x, control_data.position.y);
            if !x.is_nan() {
                let (prev_x, prev_y, prev_z) = canvas_to_unit_coords(
                    control_data.prev_position.x,
                    control_data.prev_position.y,
                );
                if !prev_x.is_nan() {
                    let (azi, _) = cartesian_to_unit_spherical(x, y, z);
                    let (azi_prev, _) = cartesian_to_unit_spherical(prev_x, prev_y, prev_z);

                    control_data.prev_position = control_data.position.clone();
                    rotation += (azi - azi_prev).rem_euclid(360.0);

                    draw(&context, rotation).unwrap();
                }
            }
        }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));
    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn draw(context: &web_sys::CanvasRenderingContext2d, rotation: f64) -> Result<(), JsValue> {
    context.clear_rect(-1.0, -1.0, 2.0, 2.0);

    // sphere
    context.set_fill_style_str(SPHERE_FILL_STYLE);
    context.begin_path();
    context.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU)?;
    context.fill();

    // coastline
    for &polyline in data::COASTLINE {
        let mut prev = None;
        for &(lon, lat) in polyline {
            let (x, y, z) = ll_to_unit_cartesian(lat, lon, rotation);
            if let Some((prev_x, prev_y, prev_z)) = prev {
                if prev_x < 0.0 || x < 0.0 {
                    context.set_line_width(COASTLINE_BACK_LINE_WIDTH);
                    context.set_stroke_style_str(COASTLINE_BACK_STROKE_STYLE);
                } else {
                    context.set_line_width(COASTLINE_FRONT_LINE_WIDTH);
                    context.set_stroke_style_str(COASTLINE_FRONT_STROKE_STYLE);
                }
                context.begin_path();
                context.move_to(prev_y, prev_z);
                context.line_to(y, z);
                context.stroke()
            }
            prev = Some((x, y, z));
        }
        context.stroke();
    }

    Ok(())
}

/// Convert latitude, longitude and rotation in degrees to Cartesian coordinates on unit sphere.
fn ll_to_unit_cartesian(lat: f64, lon: f64, rotation: f64) -> (f64, f64, f64) {
    let (azimuth, polar) = (lon + rotation, 90.0 - lat);
    let (sin_azimuth, cos_azimuth) = azimuth.to_radians().sin_cos();
    let (sin_polar, cos_polar) = polar.to_radians().sin_cos();
    (cos_azimuth * sin_polar, sin_azimuth * sin_polar, cos_polar)
}

/// Convert Cartesian coordinates to spherical coordinate azimuth and polar angles (degrees).
fn cartesian_to_unit_spherical(x: f64, y: f64, z: f64) -> (f64, f64) {
    (
        y.signum() * (x / (x * x + y * y).sqrt()).acos().to_degrees(),
        z.acos().to_degrees(),
    )
}
