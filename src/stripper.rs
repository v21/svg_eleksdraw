extern crate usvg;

use std::ops::Deref;
use xmlwriter::XmlWriter;

struct State {
    svg: xmlwriter::XmlWriter,
    gcode: String,
    pen_up: bool,
    current_svg_path: String,
}

pub fn strip(tree: &usvg::Tree, xml_opt: usvg::XmlOptions) {
    let mut state = State {
        svg: XmlWriter::new(xml_opt),
        gcode: String::default(),
        current_svg_path: String::default(),
        pen_up: false,
    };

    state.svg.start_element("svg");

    state
        .svg
        .write_attribute("xmlns", "http://www.w3.org/2000/svg");

    {
        let r = tree.svg_node().view_box.rect;
        state.svg.write_attribute(
            "viewBox",
            &format!("{} {} {} {}", r.x(), r.y(), r.width(), r.height()),
        );
    }
    state
        .svg
        .write_attribute("width", &tree.svg_node().size.width());
    state
        .svg
        .write_attribute("height", &tree.svg_node().size.height());

    for child in tree.root().children() {
        match child.borrow().deref() {
            usvg::NodeKind::Path(ref path) => {
                println!("{:?}, {}", path.data, path.transform);

                state.svg.start_element("path");
                state.svg.write_attribute("stroke", "black");
                state.svg.write_attribute("fill", "none");

                state.current_svg_path = String::default();

                for p in path.data.iter() {
                    match p {
                        usvg::PathSegment::MoveTo { x, y } => {
                            state = move_to(state, x, y);
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            state = line_to(state, x, y);
                        }
                        usvg::PathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            state = curve_to(state, x1, y1, x2, y2, x, y);
                        }
                        usvg::PathSegment::ClosePath => {}
                    }
                }

                state.svg.write_attribute("d", &state.current_svg_path);

                state.svg.end_element();
            }
            _ => {}
        }
    }
    println!("{}\n{}", state.gcode, state.svg.end_document());
}

fn move_to<'a>(mut state: State, x: &f64, y: &f64) -> State {
    state = maybe_pen_up(state);
    state.gcode.push_str(&format!("G0 X{} Y{}\n", x, y));
    state.current_svg_path.push_str(&format!("M {} {} ", x, y));
    return state;
}

fn curve_to(mut state: State, x1: &f64, y1: &f64, x2: &f64, y2: &f64, x: &f64, y: &f64) -> State {
    state = maybe_pen_down(state);
    state.gcode.push_str(&format!(
        "G5 I{} J{} P{} Q{} X{} Y{}\n",
        x1, y1, x2, y2, x, y
    ));
    state
        .current_svg_path
        .push_str(&format!("C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y));
    return state;
}

fn line_to(mut state: State, x: &f64, y: &f64) -> State {
    state = maybe_pen_down(state);
    state.gcode.push_str(&format!("G1 X{} Y{} F10000\n", x, y));
    state.current_svg_path.push_str(&format!("L {} {} ", x, y));
    return state;
}

fn maybe_pen_up(mut state: State) -> State {
    if !state.pen_up {
        state.gcode.push_str("M3 S0\n");
        state.pen_up = true;
    }
    return state;
}

fn maybe_pen_down(mut state: State) -> State {
    if state.pen_up {
        state.gcode.push_str("M3 S100\n");
        state.pen_up = false;
    }
    return state;
}

// fn recurse_strip(parent: &usvg::Node, xml: &mut XmlWriter) {
//     for n in parent.children() {
//         match *n.borrow() {}
//     }
// }
