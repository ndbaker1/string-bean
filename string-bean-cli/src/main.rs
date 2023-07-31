use std::fs::File;
use std::io::Write;

use clap::Parser;
use image;

use string_bean;

#[derive(Parser)]
struct CliArgs {
    input_file: String,
    output_file: String,
    #[clap(short = 'c', default_value_t = 500)]
    num_chords: u64,
    #[clap(short = 'o', default_value_t = 0.2)]
    line_opacity: f64,
    #[arg(short = 'a', default_value_t = 288)]
    num_anchors: u64,
    #[arg(short = 'g', default_value_t = 0)]
    num_anchor_gap: usize,
    #[arg(short = 'r', default_value_t = usize::MAX)]
    radius: usize,
    #[arg(short = 'p', default_value_t = 5.0)]
    penalty: f64,

    #[arg(short = 'w', default_value_t = 850)]
    output_width: u64,
    #[arg(short = 'h', default_value_t = 850)]
    output_height: u64,
}

fn main() -> Result<(), std::io::Error> {
    let args = CliArgs::parse();

    let img = image::open(&args.input_file).unwrap().to_luma8();
    let width = img.width() as usize;
    let height = img.height() as usize;

    let mut planner = string_bean::ThreadPlanner::new(
        args.num_chords,
        args.line_opacity,
        args.num_anchors,
        args.num_anchor_gap,
        args.radius,
        args.penalty,
        width,
        height,
        &mut img.into_vec(),
    );

    let anchors = planner.get_moves(0).unwrap();

    write_svg(&args, &anchors)?;

    Ok(())
}

fn write_svg(args: &CliArgs, anchors: &Vec<usize>) -> Result<(), std::io::Error> {
    let (x_mid, y_mid) = (
        args.output_width as f64 / 2.0,
        args.output_height as f64 / 2.0,
    );
    let radius = x_mid.min(y_mid);
    let degrees_per_anchor: f64 = 2.0 * std::f64::consts::PI / args.num_anchors as f64;

    let mut svg_file = File::create(&args.output_file)?;

    write!(
        svg_file,
        "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">\n",
        args.output_width, args.output_height
    )?;

    for anchor_pairs in anchors.windows(2) {
        let &[anchor1, anchor2] = anchor_pairs else { panic!("bad window size") };
        let (deg1, deg2) = (
            degrees_per_anchor * anchor1 as f64,
            degrees_per_anchor * anchor2 as f64,
        );
        let (x0, y0) = (x_mid + radius * deg1.cos(), y_mid + radius * deg1.sin());
        let (x1, y1) = (x_mid + radius * deg2.cos(), y_mid + radius * deg2.sin());
        write!(
            svg_file,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" opacity=\"{}\" style=\"stroke:rgb(0,0,0); stroke-width:1\" />\n",
            x0, y0, x1, y1, args.line_opacity
        )?;
    }

    write!(svg_file, "</svg>")?;

    Ok(())
}
