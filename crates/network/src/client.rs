use crate::{
    NetResult,
    codec::{decode, encode, read_control, write_control},
    identity::{Invitation, client_endpoint, connect},
};
use shared::{level::MAP_REVISION, protocol::*};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::{Mutex, mpsc as std_mpsc},
    time::{Duration, Instant},
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    time::timeout,
};

pub enum ClientEvent {
    Control {
        message: ServerControl,
        received: Instant,
    },
    Snapshot {
        message: Snapshot,
        received: Instant,
    },
    Statistics {
        rtt: Duration,
        sent_bytes: u64,
        received_bytes: u64,
        lost_packets: u64,
    },
    Disconnected(String),
}

pub struct ConnectedClient {
    pub connection: quinn::Connection,
    pub write: quinn::SendStream,
    pub read: quinn::RecvStream,
    pub welcome: ServerControl,
}

pub fn connection_error(error: quinn::ConnectionError) -> String {
    if let quinn::ConnectionError::ApplicationClosed(close) = &error {
        let reason = String::from_utf8_lossy(&close.reason);
        if !reason.is_empty() {
            return reason.into_owned();
        }
    }
    format!("З'єднання втрачено: {error}")
}

pub async fn establish(
    endpoint: &quinn::Endpoint,
    address: SocketAddr,
    invitation: &Invitation,
) -> NetResult<ConnectedClient> {
    let connection = timeout(Duration::from_secs(5), connect(endpoint, address)?)
        .await
        .map_err(|_| "Сервер не відповідає".to_string())?
        .map_err(connection_error)?;
    let handshake = async {
        let (mut write, mut read) = connection.open_bi().await.map_err(connection_error)?;
        write_control(
            &mut write,
            &Hello {
                protocol: PROTOCOL_VERSION,
                map_revision: MAP_REVISION,
                token: invitation.token,
            },
        )
        .await?;
        let welcome: ServerControl = read_control(&mut read).await?;
        match welcome {
            ServerControl::Welcome { .. } => Ok((write, read, welcome)),
            ServerControl::Rejected(reason) => Err(reason),
            _ => Err("Некоректна відповідь сервера".into()),
        }
    };
    let (write, read, welcome) = timeout(Duration::from_secs(5), handshake)
        .await
        .map_err(|_| "Час підключення вичерпано".to_string())?
        .map_err(|reason| {
            connection
                .close_reason()
                .map(connection_error)
                .unwrap_or(reason)
        })?;
    Ok(ConnectedClient {
        connection,
        write,
        read,
        welcome,
    })
}

/// The render thread only touches bounded queues. Dropping a link cancels the
/// socket worker, including DNS/handshake, without waiting on the render thread.
pub struct ClientLink {
    inputs: watch::Sender<Option<InputBatch>>,
    control: mpsc::Sender<ClientControl>,
    events: Mutex<std_mpsc::Receiver<ClientEvent>>,
    stop: Option<oneshot::Sender<()>>,
}

impl Drop for ClientLink {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

impl ClientLink {
    pub fn start(address: String, invitation: Invitation) -> NetResult<Self> {
        let (inputs, input_receiver) = watch::channel(None);
        let (control, control_receiver) = mpsc::channel(16);
        let (send_events, events) = std_mpsc::sync_channel(1024);
        let (stop, stopped) = oneshot::channel();
        std::thread::Builder::new()
            .name("midnight-network".into())
            .spawn(move || {
                let result = match crate::runtime() {
                    Ok(runtime) => {
                        let result = runtime.block_on(run(
                            address,
                            invitation,
                            input_receiver,
                            control_receiver,
                            send_events.clone(),
                            stopped,
                        ));
                        runtime.shutdown_timeout(Duration::from_millis(100));
                        result
                    }
                    Err(error) => Err(error),
                };
                if let Err(error) = result {
                    let _ = send_events.try_send(ClientEvent::Disconnected(error));
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            inputs,
            control,
            events: Mutex::new(events),
            stop: Some(stop),
        })
    }

    pub fn send_inputs(&self, batch: InputBatch) {
        self.inputs.send_replace(Some(batch));
    }
    pub fn send_control(&self, control: ClientControl) -> NetResult<()> {
        self.control
            .try_send(control)
            .map_err(|_| "Черга команд підключення переповнена".into())
    }
    pub fn poll(&self) -> Vec<ClientEvent> {
        let Ok(events) = self.events.lock() else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for _ in 0..1024 {
            match events.try_recv() {
                Ok(event) => result.push(event),
                Err(std_mpsc::TryRecvError::Empty) => break,
                Err(std_mpsc::TryRecvError::Disconnected) => {
                    if !result
                        .iter()
                        .any(|event| matches!(event, ClientEvent::Disconnected(_)))
                    {
                        result.push(ClientEvent::Disconnected(
                            "Мережеве з'єднання завершено".into(),
                        ));
                    }
                    break;
                }
            }
        }
        result
    }
}

fn deliver(
    events: &std_mpsc::SyncSender<ClientEvent>,
    event: ClientEvent,
    reliable: bool,
) -> NetResult<()> {
    match events.try_send(event) {
        Ok(()) => Ok(()),
        Err(std_mpsc::TrySendError::Full(_)) if !reliable => Ok(()),
        Err(_) => Err("Клієнт не обробляє мережеві повідомлення".into()),
    }
}

async fn run(
    address: String,
    invitation: Invitation,
    inputs: watch::Receiver<Option<InputBatch>>,
    controls: mpsc::Receiver<ClientControl>,
    events: std_mpsc::SyncSender<ClientEvent>,
    mut stopped: oneshot::Receiver<()>,
) -> NetResult<()> {
    let address = if address.contains(':') {
        address
    } else {
        format!("{address}:5000")
    };
    let addresses = tokio::select! {
        _ = &mut stopped => return Ok(()),
        result = timeout(Duration::from_secs(5), tokio::net::lookup_host(address)) => result
            .map_err(|_| "Не вдалося визначити адресу сервера".to_string())?
            .map_err(|e| e.to_string())?,
    };
    let address = addresses
        .into_iter()
        .find(SocketAddr::is_ipv4)
        .ok_or("Потрібна IPv4-адреса сервера")?;
    let endpoint = client_endpoint((Ipv4Addr::UNSPECIFIED, 0).into(), &invitation)?;
    let result = run_session(
        &endpoint,
        address,
        &invitation,
        inputs,
        controls,
        events,
        stopped,
    )
    .await;
    // Keep the endpoint driver alive briefly so CONNECTION_CLOSE reaches the
    // server even when cancellation occurred during the application handshake.
    endpoint.close(0u8.into(), "Клієнт від'єднався".as_bytes());
    let _ = timeout(Duration::from_millis(250), endpoint.wait_idle()).await;
    result
}

async fn run_session(
    endpoint: &quinn::Endpoint,
    address: SocketAddr,
    invitation: &Invitation,
    mut inputs: watch::Receiver<Option<InputBatch>>,
    mut controls: mpsc::Receiver<ClientControl>,
    events: std_mpsc::SyncSender<ClientEvent>,
    mut stopped: oneshot::Receiver<()>,
) -> NetResult<()> {
    let ConnectedClient {
        connection,
        mut write,
        mut read,
        welcome,
    } = tokio::select! {
        _ = &mut stopped => return Ok(()),
        result = establish(endpoint, address, invitation) => result?,
    };
    deliver(
        &events,
        ClientEvent::Control {
            message: welcome,
            received: Instant::now(),
        },
        true,
    )?;
    let reliable_events = events.clone();
    let mut reader = tokio::spawn(async move {
        loop {
            let message = read_control(&mut read).await?;
            deliver(
                &reliable_events,
                ClientEvent::Control {
                    message,
                    received: Instant::now(),
                },
                true,
            )?;
        }
        #[allow(unreachable_code)]
        Ok::<_, String>(())
    });
    let mut writer = tokio::spawn(async move {
        let mut heartbeat = tokio::time::interval(Duration::from_secs(1));
        let start = Instant::now();
        loop {
            let message = tokio::select! {
                control = controls.recv() => match control { Some(c) => c, None => return Ok(()) },
                _ = heartbeat.tick() => ClientControl::Ping(start.elapsed().as_micros() as u64),
            };
            timeout(Duration::from_secs(5), write_control(&mut write, &message))
                .await
                .map_err(|_| "Сервер не приймає команди".to_string())??;
        }
    });
    let mut statistics = tokio::time::interval(Duration::from_secs(1));
    let result: NetResult<()> = async {
        loop {
            tokio::select! {
                _ = &mut stopped => return Ok(()),
                result = &mut reader => return result.map_err(|e| e.to_string())?,
                result = &mut writer => return result.map_err(|e| e.to_string())?,
                closed = connection.closed() => return Err(connection_error(closed)),
                changed = inputs.changed() => {
                    if changed.is_err() { return Ok(()); }
                    let batch = inputs.borrow_and_update().clone();
                    if let Some(batch) = batch {
                        let bytes = encode(&batch)?;
                        if bytes.len() > MAX_DATAGRAM_BYTES { return Err("Input batch exceeds datagram size".into()); }
                        connection.send_datagram(bytes.into()).map_err(|e| e.to_string())?;
                    }
                },
                datagram = connection.read_datagram() => {
                    let bytes = datagram.map_err(connection_error)?;
                    let message: Snapshot = decode(&bytes, MAX_DATAGRAM_BYTES)?;
                    deliver(&events, ClientEvent::Snapshot { message, received: Instant::now() }, false)?;
                },
                _ = statistics.tick() => {
                    let stats = connection.stats();
                    deliver(&events, ClientEvent::Statistics { rtt: connection.rtt(), sent_bytes: stats.udp_tx.bytes, received_bytes: stats.udp_rx.bytes, lost_packets: stats.path.lost_packets }, false)?;
                }
            }
        }
    }.await;
    reader.abort();
    writer.abort();
    connection.close(0u8.into(), "Клієнт від'єднався".as_bytes());
    result
}
