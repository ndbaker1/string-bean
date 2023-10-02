type Precision = f64;
type FPos = (Precision, Precision);
type IPos = (usize, usize);
type PixelIntensity = (IPos, Precision);

/// A strategy for determining when to terminate the thread planner
pub trait PlanningStrategy {
    fn completed(&mut self) -> bool;
}

/// Obtain a set of anchor moves for a thread art image using a custom configuration
///
/// # Examples
///
/// ```
/// use string_bean::PlanningStrategy;
/// // create anchors to form any kind of convex polygon
/// let anchors = [(0.0, 0.0), (1.0, 1.0), (0.0, 2.0)];
/// // obtain image data and metadata
/// let (width, height) = (10, 10);
/// let image_mask = vec![255; width * height];
/// // implement your own line algorithm, use an example from this crate (grid_raytacer),
/// // or use something from https://docs.rs/line_drawing/latest/line_drawing/
/// let line_algorithm = |x0, y0, x1, y1| [];
/// // use the ThreadPlanner with configuration to create an instance
/// // that can output any number of anchor stops and start from any anchor
/// let mut planner = string_bean::ThreadPlanner::new(
///     0.2,
///     &anchors,
///     0,
///     1.0,
///     line_algorithm,
///     width,
///     height,
///     &image_mask
/// );
/// // implement a strategy to run the planner for a set number of iterations
/// struct CountPlanner {
///     target: usize,
///     count: usize,
/// }
/// impl PlanningStrategy for CountPlanner {
///     fn completed(&mut self) -> bool {
///         self.count += 1;
///         self.count > self.target
///     }
/// }
/// // compute any number of moves using a start anchor
/// let start_anchor = 0;
/// let anchors = planner.get_moves(start_anchor, CountPlanner { count: 0, target: 5 }).unwrap();
/// ```
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
    /// Contructs a thread art planner
    ///
    /// * `line_weight` - weight of line as pixel opacity between 0 and 1
    /// * `anchors` - a slice to a set of float coordinate pairs making a convex polygon
    /// * `anchor_gap_count` - number of spaces to leave between consecutive anchors
    /// * `lightness_penalty` - penality weight of darkened pixels during computation
    /// * `line_algorithm` - provided line drawing algorithm implementation, which
    ///                      returns the set of points which best represent a line from
    ///                      point A to point B
    /// * `image_width` - width of the input image
    /// * `image_height` - height of the input image
    /// * `image_mask` - a u8 slice to the grayscale representation of the input image
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
    pub fn get_moves<P>(&mut self, start_anchor: usize, mut strategy: P) -> Result<Vec<usize>, &str>
    where
        P: PlanningStrategy,
    {
        let mut anchor = start_anchor;
        let mut anchor_order = Vec::new();

        anchor_order.push(start_anchor);

        while !strategy.completed() {
            let next_anchor = self
                .next_anchor(anchor)
                .ok_or("failed to obtain next anchor.")?;

            self.apply_line(self.anchors[anchor], self.anchors[next_anchor]);

            anchor = next_anchor;
            anchor_order.push(anchor);
        }

        Ok(anchor_order)
    }

    /// Finds the next thread anchor on the perimeter based on the position of current
    fn next_anchor(&self, current: usize) -> Option<usize> {
        //  search_size = `all anchors`      - `gap on both sides`       - `current anchor`
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

    /// Apply changes from a line, persisting the pixel changes to the image mask
    fn apply_line(&mut self, src: FPos, dst: FPos) {
        for ((x, y), intensity) in self.trace_line(src, dst) {
            self.image_mask_inverted[x + y * self.image_width] -= intensity * self.line_weight;
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
                self.image_mask_inverted[x + y * self.image_width] - intensity * self.line_weight
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
            .filter(|((x, y), _)| *x + *y * self.image_width < self.image_mask_inverted.len())
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
            &[0; 100],
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
