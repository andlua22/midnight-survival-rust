//! Real authenticated QUIC clients. Run this on a different machine from the
//! server for capacity measurements; these clients do not render or run AI.
use crate::metrics::{Traffic, write_json};
use midnight_network::{
    NetResult,
    client::{ConnectedClient, connection_error, establish},
    codec::{decode, encode, read_control, write_control},
    identity::{Invitation, client_endpoint},
    tokio,
};
use serde::Serialize;
use shared::{movement::FIXED_DT, protocol::*};
use std::{
    collections::{BTreeSet, VecDeque},
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{Semaphore, mpsc, watch},
    task::JoinSet,
};

pub struct LoadOptions {
    pub server: SocketAddr,
    pub code_file: PathBuf,
    pub clients: usize,
    pub seconds: f64,
    pub spread: bool,
    pub report: PathBuf,
}

#[derive(Serialize)]
pub struct BotReport {
    pub player: PlayerId,
    pub seconds: f64,
    pub input_packets: u64,
    pub snapshots: u64,
    pub owner_snapshots: u64,
    pub owner_snapshot_hz: f64,
    pub max_snapshot_gap_ms: f64,
    pub peak_visible: usize,
    pub final_position: [f32; 3],
    pub rtt_ms: f64,
    pub traffic: Traffic,
}

#[derive(Serialize)]
pub struct LoadReport {
    pub build_profile: &'static str,
    pub scenario: &'static str,
    pub server: SocketAddr,
    pub clients: usize,
    pub requested_seconds: f64,
    pub elapsed_seconds: f64,
    pub traffic: Traffic,
    pub bots: Vec<BotReport>,
}

pub async fn run(options: LoadOptions) -> NetResult<LoadReport> {
    if options.clients == 0
        || options.clients > MAX_PLAYERS
        || !options.seconds.is_finite()
        || options.seconds <= 0.0
    {
        return Err("Потрібно 1..150 клієнтів і додатна тривалість тесту".into());
    }
    let invitation = Invitation::parse(
        &std::fs::read_to_string(&options.code_file).map_err(|e| e.to_string())?,
    )?;
    let endpoint = client_endpoint((Ipv4Addr::UNSPECIFIED, 0).into(), &invitation)?;
    let (ready_tx, mut ready_rx) = mpsc::channel(options.clients);
    let (start_tx, start_rx) = watch::channel(None);
    let semaphore = Arc::new(Semaphore::new(16));
    let mut bots = JoinSet::new();
    for index in 0..options.clients {
        let endpoint = endpoint.clone();
        let invitation = invitation.clone();
        let ready = ready_tx.clone();
        let start = start_rx.clone();
        let semaphore = semaphore.clone();
        let address = options.server;
        let duration = options.seconds;
        let spread = options.spread;
        bots.spawn(async move {
            let permit = semaphore.acquire_owned().await.map_err(|e| e.to_string())?;
            let connected = establish(&endpoint, address, &invitation).await;
            drop(permit);
            match connected {
                Ok(client) => {
                    ready.send(Ok(())).await.map_err(|e| e.to_string())?;
                    bot(client, index, spread, duration, start).await
                }
                Err(error) => {
                    let _ = ready.send(Err(error.clone())).await;
                    Err(error)
                }
            }
        });
    }
    drop(ready_tx);
    let readiness = tokio::time::timeout(Duration::from_secs(45), async {
        for _ in 0..options.clients {
            ready_rx
                .recv()
                .await
                .ok_or("Не всі клієнти підключилися")??;
        }
        Ok::<_, String>(())
    })
    .await
    .map_err(|_| "Перевищено час підключення навантажувальних клієнтів".to_string())?;
    if let Err(error) = readiness {
        bots.abort_all();
        return Err(error);
    }
    let start = Instant::now();
    start_tx.send_replace(Some(start));
    println!(
        "Підключено {} клієнтів; тест {:.0} с",
        options.clients, options.seconds
    );
    let mut reports = Vec::new();
    let mut traffic = Traffic::default();
    while let Some(result) = bots.join_next().await {
        let report = result.map_err(|e| e.to_string())??;
        traffic.add(report.traffic);
        reports.push(report);
    }
    reports.sort_by_key(|r| r.player);
    let report = LoadReport {
        build_profile: if cfg!(debug_assertions) {
            "dev"
        } else {
            "release"
        },
        scenario: if options.spread { "spread" } else { "crowd" },
        server: options.server,
        clients: options.clients,
        requested_seconds: options.seconds,
        elapsed_seconds: start.elapsed().as_secs_f64(),
        traffic,
        bots: reports,
    };
    write_json(&options.report, &report)?;
    endpoint.close(0u8.into(), b"load complete");
    let _ = tokio::time::timeout(Duration::from_millis(250), endpoint.wait_idle()).await;
    Ok(report)
}

async fn bot(
    client: ConnectedClient,
    index: usize,
    spread: bool,
    seconds: f64,
    mut start: watch::Receiver<Option<Instant>>,
) -> NetResult<BotReport> {
    let ConnectedClient {
        connection,
        mut write,
        mut read,
        welcome,
    } = client;
    let ServerControl::Welcome {
        player,
        tick,
        epoch,
        state,
        ..
    } = welcome
    else {
        return Err("Missing welcome".into());
    };
    let mut observed = tick;
    let mut observed_at = Instant::now();
    let mut current = state;
    let mut epoch = epoch;
    let mut next_tick = tick.0 + 1;
    let mut acknowledged = tick;
    let started = loop {
        if let Some(time) = *start.borrow_and_update() {
            break time;
        }
        start.changed().await.map_err(|e| e.to_string())?;
    };
    let (control_tx, mut control_rx) = mpsc::channel(512);
    let mut reader = tokio::spawn(async move {
        loop {
            let message: ServerControl = read_control(&mut read).await?;
            control_tx.send(message).await.map_err(|e| e.to_string())?;
        }
        #[allow(unreachable_code)]
        Ok::<_, String>(())
    });
    let mut writer = tokio::spawn(async move {
        let mut heartbeat = tokio::time::interval(Duration::from_secs(1));
        loop {
            heartbeat.tick().await;
            write_control(
                &mut write,
                &ClientControl::Ping(started.elapsed().as_micros() as u64),
            )
            .await?;
        }
        #[allow(unreachable_code)]
        Ok::<_, String>(())
    });
    let mut interval = tokio::time::interval(Duration::from_secs_f64(FIXED_DT as f64));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let finish = tokio::time::sleep(Duration::from_secs_f64(seconds));
    tokio::pin!(finish);
    let mut frames = VecDeque::new();
    let mut input_packets = 0;
    let mut snapshots = 0;
    let mut owner_snapshots = 0;
    let mut last_owner_snapshot: Option<Instant> = None;
    let mut max_snapshot_gap_ms = 0.0_f64;
    let mut visible = BTreeSet::new();
    let mut peak_visible = 0;
    let center = if spread {
        [
            if state.position[0] < 0.0 {
                -160.0
            } else {
                160.0
            },
            if state.position[2] < 0.0 {
                -160.0
            } else {
                160.0
            },
        ]
    } else {
        [0.0, 15.0]
    };
    let outcome: NetResult<()> = async {
        loop {
            tokio::select! {
                _ = &mut finish => return Ok(()),
                error = connection.closed() => return Err(connection_error(error)),
                result = &mut reader => return result.map_err(|e| e.to_string())?,
                result = &mut writer => return result.map_err(|e| e.to_string())?,
                control = control_rx.recv() => match control {
                    Some(ServerControl::Enter { player, .. }) => { visible.insert(player); peak_visible = peak_visible.max(visible.len()); },
                    Some(ServerControl::Leave { player, .. }) => { visible.remove(&player); },
                    Some(ServerControl::Reset { tick, epoch: new_epoch, state }) => { epoch = new_epoch; observed = tick; observed_at = Instant::now(); current = state; acknowledged = tick; next_tick = tick.0 + 1; frames.clear(); },
                    Some(ServerControl::Rejected(reason)) => return Err(reason),
                    Some(_) => {},
                    None => return Err("Control stream closed".into()),
                },
                bytes = connection.read_datagram() => {
                    let message: Snapshot = decode(&bytes.map_err(connection_error)?, MAX_DATAGRAM_BYTES)?;
                    snapshots += 1;
                    if message.tick > observed { observed = message.tick; observed_at = Instant::now(); }
                    if let Some(owner) = message.owner && owner.epoch == epoch && owner.acknowledged > acknowledged {
                        let now = Instant::now();
                        if let Some(last) = last_owner_snapshot { max_snapshot_gap_ms = max_snapshot_gap_ms.max(now.duration_since(last).as_secs_f64() * 1000.0); }
                        last_owner_snapshot = Some(now);
                        owner_snapshots += 1;
                        acknowledged = owner.acknowledged; current = owner.state;
                    }
                },
                _ = interval.tick() => {
                    let half_rtt = connection.rtt().as_secs_f64() * 0.5;
                    let estimate = observed.0 + ((observed_at.elapsed().as_secs_f64() + half_rtt) / FIXED_DT as f64) as u64;
                    let lead = ((half_rtt / FIXED_DT as f64).ceil() as u64 + 3).min(30);
                    let target = (estimate + lead).min(acknowledged.0 + INPUT_WINDOW);
                    next_tick = next_tick.max(acknowledged.0 + 1);
                    let angle = started.elapsed().as_secs_f32() * 0.4 + index as f32 * 2.4;
                    let radius = if spread { 12.0 } else { 1.5 };
                    let dx = center[0] + angle.cos() * radius - current.position[0];
                    let dz = center[1] + angle.sin() * radius - current.position[2];
                    for _ in 0..8 {
                        if next_tick > target { break; }
                        frames.push_back(InputFrame { epoch, tick: ServerTick(next_tick), movement: [0.0, if dx.hypot(dz) > 0.25 { 1.0 } else { 0.0 }], yaw: (-dx).atan2(-dz).rem_euclid(std::f32::consts::TAU), pitch: 0.0,
                            jump: next_tick % 180 == (index as u64 * 7) % 180, sprint: index.is_multiple_of(3) });
                        next_tick += 1;
                        while frames.len() > INPUT_REDUNDANCY { frames.pop_front(); }
                    }
                    if !frames.is_empty() {
                        let batch = InputBatch { frames: frames.iter().copied().collect() };
                        connection.send_datagram(encode(&batch)?.into()).map_err(|e| e.to_string())?;
                        input_packets += 1;
                    }
                }
            }
        }
    }.await;
    reader.abort();
    writer.abort();
    let stats = connection.stats();
    let rtt_ms = connection.rtt().as_secs_f64() * 1000.0;
    connection.close(0u8.into(), b"load complete");
    outcome?;
    if snapshots == 0 || owner_snapshots == 0 || input_packets == 0 {
        return Err("Клієнт не обмінявся станом гри".into());
    }
    let elapsed = started.elapsed().as_secs_f64();
    Ok(BotReport {
        player,
        seconds: elapsed,
        input_packets,
        snapshots,
        owner_snapshots,
        owner_snapshot_hz: owner_snapshots as f64 / elapsed.max(0.001),
        max_snapshot_gap_ms,
        peak_visible,
        final_position: current.position,
        rtt_ms,
        traffic: Traffic {
            sent_bytes: stats.udp_tx.bytes,
            received_bytes: stats.udp_rx.bytes,
            lost_packets: stats.path.lost_packets,
        },
    })
}
