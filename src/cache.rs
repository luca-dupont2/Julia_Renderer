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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn view() -> View { View { width: 100, height: 100, boundary: 1.0, x: 0.0, y: 0.0 } }
    fn evaluate(x: f64, y: f64) -> f64 { x * x + y }

    #[test]
    fn unchanged_pan_zoom_and_eviction() {
        let calls = AtomicUsize::new(0);
        let compute = |x, y| { calls.fetch_add(1, Ordering::Relaxed); evaluate(x, y) };
        let mut cache = Cache::new();
        let mut v = view();
        assert!(cache.update(v, compute));
        assert_eq!(calls.swap(0, Ordering::Relaxed), 10_000);
        assert!(!cache.update(v, compute));
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        v.x += 0.02;
        cache.update(v, compute);
        assert_eq!(calls.swap(0, Ordering::Relaxed), 100); // One new column.
        v.y += 0.02;
        cache.update(v, compute);
        assert_eq!(calls.swap(0, Ordering::Relaxed), 100); // One new row.
        v.boundary /= 1.1;
        cache.update(v, compute);
        let misses = calls.swap(0, Ordering::Relaxed);
        assert!(misses > 0 && misses < 10_000); // Zoom reuses some samples.
        let epsilon = TOLERANCE * v.step().0.min(v.step().1);
        for (index, sample) in cache.samples.iter().enumerate() {
            let point = v.point(index);
            assert!(v.contains(sample.0));
            assert!((sample.0.0 - point.0).hypot(sample.0.1 - point.1) <= epsilon);
            assert_eq!(sample.1, evaluate(sample.0.0, sample.0.1));
        }
        assert_eq!(cache.samples.len(), 10_000);
        v.x += 4.0;
        cache.update(v, compute);
        assert_eq!(calls.load(Ordering::Relaxed), 10_000); // No overlap.
    }

    #[test]
    fn small_moves_do_not_accumulate_error() {
        let mut cache = Cache::new();
        let mut v = view();
        for _ in 0..30 {
            cache.update(v, evaluate);
            for (index, sample) in cache.samples.iter().enumerate() {
                let point = v.point(index);
                assert!((sample.0.0 - point.0).hypot(sample.0.1 - point.1) <= 0.005);
            }
            v.x += 0.001;
        }
    }

    #[test]
    fn invalidation_resize_and_return_to_original_size() {
        let mut cache = Cache::new();
        cache.update(view(), |_, _| 1);
        cache.clear();
        cache.update(view(), |_, _| 2);
        assert!(cache.values().all(|value| value == 2));
        let resized = View { width: 50, height: 80, ..view() };
        cache.update(resized, |_, _| 2);
        assert_eq!(cache.samples.len(), 4_000);
        cache.update(view(), |_, _| 2);
        assert_eq!(cache.samples.len(), 10_000);
    }
}
