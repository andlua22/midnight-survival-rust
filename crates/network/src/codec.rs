use crate::NetResult;
use serde::{Serialize, de::DeserializeOwned};
use shared::protocol::{MAX_CONTROL_BYTES, MAX_DATAGRAM_BYTES, Snapshot};

pub fn encode<T: Serialize>(value: &T) -> NetResult<Vec<u8>> {
    postcard::to_allocvec(value).map_err(|e| e.to_string())
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8], limit: usize) -> NetResult<T> {
    if bytes.len() > limit {
        return Err("Повідомлення перевищує ліміт розміру".into());
    }
    let (value, rest) =
        postcard::take_from_bytes(bytes).map_err(|e| format!("Некоректне повідомлення: {e}"))?;
    if !rest.is_empty() {
        return Err("Зайві дані в повідомленні".into());
    }
    Ok(value)
}

pub async fn write_control<T: Serialize>(
    stream: &mut quinn::SendStream,
    value: &T,
) -> NetResult<()> {
    let bytes = encode(value)?;
    if bytes.len() > MAX_CONTROL_BYTES {
        return Err("Control message too large".into());
    }
    stream
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .await
        .map_err(|e| e.to_string())?;
    stream.write_all(&bytes).await.map_err(|e| e.to_string())
}

pub async fn read_control<T: DeserializeOwned>(stream: &mut quinn::RecvStream) -> NetResult<T> {
    let mut header = [0; 4];
    stream
        .read_exact(&mut header)
        .await
        .map_err(|e| e.to_string())?;
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > MAX_CONTROL_BYTES {
        return Err("Некоректна довжина повідомлення".into());
    }
    let mut bytes = vec![0; length];
    stream
        .read_exact(&mut bytes)
        .await
        .map_err(|e| e.to_string())?;
    decode(&bytes, MAX_CONTROL_BYTES)
}

pub fn snapshot_pages(snapshot: &Snapshot, mtu: usize) -> NetResult<Vec<Vec<u8>>> {
    let mtu = mtu.min(MAX_DATAGRAM_BYTES);
    let mut page = Snapshot {
        tick: snapshot.tick,
        population: snapshot.population,
        owner: snapshot.owner,
        players: Vec::new(),
    };
    let mut pages = Vec::new();
    // Keep the local authoritative state in its own small, independent page.
    if page.owner.is_some() {
        let bytes = encode(&page)?;
        if bytes.len() > mtu {
            return Err("QUIC datagram size is too small".into());
        }
        pages.push(bytes);
        page.owner = None;
    }
    // Postcard encodes an empty Vec length as a one-byte varint. Size each
    // replica once into a stack buffer instead of reserializing a growing page
    // for every player (quadratic work in the dense-crowd case).
    let header_size = encode(&page)?.len() - 1;
    let mut payload_size = 0;
    let mut scratch = [0u8; 128];
    for player in &snapshot.players {
        let size = postcard::to_slice(player, &mut scratch)
            .map_err(|e| e.to_string())?
            .len();
        let count = page.players.len() + 1;
        let count_size = if count < 128 { 1 } else { 2 };
        if header_size + count_size + payload_size + size > mtu && !page.players.is_empty() {
            pages.push(encode(&page)?);
            page.players.clear();
            payload_size = 0;
        }
        if header_size + 1 + size > mtu {
            return Err("Player state exceeds datagram size".into());
        }
        payload_size += size;
        page.players.push(*player);
    }
    if !page.players.is_empty() {
        pages.push(encode(&page)?);
    }
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::protocol::*;
    #[test]
    fn dense_snapshot_is_independently_decodable_and_fits_mtu() {
        let state = PlayerState::at([256.0, 2.0, -256.0]);
        let snapshot = Snapshot {
            tick: ServerTick(1000),
            population: 150,
            owner: Some(OwnerState {
                epoch: 1,
                acknowledged: ServerTick(1000),
                state,
            }),
            players: (1..150)
                .map(|id| ReplicatedPlayer {
                    player: PlayerId(id),
                    state,
                })
                .collect(),
        };
        let pages = snapshot_pages(&snapshot, 1100).unwrap();
        assert!(pages.len() > 1);
        let mut count = 0;
        for (index, bytes) in pages.iter().enumerate() {
            assert!(bytes.len() <= 1100);
            let page: Snapshot = decode(bytes, MAX_DATAGRAM_BYTES).unwrap();
            assert_eq!(page.owner.is_some(), index == 0);
            count += page.players.len();
        }
        assert_eq!(count, 149);
    }

    #[test]
    fn malformed_oversized_and_trailing_data_are_rejected() {
        assert!(decode::<InputBatch>(&[255; 32], MAX_DATAGRAM_BYTES).is_err());
        assert!(
            decode::<InputBatch>(&vec![0; MAX_DATAGRAM_BYTES + 1], MAX_DATAGRAM_BYTES).is_err()
        );
        let mut bytes = encode(&ClientControl::Resync).unwrap();
        bytes.push(0);
        assert!(decode::<ClientControl>(&bytes, MAX_CONTROL_BYTES).is_err());
        let frame = InputFrame::neutral(1, ServerTick(1), Default::default());
        let bytes = encode(&InputBatch {
            frames: vec![frame; INPUT_REDUNDANCY + 1],
        })
        .unwrap();
        assert!(decode::<InputBatch>(&bytes, MAX_DATAGRAM_BYTES).is_err());
    }

    #[test]
    fn remote_quantization_is_small_and_owner_remains_exact() {
        let mut state = PlayerState::at([-145.127, 0.9237, 256.123]);
        state.motion.yaw = 6.2;
        state.motion.pitch = -1.2;
        state.velocity = [6.781, 0.0, -2.813];
        let bytes = encode(&ReplicatedPlayer {
            player: PlayerId(149),
            state,
        })
        .unwrap();
        assert!(bytes.len() < 32);
        let remote: ReplicatedPlayer = decode(&bytes, MAX_DATAGRAM_BYTES).unwrap();
        for axis in 0..3 {
            assert!((state.position[axis] - remote.state.position[axis]).abs() <= 0.0051);
        }
        let owner = OwnerState {
            epoch: 1,
            acknowledged: ServerTick(1000),
            state,
        };
        let decoded: OwnerState = decode(&encode(&owner).unwrap(), MAX_DATAGRAM_BYTES).unwrap();
        assert_eq!(decoded.state, state);
    }
}
