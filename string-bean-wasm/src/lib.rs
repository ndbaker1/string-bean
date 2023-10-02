use string_bean::core::{grid_raytrace, CountTracker};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

type ReturnArray = js_sys::Array;

#[wasm_bindgen(js_name = planMoves)]
pub fn plan_moves(
    line_count: u32,
    line_opacity: f32,
    anchor_list: &[f64],
    anchor_gap_count: usize,
    penalty: f32,
    width: usize,
    height: usize,
    image_buffer: &[u8],
    start_anchor: usize,
) -> ReturnArray {
    let anchors = anchor_list
        .chunks_exact(2)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect::<Vec<_>>();

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

    planner
        .get_moves(start_anchor, CountTracker(line_count))
        .unwrap_or(Vec::new())
        .into_iter()
        .map(JsValue::from)
        .collect()
}
