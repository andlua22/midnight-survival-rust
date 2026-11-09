//! These tests open real UDP sockets. Explicitly enable them on a machine that
//! permits networking: cargo test -p server --test networking -- --ignored
use midnight_network::{
    client::{ClientEvent, ClientLink, ConnectedClient, establish},
    codec::{decode, encode, read_control, write_control},
    identity::{Invitation, ServerIdentity, client_endpoint, connect},
    quinn,
    server::ServerTransport,
    tokio,
};
use shared::{
    level::{MAP_REVISION, MapId},
    movement::FIXED_DT,
    prediction::Prediction,
    protocol::*,
};
use std::{
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

async fn server(
    capacity: usize,
) -> (
    SocketAddr,
    Invitation,
    oneshot::Sender<()>,
    JoinHandle<server::GameServer>,
) {
    let identity = ServerIdentity::generate().unwrap();
    let invitation = identity.invitation();
    let mut transport = ServerTransport::bind(&identity, "127.0.0.1:0".parse().unwrap())
        .expect("UDP sockets required; run outside the restricted agent sandbox");
    let address = transport.endpoint.local_addr().unwrap();
    let (stop, mut stopped) = oneshot::channel();
    let task = tokio::spawn(async move {
        let mut game = server::GameServer::new(MapId::Polygon, capacity, false);
        let mut interval = tokio::time::interval(Duration::from_secs_f64(FIXED_DT as f64));
        loop {
            tokio::select! {
                _ = &mut stopped => break,
                _ = interval.tick() => {
                    while let Ok(event) = transport.events.try_recv() { game.event(event); }
                    game.step();
                }
            }
        }
        transport.shutdown().await;
        game
    });
    (address, invitation, stop, task)
}

fn endpoint(invitation: &Invitation) -> quinn::Endpoint {
    client_endpoint("127.0.0.1:0".parse().unwrap(), invitation).unwrap()
}

struct Driver {
    connection: quinn::Connection,
    write: quinn::SendStream,
    reader: JoinHandle<()>,
    controls: mpsc::Receiver<ServerControl>,
    prediction: Prediction,
    observed: ServerTick,
    received: Instant,
    authoritative: PlayerState,
    max_height: f32,
}

impl Drop for Driver {
    fn drop(&mut self) {
        self.reader.abort();
        self.connection.close(0u8.into(), b"test done");
    }
}

impl Driver {
    fn new(client: ConnectedClient) -> Self {
        let ConnectedClient {
            connection,
            write,
            mut read,
            welcome,
        } = client;
        let ServerControl::Welcome {
            player,
            tick,
            epoch,
            map,
            state,
            ..
        } = welcome
        else {
            panic!("welcome")
        };
        let (send, controls) = mpsc::channel(512);
        let reader = tokio::spawn(async move {
            while let Ok(message) = read_control(&mut read).await {
                if send.send(message).await.is_err() {
                    break;
                }
            }
        });
        Self {
            connection,
            write,
            reader,
            controls,
            prediction: Prediction::new(map, player, tick, epoch, state),
            observed: tick,
            received: Instant::now(),
            authoritative: state,
            max_height: state.position[1],
        }
    }

    async fn drive(&mut self, seconds: f64, movement: [f32; 2], mut jump: bool) {
        let mut interval = tokio::time::interval(Duration::from_secs_f64(FIXED_DT as f64));
        let end = tokio::time::sleep(Duration::from_secs_f64(seconds));
        tokio::pin!(end);
        loop {
            tokio::select! {
                _ = &mut end => return,
                control = self.controls.recv() => match control.expect("control stream remains open") {
                    ServerControl::Enter { player, tick, state } => self.prediction.enter(player, tick, state),
                    ServerControl::Leave { player, tick } => self.prediction.leave(player, tick),
                    ServerControl::Reset { tick, epoch, state } => self.prediction.reset(tick, epoch, state),
                    _ => {},
                },
                bytes = self.connection.read_datagram() => {
                    let snapshot: Snapshot = decode(&bytes.unwrap(), MAX_DATAGRAM_BYTES).unwrap();
                    if snapshot.tick > self.observed { self.observed = snapshot.tick; self.received = Instant::now(); }
                    for remote in snapshot.players { self.prediction.remote_snapshot(snapshot.tick, remote); }
                    if let Some(owner) = snapshot.owner {
                        self.authoritative = owner.state;
                        self.max_height = self.max_height.max(owner.state.position[1]);
                        self.prediction.reconcile(owner);
                    }
                },
                _ = interval.tick() => {
                    let half_rtt = self.connection.rtt().as_secs_f64() / 2.0;
                    let current = self.observed.0 + ((self.received.elapsed().as_secs_f64() + half_rtt) / FIXED_DT as f64) as u64;
                    let target = (current + (half_rtt / FIXED_DT as f64).ceil() as u64 + 3).min(self.prediction.last_ack.0 + INPUT_WINDOW);
                    for _ in 0..8 {
                        if self.prediction.tick.0 >= target || self.prediction.needs_reset { break; }
                        let frame = InputFrame { epoch: self.prediction.epoch, tick: ServerTick(self.prediction.tick.0 + 1), movement, yaw: 0.0, pitch: 0.0, jump, sprint: false };
                        self.prediction.predict(frame).unwrap(); jump = false;
                    }
                    let batch = self.prediction.batch();
                    if !batch.frames.is_empty() { self.connection.send_datagram(encode(&batch).unwrap().into()).unwrap(); }
                    if self.prediction.needs_reset { write_control(&mut self.write, &ClientControl::Resync).await.unwrap(); }
                }
            }
        }
    }
}

#[test]
#[ignore = "opens real UDP sockets; run with --ignored outside the restricted sandbox"]
fn dropping_graphical_client_releases_its_slot_without_idle_timeout() {
    midnight_network::runtime().unwrap().block_on(async {
        let (address, invite, stop, task) = server(1).await;
        let link = ClientLink::start(address.to_string(), invite.clone()).unwrap();
        let original = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                for event in link.poll() {
                    match event {
                        ClientEvent::Control {
                            message: ServerControl::Welcome { player, .. },
                            ..
                        } => return player,
                        ClientEvent::Disconnected(reason) => panic!("connection failed: {reason}"),
                        _ => {}
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("ClientLink must connect");
        drop(link);

        let endpoint = endpoint(&invite);
        let replacement = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if let Ok(client) = establish(&endpoint, address, &invite).await {
                    break client;
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        })
        .await
        .expect("a normal GUI disconnect must free capacity well before the 10s idle timeout");
        let ServerControl::Welcome { player, .. } = replacement.welcome else {
            panic!("welcome")
        };
        assert!(player > original, "reconnect creates a fresh player");
        replacement.connection.close(0u8.into(), b"done");
        tokio::time::sleep(Duration::from_millis(100)).await;
        stop.send(()).unwrap();
        let game = task.await.unwrap();
        assert_eq!(game.population(), 0);
        assert!(game.authority.world.state(original).is_none());
    });
}

#[test]
#[ignore = "opens real UDP sockets; run with --ignored outside the restricted sandbox"]
fn encrypted_movement_collision_capacity_and_cleanup() {
    midnight_network::runtime().unwrap().block_on(async {
        let (address, invite, stop, task) = server(2).await;
        let endpoint = endpoint(&invite);
        let mut a = Driver::new(establish(&endpoint, address, &invite).await.unwrap());
        let b = establish(&endpoint, address, &invite).await.unwrap();
        assert!(
            establish(&endpoint, address, &invite).await.is_err(),
            "third player must be rejected"
        );
        let initial_x = a.authoritative.position[0];
        let ServerControl::Welcome {
            player: b_id,
            state: b_state,
            ..
        } = b.welcome
        else {
            panic!("welcome")
        };
        a.drive(0.5, [0.0; 2], false).await;
        a.drive(0.3, [1.0, 0.0], false).await;
        assert!(a.authoritative.position[0] > initial_x + 0.3);
        assert!(
            a.authoritative.position[0] < b_state.position[0] - 0.65,
            "stationary player blocks passage"
        );
        b.connection.close(0u8.into(), b"leave test");
        a.drive(1.0, [1.0, 0.0], false).await;
        assert!(
            a.authoritative.position[0] > b_state.position[0] + 0.5,
            "removed collider must stop blocking"
        );
        a.drive(0.7, [0.0; 2], false).await;
        for axis in 0..3 {
            assert!(
                (a.prediction.state().position[axis] - a.authoritative.position[axis]).abs() < 0.05
            );
        }
        drop(a);
        tokio::time::sleep(Duration::from_millis(100)).await;
        stop.send(()).unwrap();
        let game = task.await.unwrap();
        assert_eq!(game.population(), 0);
        assert!(game.authority.world.state(b_id).is_none());
    });
}

#[test]
#[ignore = "opens real UDP sockets; run with --ignored outside the restricted sandbox"]
fn certificate_access_code_and_versions_are_checked_before_spawn() {
    midnight_network::runtime().unwrap().block_on(async {
        let (address, invite, stop, task) = server(2).await;
        let mut wrong_pin = invite.clone();
        wrong_pin.fingerprint[0] ^= 1;
        assert!(
            establish(&endpoint(&wrong_pin), address, &wrong_pin)
                .await
                .is_err()
        );
        let mut wrong_token = invite.clone();
        wrong_token.token[0] ^= 1;
        let good_endpoint = endpoint(&invite);
        assert!(
            establish(&good_endpoint, address, &wrong_token)
                .await
                .is_err()
        );
        for (protocol, revision) in [
            (PROTOCOL_VERSION + 1, MAP_REVISION),
            (PROTOCOL_VERSION, MAP_REVISION + 1),
        ] {
            let connection = connect(&good_endpoint, address).unwrap().await.unwrap();
            let (mut write, mut read) = connection.open_bi().await.unwrap();
            write_control(
                &mut write,
                &Hello {
                    protocol,
                    map_revision: revision,
                    token: invite.token,
                },
            )
            .await
            .unwrap();
            match read_control::<ServerControl>(&mut read).await {
                Ok(ServerControl::Rejected(_)) | Err(_) => {}
                _ => panic!("incompatible clients must not receive a welcome"),
            }
            connection.close(0u8.into(), b"done");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        stop.send(()).unwrap();
        assert_eq!(task.await.unwrap().population(), 0);
    });
}

#[test]
#[ignore = "opens 150 real QUIC connections; run with --ignored outside the restricted sandbox"]
fn admits_150_connections_rejects_151_and_frees_every_slot() {
    midnight_network::runtime().unwrap().block_on(async {
        let (address, invite, stop, task) = server(150).await;
        let endpoint = endpoint(&invite);
        let mut clients = Vec::new();
        for _ in 0..150 {
            clients.push(establish(&endpoint, address, &invite).await.unwrap());
        }
        assert!(establish(&endpoint, address, &invite).await.is_err());
        for client in clients {
            client.connection.close(0u8.into(), b"done");
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
        let rejoined = establish(&endpoint, address, &invite).await.unwrap();
        let ServerControl::Welcome { player, .. } = rejoined.welcome else {
            panic!("welcome")
        };
        assert!(player.0 > 150, "session identities must not be recycled");
        rejoined.connection.close(0u8.into(), b"done");
        tokio::time::sleep(Duration::from_millis(100)).await;
        stop.send(()).unwrap();
        assert_eq!(task.await.unwrap().population(), 0);
    });
}

struct Proxy {
    address: SocketAddr,
    blackhole: Arc<AtomicBool>,
    tasks: Vec<JoinHandle<()>>,
}
impl Drop for Proxy {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

impl Proxy {
    async fn new(server: SocketAddr) -> Self {
        let front = Arc::new(tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let back = Arc::new(tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let address = front.local_addr().unwrap();
        let peer = Arc::new(Mutex::new(None));
        let blackhole = Arc::new(AtomicBool::new(false));
        let random = Arc::new(AtomicU64::new(42));
        let budget = Arc::new(tokio::sync::Semaphore::new(1024));
        let mut tasks = Vec::new();
        for upstream in [true, false] {
            let source = if upstream {
                front.clone()
            } else {
                back.clone()
            };
            let destination = if upstream {
                back.clone()
            } else {
                front.clone()
            };
            let peer = peer.clone();
            let blackhole = blackhole.clone();
            let random = random.clone();
            let budget = budget.clone();
            tasks.push(tokio::spawn(async move {
                let mut buffer = vec![0; 65536];
                loop {
                    let Ok((length, sender)) = source.recv_from(&mut buffer).await else {
                        break;
                    };
                    if upstream {
                        *peer.lock().unwrap() = Some(sender);
                    }
                    let target = if upstream {
                        Some(server)
                    } else {
                        *peer.lock().unwrap()
                    };
                    let Some(target) = target else {
                        continue;
                    };
                    let mut draw = random.fetch_add(0x9e3779b97f4a7c15, Ordering::Relaxed);
                    draw = (draw ^ (draw >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                    draw = (draw ^ (draw >> 27)).wrapping_mul(0x94d049bb133111eb);
                    draw ^= draw >> 31;
                    if blackhole.load(Ordering::Relaxed) || (draw >> 32) % 100 < 5 {
                        continue;
                    }
                    let Ok(permit) = budget.clone().try_acquire_owned() else {
                        continue;
                    };
                    let delay = 25 + ((draw >> 16) % 51);
                    let duplicate = (draw >> 40) % 100 < 2;
                    let payload = buffer[..length].to_vec();
                    let destination = destination.clone();
                    let blackhole = blackhole.clone();
                    tokio::spawn(async move {
                        let _permit = permit;
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        if !blackhole.load(Ordering::Relaxed) {
                            let _ = destination.send_to(&payload, target).await;
                            if duplicate {
                                let _ = destination.send_to(&payload, target).await;
                            }
                        }
                    });
                }
            }));
        }
        Self {
            address,
            blackhole,
            tasks,
        }
    }
}

#[test]
#[ignore = "real UDP, 100ms RTT, +/-25ms per-leg jitter, 5% loss and a 10s blackhole timeout"]
fn loss_jitter_reordering_reconciliation_and_timeout() {
    midnight_network::runtime().unwrap().block_on(async {
        let (address, invite, stop, task) = server(1).await;
        let proxy = Proxy::new(address).await;
        let endpoint = endpoint(&invite);
        let mut client = Driver::new(establish(&endpoint, proxy.address, &invite).await.unwrap());
        client.drive(0.6, [0.0; 2], false).await;
        client.max_height = client.authoritative.position[1];
        client.drive(1.8, [0.0, 1.0], true).await;
        assert!(
            client.max_height > 1.3,
            "jump survives redundant input transmission"
        );
        client.drive(1.2, [0.0; 2], false).await;
        for axis in 0..3 {
            assert!(
                (client.prediction.state().position[axis] - client.authoritative.position[axis])
                    .abs()
                    < 0.05
            );
        }
        proxy.blackhole.store(true, Ordering::Relaxed);
        tokio::time::timeout(Duration::from_secs(12), client.connection.closed())
            .await
            .expect("QUIC idle timeout must close within 12s");
        tokio::time::sleep(Duration::from_millis(150)).await;
        stop.send(()).unwrap();
        assert_eq!(task.await.unwrap().population(), 0);
    });
}
