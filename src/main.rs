extern crate structopt;
extern crate usvg;

mod stripper;

use std::fs::File;
use std::io::prelude::*;
use std::option::Option;
use structopt::StructOpt;

/// Convert SVG file to G-code suitable for plotting with an Eleksdraw
#[derive(StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    paths: Vec<std::path::PathBuf>,

    #[structopt(parse(from_os_str), short = "o", long = "output-dir")]
    output_dir: Option<std::path::PathBuf>,

    /// the value to send for M3 commands when lifting the pen
    #[structopt(
        short = "u",
        long = "pen-up",
        default_value = "0",
        env = "PEN_UP_HEIGHT"
    )]
    pen_up_height: f64,

    /// the value to send for M3 commands when dropping the pen
    #[structopt(
        short = "d",
        long = "pen-down",
        default_value = "100",
        env = "PEN_DOWN_HEIGHT"
    )]
    pen_down_height: f64,

    /// the value to send for G1 commands when dropping the pen
    #[structopt(
        short = "s",
        long = "max-speed",
        default_value = "10000",
        env = "MAX_G1_SPEED"
    )]
    max_line_speed: f64,
}

fn main() {
    let args = Cli::from_args();

    const XML_OPT: usvg::XmlOptions = usvg::XmlOptions {
        use_single_quote: true,
        indent: usvg::XmlIndent::Spaces(4),
        attributes_indent: usvg::XmlIndent::Spaces(4),
    };

    let params = stripper::Params {
        pen_up_height: args.pen_up_height,
        pen_down_height: args.pen_down_height,
        max_line_speed: args.max_line_speed,
    };

    for p in args.paths {
        let rtree = usvg::Tree::from_file(&p, &usvg::Options::default()).unwrap();
        let (stripped_svg, gcode) = stripper::strip(&rtree, XML_OPT, &params);

        let out_dir = args
            .output_dir
            .clone()
            .unwrap_or(p.parent().unwrap().to_path_buf());

        let mut stripped_svg_path = out_dir.clone();
        stripped_svg_path.push(p.with_extension("stripped.svg").file_name().unwrap());
        File::create(&stripped_svg_path)
            .unwrap()
            .write_all(stripped_svg.as_bytes())
            .unwrap();

        let mut gcode_path = out_dir.clone();
        gcode_path.push(p.with_extension("gcode").file_name().unwrap());
        File::create(&gcode_path)
            .unwrap()
            .write_all(gcode.as_bytes())
            .unwrap();
    }

    //print!("{}", rtree.to_string(xml_opt));
}
