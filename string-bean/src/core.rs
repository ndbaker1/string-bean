//! helper functions exposed as part of the core library definitions

use crate::{PixelIntensity, PlanningStrategy, Precision, ThreadPlanner};

/// Stop the thread planner after a certain number of lines have been drawn
pub struct CountTracker(pub u32);
impl<Z> PlanningStrategy<Z> for CountTracker {
    fn completed(&mut self, _: &Z, anchors: &[usize]) -> bool {
        anchors.len() as u32 > self.0
    }
}

/// Stop the thread planner when the loss reaches a suitable value
pub struct LossTracker {
    wait: usize,
    current: usize,
    target_loss: f64,
}
impl LossTracker {
    pub fn new(wait: usize, target_loss: f64) -> Self {
        Self {
            wait,
            current: 0,
            target_loss,
        }
    }
}
impl<'a, I, S> PlanningStrategy<ThreadPlanner<'a, I, S>> for LossTracker
where
    I: IntoIterator<Item = PixelIntensity>,
    S: Fn(Precision, Precision, Precision, Precision) -> I,
{
    fn completed(&mut self, planner: &ThreadPlanner<'a, I, S>, anchors: &[usize]) -> bool {
        // hard stop switch
        if anchors.len() > 3000 {
            return true;
        }

        // use an exponential decay function to decrease the interval of expensive computation
        if self.current >= self.wait {
            // compute loss of the entire images mask, which only
            // needs to verify inside of the points used by the planner
            // TODO.
            let loss = planner
                .image_mask_inverted
                .iter()
                .map(|a| a.abs())
                .sum::<f64>();

            // break if we have our target loss
            if loss < self.target_loss {
                return true;
            }

            // half the wait time down to a minimum
            self.wait = (self.wait / 2).max(20);
            // reset counter
            self.current = 0;
        }

        self.current += 1;

        false
    }
}

/// https://playtechs.blogspot.com/2007/03/raytracing-on-grid.html
pub fn grid_raytrace(
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

#[cfg(test)]
mod test {
    use crate::{core::grid_raytrace, ThreadPlanner};

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
