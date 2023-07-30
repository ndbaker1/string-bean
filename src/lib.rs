use std::{cmp::min, f64::consts::PI};

type FPos = (f64, f64);
type IPos = (usize, usize);

#[derive(Default)]
pub struct ThreadPlanner {
    num_chords: u64,
    chord_weight: f64,
    num_anchor_gap: usize,
    anchors: Vec<FPos>,
    image_width: usize,
    image_height: usize,
    inverted_mask: Vec<f64>,
    /// parameter to adjust for weight that negative penalties
    /// contribute to the overall penalty score.
    lightness_penalty: f64,
}

impl ThreadPlanner {
    pub fn new(
        num_chords: u64,
        chord_weight: f64,
        num_anchors: u64,
        num_anchor_gap: usize,
        radius: usize,
        penalty: f64,
        image_width: usize,
        image_height: usize,
        image_mask: &[u8],
    ) -> Self {
        let (x_mid, y_mid) = (image_width / 2, image_height / 2);
        // radius is the smallest value that fits in the image space
        let radius = min(radius, min(x_mid, y_mid)) as f64;

        let anchors = (0..num_anchors)
            .map(|anchor| anchor as f64 * 2.0 * PI / num_anchors as f64)
            .map(|angle| {
                (
                    x_mid as f64 + radius * angle.cos(),
                    y_mid as f64 + radius * angle.sin(),
                )
            })
            .collect();

        assert!(chord_weight <= 1.0 && chord_weight >= -1.0);

        let chord_weight = u8::MAX as f64 * chord_weight;
        let image_mask: Vec<_> = image_mask
            .iter()
            .map(|v| u8::MAX - v)
            .map(|v| v as f64)
            .collect();

        Self {
            num_chords,
            chord_weight,
            num_anchor_gap,
            anchors,
            image_width,
            image_height,
            inverted_mask: image_mask,
            lightness_penalty: penalty,
        }
    }

    /// Get the sequence of anchor moves to recreate the image using thread art
    pub fn get_moves(&mut self, start_anchor: usize) -> Result<Vec<usize>, ()> {
        let mut anchor = start_anchor;
        let mut anchor_order = Vec::with_capacity(self.num_chords as usize);

        anchor_order.push(start_anchor);

        for _ in 0..self.num_chords {
            let next_anchor = self.next_anchor(anchor).ok_or(())?;
            eprintln!("{anchor} -> {next_anchor}");

            self.apply_line(self.anchors[anchor], self.anchors[next_anchor]);

            anchor = next_anchor;
            anchor_order.push(anchor);
        }

        Ok(anchor_order)
    }

    /// Finds the next thread anchor on the perimeter based on the current
    fn next_anchor(&self, current: usize) -> Option<usize> {
        // basically ignore `self.num_anchor_gap` on both sides of the anchor while
        // searching the remaining anchor search space
        let search_size = self.anchors.len() - 2 * self.num_anchor_gap - 1;

        (0..search_size)
            .map(|i| (current + i + self.num_anchor_gap + 1).rem_euclid(self.anchors.len()))
            .map(|next| {
                (
                    next,
                    self.line_penalty(self.anchors[current], self.anchors[next]),
                )
            })
            .min_by(|x, y| x.1.total_cmp(&y.1))
            .map(|(i, _)| i)
    }

    /// Apply changes from a line, persisting the pixel changes to the image
    fn apply_line(&mut self, src: FPos, dst: FPos) {
        for (x, y) in self.grid_raytrace(src, dst) {
            self.inverted_mask[x + y * self.image_height] -= self.chord_weight;
        }
    }

    /// Compute the penalty for a line on the image.
    /// Penalty is weighted based on an average for each pixel touched by the line.
    fn line_penalty(&self, src: FPos, dst: FPos) -> f64 {
        let line = self.grid_raytrace(src, dst);

        if line.is_empty() {
            return f64::NEG_INFINITY;
        }

        let total_penalty: f64 = line
            .iter()
            .map(|(x, y)| self.inverted_mask[x + y * self.image_height] - self.chord_weight)
            .map(|p| match p < 0.0 {
                true => -self.lightness_penalty * p,
                false => p,
            })
            .sum();

        // return the average penalty for each point in the line
        total_penalty / line.len() as f64
    }

    /// https://playtechs.blogspot.com/2007/03/raytracing-on-grid.html
    fn grid_raytrace(&self, src: FPos, dst: FPos) -> Vec<IPos> {
        let (x0, y0) = (src.0 as i64, src.1 as i64);
        let (x1, y1) = (dst.0 as i64, dst.1 as i64);

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

        (0..n)
            .map(|_| {
                let point = (x as usize, y as usize);

                if error > 0 {
                    x += x_inc;
                    error -= dy;
                } else {
                    y += y_inc;
                    error += dx;
                }

                point
            })
            .filter(|(x, y)| *x < self.image_width && *y < self.image_height)
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// test use case for the ThreadPlanner
    #[test]
    fn main_test() {
        let width = 200;
        let height = 200;
        let mut image_mask = vec![0; width * height];
        let mut planner =
            ThreadPlanner::new(20, 0.5, 360, 10, 100, 1.0, width, height, &mut image_mask);

        let anchors = planner.get_moves(0).unwrap();
        for anchor in anchors {
            println!("{}", anchor);
        }
    }

    /// simple case for line tracing in the image pixel grid to be the same in both directions.
    #[test]
    fn line_test() {
        let mut planner = ThreadPlanner::default();
        planner.image_width = 10;
        planner.image_height = 10;

        let p1 = (2.0, 5.0);
        let p2 = (6.0, 8.0);

        let actual_line = planner.grid_raytrace(p1, p2);
        let mut expected_line = vec![
            (2, 5),
            (3, 5),
            (3, 6),
            (4, 6),
            (4, 7),
            (5, 7),
            (5, 8),
            (6, 8),
        ];

        assert_eq!(expected_line, actual_line);

        let actual_line = planner.grid_raytrace(p2, p1);
        expected_line.reverse();

        assert_eq!(expected_line, actual_line);
    }
}
