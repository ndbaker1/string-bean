use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
pub fn plan_as_json(
    line_count: u32,
    line_opacity: f32,
    anchor_count: u32,
    anchor_gap_count: usize,
    radius: usize,
    penalty: f32,
    width: usize,
    height: usize,
    image_buffer: &[u8],
    start_anchor: usize,
) -> JsValue {
    let (x_mid, y_mid) = (width / 2, height / 2);
    let radius = radius.min(x_mid.min(y_mid)) as f64;

    let anchors: Vec<_> = (0..anchor_count)
        .map(|anchor| anchor as f64 * 2.0 * std::f64::consts::PI / anchor_count as f64)
        .map(|angle| {
            (
                x_mid as f64 + radius * angle.cos(),
                y_mid as f64 + radius * angle.sin(),
            )
        })
        .collect();

    let mut planner = string_bean::ThreadPlanner::new(
        line_opacity as _,
        &anchors,
        anchor_gap_count,
        penalty as _,
        grid_raytrace,
        width,
        height,
        image_buffer,
    );

    let moves = planner
        .get_moves(start_anchor, line_count as _)
        .unwrap_or(Vec::new());

    JsValue::from_str(&format!(
        "[{}]",
        moves
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(","),
    ))
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
