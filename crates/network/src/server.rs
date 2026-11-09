use crate::{
    NetResult,
    codec::{decode, read_control, snapshot_pages, write_control},
    identity::{Invitation, ServerIdentity},
};
use shared::{level::MAP_REVISION, protocol::*};
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Semaphore, mpsc, watch},
    task::JoinHandle,
    time::timeout,
};

#[derive(Clone)]
pub struct Peer {
    pub id: u64,
    pub connection: quinn::Connection,
    control: mpsc::Sender<ServerControl>,
    snapshots: watch::Sender<Option<Arc<Snapshot>>>,
}

impl Peer {
    pub fn send(&self, message: ServerControl) -> NetResult<()> {
        self.control
            .try_send(message)
            .map_err(|_| "Черга надійних повідомлень переповнена".into())
    }
    pub fn close(&self, reason: &str) {
        self.connection.close(0u8.into(), reason.as_bytes());
    }

    /// Keep only the newest state. Encoding and QUIC connection locks belong
    /// to the transport worker, so a slow peer cannot stall a simulation tick.
    pub fn send_snapshot(&self, snapshot: Snapshot) -> NetResult<()> {
        self.snapshots
            .send(Some(Arc::new(snapshot)))
            .map_err(|_| "Мережеве з'єднання завершено".into())
    }
}

pub enum ServerEvent {
    Joined(Peer),
    Inputs { peer: u64, batch: InputBatch },
    Control { peer: u64, message: ClientControl },
    Left { peer: u64 },
}

pub struct ServerTransport {
    pub endpoint: quinn::Endpoint,
    pub events: mpsc::Receiver<ServerEvent>,
    accept_task: JoinHandle<()>,
}

impl Drop for ServerTransport {
    fn drop(&mut self) {
        self.endpoint
            .close(0u8.into(), "Сервер вимкнено".as_bytes());
        self.accept_task.abort();
    }
}

impl ServerTransport {
    /// Flush an explicit close before the owner shuts down its Tokio runtime.
    pub async fn shutdown(&mut self) {
        self.endpoint
            .close(0u8.into(), "Сервер вимкнено".as_bytes());
        self.accept_task.abort();
        self.events.close();
        let _ = timeout(Duration::from_millis(250), self.endpoint.wait_idle()).await;
    }

    pub fn bind(identity: &ServerIdentity, address: SocketAddr) -> NetResult<Self> {
        let endpoint = identity.endpoint(address)?;
        let (send, events) = mpsc::channel(4096);
        let invitation = identity.invitation();
        let accept_endpoint = endpoint.clone();
        let accept_task = tokio::spawn(async move {
            let total = Arc::new(Semaphore::new(MAX_PLAYERS + 32));
            let handshakes = Arc::new(Semaphore::new(32));
            let next_id = AtomicU64::new(1);
            while let Some(incoming) = accept_endpoint.accept().await {
                if !incoming.remote_address_validated() {
                    let _ = incoming.retry();
                    continue;
                }
                let (Ok(total_permit), Ok(handshake_permit)) = (
                    total.clone().try_acquire_owned(),
                    handshakes.clone().try_acquire_owned(),
                ) else {
                    incoming.refuse();
                    continue;
                };
                let id = next_id.fetch_add(1, Ordering::Relaxed);
                let events = send.clone();
                let invitation = invitation.clone();
                tokio::spawn(async move {
                    let _permit = total_permit;
                    let admitted =
                        timeout(Duration::from_secs(5), authenticate(incoming, &invitation)).await;
                    drop(handshake_permit);
                    if let Ok(Ok((connection, write, read))) = admitted {
                        let (control, receive) = mpsc::channel(512);
                        let (snapshots, snapshot_receiver) = watch::channel(None);
                        let peer = Peer {
                            id,
                            connection: connection.clone(),
                            control,
                            snapshots,
                        };
                        if events.send(ServerEvent::Joined(peer)).await.is_ok() {
                            let result = serve_peer(
                                id,
                                connection.clone(),
                                write,
                                read,
                                receive,
                                snapshot_receiver,
                                events.clone(),
                            )
                            .await;
                            let reason =
                                result.err().unwrap_or_else(|| "З'єднання завершено".into());
                            connection.close(0u8.into(), reason.as_bytes());
                            let _ = events.send(ServerEvent::Left { peer: id }).await;
                        }
                    }
                });
            }
        });
        Ok(Self {
            endpoint,
            events,
            accept_task,
        })
    }
}

async fn authenticate(
    incoming: quinn::Incoming,
    invitation: &Invitation,
) -> NetResult<(quinn::Connection, quinn::SendStream, quinn::RecvStream)> {
    let connection = incoming.await.map_err(|e| e.to_string())?;
    let (mut write, mut read) = connection.accept_bi().await.map_err(|e| e.to_string())?;
    let hello: Hello = read_control(&mut read).await?;
    let rejection = if hello.protocol != PROTOCOL_VERSION || hello.map_revision != MAP_REVISION {
        Some("Несумісна версія гри або карти")
    } else if !invitation.authenticates(&hello.token) {
        Some("Неправильний код доступу")
    } else if connection.max_datagram_size().is_none() {
        Some("Потрібна підтримка QUIC DATAGRAM")
    } else {
        None
    };
    if let Some(reason) = rejection {
        let _ = write_control(&mut write, &ServerControl::Rejected(reason.into())).await;
        connection.close(0u8.into(), reason.as_bytes());
        return Err(reason.into());
    }
    Ok((connection, write, read))
}

struct RateLimit {
    start: Instant,
    count: u32,
    limit: u32,
}
impl RateLimit {
    fn new(limit: u32) -> Self {
        Self {
            start: Instant::now(),
            count: 0,
            limit,
        }
    }
    fn allow(&mut self) -> bool {
        if self.start.elapsed() >= Duration::from_secs(1) {
            self.start = Instant::now();
            self.count = 0;
        }
        self.count += 1;
        self.count <= self.limit
    }
}

async fn serve_peer(
    id: u64,
    connection: quinn::Connection,
    mut write: quinn::SendStream,
    mut read: quinn::RecvStream,
    mut outgoing: mpsc::Receiver<ServerControl>,
    mut snapshots: watch::Receiver<Option<Arc<Snapshot>>>,
    events: mpsc::Sender<ServerEvent>,
) -> NetResult<()> {
    let snapshot_connection = connection.clone();
    let mut snapshot_writer = tokio::spawn(async move {
        while snapshots.changed().await.is_ok() {
            let snapshot = snapshots.borrow_and_update().clone();
            let Some(snapshot) = snapshot else {
                continue;
            };
            let mtu = snapshot_connection.max_datagram_size().unwrap_or(0);
            for bytes in snapshot_pages(&snapshot, mtu)? {
                snapshot_connection
                    .send_datagram(bytes.into())
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok::<_, String>(())
    });
    let mut writer = tokio::spawn(async move {
        while let Some(message) = outgoing.recv().await {
            timeout(Duration::from_secs(5), write_control(&mut write, &message))
                .await
                .map_err(|_| "Клієнт не приймає повідомлення".to_string())??;
        }
        Ok::<_, String>(())
    });
    let control_events = events.clone();
    let mut reader = tokio::spawn(async move {
        let mut rate = RateLimit::new(20);
        loop {
            // This future is never cancelled by an arriving datagram: framing
            // must retain partially read length prefixes and message bodies.
            let message: ClientControl = read_control(&mut read).await?;
            if !rate.allow() {
                return Err("Забагато керуючих повідомлень".into());
            }
            control_events
                .send(ServerEvent::Control { peer: id, message })
                .await
                .map_err(|_| "Сервер вимкнено".to_string())?;
        }
        #[allow(unreachable_code)]
        Ok::<_, String>(())
    });
    let datagram_connection = connection.clone();
    let mut datagrams = tokio::spawn(async move {
        let mut rate = RateLimit::new(180);
        loop {
            let bytes = datagram_connection
                .read_datagram()
                .await
                .map_err(|e| e.to_string())?;
            if !rate.allow() {
                return Err("Забагато пакетів вводу".into());
            }
            let batch: InputBatch = decode(&bytes, MAX_DATAGRAM_BYTES)?;
            if batch.frames.is_empty() || batch.frames.len() > INPUT_REDUNDANCY {
                return Err("Некоректна кількість команд".into());
            }
            for frame in &batch.frames {
                frame.validate().map_err(str::to_string)?;
            }
            // A full queue discards this input batch. Redundancy and tick
            // acknowledgements recover without blocking the simulation.
            match events.try_send(ServerEvent::Inputs { peer: id, batch }) {
                Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => {}
                Err(mpsc::error::TrySendError::Closed(_)) => return Ok(()),
            }
        }
    });
    let result = tokio::select! {
        result = &mut snapshot_writer => result,
        result = &mut writer => result,
        result = &mut reader => result,
        result = &mut datagrams => result,
        _ = connection.closed() => Ok(Ok(())),
    };
    writer.abort();
    reader.abort();
    datagrams.abort();
    snapshot_writer.abort();
    result.map_err(|e| e.to_string())?
}
