use serde::Serialize;
use std::{path::Path, time::Instant};

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct Traffic {
    pub sent_bytes: u64,
    pub received_bytes: u64,
    pub lost_packets: u64,
}

impl Traffic {
    pub fn add(&mut self, other: Self) {
        self.sent_bytes += other.sent_bytes;
        self.received_bytes += other.received_bytes;
        self.lost_packets += other.lost_packets;
    }
}

#[derive(Serialize)]
pub struct Report {
    pub protocol_version: u32,
    pub build_profile: &'static str,
    pub map: shared::level::MapId,
    pub layout: String,
    pub elapsed_seconds: f64,
    pub ticks: u64,
    pub ticks_per_second: f64,
    pub players: usize,
    pub peak_players: usize,
    pub seconds_at_capacity: f64,
    pub ticks_per_second_at_capacity: f64,
    pub tick_p99_ms_at_capacity: f64,
    pub physics_p99_ms_at_capacity: f64,
    pub replication_p99_ms_at_capacity: f64,
    pub tick_p50_ms: f64,
    pub tick_p95_ms: f64,
    pub tick_p99_ms: f64,
    pub tick_max_ms: f64,
    pub late_ticks: u64,
    pub max_tick_lag_ms: f64,
    pub peak_incoming_queue: usize,
    pub current_rss_mb: f64,
    pub peak_rss_mb: f64,
    pub process_cpu_seconds: f64,
    pub traffic: Traffic,
}

#[derive(Default)]
pub struct StepWork {
    pub physics_seconds: f64,
    pub replication_seconds: f64,
}

struct Histogram {
    bins: Vec<u64>,
    samples: u64,
    maximum: f64,
}

impl Default for Histogram {
    fn default() -> Self {
        Self {
            bins: vec![0; 10_001],
            samples: 0,
            maximum: 0.0,
        }
    }
}

impl Histogram {
    fn observe(&mut self, seconds: f64) {
        let ms = seconds * 1000.0;
        let bin = ((ms * 10.0).ceil() as usize).min(self.bins.len() - 1);
        self.bins[bin] += 1;
        self.samples += 1;
        self.maximum = self.maximum.max(ms);
    }

    fn percentile(&self, fraction: f64) -> f64 {
        let target = ((self.samples as f64 * fraction).ceil() as u64).max(1);
        let mut count = 0;
        for (index, n) in self.bins.iter().enumerate() {
            count += n;
            if count >= target {
                return index as f64 / 10.0;
            }
        }
        0.0
    }
}

pub struct Metrics {
    start: Instant,
    histogram: Histogram,
    capacity_histogram: Histogram,
    physics_histogram: Histogram,
    replication_histogram: Histogram,
    at_capacity_seconds: f64,
    last_observation: Instant,
    was_at_capacity: bool,
    late: u64,
    max_lag_seconds: f64,
    peak_players: usize,
    peak_queue: usize,
    peak_rss: f64,
    clock_ticks: f64,
}

impl Default for Metrics {
    fn default() -> Self {
        let clock_ticks = std::process::Command::new("getconf")
            .arg("CLK_TCK")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<f64>().ok())
            .filter(|v| *v > 0.0)
            .unwrap_or(100.0);
        let start = Instant::now();
        Self {
            start,
            histogram: Histogram::default(),
            capacity_histogram: Histogram::default(),
            physics_histogram: Histogram::default(),
            replication_histogram: Histogram::default(),
            at_capacity_seconds: 0.0,
            last_observation: start,
            was_at_capacity: false,
            late: 0,
            max_lag_seconds: 0.0,
            peak_players: 0,
            peak_queue: 0,
            peak_rss: 0.0,
            clock_ticks,
        }
    }
}

impl Metrics {
    pub fn elapsed(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
    pub fn observe(
        &mut self,
        seconds: f64,
        lag_seconds: f64,
        players: usize,
        capacity: usize,
        queue: usize,
        work: StepWork,
    ) {
        self.histogram.observe(seconds);
        if players == capacity {
            self.capacity_histogram.observe(seconds);
            self.physics_histogram.observe(work.physics_seconds);
            self.replication_histogram.observe(work.replication_seconds);
        }
        let now = Instant::now();
        if self.was_at_capacity {
            self.at_capacity_seconds += now.duration_since(self.last_observation).as_secs_f64();
        }
        self.last_observation = now;
        self.was_at_capacity = players == capacity;
        self.late += u64::from(lag_seconds > 1.0 / shared::movement::FIXED_HZ);
        self.max_lag_seconds = self.max_lag_seconds.max(lag_seconds);
        self.peak_players = self.peak_players.max(players);
        self.peak_queue = self.peak_queue.max(queue);
    }
    pub fn report(
        &mut self,
        map: shared::level::MapId,
        layout: &str,
        players: usize,
        traffic: Traffic,
    ) -> Report {
        let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
        let rss = status
            .lines()
            .find_map(|line| {
                line.strip_prefix("VmRSS:")
                    .and_then(|s| s.split_whitespace().next())
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or(0.0)
            / 1024.0;
        self.peak_rss = self.peak_rss.max(rss);
        let cpu = std::fs::read_to_string("/proc/self/stat")
            .ok()
            .and_then(|s| s.rsplit_once(") ").map(|(_, rest)| rest.to_owned()))
            .map(|s| {
                s.split_whitespace()
                    .skip(11)
                    .take(2)
                    .filter_map(|v| v.parse::<f64>().ok())
                    .sum::<f64>()
                    / self.clock_ticks
            })
            .unwrap_or(0.0);
        let elapsed = self.elapsed();
        let at_capacity = self.at_capacity_seconds
            + if self.was_at_capacity {
                self.last_observation.elapsed().as_secs_f64()
            } else {
                0.0
            };
        Report {
            protocol_version: shared::protocol::PROTOCOL_VERSION,
            build_profile: if cfg!(debug_assertions) {
                "dev"
            } else {
                "release"
            },
            map,
            layout: layout.into(),
            elapsed_seconds: elapsed,
            ticks: self.histogram.samples,
            ticks_per_second: self.histogram.samples as f64 / elapsed.max(0.001),
            players,
            peak_players: self.peak_players,
            seconds_at_capacity: at_capacity,
            ticks_per_second_at_capacity: self.capacity_histogram.samples as f64
                / at_capacity.max(0.001),
            tick_p99_ms_at_capacity: self.capacity_histogram.percentile(0.99),
            physics_p99_ms_at_capacity: self.physics_histogram.percentile(0.99),
            replication_p99_ms_at_capacity: self.replication_histogram.percentile(0.99),
            tick_p50_ms: self.histogram.percentile(0.5),
            tick_p95_ms: self.histogram.percentile(0.95),
            tick_p99_ms: self.histogram.percentile(0.99),
            tick_max_ms: self.histogram.maximum,
            late_ticks: self.late,
            max_tick_lag_ms: self.max_lag_seconds * 1000.0,
            peak_incoming_queue: self.peak_queue,
            current_rss_mb: rss,
            peak_rss_mb: self.peak_rss,
            process_cpu_seconds: cpu,
            traffic,
        }
    }
}

pub fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(
        &temporary,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(temporary, path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn empty_catchup_ticks_cannot_hide_slow_simulation_at_capacity() {
        let mut metrics = Metrics::default();
        for _ in 0..2 {
            metrics.observe(
                0.04,
                0.025,
                150,
                150,
                300,
                StepWork {
                    physics_seconds: 0.03,
                    replication_seconds: 0.009,
                },
            );
            metrics.last_observation = Instant::now() - Duration::from_millis(40);
        }
        for _ in 0..400 {
            metrics.observe(0.0, 0.0, 0, 150, 0, StepWork::default());
        }
        let report = metrics.report(
            shared::level::MapId::Polygon,
            "crowd",
            0,
            Traffic::default(),
        );
        assert_eq!(report.tick_p99_ms, 0.0);
        assert_eq!(report.tick_p99_ms_at_capacity, 40.0);
        assert_eq!(report.physics_p99_ms_at_capacity, 30.0);
        assert_eq!(report.replication_p99_ms_at_capacity, 9.0);
        assert!(report.seconds_at_capacity >= 0.08);
        assert!(report.ticks_per_second_at_capacity <= 25.0);
        assert_eq!(report.late_ticks, 2);
        assert_eq!(report.max_tick_lag_ms, 25.0);
    }
}
