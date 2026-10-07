use rayon::prelude::*;

// Maximum sample displacement in pixels. Set to 0.0 for exact reuse only.
const TOLERANCE: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub width: usize,
    pub height: usize,
    pub boundary: f64,
    pub x: f64,
    pub y: f64,
}

impl View {
    fn step(self) -> (f64, f64) {
        (2.0 * self.boundary / self.width as f64, 2.0 * self.boundary / self.height as f64)
    }

    fn point(self, index: usize) -> (f64, f64) {
        let (dx, dy) = self.step();
        (self.x - self.boundary + (index % self.width) as f64 * dx,
         self.boundary - self.y - (index / self.width) as f64 * dy)
    }

    fn contains(self, (x, y): (f64, f64)) -> bool {
        x >= self.x - self.boundary && x < self.x + self.boundary
            && y <= self.boundary - self.y && y > -self.boundary - self.y
    }
}

pub struct Cache<T> {
    view: Option<View>,
    samples: Vec<((f64, f64), T)>,
}

impl<T: Copy + Send + Sync> Cache<T> {
    pub fn new() -> Self {
        Self { view: None, samples: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.view = None;
        self.samples.clear();
    }

    // Samples retain their original coordinates, so approximate reuse cannot drift.
    // Only samples selected for the new viewport survive; storage is O(pixel count).
    pub fn update(&mut self, view: View, compute: impl Fn(f64, f64) -> T + Sync) -> bool {
        if self.view == Some(view) { return false; }
        let (dx, dy) = view.step();
        let epsilon = TOLERANCE * dx.min(dy);
        self.samples = (0..view.width * view.height).into_par_iter().map(|index| {
            let point = view.point(index);
            if let Some(old) = self.view {
                let (old_dx, old_dy) = old.step();
                let x = ((point.0 - old.x + old.boundary) / old_dx).round();
                let y = ((old.boundary - old.y - point.1) / old_dy).round();
                if x >= 0.0 && x < old.width as f64 && y >= 0.0 && y < old.height as f64 {
                    let sample = self.samples[y as usize * old.width + x as usize];
                    if view.contains(sample.0)
                        && (sample.0.0 - point.0).hypot(sample.0.1 - point.1) <= epsilon {
                        return sample;
                    }
                }
            }
            (point, compute(point.0, point.1))
        }).collect();
        self.view = Some(view);
        true
    }

    pub fn values(&self) -> impl Iterator<Item = T> + '_ {
        self.samples.iter().map(|sample| sample.1)
    }
}
