use string_bean::ThreadPlanner;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let input_file = &args[1];

    let img = image::open(input_file).unwrap().to_luma8();
    let width = img.width() as usize;
    let height = img.height() as usize;
    let mut image_mask = img.into_vec();

    const NUM_CHORDS: u64 = 3000;
    const NUM_ANCHORS: u64 = 288;
    const LINE_OPACITY: f64 = 0.15;

    let mut planner = ThreadPlanner::new(
        NUM_CHORDS,
        LINE_OPACITY,
        NUM_ANCHORS,
        NUM_ANCHORS as usize / 10,
        usize::MAX,
        100.0,
        width,
        height,
        &mut image_mask,
    );

    let anchors = planner.get_moves(0).unwrap();

    let width = 850.0;
    let height = 850.0;

    let (x_mid, y_mid) = (width as f64 / 2.0, height as f64 / 2.0);
    let radius = x_mid.min(y_mid);

    println!(
        r#"<!DOCTYPE html><html>
        <body>
            <svg width="{}" height="{}">
        "#,
        width, height
    );

    for anchor_pairs in anchors.windows(2) {
        const DEGREE_PER_ANCHOR: f64 = 2.0 * std::f64::consts::PI / NUM_ANCHORS as f64;

        let &[anchor1, anchor2] = anchor_pairs else { panic!("bad window") };
        let (deg1, deg2) = (
            DEGREE_PER_ANCHOR * anchor1 as f64,
            DEGREE_PER_ANCHOR * anchor2 as f64,
        );
        let (x1, y1) = (x_mid + radius * deg1.cos(), y_mid + radius * deg1.sin());
        let (x2, y2) = (x_mid + radius * deg2.cos(), y_mid + radius * deg2.sin());
        println!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" opacity="{}" style="stroke:rgb(0,0,0); stroke-width:1" />"#,
            x1, y1, x2, y2, LINE_OPACITY
        );
    }

    println!(
        r#"
            </svg>
        </body>
        </html>
        "#
    );
}
