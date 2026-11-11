//! Strict decoders for explicit source inventory message bodies.
//!
//! Layouts and XOR codes follow `export/gs.bag.scmsg` and `Share/Octets.cs`.
//! No network frame, timing rule, capacity default, or item effect is inferred.
//! Allocation limits and canonical boolean checks are decoder policy, not
//! recovered game limits. Signed values are retained for domain validation.
use crate::DataError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Decoder policy, independent of the source game's backpack capacity.
pub const MAX_INVENTORY_BODY_BYTES: usize = 16 * 1024 * 1024;
/// Decoder policy for each source list or map.
pub const MAX_INVENTORY_COLLECTION_ENTRIES: usize = 4096;
/// Decoder policy for one item's opaque extra-info payload.
pub const MAX_ITEM_EXTRA_BYTES: usize = 1024 * 1024;

// Native unpatched getType() observations. IFix can override these methods.
// Evidence: memory/evidence/inventory-native-gettypes/README.md.
pub const S_ITEMS_TYPE_ID: i32 = 59_771_964;
pub const S_ITEM_CHANGED_TYPE_ID: i32 = 59_771_867;
pub const S_ITEM_NUMBER_CHANGE_TYPE_ID: i32 = 59_771_840;
pub const S_ITEM_DURATION_CHANGED_TYPE_ID: i32 = 59_771_842;
pub const S_QUICK_USE_ITEM_CHANGED_TYPE_ID: i32 = 59_771_872;
pub const S_SET_QUICK_USE_ITEM_TYPE_ID: i32 = 59_771_914;
pub const S_BAG_SIZE_CHANGE_TYPE_ID: i32 = 59_771_878;
pub const S_SYNC_BAG_ITEM_TYPE_ID: i32 = 59_771_844;
pub const S_CLEAR_ALL_BAG_TYPE_ID: i32 = 59_771_886;
pub const S_PUT_TO_HAND_TYPE_ID: i32 = 59_771_892;

/// `gs.bag.scmsg.BagItem` field order, including its source instance identity.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BagItem {
    pub instance_id: i32,
    pub item_id: i32,
    pub number: i32,
    pub duration: i32,
    pub timeout: i32,
    pub is_bind: bool,
    pub bind_timeout: i32,
    pub extra_info: Vec<u8>,
}

/// `SItems` carries a separate source collection for each item location.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemsSnapshot {
    pub bag_items: Vec<BagItem>,
    pub skin_items: Vec<BagItem>,
    pub quick_use_items: BTreeMap<i32, BagItem>,
    pub equip_items: Vec<BagItem>,
    pub show_equip: bool,
    pub capacity: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemChanged {
    pub bind_timeout: i32,
    pub is_bind: bool,
    pub timeout: i32,
    pub item_id: i32,
    pub duration: i32,
    pub instance_id: i32,
    pub extra_info: Vec<u8>,
    pub num: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuickUseItemChanged {
    pub timeout: i32,
    pub index: i32,
    pub is_bind: bool,
    pub instance_id: i32,
    pub bind_timeout: i32,
    pub duration: i32,
    pub num: i32,
    pub item_id: i32,
    pub extra_info: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BagSizeChange {
    pub size: i32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemDurationChanged {
    pub duration: i32,
    pub instance_id: i32,
}

/// Item-count notification by config ID and binding, not an instance update.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemNumberChange {
    pub is_bind: bool,
    pub change: i32,
    pub item_id: i32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetQuickUseItem {
    pub index: i32,
    pub instance_id: i32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PutToHand {
    pub instance_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InventoryBody {
    Items(ItemsSnapshot),
    ItemChanged(ItemChanged),
    QuickUseItemChanged(QuickUseItemChanged),
    BagSizeChange(BagSizeChange),
    ItemDurationChanged(ItemDurationChanged),
    ItemNumberChange(ItemNumberChange),
    SetQuickUseItem(SetQuickUseItem),
    PutToHand(PutToHand),
    SyncBagItem(BagItem),
    ClearAllBag,
}

struct BodyReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl BodyReader<'_> {
    fn error(&self, message: &str) -> DataError {
        DataError(format!("inventory body byte {}: {message}", self.position))
    }

    fn take<const N: usize>(&mut self, code: u8) -> Result<[u8; N], DataError> {
        let end = self
            .position
            .checked_add(N)
            .ok_or_else(|| self.error("offset overflow"))?;
        let slice = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| self.error("truncated field"))?;
        let mut decoded = [0; N];
        for (target, source) in decoded.iter_mut().zip(slice) {
            *target = source ^ code;
        }
        self.position = end;
        Ok(decoded)
    }

    fn int(&mut self, code: u8) -> Result<i32, DataError> {
        Ok(i32::from_le_bytes(self.take(code)?))
    }

    fn boolean(&mut self, code: u8) -> Result<bool, DataError> {
        match self.take::<1>(code)?[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(self.error("noncanonical boolean")),
        }
    }

    fn length(&mut self, code: u8, limit: usize, minimum_bytes: usize) -> Result<usize, DataError> {
        let count = usize::try_from(self.int(code)?).map_err(|_| self.error("negative length"))?;
        if count > limit {
            return Err(self.error("length exceeds decoder policy"));
        }
        if count > (self.bytes.len() - self.position) / minimum_bytes {
            return Err(self.error("length exceeds remaining body"));
        }
        Ok(count)
    }

    fn octets(&mut self, code: u8) -> Result<Vec<u8>, DataError> {
        let count = self.length(code, MAX_ITEM_EXTRA_BYTES, 1)?;
        let end = self.position + count;
        let decoded = self.bytes[self.position..end]
            .iter()
            .map(|byte| byte ^ code)
            .collect();
        self.position = end;
        Ok(decoded)
    }

    fn bag_item(&mut self, code: u8) -> Result<BagItem, DataError> {
        Ok(BagItem {
            instance_id: self.int(code)?,
            item_id: self.int(code)?,
            number: self.int(code)?,
            duration: self.int(code)?,
            timeout: self.int(code)?,
            is_bind: self.boolean(code)?,
            bind_timeout: self.int(code)?,
            extra_info: self.octets(code)?,
        })
    }

    fn item_list(&mut self, code: u8) -> Result<Vec<BagItem>, DataError> {
        // Five ints, a bool, bindTimeout and an Octets length: 29 bytes.
        let count = self.length(code, MAX_INVENTORY_COLLECTION_ENTRIES, 29)?;
        (0..count).map(|_| self.bag_item(code)).collect()
    }

    fn quick_items(&mut self, code: u8) -> Result<BTreeMap<i32, BagItem>, DataError> {
        let count = self.length(code, MAX_INVENTORY_COLLECTION_ENTRIES, 33)?;
        let mut items = BTreeMap::new();
        for _ in 0..count {
            let index = self.int(code)?;
            let item = self.bag_item(code)?;
            if items.insert(index, item).is_some() {
                return Err(self.error("duplicate quick-use index"));
            }
        }
        Ok(items)
    }
}

fn decode_body<T>(
    bytes: &[u8],
    read: impl FnOnce(&mut BodyReader<'_>) -> Result<T, DataError>,
) -> Result<T, DataError> {
    if bytes.len() > MAX_INVENTORY_BODY_BYTES {
        return Err(DataError("inventory body exceeds decoder policy".into()));
    }
    let mut reader = BodyReader { bytes, position: 0 };
    let body = read(&mut reader)?;
    if reader.position != bytes.len() {
        return Err(reader.error("trailing bytes"));
    }
    Ok(body)
}

/// Decode one complete plain `BagItem` record, without a message envelope.
pub fn read_bag_item_body(bytes: &[u8]) -> Result<BagItem, DataError> {
    decode_body(bytes, |r| r.bag_item(0))
}

/// Decode one complete default `SItems` body with its per-field XOR codes.
pub fn read_items_body(bytes: &[u8]) -> Result<ItemsSnapshot, DataError> {
    decode_body(bytes, |r| {
        Ok(ItemsSnapshot {
            bag_items: r.item_list(123)?,
            skin_items: r.item_list(14)?,
            quick_use_items: r.quick_items(7)?,
            equip_items: r.item_list(63)?,
            show_equip: r.boolean(81)?,
            capacity: r.int(106)?,
        })
    })
}

pub fn read_item_changed_body(bytes: &[u8]) -> Result<ItemChanged, DataError> {
    decode_body(bytes, |r| {
        Ok(ItemChanged {
            bind_timeout: r.int(99)?,
            is_bind: r.boolean(109)?,
            timeout: r.int(100)?,
            item_id: r.int(115)?,
            duration: r.int(82)?,
            instance_id: r.int(0)?,
            extra_info: r.octets(4)?,
            num: r.int(109)?,
        })
    })
}

pub fn read_quick_use_item_changed_body(bytes: &[u8]) -> Result<QuickUseItemChanged, DataError> {
    decode_body(bytes, |r| {
        Ok(QuickUseItemChanged {
            timeout: r.int(46)?,
            index: r.int(118)?,
            is_bind: r.boolean(97)?,
            instance_id: r.int(49)?,
            bind_timeout: r.int(47)?,
            duration: r.int(126)?,
            num: r.int(67)?,
            item_id: r.int(7)?,
            extra_info: r.octets(104)?,
        })
    })
}

pub fn read_bag_size_change_body(bytes: &[u8]) -> Result<BagSizeChange, DataError> {
    decode_body(bytes, |r| Ok(BagSizeChange { size: r.int(77)? }))
}

pub fn read_item_duration_changed_body(bytes: &[u8]) -> Result<ItemDurationChanged, DataError> {
    decode_body(bytes, |r| {
        Ok(ItemDurationChanged {
            duration: r.int(101)?,
            instance_id: r.int(88)?,
        })
    })
}

pub fn read_item_number_change_body(bytes: &[u8]) -> Result<ItemNumberChange, DataError> {
    decode_body(bytes, |r| {
        Ok(ItemNumberChange {
            is_bind: r.boolean(36)?,
            change: r.int(57)?,
            item_id: r.int(123)?,
        })
    })
}

pub fn read_set_quick_use_item_body(bytes: &[u8]) -> Result<SetQuickUseItem, DataError> {
    decode_body(bytes, |r| {
        Ok(SetQuickUseItem {
            index: r.int(47)?,
            instance_id: r.int(123)?,
        })
    })
}

pub fn read_put_to_hand_body(bytes: &[u8]) -> Result<PutToHand, DataError> {
    decode_body(bytes, |r| {
        Ok(PutToHand {
            instance_id: r.int(127)?,
        })
    })
}

pub fn read_sync_bag_item_body(bytes: &[u8]) -> Result<BagItem, DataError> {
    decode_body(bytes, |r| r.bag_item(62))
}

/// Dispatch only recovered native base-method IDs; `bytes` is an explicit body.
pub fn read_inventory_body(type_id: i32, bytes: &[u8]) -> Result<InventoryBody, DataError> {
    match type_id {
        S_ITEMS_TYPE_ID => read_items_body(bytes).map(InventoryBody::Items),
        S_ITEM_CHANGED_TYPE_ID => read_item_changed_body(bytes).map(InventoryBody::ItemChanged),
        S_QUICK_USE_ITEM_CHANGED_TYPE_ID => {
            read_quick_use_item_changed_body(bytes).map(InventoryBody::QuickUseItemChanged)
        }
        S_BAG_SIZE_CHANGE_TYPE_ID => {
            read_bag_size_change_body(bytes).map(InventoryBody::BagSizeChange)
        }
        S_ITEM_DURATION_CHANGED_TYPE_ID => {
            read_item_duration_changed_body(bytes).map(InventoryBody::ItemDurationChanged)
        }
        S_ITEM_NUMBER_CHANGE_TYPE_ID => {
            read_item_number_change_body(bytes).map(InventoryBody::ItemNumberChange)
        }
        S_SET_QUICK_USE_ITEM_TYPE_ID => {
            read_set_quick_use_item_body(bytes).map(InventoryBody::SetQuickUseItem)
        }
        S_PUT_TO_HAND_TYPE_ID => read_put_to_hand_body(bytes).map(InventoryBody::PutToHand),
        S_SYNC_BAG_ITEM_TYPE_ID => read_sync_bag_item_body(bytes).map(InventoryBody::SyncBagItem),
        S_CLEAR_ALL_BAG_TYPE_ID => decode_body(bytes, |_| Ok(InventoryBody::ClearAllBag)),
        _ => Err(DataError(format!(
            "unsupported source inventory message type {type_id}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic wire fixtures, independently laid out from the source field
    // order and XOR-code table. These are not a runtime starter inventory.
    const PLAIN_ITEM: &[u8] = &[
        0x04, 0x03, 0x02, 0x01, 0x58, 0xc3, 0x00, 0x00, 0xfe, 0xff, 0xff, 0xff, 0x11, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x80, 0x01, 0xff, 0xff, 0xff, 0xff, 0x03, 0x00, 0x00, 0x00, 0x00,
        0xff, 0x81,
    ];
    const SNAPSHOT: &[u8] = &[
        0x7a, 0x7b, 0x7b, 0x7b, 0x6a, 0x7b, 0x7b, 0x7b, 0x23, 0xb8, 0x7b, 0x7b, 0x7e, 0x7b, 0x7b,
        0x7b, 0x72, 0x7b, 0x7b, 0x7b, 0x84, 0x84, 0x84, 0x84, 0x7a, 0x85, 0x84, 0x84, 0x84, 0x79,
        0x7b, 0x7b, 0x7b, 0x7b, 0x84, 0x0f, 0x0e, 0x0e, 0x0e, 0x1c, 0x0e, 0x0e, 0x0e, 0x9e, 0x30,
        0x0e, 0x0e, 0x0f, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e,
        0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x06, 0x07, 0x07, 0x07, 0xfe, 0xf8, 0xf8,
        0xf8, 0x14, 0x07, 0x07, 0x07, 0x47, 0x72, 0x07, 0x07, 0x05, 0x07, 0x07, 0x07, 0x04, 0x07,
        0x07, 0x07, 0x03, 0x07, 0x07, 0x07, 0x07, 0x02, 0x07, 0x07, 0x07, 0x06, 0x07, 0x07, 0x07,
        0x87, 0x3e, 0x3f, 0x3f, 0x3f, 0x2b, 0x3f, 0x3f, 0x3f, 0x26, 0x46, 0x3f, 0x3f, 0x3e, 0x3f,
        0x3f, 0x3f, 0xc0, 0xc0, 0xc0, 0xc0, 0x3f, 0x3f, 0x3f, 0x3f, 0x3e, 0x3f, 0x3f, 0x3f, 0x3f,
        0x3d, 0x3f, 0x3f, 0x3f, 0x2f, 0x1f, 0x50, 0x76, 0x6a, 0x6a, 0x6a,
    ];
    const ITEM_CHANGED: &[u8] = &[
        0x9c, 0x9c, 0x9c, 0x9c, 0x6c, 0x9a, 0x9b, 0x9b, 0x9b, 0x2b, 0xb0, 0x73, 0x73, 0x43, 0x52,
        0x52, 0x52, 0x04, 0x03, 0x02, 0x01, 0x07, 0x04, 0x04, 0x04, 0x04, 0xfb, 0x85, 0x90, 0x92,
        0x92, 0x92,
    ];
    const QUICK_CHANGED: &[u8] = &[
        0xd8, 0xd1, 0xd1, 0xd1, 0x8f, 0x89, 0x89, 0x89, 0x60, 0x6a, 0x31, 0x31, 0x31, 0xd1, 0xd0,
        0xd0, 0xd0, 0x7b, 0x7e, 0x7e, 0x7e, 0x40, 0x43, 0x43, 0x43, 0x5f, 0xc4, 0x07, 0x07, 0x6a,
        0x68, 0x68, 0x68, 0x68, 0x97,
    ];

    fn plain_item() -> BagItem {
        BagItem {
            instance_id: 0x0102_0304,
            item_id: 50008,
            number: -2,
            duration: 17,
            timeout: i32::MIN,
            is_bind: true,
            bind_timeout: -1,
            extra_info: vec![0, 255, 129],
        }
    }

    #[test]
    fn plain_and_uniform_code_records_preserve_signed_fields_and_opaque_octets() {
        assert_eq!(read_bag_item_body(PLAIN_ITEM).unwrap(), plain_item());
        let coded: Vec<_> = PLAIN_ITEM.iter().map(|byte| byte ^ 0xab).collect();
        assert_eq!(
            decode_body(&coded, |r| r.bag_item(0xab)).unwrap(),
            plain_item()
        );
        let synced: Vec<_> = PLAIN_ITEM.iter().map(|byte| byte ^ 62).collect();
        assert_eq!(read_sync_bag_item_body(&synced).unwrap(), plain_item());
    }

    #[test]
    fn source_snapshot_masks_decode_distinct_collections_and_signed_quick_index() {
        let snapshot = read_items_body(SNAPSHOT).unwrap();
        assert_eq!((snapshot.capacity, snapshot.show_equip), (28, true));
        assert_eq!(
            snapshot.bag_items,
            vec![BagItem {
                instance_id: 17,
                item_id: 50008,
                number: 5,
                duration: 9,
                timeout: -1,
                is_bind: true,
                bind_timeout: -2,
                extra_info: vec![0, 255],
            }]
        );
        assert_eq!(
            snapshot.skin_items,
            vec![BagItem {
                instance_id: 18,
                item_id: 16016,
                number: 1,
                duration: 0,
                timeout: 0,
                is_bind: false,
                bind_timeout: 0,
                extra_info: vec![],
            }]
        );
        assert_eq!(
            snapshot.quick_use_items,
            BTreeMap::from([(
                -7,
                BagItem {
                    instance_id: 19,
                    item_id: 30016,
                    number: 2,
                    duration: 3,
                    timeout: 4,
                    is_bind: false,
                    bind_timeout: 5,
                    extra_info: vec![128],
                }
            )])
        );
        assert_eq!(
            snapshot.equip_items,
            vec![BagItem {
                instance_id: 20,
                item_id: 31001,
                number: 1,
                duration: -1,
                timeout: 0,
                is_bind: true,
                bind_timeout: 0,
                extra_info: vec![16, 32],
            }]
        );
        assert_eq!(
            read_inventory_body(S_ITEMS_TYPE_ID, SNAPSHOT).unwrap(),
            InventoryBody::Items(snapshot)
        );
    }

    #[test]
    fn change_bodies_follow_their_own_orders_and_masks() {
        assert_eq!(
            read_item_changed_body(ITEM_CHANGED).unwrap(),
            ItemChanged {
                bind_timeout: -1,
                is_bind: true,
                timeout: -2,
                item_id: 50008,
                duration: 17,
                instance_id: 0x0102_0304,
                extra_info: vec![0, 255, 129],
                num: -3,
            }
        );
        assert_eq!(
            read_quick_use_item_changed_body(QUICK_CHANGED).unwrap(),
            QuickUseItemChanged {
                timeout: -10,
                index: -7,
                is_bind: true,
                instance_id: 91,
                bind_timeout: -2,
                duration: 5,
                num: 3,
                item_id: 50008,
                extra_info: vec![0, 255],
            }
        );
        assert_eq!(
            read_bag_size_change_body(&[0xb2, 0xb2, 0xb2, 0xb2]).unwrap(),
            BagSizeChange { size: -1 }
        );
        assert_eq!(
            read_item_duration_changed_body(&[0x9a, 0x9a, 0x9a, 0x9a, 0x4b, 0x58, 0x58, 0x58])
                .unwrap(),
            ItemDurationChanged {
                duration: -1,
                instance_id: 19
            }
        );
        assert_eq!(
            read_item_number_change_body(&[0x25, 0xc7, 0xc6, 0xc6, 0xc6, 0x23, 0xb8, 0x7b, 0x7b])
                .unwrap(),
            ItemNumberChange {
                is_bind: true,
                change: -2,
                item_id: 50008
            }
        );
        assert_eq!(
            read_set_quick_use_item_body(&[0xd6, 0xd0, 0xd0, 0xd0, 0x68, 0x7b, 0x7b, 0x7b])
                .unwrap(),
            SetQuickUseItem {
                index: -7,
                instance_id: 19
            }
        );
        assert_eq!(
            read_put_to_hand_body(&[0x6c, 0x7f, 0x7f, 0x7f]).unwrap(),
            PutToHand { instance_id: 19 }
        );
    }

    #[test]
    fn every_truncated_fixture_and_trailing_byte_is_rejected() {
        for (type_id, bytes) in [
            (S_ITEMS_TYPE_ID, SNAPSHOT),
            (S_ITEM_CHANGED_TYPE_ID, ITEM_CHANGED),
            (S_QUICK_USE_ITEM_CHANGED_TYPE_ID, QUICK_CHANGED),
        ] {
            for end in 0..bytes.len() {
                assert!(
                    read_inventory_body(type_id, &bytes[..end]).is_err(),
                    "type {type_id}, prefix {end}"
                );
            }
            let mut trailing = bytes.to_vec();
            trailing.push(0);
            assert!(read_inventory_body(type_id, &trailing).is_err());
        }
        for end in 0..PLAIN_ITEM.len() {
            assert!(read_bag_item_body(&PLAIN_ITEM[..end]).is_err());
        }
        let mut trailing = PLAIN_ITEM.to_vec();
        trailing.push(0);
        assert!(read_bag_item_body(&trailing).is_err());
    }

    #[test]
    fn invalid_lengths_counts_and_booleans_fail_before_publication() {
        let mut invalid = PLAIN_ITEM.to_vec();
        invalid[20] = 2;
        assert!(read_bag_item_body(&invalid).is_err());
        for length in [-1, i32::MAX, 4] {
            let mut invalid = PLAIN_ITEM.to_vec();
            invalid[25..29].copy_from_slice(&length.to_le_bytes());
            assert!(read_bag_item_body(&invalid).is_err());
        }
        for count in [-1, MAX_INVENTORY_COLLECTION_ENTRIES as i32 + 1, i32::MAX] {
            let mut invalid = SNAPSHOT.to_vec();
            invalid[..4].copy_from_slice(&count.to_le_bytes().map(|byte| byte ^ 123));
            assert!(read_items_body(&invalid).is_err());
        }
        assert!(read_items_body(&[0x7a, 0x7b, 0x7b, 0x7b]).is_err());
        let mut invalid = SNAPSHOT.to_vec();
        let show_equip = invalid.len() - 5;
        invalid[show_equip] = 2 ^ 81;
        assert!(read_items_body(&invalid).is_err());
    }

    #[test]
    fn duplicate_quick_indexes_reject_the_complete_body() {
        let mut bytes = vec![123; 4];
        bytes.extend([14; 4]);
        bytes.extend([2 ^ 7, 7, 7, 7]);
        for _ in 0..2 {
            bytes.extend((-7i32).to_le_bytes().map(|byte| byte ^ 7));
            bytes.extend(PLAIN_ITEM.iter().map(|byte| byte ^ 7));
        }
        bytes.extend([63; 4]);
        bytes.push(81);
        bytes.extend([106; 4]);
        let error = read_items_body(&bytes).unwrap_err();
        assert!(error.0.contains("duplicate quick-use index"));
    }

    #[test]
    fn dispatcher_accepts_only_known_complete_bodies() {
        assert_eq!(
            read_inventory_body(S_CLEAR_ALL_BAG_TYPE_ID, &[]).unwrap(),
            InventoryBody::ClearAllBag
        );
        assert!(read_inventory_body(S_CLEAR_ALL_BAG_TYPE_ID, &[0]).is_err());
        assert!(read_inventory_body(0, &[]).is_err());
        let oversized = vec![0; MAX_INVENTORY_BODY_BYTES + 1];
        assert!(read_items_body(&oversized).is_err());
    }
}
