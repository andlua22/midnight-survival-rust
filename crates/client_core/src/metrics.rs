use serde::Serialize;

pub const WARMUP_SECONDS: f64 = 10.0;
pub const CAPTURE_SECONDS: f64 = 60.0;

/// Constant-memory frame statistics for long online sessions. Percentiles are
/// rounded up to 0.1 ms; mean/FPS retain the full measured duration.
pub struct FrameHistogram {
    bins: Vec<u64>,
    frames: usize,
    total_ms: f64,
    maximum_ms: f64,
}

impl Default for FrameHistogram {
    fn default() -> Self {
        Self {
            bins: vec![0; 10_001],
            frames: 0,
            total_ms: 0.0,
            maximum_ms: 0.0,
        }
    }
}

impl FrameHistogram {
    pub fn observe(&mut self, ms: f64) {
        if !ms.is_finite() || ms <= 0.0 {
            return;
        }
        self.frames += 1;
        self.total_ms += ms;
        self.maximum_ms = self.maximum_ms.max(ms);
        let bin = ((ms * 10.0).ceil() as usize).min(self.bins.len() - 1);
        self.bins[bin] += 1;
    }
    pub fn statistics(&self) -> Option<FrameStatistics> {
        if self.frames == 0 {
            return None;
        }
        let percentile = |fraction: f64| {
            let target = (self.frames as f64 * fraction).ceil() as u64;
            let mut count = 0;
            for (bin, &n) in self.bins.iter().enumerate() {
                count += n;
                if count >= target {
                    return if bin == self.bins.len() - 1 {
                        self.maximum_ms
                    } else {
                        bin as f64 / 10.0
                    };
                }
            }
            self.maximum_ms
        };
        Some(FrameStatistics {
            frames: self.frames,
            duration_seconds: self.total_ms / 1000.0,
            average_fps: self.frames as f64 * 1000.0 / self.total_ms,
            mean_ms: self.total_ms / self.frames as f64,
            p95_ms: percentile(0.95),
            p99_ms: percentile(0.99),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrameStatistics {
    pub frames: usize,
    pub duration_seconds: f64,
    pub average_fps: f64,
    pub mean_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

impl FrameStatistics {
    pub fn from_samples(samples_ms: &[f64]) -> Option<Self> {
        let mut sorted: Vec<f64> = samples_ms
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v > 0.0)
            .collect();
        if sorted.is_empty() {
            return None;
        }
        sorted.sort_by(f64::total_cmp);
        let frames = sorted.len();
        let total: f64 = sorted.iter().sum();
        let percentile =
            |fraction: f64| sorted[((frames as f64 * fraction).ceil() as usize).saturating_sub(1)];
        Some(Self {
            frames,
            duration_seconds: total / 1000.0,
            average_fps: frames as f64 * 1000.0 / total,
            mean_ms: total / frames as f64,
            p95_ms: percentile(0.95),
            p99_ms: percentile(0.99),
        })
    }
}

#[derive(Default)]
pub struct BenchmarkRecorder {
    elapsed: f64,
    samples: Vec<f64>,
    finished: bool,
}

impl BenchmarkRecorder {
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn observe(&mut self, delta_seconds: f64) -> Option<FrameStatistics> {
        if self.finished || !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return None;
        }
        let before = self.elapsed;
        self.elapsed += delta_seconds;
        if before >= WARMUP_SECONDS {
            self.samples.push(delta_seconds * 1000.0);
        }
        if self.elapsed >= WARMUP_SECONDS + CAPTURE_SECONDS {
            self.finished = true;
            return FrameStatistics::from_samples(&self.samples);
        }
        None
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Serialize)]
pub struct BenchmarkReport {
    pub gpu: String,
    pub backend: String,
    pub resolution: [u32; 2],
    pub build_profile: String,
    pub vsync: bool,
    pub warmup_seconds: f64,
    pub statistics: FrameStatistics,
    pub target_fps: u32,
    /// The agreed target applies to a release run on an RTX 4060 at 1080p.
    pub target_conditions_met: bool,
    pub target_achieved: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_session_histogram_keeps_slow_frames_and_exact_mean() {
        let mut histogram = FrameHistogram::default();
        for _ in 0..9900 {
            histogram.observe(10.02);
        }
        for _ in 0..100 {
            histogram.observe(50.03);
        }
        histogram.observe(f64::NAN);
        let stats = histogram.statistics().unwrap();
        assert_eq!(stats.frames, 10000);
        assert_eq!(stats.p95_ms, 10.1);
        assert!((stats.mean_ms - 10.4201).abs() < 1e-8);
    }

    #[test]
    fn fps_uses_total_elapsed_time_and_percentiles_include_slow_frames() {
        let mut samples = vec![5.0; 95];
        samples.extend([50.0; 5]);
        let stats = FrameStatistics::from_samples(&samples).unwrap();
        assert!((stats.average_fps - 100_000.0 / 725.0).abs() < 1e-8);
        assert_eq!(stats.p95_ms, 5.0);
        assert_eq!(stats.p99_ms, 50.0);
        assert!(FrameStatistics::from_samples(&[0.0, f64::NAN]).is_none());
    }

    #[test]
    fn benchmark_excludes_warmup_and_finishes_once() {
        let mut recorder = BenchmarkRecorder::default();
        for _ in 0..10 {
            assert!(recorder.observe(1.0).is_none());
        }
        for _ in 0..59 {
            assert!(recorder.observe(1.0).is_none());
        }
        let stats = recorder.observe(1.0).unwrap();
        assert_eq!(stats.frames, 60);
        assert_eq!(stats.duration_seconds, 60.0);
        assert!(recorder.observe(1.0).is_none());
        recorder.reset();
        assert_eq!(recorder.elapsed(), 0.0);
        assert!(!recorder.is_finished());
    }
}
