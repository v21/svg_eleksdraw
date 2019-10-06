extern crate structopt;
extern crate usvg;

mod stripper;

use structopt::StructOpt;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();

    const xml_opt: usvg::XmlOptions = usvg::XmlOptions {
        use_single_quote: true,
        indent: usvg::XmlIndent::Spaces(4),
        attributes_indent: usvg::XmlIndent::Spaces(4),
    };
    let rtree = usvg::Tree::from_file(&args.path, &usvg::Options::default()).unwrap();
    let stripped_tree = stripper::strip(&rtree, xml_opt);
    print!("{}", rtree.to_string(xml_opt));
}
