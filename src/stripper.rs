extern crate kurbo;
extern crate usvg;

use kurbo::{CubicBez, Line, ParamCurve, ParamCurveArclen, Point};
use std::ops::Deref;
use usvg::Rect;
use xmlwriter::XmlWriter;

struct State {
    svg: xmlwriter::XmlWriter,
    gcode: String,
    pen_up: bool,
    current_svg_path: String,
    current_loc: Point,
    path_start: Option<(f64, f64)>,
    params: Params,
    view_box: Rect,
}

#[derive(Clone)]
pub struct Params {
    pub pen_up_height: f64,
    pub pen_down_height: f64,
    pub max_line_speed: f64,
}

pub fn strip(tree: &usvg::Tree, xml_opt: usvg::XmlOptions, params: &Params) -> (String, String) {
    let mut state = State {
        svg: XmlWriter::new(xml_opt),
        gcode: String::default(),
        current_svg_path: String::default(),
        pen_up: false,
        current_loc: Point::ORIGIN,
        params: (*params).clone(),
        path_start: None,
        view_box: tree.svg_node().view_box.rect.clone(),
    };

    state.svg.start_element("svg");

    state
        .svg
        .write_attribute("xmlns", "http://www.w3.org/2000/svg");

    let r = state.view_box;
    state.svg.write_attribute(
        "viewBox",
        &format!("{} {} {} {}", r.x(), r.y(), r.width(), r.height()),
    );
    state
        .svg
        .write_attribute("width", &tree.svg_node().size.width());
    state
        .svg
        .write_attribute("height", &tree.svg_node().size.height());

    state = check_bounds_and_pause(state);

    for child in tree.root().children() {
        match child.borrow().deref() {
            usvg::NodeKind::Path(ref path) => {
                //println!("{:?}, {}", path.data, path.transform);

                state.svg.start_element("path");
                state.svg.write_attribute("stroke-width", "0.1");
                state.svg.write_attribute("stroke", "black");
                state.svg.write_attribute("fill", "none");

                state.current_svg_path = String::default();
                let a = |&x, &y| path.transform.apply(x, y);

                for p in path.data.iter() {
                    match p {
                        usvg::PathSegment::MoveTo { x, y } => {
                            let (tx, ty) = a(x, y);
                            state = move_to(state, &tx, &ty);
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            let (tx, ty) = a(x, y);
                            state = line_to(state, &tx, &ty);
                        }
                        usvg::PathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            let (tx1, ty1) = a(x1, y1);
                            let (tx2, ty2) = a(x2, y2);
                            let (tx, ty) = a(x, y);
                            state = curve_to(state, &tx1, &ty1, &tx2, &ty2, &tx, &ty);
                        }
                        usvg::PathSegment::ClosePath => match state.path_start {
                            Some((x, y)) => state = line_to(state, &x, &y),
                            None => {}
                        },
                    }
                }

                state.path_start = None;

                state.svg.write_attribute("d", &state.current_svg_path);

                state.svg.end_element();
            }
            _ => {}
        }
    }

    state.gcode.push_str("M3 S0\n"); //full pen up
    state.gcode.push_str(&format!("G0 X0 Y0\n")); //return to origin

    return (state.svg.end_document(), state.gcode);
    //println!("{}\n{}", state.gcode, state.svg.end_document());
}

fn check_bounds_and_pause(mut state: State) -> State {
    let r = state.view_box;

    state.svg.start_element("path");
    state.svg.write_attribute("stroke", "red");
    state.svg.write_attribute("fill", "none");

    state.gcode.push_str("M3 S0\n"); //full pen up

    let mut path = String::new();

    path.push_str(&format!("M {} {} ", r.left(), r.top()));

    let mut do_corner = |x, y| {
        state.gcode.push_str(&format!("G0 X{} Y{}\n", x, y));

        path.push_str(&format!("L {} {} ", &x, &y));
    };

    do_corner(r.left(), r.top());
    do_corner(r.left(), r.bottom());
    do_corner(r.right(), r.bottom());
    do_corner(r.right(), r.top());
    do_corner(r.left(), r.top());

    state.gcode.push_str("G4 P3\n"); //wait

    state.svg.write_attribute("d", &path);

    state.svg.end_element();

    return state;
}

fn move_to<'a>(mut state: State, x: &f64, y: &f64) -> State {
    state = maybe_pen_up(state);

    let (x, y) = clamp_to_viewbox(&state, &x, &y);
    state.path_start = Some((x, y));
    state.gcode.push_str(&format!("G0 X{} Y{}\n", x, y));
    state.current_svg_path.push_str(&format!("M {} {} ", x, y));
    state.current_loc = Point::new(x, y);
    return state;
}

fn curve_to(mut state: State, x1: &f64, y1: &f64, x2: &f64, y2: &f64, x: &f64, y: &f64) -> State {
    // state = maybe_pen_down(state);

    let bez = CubicBez::new(
        state.current_loc,
        Point::new(*x1, *y1),
        Point::new(*x2, *y2),
        Point::new(*x, *y),
    );

    for l in bezier_to_lines(bez) {
        state = line_to(state, &l.end().x, &l.end().y);
    }

    // state.gcode.push_str(&format!(
    //     "G5 I{} J{} P{} Q{} X{} Y{}\n",
    //     x1, y1, x2, y2, x, y
    // ));
    // state
    //     .current_svg_path
    //     .push_str(&format!("C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y));

    // state.current_loc = Point::new(*x, *y);
    return state;
}

fn line_to(mut state: State, x: &f64, y: &f64) -> State {
    state = maybe_pen_down(state);

    let (x, y) = clamp_to_viewbox(&state, &x, &y);
    state.gcode.push_str(&format!(
        "G1 X{} Y{} F{}\n",
        x, y, &state.params.max_line_speed
    ));
    state
        .current_svg_path
        .push_str(&format!("L {} {} ", &x, &y));

    state.current_loc = Point::new(x, y);
    return state;
}

fn maybe_pen_up(mut state: State) -> State {
    if !state.pen_up {
        state
            .gcode
            .push_str(&format!("M3 S{}\n", &state.params.pen_up_height));
        state.pen_up = true;
    }
    return state;
}

fn maybe_pen_down(mut state: State) -> State {
    if state.pen_up {
        state
            .gcode
            .push_str(&format!("M3 S{}\n", &state.params.pen_down_height));
        state.pen_up = false;
    }
    return state;
}

fn bezier_to_lines(bez: CubicBez) -> Vec<Line> {
    let count = i64::max((bez.arclen(0.01).floor() * 10.) as i64, 4);
    let mut pos = bez.start();
    let mut lines: Vec<Line> = Vec::new();

    for i in 1..=count {
        let t = i as f64 / count as f64;
        let new_pos = bez.eval(t);
        lines.push(Line::new(pos, new_pos));
        pos = new_pos;
    }

    return lines;
}

fn clamp_to_viewbox(state: &State, x: &f64, y: &f64) -> (f64, f64) {
    (
        f64::min(state.view_box.right(), f64::max(state.view_box.left(), *x)),
        f64::min(state.view_box.bottom(), f64::max(state.view_box.top(), *y)),
    )
}

// fn recurse_strip(parent: &usvg::Node, xml: &mut XmlWriter) {
//     for n in parent.children() {
//         match *n.borrow() {}
//     }
// }
