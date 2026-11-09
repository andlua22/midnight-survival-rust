use crate::{
    interface::{Interface, Telemetry},
    launch::ClientOptions,
    metrics::{FrameHistogram, WARMUP_SECONDS},
    online::Online,
    storage::write_report,
};
use bevy::{prelude::*, window::PrimaryWindow};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    thread::JoinHandle,
    time::{Duration, Instant},
};

struct ReportWriter {
    sender: Option<mpsc::SyncSender<Vec<u8>>>,
    thread: Option<JoinHandle<()>>,
    error: Arc<Mutex<Option<String>>>,
}

impl ReportWriter {
    fn new(path: PathBuf) -> Result<Self, String> {
        let (sender, receiver) = mpsc::sync_channel::<Vec<u8>>(1);
        let error = Arc::new(Mutex::new(None));
        let errors = error.clone();
        let thread = std::thread::Builder::new()
            .name("network-report".into())
            .spawn(move || {
                while let Ok(bytes) = receiver.recv() {
                    if let Err(problem) = write_report(&path, &bytes)
                        && let Ok(mut slot) = errors.lock()
                    {
                        *slot = Some(problem.to_string());
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            sender: Some(sender),
            thread: Some(thread),
            error,
        })
    }
    fn send(&self, report: &serde_json::Value) {
        if let (Some(sender), Ok(bytes)) = (&self.sender, serde_json::to_vec_pretty(report)) {
            let _ = sender.try_send(bytes);
        }
    }
}

impl Drop for ReportWriter {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[derive(Resource)]
pub(crate) struct OnlineCapture {
    writer: Option<ReportWriter>,
    error: Option<String>,
    active: bool,
    started: Instant,
    last_report: Instant,
    frames: FrameHistogram,
    unfocused_frames: u64,
    resolution: [u32; 2],
    resolution_changes: u64,
    last_prediction: Option<(shared::protocol::PlayerId, u64, f32)>,
    vsync: bool,
}

impl OnlineCapture {
    pub fn new(options: &ClientOptions) -> Self {
        let (writer, error) = match options.network_report.clone().map(ReportWriter::new) {
            Some(Ok(writer)) => (Some(writer), None),
            Some(Err(error)) => (None, Some(error)),
            None => (None, None),
        };
        Self {
            writer,
            error,
            active: false,
            started: Instant::now(),
            last_report: Instant::now(),
            frames: FrameHistogram::default(),
            unfocused_frames: 0,
            resolution: [0; 2],
            resolution_changes: 0,
            last_prediction: None,
            vsync: !options.no_vsync,
        }
    }
}

pub(crate) fn capture(
    mut capture: ResMut<OnlineCapture>,
    online: Res<Online>,
    time: Res<Time<Real>>,
    telemetry: Res<Telemetry>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut interface: ResMut<Interface>,
    mut exit: MessageReader<AppExit>,
) {
    if let Some(error) = capture.error.take() {
        interface.notice = Some(format!("Звіт мережевого тесту: {error}"));
    }
    let Some(writer) = &capture.writer else {
        return;
    };
    let writer_error = writer.error.lock().ok().and_then(|mut slot| slot.take());
    if let Some(error) = writer_error {
        interface.notice = Some(format!("Звіт мережевого тесту: {error}"));
    }
    if online.connected && !capture.active {
        capture.active = true;
        capture.started = Instant::now();
        capture.last_report = Instant::now();
        capture.frames = FrameHistogram::default();
        capture.unfocused_frames = 0;
        capture.resolution_changes = 0;
    }
    if !capture.active {
        return;
    }
    if online.connected {
        capture.last_prediction = online.prediction_statistics();
        if let Ok(window) = windows.single() {
            let resolution = [window.physical_width(), window.physical_height()];
            if resolution != capture.resolution {
                if capture.resolution != [0; 2] {
                    capture.resolution_changes += 1;
                }
                capture.resolution = resolution;
            }
            if capture.started.elapsed().as_secs_f64() >= WARMUP_SECONDS {
                if window.focused && !resolution.contains(&0) {
                    capture.frames.observe(time.delta_secs_f64() * 1000.0);
                } else {
                    capture.unfocused_frames += 1;
                }
            }
        }
    }
    let final_report = !online.connected || exit.read().next().is_some();
    if final_report || capture.last_report.elapsed() >= Duration::from_secs(5) {
        let (player, corrections, maximum) = capture
            .last_prediction
            .map(|(p, c, m)| (Some(p), c, m))
            .unwrap_or((None, 0, 0.0));
        let report = serde_json::json!({
            "protocol_version": shared::protocol::PROTOCOL_VERSION,
            "build_profile": if cfg!(debug_assertions) { "dev" } else { "release" },
            "server": online.address, "player": player, "status": online.status,
            "gpu": telemetry.gpu, "backend": telemetry.backend, "resolution": capture.resolution,
            "resolution_changes": capture.resolution_changes, "vsync": capture.vsync,
            "elapsed_seconds": capture.started.elapsed().as_secs_f64(), "warmup_seconds": WARMUP_SECONDS,
            "unfocused_frames": capture.unfocused_frames, "frames": capture.frames.statistics(),
            "population": online.population, "rtt_ms": online.rtt.as_secs_f64() * 1000.0,
            "sent_bytes": online.sent_bytes, "received_bytes": online.received_bytes, "lost_packets": online.lost_packets,
            "corrections": corrections, "maximum_correction_m": maximum,
        });
        capture.writer.as_ref().unwrap().send(&report);
        capture.last_report = Instant::now();
    }
    if final_report {
        capture.active = false;
    }
}
