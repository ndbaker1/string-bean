use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
pub fn plan_as_json(
    num_chords: u32,
    line_opacity: f32,
    num_anchors: u32,
    num_anchors_gap: usize,
    radius: usize,
    penalty: f32,
    width: usize,
    height: usize,
    image_buffer: &[u8],
    start_anchor: usize,
) -> JsValue {
    let mut planner = string_bean::ThreadPlanner::new(
        num_chords as _,
        line_opacity as _,
        num_anchors as _,
        num_anchors_gap as _,
        radius,
        penalty as _,
        width,
        height,
        image_buffer,
    );

    let moves = planner.get_moves(start_anchor).unwrap_or(Vec::new());

    JsValue::from_str(&format!(
        "[{}]",
        moves
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(","),
    ))
}
