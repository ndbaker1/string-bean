use std::fs::File;
use std::io::Write;
use std::path::Path;

use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    input_file: String,
    output_file: String,
    #[clap(short = 'c', default_value_t = 500)]
    line_count: usize,
    #[clap(short = 'o', default_value_t = 0.2)]
    line_opacity: f64,
    #[arg(short = 'a', default_value_t = 288)]
    anchor_count: u64,
    #[arg(short = 'g', default_value_t = 0)]
    anchor_gap_count: usize,
    #[arg(short = 'r', default_value_t = usize::MAX)]
    radius: usize,
    #[arg(short = 'p', default_value_t = 5.0)]
    penalty: f64,

    #[arg(default_value_t = 850)]
    width: u64,
    #[arg(default_value_t = 850)]
    height: u64,
}

fn main() -> Result<(), std::io::Error> {
    let args = CliArgs::parse();

    let img = image::open(&args.input_file).unwrap().to_luma8();
    let width = img.width() as usize;
    let height = img.height() as usize;

    let (x_mid, y_mid) = (width / 2, height / 2);
    let radius = args.radius.min(x_mid.min(y_mid)) as f64;

    let anchors: Vec<_> = (0..args.anchor_count)
        .map(|anchor| anchor as f64 * 2.0 * std::f64::consts::PI / args.anchor_count as f64)
        .map(|angle| {
            (
                x_mid as f64 + radius * angle.cos(),
                y_mid as f64 + radius * angle.sin(),
            )
        })
        .collect();

    let mut planner = string_bean::ThreadPlanner::new(
        args.line_opacity,
        &anchors,
        args.anchor_gap_count,
        args.penalty,
        grid_raytrace,
        width,
        height,
        &img.into_vec(),
    );

    let anchors = planner.get_moves(0, args.line_count).unwrap();

    write_svg(&args, &anchors)?;

    Ok(())
}

fn write_svg(args: &CliArgs, anchors: &[usize]) -> Result<(), std::io::Error> {
    let (x_mid, y_mid) = (args.width as f64 / 2.0, args.height as f64 / 2.0);
    let radius = x_mid.min(y_mid);
    let degrees_per_anchor: f64 = 2.0 * std::f64::consts::PI / args.anchor_count as f64;

    let mut svg_file = File::create(&Path::new(&args.output_file).with_extension("svg"))?;

    writeln!(
        svg_file,
        "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
        args.width, args.height
    )?;

    for anchor_pairs in anchors.windows(2) {
        let &[anchor1, anchor2] = anchor_pairs else { panic!("bad window size") };
        let (deg1, deg2) = (
            degrees_per_anchor * anchor1 as f64,
            degrees_per_anchor * anchor2 as f64,
        );
        let (x0, y0) = (x_mid + radius * deg1.cos(), y_mid + radius * deg1.sin());
        let (x1, y1) = (x_mid + radius * deg2.cos(), y_mid + radius * deg2.sin());
        writeln!(
            svg_file,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" opacity=\"{}\" style=\"stroke:rgb(0,0,0); stroke-width:1\" />",
            x0, y0, x1, y1, args.line_opacity
        )?;
    }

    writeln!(svg_file, "</svg>")?;

    Ok(())
}

/// https://playtechs.blogspot.com/2007/03/raytracing-on-grid.html
fn grid_raytrace(
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) -> impl Iterator<Item = ((usize, usize), f64)> {
    let (x0, y0) = (x0 as i64, y0 as i64);
    let (x1, y1) = (x1 as i64, y1 as i64);

    let mut dx = (x1 - x0).abs();
    let mut dy = (y1 - y0).abs();
    let mut x = x0;
    let mut y = y0;

    let n = 1 + dx + dy;
    let x_inc = (x1 - x0).signum();
    let y_inc = (y1 - y0).signum();

    let mut error = dx - dy;
    dx *= 2;
    dy *= 2;

    (0..n).map(move |_| {
        let point = ((x as usize, y as usize), 1.0);

        if error > 0 {
            x += x_inc;
            error -= dy;
        } else {
            y += y_inc;
            error += dx;
        }

        point
    })
}
