//! Strict source configuration readers for the APK's concatenated Octets records.
//! Field order follows cfg declarations and Unmarshal pop sequences in export/.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub mod clearance;
pub mod controller;
pub mod inventory_wire;
pub mod package;
pub mod player_state;
pub mod posture;
pub mod source_bag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataError(pub String);
impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for DataError {}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], DataError> {
        let end = self
            .position
            .checked_add(N)
            .ok_or_else(|| self.error("offset overflow"))?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| self.error("truncated record"))?;
        let mut result = [0; N];
        result.copy_from_slice(bytes);
        self.position = end;
        Ok(result)
    }
    fn error(&self, message: &str) -> DataError {
        DataError(format!("byte {}: {message}", self.position))
    }
    fn int(&mut self) -> Result<i32, DataError> {
        Ok(i32::from_le_bytes(self.take()?))
    }
    fn float(&mut self) -> Result<f32, DataError> {
        let value = f32::from_le_bytes(self.take()?);
        if !value.is_finite() {
            return Err(self.error("non-finite float"));
        }
        Ok(value)
    }
    fn boolean(&mut self) -> Result<bool, DataError> {
        match self.take::<1>()?[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(self.error("invalid boolean")),
        }
    }
    fn length(&mut self, limit: usize) -> Result<usize, DataError> {
        let count = usize::try_from(self.int()?).map_err(|_| self.error("negative length"))?;
        if count > limit || count > self.bytes.len() - self.position {
            return Err(self.error("length exceeds data or limit"));
        }
        Ok(count)
    }
    fn string(&mut self) -> Result<String, DataError> {
        let count = self.length(1024 * 1024)?;
        let end = self.position + count;
        let text = std::str::from_utf8(&self.bytes[self.position..end])
            .map_err(|_| self.error("invalid UTF-8"))?
            .to_owned();
        self.position = end;
        Ok(text)
    }
    fn list<T>(
        &mut self,
        mut read: impl FnMut(&mut Self) -> Result<T, DataError>,
    ) -> Result<Vec<T>, DataError> {
        let count = self.length(4096)?;
        (0..count).map(|_| read(self)).collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemConfig {
    pub id: i32,
    pub name: String,
    pub kind: i32,
    pub child_type: i32,
    pub quality: i32,
    pub can_discard: bool,
    pub durability: i32,
    pub durability_down_value: i32,
    pub quick_use: bool,
    pub can_use: bool,
    pub icon: String,
    pub description: String,
    pub sell_scene: i32,
    pub sell_type: i32,
    pub price: i32,
    pub item_id: i32,
    pub pileable: bool,
    pub drop_id: i32,
    pub level: i32,
    pub model_path: String,
    pub max_pile: i32,
    pub drop_model: String,
    pub use_time: f32,
    pub exist_time: i32,
    pub extras: Vec<f32>,
    pub extra_strings: Vec<String>,
    pub sex_limit: i32,
    pub model_show_position: String,
    pub model_show_angle: String,
    pub model_show_scale: String,
    pub gift_preview: bool,
    pub order: i32,
    pub can_save: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecipeConfig {
    pub id: i32,
    pub time: i32,
    pub material: BTreeMap<i32, i32>,
    pub output_count: i32,
}

/// Blueprint fields in cfg.DrawingCfg declaration/Unmarshal order. Mode,
/// workbench and unlock flags are retained without inferring eligibility.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DrawingConfig {
    pub id: i32,
    pub name: String,
    pub big_type: i32,
    pub small_type: i32,
    pub normal: bool,
    pub show_locked: bool,
    pub default_unlocked: bool,
    pub civilization_normal: bool,
    pub civilization_show_locked: bool,
    pub civilization_default: bool,
    pub nostalgia: bool,
    pub nostalgia_show_locked: bool,
    pub nostalgia_default: bool,
    pub rare: bool,
    pub level_limit: i32,
    pub workbench: i32,
    pub workbench_level: i32,
    pub civilized_workbench: i32,
    pub civilized_workbench_level: i32,
    pub time: i32,
    pub material: BTreeMap<i32, i32>,
    pub target_item_id: i32,
    pub once_max_create: i32,
    pub order: i32,
    pub base_number: i32,
    pub product_count: i32,
    pub talent: bool,
    pub info_panel_type: i32,
    pub battle_type_limit: i32,
    pub toolbox_level: i32,
    pub custom_need_show: bool,
    pub firepower_normal: bool,
    pub firepower_show_locked: bool,
    pub firepower_default: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreeConfig {
    pub id: i32,
    pub name: String,
    pub hp: i32,
    pub item_id: i32,
    pub cutting_tools: Vec<i32>,
    pub tree_type: i32,
    pub down_effect: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EquipmentSlots {
    pub id: i32,
    pub slots: Vec<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UseItemConfig {
    pub id: i32,
    pub standing_time: f32,
    pub lying_time: f32,
    pub walk_speed_rate: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GameplayCatalog {
    pub items: BTreeMap<i32, ItemConfig>,
    pub recipes: BTreeMap<i32, RecipeConfig>,
    pub drawings: BTreeMap<i32, DrawingConfig>,
    pub trees: BTreeMap<i32, TreeConfig>,
    pub equipment_slots: BTreeMap<i32, EquipmentSlots>,
    pub use_items: BTreeMap<i32, UseItemConfig>,
}

fn records<T>(
    bytes: &[u8],
    mut read: impl FnMut(&mut Reader<'_>) -> Result<(i32, T), DataError>,
) -> Result<BTreeMap<i32, T>, DataError> {
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(DataError("configuration exceeds 64 MiB".into()));
    }
    let mut reader = Reader { bytes, position: 0 };
    let mut result = BTreeMap::new();
    while reader.position < bytes.len() {
        if result.len() >= 100_000 {
            return Err(reader.error("record count exceeds limit"));
        }
        let (id, value) = read(&mut reader)?;
        if result.insert(id, value).is_some() {
            return Err(reader.error("duplicate record ID"));
        }
    }
    Ok(result)
}

pub fn read_items(bytes: &[u8]) -> Result<BTreeMap<i32, ItemConfig>, DataError> {
    records(bytes, |r| {
        let item = ItemConfig {
            id: r.int()?,
            name: r.string()?,
            kind: r.int()?,
            child_type: r.int()?,
            quality: r.int()?,
            can_discard: r.boolean()?,
            durability: r.int()?,
            durability_down_value: r.int()?,
            quick_use: r.boolean()?,
            can_use: r.boolean()?,
            icon: r.string()?,
            description: r.string()?,
            sell_scene: r.int()?,
            sell_type: r.int()?,
            price: r.int()?,
            item_id: r.int()?,
            pileable: r.boolean()?,
            drop_id: r.int()?,
            level: r.int()?,
            model_path: r.string()?,
            max_pile: r.int()?,
            drop_model: r.string()?,
            use_time: r.float()?,
            exist_time: r.int()?,
            extras: r.list(Reader::float)?,
            extra_strings: r.list(Reader::string)?,
            sex_limit: r.int()?,
            model_show_position: r.string()?,
            model_show_angle: r.string()?,
            model_show_scale: r.string()?,
            gift_preview: r.boolean()?,
            order: r.int()?,
            can_save: r.boolean()?,
        };
        Ok((item.id, item))
    })
}

pub fn read_recipes(bytes: &[u8]) -> Result<BTreeMap<i32, RecipeConfig>, DataError> {
    records(bytes, |r| {
        let id = r.int()?;
        let time = r.int()?;
        let pairs = r.list(|r| Ok((r.int()?, r.int()?)))?;
        let mut material = BTreeMap::new();
        for (item, count) in pairs {
            if count <= 0 || material.insert(item, count).is_some() {
                return Err(r.error("invalid or duplicate recipe material"));
            }
        }
        let output_count = r.int()?;
        if time < 0 || output_count <= 0 {
            return Err(r.error("invalid recipe timing/output"));
        }
        Ok((
            id,
            RecipeConfig {
                id,
                time,
                material,
                output_count,
            },
        ))
    })
}

pub fn read_drawings(bytes: &[u8]) -> Result<BTreeMap<i32, DrawingConfig>, DataError> {
    records(bytes, |r| {
        let id = r.int()?;
        let name = r.string()?;
        let big_type = r.int()?;
        let small_type = r.int()?;
        let normal = r.boolean()?;
        let show_locked = r.boolean()?;
        let default_unlocked = r.boolean()?;
        let civilization_normal = r.boolean()?;
        let civilization_show_locked = r.boolean()?;
        let civilization_default = r.boolean()?;
        let nostalgia = r.boolean()?;
        let nostalgia_show_locked = r.boolean()?;
        let nostalgia_default = r.boolean()?;
        let rare = r.boolean()?;
        let level_limit = r.int()?;
        let workbench = r.int()?;
        let workbench_level = r.int()?;
        let civilized_workbench = r.int()?;
        let civilized_workbench_level = r.int()?;
        let time = r.int()?;
        let mut material = BTreeMap::new();
        for (item, count) in r.list(|r| Ok((r.int()?, r.int()?)))? {
            if count <= 0 || material.insert(item, count).is_some() {
                return Err(r.error("invalid or duplicate blueprint material"));
            }
        }
        let drawing = DrawingConfig {
            id,
            name,
            big_type,
            small_type,
            normal,
            show_locked,
            default_unlocked,
            civilization_normal,
            civilization_show_locked,
            civilization_default,
            nostalgia,
            nostalgia_show_locked,
            nostalgia_default,
            rare,
            level_limit,
            workbench,
            workbench_level,
            civilized_workbench,
            civilized_workbench_level,
            time,
            material,
            target_item_id: r.int()?,
            once_max_create: r.int()?,
            order: r.int()?,
            base_number: r.int()?,
            product_count: r.int()?,
            talent: r.boolean()?,
            info_panel_type: r.int()?,
            battle_type_limit: r.int()?,
            toolbox_level: r.int()?,
            custom_need_show: r.boolean()?,
            firepower_normal: r.boolean()?,
            firepower_show_locked: r.boolean()?,
            firepower_default: r.boolean()?,
        };
        Ok((id, drawing))
    })
}

pub fn read_trees(bytes: &[u8]) -> Result<BTreeMap<i32, TreeConfig>, DataError> {
    records(bytes, |r| {
        let tree = TreeConfig {
            id: r.int()?,
            name: r.string()?,
            hp: r.int()?,
            item_id: r.int()?,
            cutting_tools: r.list(Reader::int)?,
            tree_type: r.int()?,
            down_effect: r.int()?,
        };
        Ok((tree.id, tree))
    })
}

pub fn read_equipment_slots(bytes: &[u8]) -> Result<BTreeMap<i32, EquipmentSlots>, DataError> {
    records(bytes, |r| {
        let config = EquipmentSlots {
            id: r.int()?,
            slots: r.list(Reader::int)?,
        };
        Ok((config.id, config))
    })
}

pub fn read_use_items(bytes: &[u8]) -> Result<BTreeMap<i32, UseItemConfig>, DataError> {
    records(bytes, |r| {
        let config = UseItemConfig {
            id: r.int()?,
            standing_time: r.float()?,
            lying_time: r.float()?,
            walk_speed_rate: r.float()?,
        };
        Ok((config.id, config))
    })
}

/// Variable-capacity inventory retaining source item IDs. Capacity is supplied
/// by the caller's recovered bag rules, never inferred from prototype slots.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceInventory {
    pub slots: Vec<Option<SourceStack>>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceStack {
    pub item: i32,
    pub count: u32,
}
impl SourceInventory {
    pub fn new(capacity: usize) -> Result<Self, DataError> {
        if capacity > 4096 {
            return Err(DataError("inventory exceeds slot limit".into()));
        }
        Ok(Self {
            slots: vec![None; capacity],
        })
    }
    pub fn validate(&self, items: &BTreeMap<i32, ItemConfig>) -> Result<(), DataError> {
        if self.slots.len() > 4096 {
            return Err(DataError("inventory exceeds slot limit".into()));
        }
        for stack in self.slots.iter().flatten() {
            if stack.count == 0 || stack.count > stack_limit(items, stack.item)? {
                return Err(DataError("invalid stack count".into()));
            }
        }
        Ok(())
    }
    pub fn insert(
        &mut self,
        items: &BTreeMap<i32, ItemConfig>,
        item: i32,
        count: u32,
    ) -> Result<(), DataError> {
        self.validate(items)?;
        if count == 0 {
            return Err(DataError("empty insertion".into()));
        }
        let limit = stack_limit(items, item)?;
        let mut staged = self.clone();
        let mut remaining = count;
        for stack in staged
            .slots
            .iter_mut()
            .flatten()
            .filter(|stack| stack.item == item)
        {
            let added = remaining.min(limit - stack.count);
            stack.count += added;
            remaining -= added;
        }
        for slot in staged.slots.iter_mut().filter(|slot| slot.is_none()) {
            if remaining == 0 {
                break;
            }
            let added = remaining.min(limit);
            *slot = Some(SourceStack { item, count: added });
            remaining -= added;
        }
        if remaining != 0 {
            return Err(DataError("inventory is full".into()));
        }
        *self = staged;
        Ok(())
    }
    pub fn remove(&mut self, item: i32, count: u32) -> Result<(), DataError> {
        if self.slots.len() > 4096 {
            return Err(DataError("inventory exceeds slot limit".into()));
        }
        let available: u64 = self
            .slots
            .iter()
            .flatten()
            .filter(|stack| stack.item == item)
            .map(|stack| u64::from(stack.count))
            .sum();
        if count == 0 || available < u64::from(count) {
            return Err(DataError("insufficient materials".into()));
        }
        let mut remaining = count;
        for slot in &mut self.slots {
            if let Some(stack) = slot
                && stack.item == item
            {
                let removed = remaining.min(stack.count);
                stack.count -= removed;
                remaining -= removed;
                if stack.count == 0 {
                    *slot = None;
                }
            }
        }
        Ok(())
    }

    /// Applies one externally authorized, completed craft atomically. The caller
    /// owns time conversion, unlock/workbench checks and authoritative scheduling.
    pub fn complete_drawing(
        &mut self,
        catalog: &GameplayCatalog,
        drawing_id: i32,
    ) -> Result<(), DataError> {
        self.validate(&catalog.items)?;
        let drawing = catalog
            .drawings
            .get(&drawing_id)
            .ok_or_else(|| DataError(format!("unknown blueprint {drawing_id}")))?;
        let output = u32::try_from(drawing.product_count)
            .ok()
            .filter(|&count| count > 0)
            .ok_or_else(|| DataError("blueprint output count is unresolved".into()))?;
        let mut staged = self.clone();
        for (&item, &count) in &drawing.material {
            let count = u32::try_from(count)
                .ok()
                .filter(|&count| count > 0)
                .ok_or_else(|| DataError("invalid blueprint material count".into()))?;
            staged.remove(item, count)?;
        }
        staged.insert(&catalog.items, drawing.target_item_id, output)?;
        *self = staged;
        Ok(())
    }
}
fn stack_limit(items: &BTreeMap<i32, ItemConfig>, item: i32) -> Result<u32, DataError> {
    let config = items
        .get(&item)
        .ok_or_else(|| DataError(format!("unknown source item {item}")))?;
    if !config.pileable {
        return Ok(1);
    }
    u32::try_from(config.max_pile)
        .ok()
        .filter(|&limit| limit > 0)
        .ok_or_else(|| {
            DataError(format!(
                "item {item} has no finite pile limit; runtime handling is unresolved"
            ))
        })
}

impl GameplayCatalog {
    pub fn validate_references(&self) -> Vec<String> {
        let mut missing = BTreeSet::new();
        for recipe in self.recipes.values() {
            for id in recipe.material.keys() {
                if !self.items.contains_key(id) {
                    missing.insert(format!("recipe {} references item {id}", recipe.id));
                }
            }
        }
        for drawing in self.drawings.values() {
            for id in drawing
                .material
                .keys()
                .chain(std::iter::once(&drawing.target_item_id))
            {
                if !self.items.contains_key(id) {
                    missing.insert(format!("blueprint {} references item {id}", drawing.id));
                }
            }
        }
        missing.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn item(id: i32, pileable: bool, max_pile: i32) -> ItemConfig {
        ItemConfig {
            id,
            name: format!("item-{id}"),
            kind: 0,
            child_type: 0,
            quality: 0,
            can_discard: true,
            durability: 0,
            durability_down_value: 0,
            quick_use: false,
            can_use: false,
            icon: String::new(),
            description: String::new(),
            sell_scene: 0,
            sell_type: 0,
            price: 0,
            item_id: 0,
            pileable,
            drop_id: 0,
            level: 0,
            model_path: String::new(),
            max_pile,
            drop_model: String::new(),
            use_time: 0.0,
            exist_time: 0,
            extras: vec![],
            extra_strings: vec![],
            sex_limit: 0,
            model_show_position: String::new(),
            model_show_angle: String::new(),
            model_show_scale: String::new(),
            gift_preview: false,
            order: 0,
            can_save: true,
        }
    }

    fn blueprint_bytes() -> Vec<u8> {
        let mut raw = Vec::new();
        for n in [70001i32, 0, 1, 2] {
            raw.extend(n.to_le_bytes());
        }
        raw.extend([1, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        for n in [
            0i32, 1, 2, 0, 0, 15, 2, 50001, 3, 50002, 2, 60001, 10, 1, 1, 2,
        ] {
            raw.extend(n.to_le_bytes());
        }
        raw.push(0);
        for n in [0i32, 0, 0] {
            raw.extend(n.to_le_bytes());
        }
        raw.extend([0, 0, 0, 0]);
        raw
    }

    pub(crate) fn catalog() -> GameplayCatalog {
        GameplayCatalog {
            items: BTreeMap::from([
                (50001, item(50001, true, 5)),
                (50002, item(50002, true, 4)),
                (60001, item(60001, false, 0)),
                (60002, item(60002, true, 0)),
            ]),
            recipes: BTreeMap::new(),
            drawings: read_drawings(&blueprint_bytes()).unwrap(),
            trees: BTreeMap::new(),
            equipment_slots: BTreeMap::new(),
            use_items: BTreeMap::new(),
        }
    }
    #[test]
    fn decodes_source_recipe_and_rejects_truncation_duplicate_or_negative_lengths() {
        let values: [i32; 8] = [51010, 5, 2, 50008, 3, 50010, 1, 1];
        let raw: Vec<_> = values.into_iter().flat_map(i32::to_le_bytes).collect();
        let records = read_recipes(&raw).unwrap();
        assert_eq!(
            records[&51010].material,
            BTreeMap::from([(50008, 3), (50010, 1)])
        );
        for length in 1..raw.len() {
            assert!(read_recipes(&raw[..length]).is_err());
        }
        assert!(read_recipes(&[raw.clone(), raw.clone()].concat()).is_err());
        let mut corrupt = raw;
        corrupt[8..12].copy_from_slice(&(-1i32).to_le_bytes());
        assert!(read_recipes(&corrupt).is_err());
    }
    #[test]
    fn malformed_utf8_boolean_and_nan_fail_before_publication() {
        let mut reader = Reader {
            bytes: &[1, 0, 0, 0, 0xff],
            position: 0,
        };
        assert!(reader.string().is_err());
        assert!(
            Reader {
                bytes: &[2],
                position: 0
            }
            .boolean()
            .is_err()
        );
        assert!(
            Reader {
                bytes: &f32::NAN.to_le_bytes(),
                position: 0
            }
            .float()
            .is_err()
        );
    }

    #[test]
    fn blueprints_preserve_requirements_and_reject_every_truncated_record() {
        let bytes = blueprint_bytes();
        let blueprint = read_drawings(&bytes).unwrap().remove(&70001).unwrap();
        assert_eq!(blueprint.material, BTreeMap::from([(50001, 3), (50002, 2)]));
        assert_eq!(
            (
                blueprint.workbench,
                blueprint.workbench_level,
                blueprint.time
            ),
            (1, 2, 15)
        );
        assert_eq!(
            (blueprint.target_item_id, blueprint.product_count),
            (60001, 2)
        );
        assert!(blueprint.default_unlocked);
        for length in 1..bytes.len() {
            assert!(read_drawings(&bytes[..length]).is_err());
        }
    }

    #[test]
    fn source_stacks_respect_capacity_and_unknown_or_unresolved_items_roll_back() {
        let catalog = catalog();
        let mut inventory = SourceInventory::new(2).unwrap();
        inventory.insert(&catalog.items, 50001, 7).unwrap();
        assert_eq!(
            inventory.slots,
            vec![
                Some(SourceStack {
                    item: 50001,
                    count: 5
                }),
                Some(SourceStack {
                    item: 50001,
                    count: 2
                }),
            ]
        );
        let before = inventory.clone();
        for (item, count) in [(50001, 4), (123456, 1), (60002, 1), (50001, 0)] {
            assert!(inventory.insert(&catalog.items, item, count).is_err());
            assert_eq!(inventory, before);
        }
        assert!(inventory.remove(50001, 8).is_err());
        assert_eq!(inventory, before);
        inventory.remove(50001, 7).unwrap();
        assert!(inventory.slots.iter().all(Option::is_none));
    }

    #[test]
    fn completed_craft_consumes_exact_materials_and_never_loses_items_on_failure() {
        let catalog = catalog();
        let mut inventory = SourceInventory::new(3).unwrap();
        inventory.insert(&catalog.items, 50001, 4).unwrap();
        inventory.insert(&catalog.items, 50002, 2).unwrap();
        inventory.complete_drawing(&catalog, 70001).unwrap();
        inventory.validate(&catalog.items).unwrap();
        assert_eq!(
            inventory
                .slots
                .iter()
                .flatten()
                .filter(|s| s.item == 60001)
                .count(),
            2
        );
        assert_eq!(
            inventory
                .slots
                .iter()
                .flatten()
                .find(|s| s.item == 50001)
                .unwrap()
                .count,
            1
        );
        let before = inventory.clone();
        assert!(inventory.complete_drawing(&catalog, 70001).is_err());
        assert_eq!(inventory, before);

        let mut full = SourceInventory::new(2).unwrap();
        full.insert(&catalog.items, 50001, 4).unwrap();
        full.insert(&catalog.items, 50002, 2).unwrap();
        let before = full.clone();
        assert!(full.complete_drawing(&catalog, 70001).is_err());
        assert_eq!(full, before);
    }
}
