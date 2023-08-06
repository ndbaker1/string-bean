type Precision = f64;
type FPos = (Precision, Precision);
type IPos = (usize, usize);
type PixelIntensity = (IPos, Precision);

/// Obtain a set of anchor moves for a thread art image using a custom configuration
pub struct ThreadPlanner<'a, I, S>
where
    I: IntoIterator<Item = PixelIntensity>,
    S: Fn(Precision, Precision, Precision, Precision) -> I,
{
    /// weight of line as pixel opacity between 0 and 1
    line_weight: Precision,
    /// set of anchor coordinates to use when drawing lines
    anchors: &'a [FPos],
    /// minimum number of anchors to branch out on the right and left each planning step
    anchor_gap_count: usize,
    /// weighting factor for negative penalties during planning
    lightness_penalty: Precision,
    /// line drawing algorithm returning a collection of
    /// pixel-intensity pairs that can be turned into an iterator
    line_algorithm: S,
    image_width: usize,
    image_height: usize,
    /// inverted grayscale image mask used to compute pentalties and
    /// persist pixel changes during thread planning
    image_mask_inverted: Vec<Precision>,
}

impl<'a, I, S> ThreadPlanner<'a, I, S>
where
    I: IntoIterator<Item = PixelIntensity>,
    S: Fn(Precision, Precision, Precision, Precision) -> I,
{
    pub fn new(
        line_weight: Precision,
        anchors: &'a [FPos],
        anchor_gap_count: usize,
        lightness_penalty: Precision,
        line_algorithm: S,
        image_width: usize,
        image_height: usize,
        image_mask: &[u8],
    ) -> Self {
        assert!(
            (0.0..1.0).contains(&line_weight),
            "line weight needs to be in the range [0,1]"
        );
        // convert line weight into the normalized u8 pixel range
        let line_weight: Precision = u8::MAX as Precision * line_weight;

        let image_mask_inverted: Vec<_> = image_mask
            .iter()
            .map(|v| u8::MAX - v)
            .map(|v| v as Precision)
            .collect();

        // TODO: either trust the user or perform a runtime check that the points form a convex polygon

        Self {
            line_weight,
            anchor_gap_count,
            anchors,
            image_width,
            image_height,
            image_mask_inverted,
            lightness_penalty,
            line_algorithm,
        }
    }

    /// Get the sequence of anchor moves to recreate the image using thread art
    pub fn get_moves(&mut self, start_anchor: usize, line_count: usize) -> Result<Vec<usize>, ()> {
        let mut anchor = start_anchor;
        let mut anchor_order = Vec::with_capacity(line_count);

        anchor_order.push(start_anchor);

        for _ in 0..line_count {
            let next_anchor = self.next_anchor(anchor).ok_or(())?;

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
        let search_size = self.anchors.len() - 2 * self.anchor_gap_count - 1;

        (0..search_size)
            .map(|i| (current + i + self.anchor_gap_count + 1).rem_euclid(self.anchors.len()))
            .map(|next| {
                (
                    next,
                    self.penalty(self.anchors[current], self.anchors[next]),
                )
            })
            .min_by(|x, y| x.1.total_cmp(&y.1))
            .map(|(i, _)| i)
    }

    /// Apply changes from a line, persisting the pixel changes to the image
    fn apply_line(&mut self, src: FPos, dst: FPos) {
        for ((x, y), intensity) in self.trace_line(src, dst) {
            self.image_mask_inverted[x + y * self.image_height] -= intensity * self.line_weight;
        }
    }

    /// Compute the penalty for a line on the image.
    /// Penalty is weighted based on an average for each pixel touched by the line.
    fn penalty(&self, src: FPos, dst: FPos) -> Precision {
        let line = self.trace_line(src, dst);

        if line.is_empty() {
            return Precision::NEG_INFINITY;
        }

        let line_length: Precision = line.len() as _;
        let line_penalty: Precision = line
            .into_iter()
            .map(|((x, y), intensity)| {
                self.image_mask_inverted[x + y * self.image_height] - intensity * self.line_weight
            })
            .map(|p| match p < 0.0 {
                true => -self.lightness_penalty * p,
                false => p,
            })
            .sum();

        // return the average penalty for each point in the line
        line_penalty / line_length
    }

    /// Gets line from source to destination with additional bounds checks
    fn trace_line(&self, src: FPos, dst: FPos) -> Vec<PixelIntensity> {
        (self.line_algorithm)(src.0, src.1, dst.0, dst.1)
            .into_iter()
            .filter(|((x, y), _)| *x < self.image_width && *y < self.image_height)
            .filter(|((x, y), _)| *x + *y * self.image_height < self.image_mask_inverted.len())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// https://playtechs.blogspot.com/2007/03/raytracing-on-grid.html
    fn grid_raytrace(
        x0: Precision,
        y0: Precision,
        x1: Precision,
        y1: Precision,
    ) -> impl Iterator<Item = PixelIntensity> {
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

    /// test use case for the ThreadPlanner
    #[test]
    fn main_test() {
        const RADIUS: usize = 200;

        const NUM_ANCHORS: usize = 100;
        let anchors: Vec<_> = (0..NUM_ANCHORS)
            .map(|anchor| {
                anchor as Precision * 2.0 * std::f64::consts::PI / NUM_ANCHORS as Precision
            })
            .map(|angle| {
                (
                    RADIUS as Precision * (1.0 + angle.cos()),
                    RADIUS as Precision * (1.0 + angle.sin()),
                )
            })
            .collect();

        let mut planner = ThreadPlanner::new(
            0.5,
            &anchors,
            10,
            1.0,
            grid_raytrace,
            RADIUS,
            RADIUS,
            &vec![0; RADIUS * RADIUS],
        );

        let anchors = planner.get_moves(0, 20).unwrap();
        for anchor in anchors {
            println!("{}", anchor);
        }
    }

    /// simple case for line tracing in the image pixel grid to be the same in both directions.
    #[test]
    fn line_test() {
        let p1 = (2.0, 5.0);
        let p2 = (6.0, 8.0);

        let planner = ThreadPlanner::new(
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            grid_raytrace,
            10,
            10,
            Default::default(),
        );

        let actual_line = planner.trace_line(p1, p2);
        let mut expected_line = vec![
            ((2, 5), 1.0),
            ((3, 5), 1.0),
            ((3, 6), 1.0),
            ((4, 6), 1.0),
            ((4, 7), 1.0),
            ((5, 7), 1.0),
            ((5, 8), 1.0),
            ((6, 8), 1.0),
        ];

        assert_eq!(expected_line, actual_line);

        let actual_line = planner.trace_line(p2, p1);
        expected_line.reverse();

        assert_eq!(expected_line, actual_line);
    }
}
