//! Shared authoritative survival state: vitals, inventory, crafting and resource harvesting.
//! Used by both offline simulation and the dedicated multiplayer server.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    level::MapId,
    movement::EYE_OFFSET,
    protocol::{PlayerId, ServerTick},
};

pub const MAX_HEALTH: f32 = 100.0;
pub const MAX_HYDRATION: f32 = 250.0;
pub const MAX_CALORIES: f32 = 500.0;
pub const MAX_RADIATION: f32 = 100.0;
pub const SPRINT_MIN_HYDRATION: f32 = 40.0;
pub const NORMAL_BODY_TEMP_C: f32 = 36.6;
pub const CAMPFIRE_WARMTH_RADIUS: f32 = 8.0;
pub const DAY_CYCLE_TICKS: u64 = 60 * 300;
pub const RADTOWN_CENTER: [f32; 3] = [-22.0, 0.0, -10.5];
pub const RADTOWN_RADIUS: f32 = 6.5;
pub const NUCLEAR_STRIKE_CENTER: [f32; 3] = [8.0, 0.0, 14.0];
pub const NUCLEAR_STRIKE_RADIUS: f32 = 10.5;
pub const NUCLEAR_SIREN_TICKS: u32 = 60 * 15;
pub const NUCLEAR_SHOCKWAVE_TICKS: u32 = 60 * 8;
pub const NUCLEAR_FALLOUT_TICKS: u32 = 60 * 60;

pub const HOTBAR_SLOTS: usize = 6;
pub const BACKPACK_SLOTS: usize = 18;
pub const ACTION_COOLDOWN_TICKS: u64 = 36;
pub const NODE_RESPAWN_TICKS: u64 = 60 * 30;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vitals {
    pub health: f32,
    pub hydration: f32,
    pub calories: f32,
    pub body_temp_c: f32,
    pub comfort: f32,
    pub radiation: f32,
}

impl Default for Vitals {
    fn default() -> Self {
        Self {
            health: MAX_HEALTH,
            hydration: MAX_HYDRATION,
            calories: MAX_CALORIES,
            body_temp_c: NORMAL_BODY_TEMP_C,
            comfort: 0.0,
            radiation: 0.0,
        }
    }
}

impl Vitals {
    pub fn can_sprint(&self) -> bool {
        self.hydration >= SPRINT_MIN_HYDRATION
    }

    pub fn step(&mut self, sprinting: bool, dt: f32) {
        self.step_environment(sprinting, 18.0, 1.0, false, dt);
    }

    pub fn step_environment(
        &mut self,
        sprinting: bool,
        ambient_temp_c: f32,
        hydration_mult: f32,
        near_campfire: bool,
        dt: f32,
    ) {
        self.step_full(
            sprinting,
            ambient_temp_c,
            hydration_mult,
            near_campfire,
            false,
            dt,
        );
    }

    pub fn step_full(
        &mut self,
        sprinting: bool,
        ambient_temp_c: f32,
        hydration_mult: f32,
        near_campfire: bool,
        in_radiation_zone: bool,
        dt: f32,
    ) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        self.comfort = if near_campfire { 100.0 } else { 0.0 };
        if in_radiation_zone {
            self.radiation = (self.radiation + 2.2 * dt).clamp(0.0, MAX_RADIATION);
        } else {
            self.radiation = (self.radiation - 0.6 * dt).clamp(0.0, MAX_RADIATION);
        }
        let target_body_temp =
            (NORMAL_BODY_TEMP_C + (ambient_temp_c - 18.0) * 0.075).clamp(32.5, 40.5);
        let temp_rate = (0.35 * dt).clamp(0.0, 1.0);
        self.body_temp_c += (target_body_temp - self.body_temp_c) * temp_rate;

        let heat_factor = if self.body_temp_c > 37.8 { 1.35 } else { 1.0 };
        let cold_factor = if self.body_temp_c < 35.4 { 1.45 } else { 1.0 };

        let hydration_drain =
            (if sprinting { 0.32 } else { 0.14 }) * hydration_mult.max(0.5) * heat_factor;
        let calorie_drain = (if sprinting { 0.55 } else { 0.22 }) * cold_factor;
        self.hydration = (self.hydration - hydration_drain * dt).clamp(0.0, MAX_HYDRATION);
        self.calories = (self.calories - calorie_drain * dt).clamp(0.0, MAX_CALORIES);

        if self.hydration <= 0.0 {
            self.health = (self.health - 1.5 * dt).clamp(0.0, MAX_HEALTH);
        }
        if self.calories <= 0.0 {
            self.health = (self.health - 1.0 * dt).clamp(0.0, MAX_HEALTH);
        }
        if self.body_temp_c < 34.2 {
            self.health = (self.health - 0.8 * dt).clamp(0.0, MAX_HEALTH);
        }
        if self.radiation > 25.0 {
            self.health =
                (self.health - ((self.radiation - 25.0) * 0.03) * dt).clamp(0.0, MAX_HEALTH);
        }
        if self.health > 0.0
            && self.health < MAX_HEALTH
            && self.calories > 200.0
            && self.hydration > 100.0
            && self.body_temp_c >= 35.2
            && self.radiation < 15.0
        {
            let regen = if near_campfire { 1.2 } else { 0.6 };
            self.health = (self.health + regen * dt).clamp(0.0, MAX_HEALTH);
        }
    }

    pub fn apply_damage(&mut self, amount: f32) -> bool {
        if amount.is_finite() && amount > 0.0 {
            self.health = (self.health - amount).clamp(0.0, MAX_HEALTH);
        }
        self.health <= 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    Rock,
    StoneHatchet,
    StonePickaxe,
    WoodenSpear,
    HuntingBow,
    Revolver,
    AssaultRifle,
    BuildingPlan,
    Hammer,
    Wood,
    Stone,
    MetalFragments,
    Scrap,
    Arrow,
    PistolAmmo,
    RifleAmmo,
    ElectricFuse,
    GreenKeycard,
    Bandage,
    WaterBottle,
    Ration,
    AntiRadPills,
    LowGradeFuel,
    C4Explosive,
    TeleportGun,
}

impl ItemKind {
    pub const ALL: [Self; 25] = [
        Self::Rock,
        Self::StoneHatchet,
        Self::StonePickaxe,
        Self::WoodenSpear,
        Self::HuntingBow,
        Self::Revolver,
        Self::AssaultRifle,
        Self::BuildingPlan,
        Self::Hammer,
        Self::Wood,
        Self::Stone,
        Self::MetalFragments,
        Self::Scrap,
        Self::Arrow,
        Self::PistolAmmo,
        Self::RifleAmmo,
        Self::ElectricFuse,
        Self::GreenKeycard,
        Self::Bandage,
        Self::WaterBottle,
        Self::Ration,
        Self::AntiRadPills,
        Self::LowGradeFuel,
        Self::C4Explosive,
        Self::TeleportGun,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Rock => "Камінь",
            Self::StoneHatchet => "Кам'яна сокира",
            Self::StonePickaxe => "Кам'яне кайло",
            Self::WoodenSpear => "Дерев'яний спис",
            Self::HuntingBow => "Мисливський лук",
            Self::Revolver => "Саморобний револьвер",
            Self::AssaultRifle => "Штурмова гвинтівка АК-47",
            Self::BuildingPlan => "Будівельний план",
            Self::Hammer => "Киянка",
            Self::Wood => "Деревина",
            Self::Stone => "Каміння",
            Self::MetalFragments => "Металеві фрагменти",
            Self::Scrap => "Скрап (Металобрухт)",
            Self::Arrow => "Стріли",
            Self::PistolAmmo => "Набої 9мм",
            Self::RifleAmmo => "Набої 5.56мм",
            Self::ElectricFuse => "Електричний запобіжник",
            Self::GreenKeycard => "Зелена картка РТ",
            Self::Bandage => "Бинт",
            Self::WaterBottle => "Фляга води",
            Self::Ration => "Сухпайок",
            Self::AntiRadPills => "Антирад-таблетки",
            Self::LowGradeFuel => "Паливо низької якості",
            Self::C4Explosive => "Вибухівка C4",
            Self::TeleportGun => "Телепорт-гармата (Teleport Gun)",
        }
    }

    pub fn label_en(self) -> &'static str {
        match self {
            Self::Rock => "Rock",
            Self::StoneHatchet => "Stone Hatchet",
            Self::StonePickaxe => "Stone Pickaxe",
            Self::WoodenSpear => "Wooden Spear",
            Self::HuntingBow => "Hunting Bow",
            Self::Revolver => "Handmade Revolver",
            Self::AssaultRifle => "Assault Rifle (AK-47)",
            Self::BuildingPlan => "Building Plan",
            Self::Hammer => "Wooden Hammer",
            Self::Wood => "Wood Logs",
            Self::Stone => "Stone Rocks",
            Self::MetalFragments => "Metal Fragments",
            Self::Scrap => "Scrap Metal",
            Self::Arrow => "Wooden Arrows",
            Self::PistolAmmo => "9mm Pistol Ammo",
            Self::RifleAmmo => "5.56mm Rifle Ammo",
            Self::ElectricFuse => "Electric Fuse",
            Self::GreenKeycard => "Green Keycard",
            Self::Bandage => "Medical Bandage",
            Self::WaterBottle => "Water Canteen",
            Self::Ration => "Military Ration",
            Self::AntiRadPills => "Anti-Rad Pills",
            Self::LowGradeFuel => "Low Grade Fuel",
            Self::C4Explosive => "Timed Explosive (C4)",
            Self::TeleportGun => "Teleportation Gun",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Rock => "КАМІНЬ",
            Self::StoneHatchet => "СОКИРА",
            Self::StonePickaxe => "КАЙЛО",
            Self::WoodenSpear => "СПИС",
            Self::HuntingBow => "ЛУК",
            Self::Revolver => "РЕВОЛЬВ.",
            Self::AssaultRifle => "АК-47",
            Self::BuildingPlan => "ПЛАН",
            Self::Hammer => "КИЯНКА",
            Self::Wood => "ДЕРЕВО",
            Self::Stone => "РУДА",
            Self::MetalFragments => "МЕТАЛ",
            Self::Scrap => "СКРАП",
            Self::Arrow => "СТРІЛИ",
            Self::PistolAmmo => "9ММ",
            Self::RifleAmmo => "5.56ММ",
            Self::ElectricFuse => "РЕЗИСТОР",
            Self::GreenKeycard => "КАРТКА",
            Self::Bandage => "БИНТ",
            Self::WaterBottle => "ВОДА",
            Self::Ration => "ЇЖА",
            Self::AntiRadPills => "АНТИРАД",
            Self::LowGradeFuel => "ПАЛИВО",
            Self::C4Explosive => "C4",
            Self::TeleportGun => "ТЕЛЕПОРТ",
        }
    }

    pub fn max_stack(self) -> u16 {
        match self {
            Self::Rock
            | Self::StoneHatchet
            | Self::StonePickaxe
            | Self::WoodenSpear
            | Self::HuntingBow
            | Self::Revolver
            | Self::AssaultRifle
            | Self::BuildingPlan
            | Self::Hammer
            | Self::ElectricFuse
            | Self::GreenKeycard
            | Self::TeleportGun => 1,
            Self::Wood | Self::Stone | Self::MetalFragments | Self::Scrap => 1000,
            Self::LowGradeFuel => 500,
            Self::Arrow => 64,
            Self::PistolAmmo | Self::RifleAmmo => 128,
            Self::Bandage | Self::Ration | Self::AntiRadPills | Self::C4Explosive => 5,
            Self::WaterBottle => 3,
        }
    }

    pub fn reach_meters(self) -> f32 {
        match self {
            Self::TeleportGun => 800.0,
            Self::BuildingPlan | Self::Hammer | Self::C4Explosive => 4.2,
            Self::WoodenSpear => 3.3,
            _ => 2.8,
        }
    }

    pub fn harvest_yield(self, resource: ResourceKind) -> u16 {
        match (self, resource) {
            (Self::StoneHatchet, ResourceKind::Tree) => 25,
            (Self::StonePickaxe, ResourceKind::Rock) => 25,
            (Self::Rock | Self::StoneHatchet | Self::StonePickaxe, _) => 10,
            (Self::WoodenSpear, _) => 5,
            _ => 0,
        }
    }

    pub fn melee_damage(self) -> f32 {
        match self {
            Self::WoodenSpear => 28.0,
            Self::StoneHatchet | Self::StonePickaxe => 18.0,
            Self::Rock => 12.0,
            _ => 5.0,
        }
    }

    pub fn ranged_spec(self) -> Option<(ItemKind, f32, f32, f32)> {
        // Returns (ammo_kind, base_damage, muzzle_speed_m_s, max_range_m)
        match self {
            Self::HuntingBow => Some((Self::Arrow, 38.0, 55.0, 65.0)),
            Self::Revolver => Some((Self::PistolAmmo, 34.0, 140.0, 75.0)),
            Self::AssaultRifle => Some((Self::RifleAmmo, 46.0, 220.0, 95.0)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    pub kind: ItemKind,
    pub count: u16,
}

impl ItemStack {
    pub fn new(kind: ItemKind, count: u16) -> Option<Self> {
        (count > 0).then(|| Self {
            kind,
            count: count.min(kind.max_stack()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotIndex {
    Hotbar(u8),
    Backpack(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub hotbar: [Option<ItemStack>; HOTBAR_SLOTS],
    pub backpack: [Option<ItemStack>; BACKPACK_SLOTS],
    pub active_slot: u8,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::starter()
    }
}

impl Inventory {
    pub fn empty() -> Self {
        Self {
            hotbar: [None; HOTBAR_SLOTS],
            backpack: [None; BACKPACK_SLOTS],
            active_slot: 0,
        }
    }

    pub fn starter() -> Self {
        let mut hotbar = [None; HOTBAR_SLOTS];
        hotbar[0] = ItemStack::new(ItemKind::Rock, 1);
        hotbar[1] = ItemStack::new(ItemKind::Bandage, 2);
        hotbar[2] = ItemStack::new(ItemKind::WaterBottle, 1);
        hotbar[3] = ItemStack::new(ItemKind::Ration, 1);
        hotbar[4] = ItemStack::new(ItemKind::BuildingPlan, 1);
        hotbar[5] = ItemStack::new(ItemKind::Hammer, 1);
        Self {
            hotbar,
            backpack: [None; BACKPACK_SLOTS],
            active_slot: 0,
        }
    }

    pub fn select_slot(&mut self, slot: u8) {
        if (slot as usize) < HOTBAR_SLOTS {
            self.active_slot = slot;
        }
    }

    pub fn active_item(&self) -> Option<ItemStack> {
        self.hotbar
            .get(self.active_slot as usize)
            .copied()
            .flatten()
    }

    pub fn slot(&self, index: SlotIndex) -> Option<ItemStack> {
        match index {
            SlotIndex::Hotbar(i) => self.hotbar.get(i as usize).copied().flatten(),
            SlotIndex::Backpack(i) => self.backpack.get(i as usize).copied().flatten(),
        }
    }

    fn slot_mut(&mut self, index: SlotIndex) -> Option<&mut Option<ItemStack>> {
        match index {
            SlotIndex::Hotbar(i) => self.hotbar.get_mut(i as usize),
            SlotIndex::Backpack(i) => self.backpack.get_mut(i as usize),
        }
    }

    pub fn move_slot(&mut self, from: SlotIndex, to: SlotIndex) -> bool {
        if from == to {
            return false;
        }
        let Some(source) = self.slot(from) else {
            return false;
        };
        let target = self.slot(to);
        if let Some(mut dest) = target
            && dest.kind == source.kind
            && dest.count < dest.kind.max_stack()
        {
            let space = dest.kind.max_stack() - dest.count;
            let transfer = source.count.min(space);
            dest.count += transfer;
            let remaining = source.count - transfer;
            if let Some(slot) = self.slot_mut(to) {
                *slot = Some(dest);
            }
            if let Some(slot) = self.slot_mut(from) {
                *slot = ItemStack::new(source.kind, remaining);
            }
            return true;
        }
        if let Some(slot) = self.slot_mut(to) {
            *slot = Some(source);
        } else {
            return false;
        }
        if let Some(slot) = self.slot_mut(from) {
            *slot = target;
        }
        true
    }

    pub fn add_item(&mut self, kind: ItemKind, mut amount: u16) -> u16 {
        let max = kind.max_stack();
        for slot in self
            .hotbar
            .iter_mut()
            .chain(self.backpack.iter_mut())
            .flatten()
        {
            if slot.kind == kind && slot.count < max {
                let added = amount.min(max - slot.count);
                slot.count += added;
                amount -= added;
                if amount == 0 {
                    return 0;
                }
            }
        }
        // Raw materials and ammunition go into the backpack first so hotbar slots
        // remain free for crafted tools, weapons, and consumables.
        if matches!(
            kind,
            ItemKind::Wood
                | ItemKind::Stone
                | ItemKind::MetalFragments
                | ItemKind::Scrap
                | ItemKind::LowGradeFuel
                | ItemKind::Arrow
                | ItemKind::PistolAmmo
                | ItemKind::RifleAmmo
        ) {
            for slot in self.backpack.iter_mut().chain(self.hotbar.iter_mut()) {
                if slot.is_none() && amount > 0 {
                    let added = amount.min(max);
                    *slot = ItemStack::new(kind, added);
                    amount -= added;
                    if amount == 0 {
                        return 0;
                    }
                }
            }
        } else {
            for slot in self.hotbar.iter_mut().chain(self.backpack.iter_mut()) {
                if slot.is_none() && amount > 0 {
                    let added = amount.min(max);
                    *slot = ItemStack::new(kind, added);
                    amount -= added;
                    if amount == 0 {
                        return 0;
                    }
                }
            }
        }
        amount
    }

    pub fn count_item(&self, kind: ItemKind) -> u32 {
        self.hotbar
            .iter()
            .chain(self.backpack.iter())
            .flatten()
            .filter(|stack| stack.kind == kind)
            .map(|stack| stack.count as u32)
            .sum()
    }

    pub fn remove_item(&mut self, kind: ItemKind, mut amount: u16) -> bool {
        if self.count_item(kind) < amount as u32 {
            return false;
        }
        for slot in self.backpack.iter_mut().chain(self.hotbar.iter_mut()) {
            let Some(stack) = slot else {
                continue;
            };
            if stack.kind != kind {
                continue;
            }
            let take = amount.min(stack.count);
            stack.count -= take;
            amount -= take;
            if stack.count == 0 {
                *slot = None;
            }
            if amount == 0 {
                break;
            }
        }
        true
    }

    pub fn consume_active(&mut self) -> Option<ItemKind> {
        let slot = self.hotbar.get_mut(self.active_slot as usize)?;
        let stack = slot.as_mut()?;
        let kind = stack.kind;
        stack.count -= 1;
        if stack.count == 0 {
            *slot = None;
        }
        Some(kind)
    }

    pub fn can_craft(&self, recipe: RecipeId) -> bool {
        recipe
            .spec()
            .ingredients
            .iter()
            .all(|&(kind, needed)| self.count_item(kind) >= needed as u32)
    }

    pub fn craft(&mut self, recipe: RecipeId) -> Result<ItemKind, &'static str> {
        let spec = recipe.spec();
        if !self.can_craft(recipe) {
            return Err("Недостатньо ресурсів для крафту");
        }
        // Verify there is room for the output before removing ingredients.
        let mut probe = self.clone();
        for &(kind, needed) in spec.ingredients {
            probe.remove_item(kind, needed);
        }
        if probe.add_item(spec.output, spec.output_count) > 0 {
            return Err("Інвентар переповнений");
        }
        *self = probe;
        Ok(spec.output)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RecipeId {
    StoneHatchet,
    StonePickaxe,
    WoodenSpear,
    HuntingBow,
    Arrow,
    Revolver,
    PistolAmmo,
    AssaultRifle,
    RifleAmmo,
    ElectricFuse,
    LowGradeFuel,
    C4Explosive,
    BuildingPlan,
    Hammer,
    Bandage,
}

pub struct RecipeSpec {
    pub id: RecipeId,
    pub output: ItemKind,
    pub output_count: u16,
    pub ingredients: &'static [(ItemKind, u16)],
    pub description: &'static str,
}

impl RecipeId {
    pub const ALL: [Self; 15] = [
        Self::StoneHatchet,
        Self::StonePickaxe,
        Self::WoodenSpear,
        Self::HuntingBow,
        Self::Arrow,
        Self::Revolver,
        Self::PistolAmmo,
        Self::AssaultRifle,
        Self::RifleAmmo,
        Self::ElectricFuse,
        Self::LowGradeFuel,
        Self::C4Explosive,
        Self::BuildingPlan,
        Self::Hammer,
        Self::Bandage,
    ];

    pub fn spec(self) -> RecipeSpec {
        match self {
            Self::StoneHatchet => RecipeSpec {
                id: Self::StoneHatchet,
                output: ItemKind::StoneHatchet,
                output_count: 1,
                ingredients: &[(ItemKind::Wood, 60), (ItemKind::Stone, 30)],
                description: "Швидке рубання дерев (+25 деревини за удар)",
            },
            Self::StonePickaxe => RecipeSpec {
                id: Self::StonePickaxe,
                output: ItemKind::StonePickaxe,
                output_count: 1,
                ingredients: &[(ItemKind::Wood, 60), (ItemKind::Stone, 30)],
                description: "Ефективний видобуток каміння (+25 каміння за удар)",
            },
            Self::WoodenSpear => RecipeSpec {
                id: Self::WoodenSpear,
                output: ItemKind::WoodenSpear,
                output_count: 1,
                ingredients: &[(ItemKind::Wood, 100)],
                description: "Збільшена дистанція удару (3.3 м) і висока шкода",
            },
            Self::HuntingBow => RecipeSpec {
                id: Self::HuntingBow,
                output: ItemKind::HuntingBow,
                output_count: 1,
                ingredients: &[(ItemKind::Wood, 120), (ItemKind::Stone, 20)],
                description: "Мисливський лук (38 шкоди, балістика стріл)",
            },
            Self::Arrow => RecipeSpec {
                id: Self::Arrow,
                output: ItemKind::Arrow,
                output_count: 4,
                ingredients: &[(ItemKind::Wood, 25), (ItemKind::Stone, 10)],
                description: "Комплект 4 дерев'яних стріл для лука",
            },
            Self::Revolver => RecipeSpec {
                id: Self::Revolver,
                output: ItemKind::Revolver,
                output_count: 1,
                ingredients: &[
                    (ItemKind::MetalFragments, 80),
                    (ItemKind::Wood, 60),
                    (ItemKind::Scrap, 15),
                ],
                description: "Револьвер (34 шкоди, хітскан до 25м + балістика)",
            },
            Self::PistolAmmo => RecipeSpec {
                id: Self::PistolAmmo,
                output: ItemKind::PistolAmmo,
                output_count: 8,
                ingredients: &[(ItemKind::MetalFragments, 20), (ItemKind::Stone, 15)],
                description: "Комплект 8 пістолетних набоїв (9мм)",
            },
            Self::AssaultRifle => RecipeSpec {
                id: Self::AssaultRifle,
                output: ItemKind::AssaultRifle,
                output_count: 1,
                ingredients: &[
                    (ItemKind::MetalFragments, 200),
                    (ItemKind::Wood, 100),
                    (ItemKind::Scrap, 40),
                ],
                description: "Штурмова гвинтівка АК (46 шкоди, 5.56мм, висока дальність)",
            },
            Self::RifleAmmo => RecipeSpec {
                id: Self::RifleAmmo,
                output: ItemKind::RifleAmmo,
                output_count: 10,
                ingredients: &[(ItemKind::MetalFragments, 30), (ItemKind::Stone, 20)],
                description: "Комплект 10 гвинтівкових набоїв (5.56мм)",
            },
            Self::ElectricFuse => RecipeSpec {
                id: Self::ElectricFuse,
                output: ItemKind::ElectricFuse,
                output_count: 1,
                ingredients: &[(ItemKind::MetalFragments, 20), (ItemKind::Scrap, 5)],
                description: "Електричний запобіжник для щитка генератора в Радтауні (РТ)",
            },
            Self::LowGradeFuel => RecipeSpec {
                id: Self::LowGradeFuel,
                output: ItemKind::LowGradeFuel,
                output_count: 25,
                ingredients: &[(ItemKind::Wood, 30), (ItemKind::Stone, 15)],
                description: "Паливо (25 од.) для Багі, Хамера, Мінікоптера та крафту C4",
            },
            Self::C4Explosive => RecipeSpec {
                id: Self::C4Explosive,
                output: ItemKind::C4Explosive,
                output_count: 1,
                ingredients: &[
                    (ItemKind::MetalFragments, 120),
                    (ItemKind::Scrap, 40),
                    (ItemKind::LowGradeFuel, 30),
                ],
                description: "Вибухівка C4 (320 шкоди по спорудах та БТР «Бредлі»)",
            },
            Self::BuildingPlan => RecipeSpec {
                id: Self::BuildingPlan,
                output: ItemKind::BuildingPlan,
                output_count: 1,
                ingredients: &[(ItemKind::Wood, 20)],
                description: "Зведення фундаментів, стін (обертання Q/ПКМ), прорізів і Шафи",
            },
            Self::Hammer => RecipeSpec {
                id: Self::Hammer,
                output: ItemKind::Hammer,
                output_count: 1,
                ingredients: &[(ItemKind::Wood, 40)],
                description: "Ремонт і покращення будівель (Солома -> Дерево -> Камінь -> Метал)",
            },
            Self::Bandage => RecipeSpec {
                id: Self::Bandage,
                output: ItemKind::Bandage,
                output_count: 1,
                ingredients: &[(ItemKind::Wood, 20)],
                description: "Перев'язка поранень (+15 здоров'я)",
            },
        }
    }
}

pub const TC_PRIVILEGE_RADIUS: f32 = 30.0;
pub const BUILD_GRID_SIZE: f32 = 3.0;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureKind {
    #[default]
    Foundation,
    Wall,
    Doorway,
    FloorCeiling,
    ToolCupboard,
    Campfire,
}

impl StructureKind {
    pub const ALL: [Self; 6] = [
        Self::Foundation,
        Self::Wall,
        Self::Doorway,
        Self::FloorCeiling,
        Self::ToolCupboard,
        Self::Campfire,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::Foundation => Self::Wall,
            Self::Wall => Self::Doorway,
            Self::Doorway => Self::FloorCeiling,
            Self::FloorCeiling => Self::ToolCupboard,
            Self::ToolCupboard => Self::Campfire,
            Self::Campfire => Self::Foundation,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Foundation => "Фундамент",
            Self::Wall => "Стіна",
            Self::Doorway => "Дверний проріз",
            Self::FloorCeiling => "Перекриття / Дах",
            Self::ToolCupboard => "Шафа з інструментами (TC)",
            Self::Campfire => "Багаття (обігрів та комфорт)",
        }
    }

    pub fn build_cost_wood(self) -> u16 {
        match self {
            Self::ToolCupboard => 50,
            Self::Campfire => 30,
            _ => 20,
        }
    }

    pub fn half_extents(self) -> [f32; 3] {
        match self {
            Self::Foundation => [1.5, 0.10, 1.5],
            Self::Wall | Self::Doorway => [1.5, 1.25, 0.15],
            Self::FloorCeiling => [1.5, 0.12, 1.5],
            Self::ToolCupboard => [0.45, 0.85, 0.35],
            Self::Campfire => [0.45, 0.22, 0.45],
        }
    }

    pub fn oriented_half_extents(self, rotation: u8) -> [f32; 3] {
        let ext = self.half_extents();
        if rotation % 2 == 1 {
            [ext[2], ext[1], ext[0]]
        } else {
            ext
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuildingTier {
    #[default]
    Twig,
    Wood,
    Stone,
    Metal,
}

impl BuildingTier {
    pub fn label(self) -> &'static str {
        match self {
            Self::Twig => "Солома",
            Self::Wood => "Дерево",
            Self::Stone => "Камінь",
            Self::Metal => "Метал",
        }
    }

    pub fn max_health(self) -> u16 {
        match self {
            Self::Twig => 50,
            Self::Wood => 250,
            Self::Stone => 500,
            Self::Metal => 1000,
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Twig => [0.68, 0.56, 0.35],
            Self::Wood => [0.50, 0.34, 0.20],
            Self::Stone => [0.52, 0.56, 0.60],
            Self::Metal => [0.32, 0.42, 0.49],
        }
    }

    pub fn next_upgrade(self) -> Option<(Self, ItemKind, u16)> {
        match self {
            Self::Twig => Some((Self::Wood, ItemKind::Wood, 60)),
            Self::Wood => Some((Self::Stone, ItemKind::Stone, 100)),
            Self::Stone => Some((Self::Metal, ItemKind::Stone, 200)),
            Self::Metal => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlacedStructure {
    pub id: u32,
    pub owner: PlayerId,
    pub kind: StructureKind,
    pub tier: BuildingTier,
    pub position: [f32; 3],
    #[serde(default)]
    pub rotation: u8,
    pub health: u16,
    pub max_health: u16,
}

pub fn socket_edge_label(rotation: u8) -> &'static str {
    match rotation % 4 {
        0 => "Північ (-Z, 0°)",
        1 => "Схід (+X, 90°)",
        2 => "Південь (+Z, 180°)",
        _ => "Захід (-X, 270°)",
    }
}

pub fn preview_socket_placement(
    player_position: [f32; 3],
    yaw: f32,
    pitch: f32,
    kind: StructureKind,
    build_rotation: u8,
    structures: &[PlacedStructure],
) -> ([f32; 3], u8) {
    let dir = look_direction(yaw, pitch);
    let forward = [
        player_position[0] + dir[0] * 3.0,
        player_position[1],
        player_position[2] + dir[2] * 3.0,
    ];
    let snap = |v: f32| ((v / BUILD_GRID_SIZE).round() * BUILD_GRID_SIZE).clamp(-5000.0, 5000.0);
    let mut gx = snap(forward[0]);
    let mut gz = snap(forward[2]);

    // Snap walls, doorways, ceilings, and deployables to the closest existing Foundation socket cell
    // within 2.5m so rotating around a foundation snaps cleanly to all 4 edges.
    if kind != StructureKind::Foundation {
        let mut best_dist_sq = 2.5_f32 * 2.5_f32;
        for s in structures {
            if s.kind == StructureKind::Foundation {
                let dx = s.position[0] - forward[0];
                let dz = s.position[2] - forward[2];
                let d2 = dx * dx + dz * dz;
                if d2 < best_dist_sq {
                    best_dist_sq = d2;
                    gx = s.position[0];
                    gz = s.position[2];
                }
            }
        }
    }

    let auto_edge = if dir[0].abs() > dir[2].abs() {
        if dir[0] >= 0.0 { 1 } else { 3 }
    } else if dir[2] >= 0.0 {
        2
    } else {
        0
    };
    let rot = (auto_edge + build_rotation) % 4;
    let pos = match kind {
        StructureKind::Foundation => [gx, 0.10, gz],
        StructureKind::Wall | StructureKind::Doorway => match rot {
            0 => [gx, 1.45, gz - 1.35],
            1 => [gx + 1.35, 1.45, gz],
            2 => [gx, 1.45, gz + 1.35],
            _ => [gx - 1.35, 1.45, gz],
        },
        StructureKind::FloorCeiling => [gx, 2.75, gz],
        StructureKind::ToolCupboard => [gx, 1.05, gz],
        StructureKind::Campfire => [gx, 0.22, gz],
    };
    (pos, rot)
}

pub fn preview_placement(
    player_position: [f32; 3],
    yaw: f32,
    pitch: f32,
    kind: StructureKind,
) -> [f32; 3] {
    preview_socket_placement(player_position, yaw, pitch, kind, 0, &[]).0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Biome {
    Temperate,
    Desert,
    Jungle,
    Winter,
}

impl Biome {
    pub const ALL: [Self; 4] = [Self::Temperate, Self::Desert, Self::Jungle, Self::Winter];

    pub fn at(position: [f32; 3]) -> Self {
        if -position[2] > 600.0 {
            Self::Winter
        } else if -position[2] < -700.0 {
            Self::Desert
        } else if position[0] > 1800.0 || position[0] < -2800.0 {
            Self::Jungle
        } else {
            Self::Temperate
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Temperate => "Помірний ліс",
            Self::Desert => "Пустеля",
            Self::Jungle => "Джунглі",
            Self::Winter => "Арктична зима",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Temperate => "ЛІС",
            Self::Desert => "ПУСТЕЛЯ",
            Self::Jungle => "ДЖУНГЛІ",
            Self::Winter => "ЗИМА",
        }
    }

    pub fn base_temperature_c(self) -> f32 {
        match self {
            Self::Temperate => 18.0,
            Self::Desert => 38.0,
            Self::Jungle => 30.0,
            Self::Winter => -18.0,
        }
    }

    pub fn hydration_multiplier(self) -> f32 {
        match self {
            Self::Desert => 1.8,
            Self::Jungle => 1.2,
            Self::Temperate => 1.0,
            Self::Winter => 0.9,
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Temperate => [0.24, 0.36, 0.22],
            Self::Desert => [0.64, 0.51, 0.29],
            Self::Jungle => [0.09, 0.44, 0.18],
            Self::Winter => [0.74, 0.82, 0.88],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherKind {
    Clear,
    Cloudy,
    RainStorm,
    Fog,
    Sandstorm,
    Blizzard,
}

impl WeatherKind {
    pub fn at_tick(tick: ServerTick, biome: Biome) -> Self {
        let phase = (tick.0 / (60 * 45)) % 4;
        match (biome, phase) {
            (_, 0) => Self::Clear,
            (_, 1) => Self::Cloudy,
            (Biome::Desert, 2) => Self::Sandstorm,
            (Biome::Winter, 2) => Self::Blizzard,
            (Biome::Jungle, 2) => Self::RainStorm,
            (Biome::Temperate, 2) => Self::Fog,
            (Biome::Winter, 3) => Self::Fog,
            (_, _) => Self::RainStorm,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Clear => "Ясно",
            Self::Cloudy => "Мінлива хмарність",
            Self::RainStorm => "Злива з грозою",
            Self::Fog => "Густий туман",
            Self::Sandstorm => "Піщана буря",
            Self::Blizzard => "Хуртовина",
        }
    }

    pub fn temperature_offset_c(self) -> f32 {
        match self {
            Self::Clear => 2.0,
            Self::Cloudy => 0.0,
            Self::RainStorm => -4.0,
            Self::Fog => -2.0,
            Self::Sandstorm => 5.0,
            Self::Blizzard => -10.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldEnvironment {
    pub time_hours: f32,
    pub is_night: bool,
    pub biome: Biome,
    pub weather: WeatherKind,
    pub ambient_temp_c: f32,
    pub near_campfire: bool,
    pub in_radtown: bool,
    pub wind_offset: [f32; 2],
}

pub fn time_of_day_hours(tick: ServerTick) -> f32 {
    let progress = (tick.0 % DAY_CYCLE_TICKS) as f32 / DAY_CYCLE_TICKS as f32;
    (10.0 + progress * 24.0).rem_euclid(24.0)
}

pub fn format_time_of_day(hours: f32) -> String {
    let total_minutes = ((hours.rem_euclid(24.0)) * 60.0).round() as u32 % (24 * 60);
    let h = total_minutes / 60;
    let m = total_minutes % 60;
    format!("{h:02}:{m:02}")
}

pub const RADTOWN_SUPPLY_CRATE_POS: [f32; 3] = [-16.5, 0.45, -7.5];
pub const RADTOWN_RECYCLER_POS: [f32; 3] = [-19.0, 0.65, -7.5];
pub const RADTOWN_FUSEBOX_POS: [f32; 3] = [-20.2, 1.15, -10.0];
pub const RADTOWN_READER_POS: [f32; 3] = [-21.4, 1.15, -10.0];
pub const RADTOWN_MILITARY_CRATE_POS: [f32; 3] = [-23.2, 0.50, -11.5];
pub const RADTOWN_NUCLEAR_BUTTON_POS: [f32; 3] = [-25.2, 1.15, -13.2];
pub const FALLOUT_CRATE_POS: [f32; 3] = [8.0, 0.55, 14.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonumentHotspot {
    SupplyCrate,
    Recycler,
    FuseBox,
    CardReader,
    MilitaryCrate,
    NuclearSiloButton,
    FalloutCrate,
}

impl MonumentHotspot {
    pub const ALL: [Self; 7] = [
        Self::SupplyCrate,
        Self::Recycler,
        Self::FuseBox,
        Self::CardReader,
        Self::MilitaryCrate,
        Self::NuclearSiloButton,
        Self::FalloutCrate,
    ];

    pub fn position(self) -> [f32; 3] {
        match self {
            Self::SupplyCrate => RADTOWN_SUPPLY_CRATE_POS,
            Self::Recycler => RADTOWN_RECYCLER_POS,
            Self::FuseBox => RADTOWN_FUSEBOX_POS,
            Self::CardReader => RADTOWN_READER_POS,
            Self::MilitaryCrate => RADTOWN_MILITARY_CRATE_POS,
            Self::NuclearSiloButton => RADTOWN_NUCLEAR_BUTTON_POS,
            Self::FalloutCrate => FALLOUT_CRATE_POS,
        }
    }

    pub fn half_extents(self) -> [f32; 3] {
        match self {
            Self::SupplyCrate => [0.65, 0.45, 0.65],
            Self::Recycler => [0.75, 0.70, 0.65],
            Self::FuseBox => [0.45, 0.55, 0.35],
            Self::CardReader => [0.45, 0.55, 0.35],
            Self::MilitaryCrate => [0.75, 0.50, 0.65],
            Self::NuclearSiloButton => [0.45, 0.55, 0.45],
            Self::FalloutCrate => [0.85, 0.55, 0.85],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SupplyCrate => "Ящик постачання РТ (Запобіжник + Зелена картка + Паливо)",
            Self::Recycler => "Переробник РТ (Розбір ресурсів -> Метал + Скрап)",
            Self::FuseBox => "Електрощиток та рубильник генератора РТ",
            Self::CardReader => "Зчитувач зеленої картки доступу бункера РТ",
            Self::MilitaryCrate => "Військовий ящик бункера РТ (Зброя + C4 + Набої)",
            Self::NuclearSiloButton => "Пульт запуску ЯДЕРНОЇ БОЄГОЛОВКИ бункера РТ",
            Self::FalloutCrate => "Елітний радіоактивний контейнер у кратері ядерного вибуху",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NuclearPhase {
    #[default]
    Inactive,
    SirenWarning,
    DetonationShockwave,
    Fallout,
}

impl NuclearPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Inactive => "Спокійно",
            Self::SirenWarning => "СИРЕНА ЯДЕРНОЇ ТРИВОГИ",
            Self::DetonationShockwave => "ЯДЕРНИЙ ВИБУХ ТА УДАРНА ХВИЛЯ",
            Self::Fallout => "РАДІОАКТИВНИЙ ПОПІЛ (FALLOUT)",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NuclearStrikeState {
    pub phase: NuclearPhase,
    pub ticks_remaining: u32,
    pub fallout_crate_looted: bool,
}

impl NuclearStrikeState {
    pub fn is_active(&self) -> bool {
        self.phase != NuclearPhase::Inactive
    }

    pub fn launch(&mut self) -> bool {
        if self.phase != NuclearPhase::Inactive {
            return false;
        }
        self.phase = NuclearPhase::SirenWarning;
        self.ticks_remaining = NUCLEAR_SIREN_TICKS;
        self.fallout_crate_looted = false;
        true
    }

    pub fn step(&mut self) -> Option<NuclearPhase> {
        if self.phase == NuclearPhase::Inactive {
            return None;
        }
        if self.ticks_remaining > 0 {
            self.ticks_remaining -= 1;
        }
        if self.ticks_remaining == 0 {
            match self.phase {
                NuclearPhase::Inactive => None,
                NuclearPhase::SirenWarning => {
                    self.phase = NuclearPhase::DetonationShockwave;
                    self.ticks_remaining = NUCLEAR_SHOCKWAVE_TICKS;
                    Some(NuclearPhase::DetonationShockwave)
                }
                NuclearPhase::DetonationShockwave => {
                    self.phase = NuclearPhase::Fallout;
                    self.ticks_remaining = NUCLEAR_FALLOUT_TICKS;
                    Some(NuclearPhase::Fallout)
                }
                NuclearPhase::Fallout => {
                    self.phase = NuclearPhase::Inactive;
                    self.ticks_remaining = 0;
                    self.fallout_crate_looted = false;
                    Some(NuclearPhase::Inactive)
                }
            }
        } else {
            None
        }
    }
}

pub fn is_in_nuclear_blast(position: [f32; 3]) -> bool {
    let dx = position[0] - NUCLEAR_STRIKE_CENTER[0];
    let dz = position[2] - NUCLEAR_STRIKE_CENTER[2];
    dx * dx + dz * dz <= NUCLEAR_STRIKE_RADIUS * NUCLEAR_STRIKE_RADIUS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcKind {
    ZombieWalker,
    RadZombie,
    ScientistNpc,
    BradleyApc,
    PatrolHeli,
}

impl NpcKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::ZombieWalker => "Зомбі-блукач",
            Self::RadZombie => "Радіоактивний зомбі",
            Self::ScientistNpc => "Вчений-патрульний РТ",
            Self::BradleyApc => "БТР «Бредлі» (Танк РТ)",
            Self::PatrolHeli => "Патрульний вертоліт",
        }
    }

    pub fn max_health(self) -> u16 {
        match self {
            Self::ZombieWalker => 90,
            Self::RadZombie => 130,
            Self::ScientistNpc => 150,
            Self::BradleyApc => 600,
            Self::PatrolHeli => 500,
        }
    }

    pub fn half_extents(self) -> [f32; 3] {
        match self {
            Self::ZombieWalker | Self::RadZombie | Self::ScientistNpc => [0.48, 0.92, 0.48],
            Self::BradleyApc => [1.75, 1.15, 2.35],
            Self::PatrolHeli => [1.85, 1.10, 2.50],
        }
    }

    pub fn is_humanoid(self) -> bool {
        matches!(
            self,
            Self::ZombieWalker | Self::RadZombie | Self::ScientistNpc
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NpcEntity {
    pub id: u16,
    pub kind: NpcKind,
    pub position: [f32; 3],
    pub yaw: f32,
    pub health: u16,
    pub max_health: u16,
    pub waypoint_index: u8,
    pub respawn_at: Option<ServerTick>,
}

impl NpcEntity {
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }
}

pub fn player_triggers_patrol_heli(inventory: &Inventory) -> bool {
    inventory
        .hotbar
        .iter()
        .flatten()
        .any(|stack| matches!(stack.kind, ItemKind::Revolver | ItemKind::AssaultRifle))
}

pub fn cargo_ship_transform(tick: ServerTick) -> ([f32; 3], f32) {
    let angle = (tick.0 as f32 / (60.0 * 120.0)) * std::f32::consts::TAU;
    let radius = 41.5;
    let x = angle.cos() * radius;
    let z = angle.sin() * radius;
    let yaw = -angle;
    ([x, -0.35, z], yaw)
}

pub fn map_npcs(map: MapId) -> Vec<NpcEntity> {
    if map != MapId::Polygon {
        return Vec::new();
    }
    let initial = [
        (NpcKind::ZombieWalker, [-8.5, 0.92, 1.5], 0.0),
        (NpcKind::ZombieWalker, [14.0, 0.92, -2.0], 1.57),
        (NpcKind::RadZombie, [-15.5, 0.92, -5.5], 0.8),
        (NpcKind::ScientistNpc, [-18.0, 0.92, -8.5], 0.4),
        (NpcKind::ScientistNpc, [-21.2, 0.92, -9.0], -0.4),
        (NpcKind::BradleyApc, [-11.5, 1.15, -9.5], 0.0),
        (NpcKind::PatrolHeli, [0.0, 14.5, -14.0], 0.0),
    ];
    initial
        .into_iter()
        .enumerate()
        .map(|(i, (kind, position, yaw))| NpcEntity {
            id: i as u16 + 1,
            kind,
            position,
            yaw,
            health: kind.max_health(),
            max_health: kind.max_health(),
            waypoint_index: 0,
            respawn_at: None,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleKind {
    Bicycle,
    DuneBuggy,
    ArmoredHumvee,
    Minicopter,
}

impl VehicleKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bicycle => "Велосипед",
            Self::DuneBuggy => "Багі (Dune Buggy)",
            Self::ArmoredHumvee => "Броньований Хамер",
            Self::Minicopter => "Мінікоптер",
        }
    }

    pub fn max_health(self) -> u16 {
        match self {
            Self::Bicycle => 150,
            Self::DuneBuggy => 280,
            Self::ArmoredHumvee => 500,
            Self::Minicopter => 320,
        }
    }

    pub fn uses_fuel(self) -> bool {
        !matches!(self, Self::Bicycle)
    }

    pub fn half_extents(self) -> [f32; 3] {
        match self {
            Self::Bicycle => [0.45, 0.65, 0.95],
            Self::DuneBuggy => [1.10, 0.75, 1.75],
            Self::ArmoredHumvee => [1.30, 1.05, 2.15],
            Self::Minicopter => [1.15, 0.95, 1.65],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VehicleEntity {
    pub id: u16,
    pub kind: VehicleKind,
    pub position: [f32; 3],
    pub yaw: f32,
    pub health: u16,
    pub max_health: u16,
    pub fuel: u16,
    pub driver: Option<PlayerId>,
}

pub fn map_vehicles(map: MapId) -> Vec<VehicleEntity> {
    if map != MapId::Polygon {
        return Vec::new();
    }
    let placements = [
        (VehicleKind::Bicycle, [3.8, 0.0, 11.5], 0.0, 0),
        (VehicleKind::DuneBuggy, [7.2, 0.0, 13.5], 0.35, 45),
        (VehicleKind::ArmoredHumvee, [-6.8, 0.0, 11.5], -0.25, 65),
        (VehicleKind::Minicopter, [-11.5, 0.0, 6.5], 0.0, 55),
    ];
    placements
        .into_iter()
        .enumerate()
        .map(|(i, (kind, position, yaw, fuel))| VehicleEntity {
            id: i as u16 + 1,
            kind,
            position,
            yaw,
            health: kind.max_health(),
            max_health: kind.max_health(),
            fuel,
            driver: None,
        })
        .collect()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonumentPuzzleState {
    pub fuse_ticks_remaining: u32,
    pub switch_on: bool,
    pub door_open_ticks: u32,
    pub supply_crate_cooldown: u32,
    pub military_crate_cooldown: u32,
}

impl MonumentPuzzleState {
    pub fn is_powered(&self) -> bool {
        self.fuse_ticks_remaining > 0 && self.switch_on
    }

    pub fn is_door_open(&self) -> bool {
        self.door_open_ticks > 0
    }

    pub fn step(&mut self) -> bool {
        let prev_powered = self.is_powered();
        let prev_door = self.is_door_open();
        let prev_supply_ready = self.supply_crate_cooldown == 0;
        let prev_mil_ready = self.military_crate_cooldown == 0;
        if self.fuse_ticks_remaining > 0 {
            self.fuse_ticks_remaining -= 1;
            if self.fuse_ticks_remaining == 0 {
                self.switch_on = false;
            }
        }
        if self.door_open_ticks > 0 {
            self.door_open_ticks -= 1;
        }
        if self.supply_crate_cooldown > 0 {
            self.supply_crate_cooldown -= 1;
        }
        if self.military_crate_cooldown > 0 {
            self.military_crate_cooldown -= 1;
        }
        prev_powered != self.is_powered()
            || prev_door != self.is_door_open()
            || prev_supply_ready != (self.supply_crate_cooldown == 0)
            || prev_mil_ready != (self.military_crate_cooldown == 0)
    }
}

pub fn is_in_radtown(position: [f32; 3]) -> bool {
    let dx = position[0] - RADTOWN_CENTER[0];
    let dz = position[2] - RADTOWN_CENTER[2];
    dx * dx + dz * dz <= RADTOWN_RADIUS * RADTOWN_RADIUS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Tree,
    Rock,
}

impl ResourceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tree => "Дерево",
            Self::Rock => "Кам'яна жила",
        }
    }

    pub fn item(self) -> ItemKind {
        match self {
            Self::Tree => ItemKind::Wood,
            Self::Rock => ItemKind::Stone,
        }
    }

    pub fn max_health(self) -> u16 {
        match self {
            Self::Tree => 200,
            Self::Rock => 250,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Self::Tree => 0.55,
            Self::Rock => 0.95,
        }
    }

    pub fn height(self) -> f32 {
        match self {
            Self::Tree => 4.2,
            Self::Rock => 1.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: u16,
    pub kind: ResourceKind,
    pub position: [f32; 3],
    pub health: u16,
    pub max_health: u16,
    pub respawn_at: Option<ServerTick>,
}

impl ResourceNode {
    pub fn is_available(&self) -> bool {
        self.health > 0
    }
}

pub fn map_resource_nodes(map: MapId) -> Vec<ResourceNode> {
    if map != MapId::Polygon {
        return Vec::new();
    }
    let placements = [
        (ResourceKind::Tree, [-17.5, 0.0, 8.0]),
        (ResourceKind::Tree, [-21.0, 0.0, -2.0]),
        (ResourceKind::Tree, [-18.0, 0.0, -18.0]),
        (ResourceKind::Tree, [17.5, 0.0, 8.0]),
        (ResourceKind::Tree, [21.0, 0.0, -4.0]),
        (ResourceKind::Tree, [8.0, 0.0, -22.0]),
        (ResourceKind::Rock, [-17.5, 0.0, 2.0]),
        (ResourceKind::Rock, [17.5, 0.0, 2.0]),
        (ResourceKind::Rock, [21.0, 0.0, 12.0]),
        (ResourceKind::Rock, [-21.0, 0.0, 14.0]),
        (ResourceKind::Rock, [4.0, 0.0, -22.0]),
        (ResourceKind::Rock, [-4.0, 0.0, -22.0]),
        // Jungle biome (x > 10.0, z in -10.0..10.0) dense tropical trees & rocks
        (ResourceKind::Tree, [14.5, 0.0, -6.5]),
        (ResourceKind::Tree, [19.5, 0.0, -1.5]),
        (ResourceKind::Tree, [24.0, 0.0, 4.5]),
        (ResourceKind::Tree, [15.5, 0.0, 6.5]),
        (ResourceKind::Tree, [25.5, 0.0, -6.0]),
        (ResourceKind::Rock, [22.5, 0.0, 0.5]),
    ];
    placements
        .into_iter()
        .enumerate()
        .map(|(index, (kind, position))| ResourceNode {
            id: index as u16 + 1,
            kind,
            position,
            health: kind.max_health(),
            max_health: kind.max_health(),
            respawn_at: None,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerSurvival {
    pub vitals: Vitals,
    pub inventory: Inventory,
    pub selected_structure: StructureKind,
    #[serde(default)]
    pub build_rotation: u8,
    #[serde(default)]
    pub mounted_vehicle: Option<u16>,
    pub last_action_tick: ServerTick,
    pub notice: Option<String>,
}

impl Default for PlayerSurvival {
    fn default() -> Self {
        Self {
            vitals: Vitals::default(),
            inventory: Inventory::starter(),
            selected_structure: StructureKind::Foundation,
            build_rotation: 0,
            mounted_vehicle: None,
            last_action_tick: ServerTick(0),
            notice: None,
        }
    }
}

impl PlayerSurvival {
    pub fn cycle_structure(&mut self) -> StructureKind {
        self.selected_structure = self.selected_structure.next();
        self.notice = Some(format!(
            "Форма будівництва: {} ({} Деревина)",
            self.selected_structure.label(),
            self.selected_structure.build_cost_wood()
        ));
        self.selected_structure
    }

    pub fn rotate_structure(&mut self) -> u8 {
        self.build_rotation = (self.build_rotation + 1) % 4;
        self.notice = Some(format!(
            "Поворот сокета будівництва: +{}° ({})",
            u16::from(self.build_rotation) * 90,
            socket_edge_label(self.build_rotation)
        ));
        self.build_rotation
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionOutcome {
    Consumed {
        item: ItemKind,
    },
    Harvested {
        node_id: u16,
        item: ItemKind,
        amount: u16,
        node_health: u16,
    },
    BuiltStructure(PlacedStructure),
    UpgradedStructure(PlacedStructure),
    DamagedStructure {
        id: u32,
        destroyed: bool,
    },
    DamagedPlayer {
        target: PlayerId,
        damage: f32,
        defeated: bool,
    },
    DamagedNpc {
        npc_id: u16,
        damage: f32,
        defeated: bool,
    },
    FiredWeapon {
        weapon: ItemKind,
        start: [f32; 3],
        end: [f32; 3],
        hit_player: Option<PlayerId>,
        headshot: bool,
        damage: f32,
    },
    MonumentInteracted(MonumentPuzzleState),
    SwungEmpty,
}

pub struct SurvivalWorld {
    source_rules_pending: bool,
    pub nodes: Vec<ResourceNode>,
    pub structures: Vec<PlacedStructure>,
    next_structure_id: u32,
    pub players: BTreeMap<PlayerId, PlayerSurvival>,
    pub monument: MonumentPuzzleState,
    pub npcs: Vec<NpcEntity>,
    pub vehicles: Vec<VehicleEntity>,
    pub nuclear: NuclearStrikeState,
}

impl SurvivalWorld {
    pub fn new(map: MapId) -> Self {
        Self {
            source_rules_pending: map == MapId::ImportedIsland,
            nodes: map_resource_nodes(map),
            structures: Vec::new(),
            next_structure_id: 1,
            players: BTreeMap::new(),
            monument: MonumentPuzzleState::default(),
            npcs: map_npcs(map),
            vehicles: map_vehicles(map),
            nuclear: NuclearStrikeState::default(),
        }
    }

    pub fn has_building_privilege(&self, actor: PlayerId, position: [f32; 3]) -> bool {
        let mut any_tc_in_range = false;
        for structure in &self.structures {
            if structure.kind != StructureKind::ToolCupboard {
                continue;
            }
            let dx = structure.position[0] - position[0];
            let dz = structure.position[2] - position[2];
            if dx * dx + dz * dz <= TC_PRIVILEGE_RADIUS * TC_PRIVILEGE_RADIUS {
                any_tc_in_range = true;
                if structure.owner == actor {
                    return true;
                }
            }
        }
        !any_tc_in_range
    }

    pub fn targeted_structure(
        &self,
        player_position: [f32; 3],
        yaw: f32,
        pitch: f32,
        max_distance: f32,
    ) -> Option<&PlacedStructure> {
        let origin = [
            player_position[0],
            player_position[1] + EYE_OFFSET,
            player_position[2],
        ];
        let dir = look_direction(yaw, pitch);
        let mut best: Option<(f32, &PlacedStructure)> = None;
        for structure in &self.structures {
            let ext = structure.kind.oriented_half_extents(structure.rotation);
            let min = [
                structure.position[0] - ext[0],
                structure.position[1] - ext[1],
                structure.position[2] - ext[2],
            ];
            let max = [
                structure.position[0] + ext[0],
                structure.position[1] + ext[1],
                structure.position[2] + ext[2],
            ];
            if let Some(distance) = ray_aabb(origin, dir, min, max, max_distance)
                && best.is_none_or(|(d, _)| distance < d)
            {
                best = Some((distance, structure));
            }
        }
        best.map(|(_, s)| s)
    }

    pub fn targeted_monument_hotspot(
        &self,
        player_position: [f32; 3],
        yaw: f32,
        pitch: f32,
        max_distance: f32,
    ) -> Option<MonumentHotspot> {
        let origin = [
            player_position[0],
            player_position[1] + EYE_OFFSET,
            player_position[2],
        ];
        let dir = look_direction(yaw, pitch);
        let mut best: Option<(f32, MonumentHotspot)> = None;
        for hotspot in MonumentHotspot::ALL {
            if hotspot == MonumentHotspot::FalloutCrate
                && (self.nuclear.phase != NuclearPhase::Fallout
                    || self.nuclear.fallout_crate_looted)
            {
                continue;
            }
            let pos = hotspot.position();
            let ext = hotspot.half_extents();
            let min = [pos[0] - ext[0], pos[1] - ext[1], pos[2] - ext[2]];
            let max = [pos[0] + ext[0], pos[1] + ext[1], pos[2] + ext[2]];
            if let Some(distance) = ray_aabb(origin, dir, min, max, max_distance)
                && best.is_none_or(|(d, _)| distance < d)
            {
                best = Some((distance, hotspot));
            }
        }
        if let Some((_, hotspot)) = best {
            return Some(hotspot);
        }
        // Fallback proximity check within 1.85m so interacting in tight bunker corridors is effortless.
        let mut closest_prox: Option<(f32, MonumentHotspot)> = None;
        for hotspot in MonumentHotspot::ALL {
            if hotspot == MonumentHotspot::FalloutCrate
                && (self.nuclear.phase != NuclearPhase::Fallout
                    || self.nuclear.fallout_crate_looted)
            {
                continue;
            }
            let pos = hotspot.position();
            let dx = pos[0] - player_position[0];
            let dz = pos[2] - player_position[2];
            let d2 = dx * dx + dz * dz;
            if d2 <= 1.85 * 1.85 && closest_prox.is_none_or(|(best_d2, _)| d2 < best_d2) {
                closest_prox = Some((d2, hotspot));
            }
        }
        closest_prox.map(|(_, h)| h)
    }

    pub fn targeted_npc(
        &self,
        player_position: [f32; 3],
        yaw: f32,
        pitch: f32,
        max_distance: f32,
    ) -> Option<&NpcEntity> {
        let origin = [
            player_position[0],
            player_position[1] + EYE_OFFSET,
            player_position[2],
        ];
        let dir = look_direction(yaw, pitch);
        let mut best: Option<(f32, &NpcEntity)> = None;
        for npc in &self.npcs {
            if !npc.is_alive() {
                continue;
            }
            let ext = npc.kind.half_extents();
            let min = [
                npc.position[0] - ext[0],
                npc.position[1] - ext[1],
                npc.position[2] - ext[2],
            ];
            let max = [
                npc.position[0] + ext[0],
                npc.position[1] + ext[1],
                npc.position[2] + ext[2],
            ];
            if let Some(distance) = ray_aabb(origin, dir, min, max, max_distance)
                && best.is_none_or(|(d, _)| distance < d)
            {
                best = Some((distance, npc));
            }
        }
        best.map(|(_, npc)| npc)
    }

    pub fn targeted_vehicle(
        &self,
        player_position: [f32; 3],
        yaw: f32,
        pitch: f32,
        max_distance: f32,
    ) -> Option<&VehicleEntity> {
        let origin = [
            player_position[0],
            player_position[1] + EYE_OFFSET,
            player_position[2],
        ];
        let dir = look_direction(yaw, pitch);
        let mut best: Option<(f32, &VehicleEntity)> = None;
        for vehicle in &self.vehicles {
            if vehicle.health == 0 {
                continue;
            }
            let ext = vehicle.kind.half_extents();
            let min = [
                vehicle.position[0] - ext[0],
                vehicle.position[1],
                vehicle.position[2] - ext[2],
            ];
            let max = [
                vehicle.position[0] + ext[0],
                vehicle.position[1] + ext[1] * 2.0,
                vehicle.position[2] + ext[2],
            ];
            if let Some(distance) = ray_aabb(origin, dir, min, max, max_distance)
                && best.is_none_or(|(d, _)| distance < d)
            {
                best = Some((distance, vehicle));
            }
        }
        if let Some((_, v)) = best {
            return Some(v);
        }
        let mut closest_prox: Option<(f32, &VehicleEntity)> = None;
        for vehicle in &self.vehicles {
            if vehicle.health == 0 {
                continue;
            }
            let dx = vehicle.position[0] - player_position[0];
            let dz = vehicle.position[2] - player_position[2];
            let d2 = dx * dx + dz * dz;
            if d2 <= 2.8 * 2.8 && closest_prox.is_none_or(|(best_d2, _)| d2 < best_d2) {
                closest_prox = Some((d2, vehicle));
            }
        }
        closest_prox.map(|(_, v)| v)
    }

    pub fn toggle_mount_vehicle(
        &mut self,
        actor: PlayerId,
        actor_position: [f32; 3],
        yaw: f32,
        pitch: f32,
    ) -> Result<Option<u16>, &'static str> {
        let Some(current_mounted) = self.players.get(&actor).map(|p| p.mounted_vehicle) else {
            return Err("Гравця не знайдено");
        };
        if let Some(vid) = current_mounted {
            if let Some(v) = self.vehicles.iter_mut().find(|v| v.id == vid) {
                v.driver = None;
            }
            if let Some(state) = self.players.get_mut(&actor) {
                state.mounted_vehicle = None;
                state.notice = Some("Ви вийшли з транспорту".into());
            }
            return Ok(None);
        }

        let Some(target_id) = self
            .targeted_vehicle(actor_position, yaw, pitch, 4.0)
            .map(|v| v.id)
        else {
            return Err("Поблизу немає справного транспорту (підійдіть ближче і натисніть F)");
        };
        let vehicle_idx = self
            .vehicles
            .iter()
            .position(|v| v.id == target_id)
            .ok_or("Транспорт не знайдено")?;
        if self.vehicles[vehicle_idx].driver.is_some() {
            return Err("Цей транспорт уже зайнятий іншим водієм");
        }
        let uses_fuel = self.vehicles[vehicle_idx].kind.uses_fuel();
        let state = self.players.get_mut(&actor).unwrap();
        if uses_fuel && self.vehicles[vehicle_idx].fuel < 100 {
            let available = state.inventory.count_item(ItemKind::LowGradeFuel).min(25) as u16;
            if available > 0 {
                state
                    .inventory
                    .remove_item(ItemKind::LowGradeFuel, available);
                self.vehicles[vehicle_idx].fuel =
                    (self.vehicles[vehicle_idx].fuel + available).min(250);
            }
        }
        if uses_fuel && self.vehicles[vehicle_idx].fuel == 0 {
            return Err("Бак порожній! Скрафтіть або візьміть Паливо низької якості");
        }
        self.vehicles[vehicle_idx].driver = Some(actor);
        let v_label = self.vehicles[vehicle_idx].kind.label();
        let v_fuel = self.vehicles[vehicle_idx].fuel;
        state.mounted_vehicle = Some(target_id);
        state.notice = Some(if uses_fuel {
            format!("Ви сіли за кермо: {v_label} (Паливо: {v_fuel} од., F — вийти)")
        } else {
            format!("Ви сіли на {v_label} (F — злізти)")
        });
        Ok(Some(target_id))
    }

    pub fn join(&mut self, id: PlayerId) -> &PlayerSurvival {
        let pending = self.source_rules_pending;
        self.players.entry(id).or_insert_with(|| {
            let mut state = PlayerSurvival::default();
            if pending {
                state.inventory = Inventory::empty();
                state.notice =
                    Some("Source inventory and gameplay rules are being reconstructed.".into());
            }
            state
        })
    }

    pub fn source_rules_pending(&self) -> bool {
        self.source_rules_pending
    }

    pub fn leave(&mut self, id: PlayerId) {
        if let Some(removed) = self.players.remove(&id)
            && let Some(vid) = removed.mounted_vehicle
            && let Some(v) = self.vehicles.iter_mut().find(|v| v.id == vid)
        {
            v.driver = None;
        }
    }

    pub fn player(&self, id: PlayerId) -> Option<&PlayerSurvival> {
        self.players.get(&id)
    }

    pub fn player_mut(&mut self, id: PlayerId) -> Option<&mut PlayerSurvival> {
        self.players.get_mut(&id)
    }

    pub fn can_sprint(&self, id: PlayerId) -> bool {
        self.players
            .get(&id)
            .is_none_or(|state| state.mounted_vehicle.is_some() || state.vitals.can_sprint())
    }

    pub fn environment_at(&self, tick: ServerTick, position: [f32; 3]) -> WorldEnvironment {
        let time_hours = time_of_day_hours(tick);
        let is_night = !(6.0..=20.0).contains(&time_hours);
        let biome = Biome::at(position);
        let weather = WeatherKind::at_tick(tick, biome);
        let day_phase = ((time_hours - 6.0) / 24.0 * std::f32::consts::TAU).sin();
        let diurnal_swing = match biome {
            Biome::Desert => day_phase * 10.0 - if is_night { 12.0 } else { 0.0 },
            Biome::Winter => day_phase * 4.0 - if is_night { 6.0 } else { 0.0 },
            _ => day_phase * 5.0 - if is_night { 4.0 } else { 0.0 },
        };
        let near_campfire = self.structures.iter().any(|s| {
            if s.kind != StructureKind::Campfire {
                return false;
            }
            let dx = s.position[0] - position[0];
            let dz = s.position[2] - position[2];
            dx * dx + dz * dz <= CAMPFIRE_WARMTH_RADIUS * CAMPFIRE_WARMTH_RADIUS
        });
        let campfire_bonus = if near_campfire { 18.0 } else { 0.0 };
        let ambient_temp_c = biome.base_temperature_c()
            + diurnal_swing
            + weather.temperature_offset_c()
            + campfire_bonus;
        let in_radtown = is_in_radtown(position)
            || (matches!(
                self.nuclear.phase,
                NuclearPhase::DetonationShockwave | NuclearPhase::Fallout
            ) && is_in_nuclear_blast(position));
        let t = tick.0 as f32 / 60.0;
        let wind_offset = [t * 0.65, t * 0.28];
        WorldEnvironment {
            time_hours,
            is_night,
            biome,
            weather,
            ambient_temp_c,
            near_campfire,
            in_radtown,
            wind_offset,
        }
    }

    /// Tick resource respawns, monument timers, nuclear strike, vehicles, NPCs, and periodic vitals drain.
    /// Returns node IDs that respawned and players whose vitals reached 0 HP (requiring respawn).
    pub fn step(
        &mut self,
        tick: ServerTick,
        sprinting_players: impl IntoIterator<Item = (PlayerId, bool)>,
    ) -> (Vec<ResourceNode>, Vec<PlayerId>) {
        self.step_with_positions(
            tick,
            sprinting_players
                .into_iter()
                .map(|(id, sprinting)| (id, sprinting, [0.0, 2.0, 0.0])),
        )
    }

    pub fn step_with_positions(
        &mut self,
        tick: ServerTick,
        players: impl IntoIterator<Item = (PlayerId, bool, [f32; 3])>,
    ) -> (Vec<ResourceNode>, Vec<PlayerId>) {
        if self.source_rules_pending {
            return (Vec::new(), Vec::new());
        }
        self.monument.step();
        let transitioned_nuclear = self.nuclear.step();
        let mut respawned_nodes = Vec::new();
        for node in &mut self.nodes {
            if let Some(respawn_tick) = node.respawn_at
                && tick >= respawn_tick
            {
                node.health = node.max_health;
                node.respawn_at = None;
                respawned_nodes.push(*node);
            }
        }
        for npc in &mut self.npcs {
            if let Some(respawn_tick) = npc.respawn_at
                && tick >= respawn_tick
            {
                npc.health = npc.max_health;
                npc.respawn_at = None;
            }
        }

        let player_list: Vec<(PlayerId, bool, [f32; 3])> = players.into_iter().collect();
        let mut defeated = Vec::new();

        // Sync mounted vehicle positions with their drivers every tick.
        for &(id, moving, pos) in &player_list {
            let mounted = self.players.get(&id).and_then(|p| p.mounted_vehicle);
            if let Some(vid) = mounted {
                if let Some(v) = self
                    .vehicles
                    .iter_mut()
                    .find(|v| v.id == vid && v.health > 0)
                {
                    v.driver = Some(id);
                    let vy = if v.kind == VehicleKind::Minicopter && pos[1] > 1.2 {
                        pos[1] - 0.65
                    } else {
                        0.0
                    };
                    let dx = pos[0] - v.position[0];
                    let dz = pos[2] - v.position[2];
                    if dx * dx + dz * dz > 1e-4 {
                        v.yaw = (-dx).atan2(-dz);
                    }
                    v.position = [pos[0], vy, pos[2]];
                    if tick.0.is_multiple_of(60) && moving && v.kind.uses_fuel() {
                        v.fuel = v.fuel.saturating_sub(1);
                        if v.fuel == 0 {
                            v.driver = None;
                            if let Some(p) = self.players.get_mut(&id) {
                                p.mounted_vehicle = None;
                                p.notice = Some("Закінчилося паливо! Транспорт зупинився".into());
                            }
                        }
                    }
                } else if let Some(p) = self.players.get_mut(&id) {
                    p.mounted_vehicle = None;
                }
            }
        }

        // Apply Nuclear Detonation Shockwave destruction when transitioning from SirenWarning -> DetonationShockwave.
        if transitioned_nuclear == Some(NuclearPhase::DetonationShockwave) {
            self.structures.retain_mut(|s| {
                if !is_in_nuclear_blast(s.position) {
                    return true;
                }
                match s.tier {
                    BuildingTier::Twig | BuildingTier::Wood => false,
                    BuildingTier::Stone | BuildingTier::Metal => {
                        s.health = s.health.saturating_sub(220);
                        s.health > 0
                    }
                }
            });
            for &(id, _, pos) in &player_list {
                if is_in_nuclear_blast(pos)
                    && let Some(p) = self.players.get_mut(&id)
                {
                    p.vitals.radiation = (p.vitals.radiation + 35.0).clamp(0.0, MAX_RADIATION);
                    if p.vitals.apply_damage(65.0) {
                        if let Some(vid) = p.mounted_vehicle.take()
                            && let Some(v) = self.vehicles.iter_mut().find(|v| v.id == vid)
                        {
                            v.driver = None;
                        }
                        p.vitals = Vitals::default();
                        p.inventory = Inventory::starter();
                        p.notice = Some("Вас спопелила ядерна ударна хвиля!".into());
                        defeated.push(id);
                    } else {
                        p.notice = Some("ЯДЕРНА УДАРНА ХВИЛЯ! (-65 ХП, +35 Рад)".into());
                    }
                }
            }
        }

        // Step NPC AI and player vitals every 15 ticks (4 Hz = 0.25s) to keep per-tick work tiny.
        if tick.0.is_multiple_of(15) {
            let dt = 0.25_f32;
            let is_night = !(6.0..=20.0).contains(&time_of_day_hours(tick));
            let attack_tick = tick.0.is_multiple_of(60);
            const BRADLEY_WAYPOINTS: [[f32; 3]; 4] = [
                [-11.5, 1.15, -9.5],
                [-4.0, 1.15, -9.5],
                [-4.0, 1.15, -18.0],
                [-15.0, 1.15, -18.0],
            ];

            for i in 0..self.npcs.len() {
                if !self.npcs[i].is_alive() {
                    continue;
                }
                let kind = self.npcs[i].kind;
                let npc_pos = self.npcs[i].position;
                match kind {
                    NpcKind::PatrolHeli => {
                        let angle = (tick.0 as f32 / (60.0 * 35.0)) * std::f32::consts::TAU;
                        let r = 18.0;
                        let nx = angle.cos() * r;
                        let nz = angle.sin() * r - 4.0;
                        let dx = nx - npc_pos[0];
                        let dz = nz - npc_pos[2];
                        self.npcs[i].position = [nx, 14.5, nz];
                        if dx * dx + dz * dz > 1e-5 {
                            self.npcs[i].yaw = (-dx).atan2(-dz);
                        }
                        if attack_tick {
                            let mut armed_target: Option<(f32, PlayerId)> = None;
                            for &(pid, _, ppos) in &player_list {
                                let Some(pstate) = self.players.get(&pid) else {
                                    continue;
                                };
                                // PatrolHeli aggression rule: ONLY targets players carrying firearms in hotbar!
                                if !player_triggers_patrol_heli(&pstate.inventory) {
                                    continue;
                                }
                                let pdx = ppos[0] - nx;
                                let pdz = ppos[2] - nz;
                                let d2 = pdx * pdx + pdz * pdz;
                                if d2 <= 28.0 * 28.0
                                    && armed_target.is_none_or(|(best, _)| d2 < best)
                                {
                                    armed_target = Some((d2, pid));
                                }
                            }
                            if let Some((_, target_id)) = armed_target
                                && let Some(pstate) = self.players.get_mut(&target_id)
                            {
                                let dmg = 14.0;
                                if pstate.vitals.apply_damage(dmg) {
                                    pstate.vitals = Vitals::default();
                                    pstate.inventory = Inventory::starter();
                                    pstate.mounted_vehicle = None;
                                    pstate.notice =
                                        Some("Вас ліквідував Патрульний вертоліт!".into());
                                    if !defeated.contains(&target_id) {
                                        defeated.push(target_id);
                                    }
                                } else {
                                    pstate.notice = Some(
                                        "Патрульний вертоліт атакує вас через зброю в руках! (-14 ХП)"
                                            .into(),
                                    );
                                }
                            }
                        }
                    }
                    NpcKind::BradleyApc => {
                        let wp_idx =
                            (self.npcs[i].waypoint_index as usize) % BRADLEY_WAYPOINTS.len();
                        let target_wp = BRADLEY_WAYPOINTS[wp_idx];
                        let dx = target_wp[0] - npc_pos[0];
                        let dz = target_wp[2] - npc_pos[2];
                        let dist = (dx * dx + dz * dz).sqrt();
                        if dist < 0.8 {
                            self.npcs[i].waypoint_index =
                                (self.npcs[i].waypoint_index + 1) % BRADLEY_WAYPOINTS.len() as u8;
                        } else {
                            let step_len = (2.6 * dt).min(dist);
                            self.npcs[i].position[0] += (dx / dist) * step_len;
                            self.npcs[i].position[2] += (dz / dist) * step_len;
                            self.npcs[i].yaw = (-dx).atan2(-dz);
                        }
                        if attack_tick {
                            let mut closest: Option<(f32, PlayerId)> = None;
                            for &(pid, _, ppos) in &player_list {
                                if ppos[2] > 4.0 {
                                    continue;
                                }
                                let pdx = ppos[0] - self.npcs[i].position[0];
                                let pdz = ppos[2] - self.npcs[i].position[2];
                                let d2 = pdx * pdx + pdz * pdz;
                                if d2 <= 15.0 * 15.0 && closest.is_none_or(|(best, _)| d2 < best) {
                                    closest = Some((d2, pid));
                                }
                            }
                            if let Some((_, target_id)) = closest
                                && let Some(pstate) = self.players.get_mut(&target_id)
                            {
                                if pstate.vitals.apply_damage(16.0) {
                                    pstate.vitals = Vitals::default();
                                    pstate.inventory = Inventory::starter();
                                    pstate.mounted_vehicle = None;
                                    pstate.notice =
                                        Some("Вас знищив гарматний вогонь БТР «Бредлі»!".into());
                                    if !defeated.contains(&target_id) {
                                        defeated.push(target_id);
                                    }
                                } else {
                                    pstate.notice =
                                        Some("Вогневий удар БТР «Бредлі» (-16 ХП)".into());
                                }
                            }
                        }
                    }
                    NpcKind::ScientistNpc => {
                        let mut closest: Option<(f32, PlayerId, [f32; 3])> = None;
                        for &(pid, _, ppos) in &player_list {
                            if ppos[2] > 4.0 {
                                continue;
                            }
                            let pdx = ppos[0] - npc_pos[0];
                            let pdz = ppos[2] - npc_pos[2];
                            let d2 = pdx * pdx + pdz * pdz;
                            if d2 <= 13.5 * 13.5 && closest.is_none_or(|(best, _, _)| d2 < best) {
                                closest = Some((d2, pid, ppos));
                            }
                        }
                        if let Some((_, target_id, ppos)) = closest {
                            let dx = ppos[0] - npc_pos[0];
                            let dz = ppos[2] - npc_pos[2];
                            self.npcs[i].yaw = (-dx).atan2(-dz);
                            if attack_tick && let Some(pstate) = self.players.get_mut(&target_id) {
                                if pstate.vitals.apply_damage(10.0) {
                                    pstate.vitals = Vitals::default();
                                    pstate.inventory = Inventory::starter();
                                    pstate.mounted_vehicle = None;
                                    pstate.notice =
                                        Some("Вас застрелив вчений-патрульний РТ!".into());
                                    if !defeated.contains(&target_id) {
                                        defeated.push(target_id);
                                    }
                                } else {
                                    pstate.notice =
                                        Some("Вчений-патрульний РТ влучив у вас (-10 ХП)".into());
                                }
                            }
                        }
                    }
                    NpcKind::ZombieWalker | NpcKind::RadZombie => {
                        let mut closest: Option<(f32, PlayerId, [f32; 3])> = None;
                        for &(pid, _, ppos) in &player_list {
                            if ppos[2] > 6.0 {
                                continue;
                            }
                            let pdx = ppos[0] - npc_pos[0];
                            let pdz = ppos[2] - npc_pos[2];
                            let d2 = pdx * pdx + pdz * pdz;
                            if d2 <= 11.5 * 11.5 && closest.is_none_or(|(best, _, _)| d2 < best) {
                                closest = Some((d2, pid, ppos));
                            }
                        }
                        if let Some((d2, target_id, ppos)) = closest {
                            let dist = d2.sqrt();
                            let dx = ppos[0] - npc_pos[0];
                            let dz = ppos[2] - npc_pos[2];
                            self.npcs[i].yaw = (-dx).atan2(-dz);
                            if dist > 1.35 {
                                let speed = if is_night { 3.5 } else { 2.2 };
                                let step_len = (speed * dt).min(dist - 1.25);
                                self.npcs[i].position[0] += (dx / dist) * step_len;
                                self.npcs[i].position[2] += (dz / dist) * step_len;
                            }
                            if dist <= 2.1
                                && attack_tick
                                && let Some(pstate) = self.players.get_mut(&target_id)
                            {
                                let dmg = if kind == NpcKind::RadZombie {
                                    pstate.vitals.radiation =
                                        (pstate.vitals.radiation + 10.0).clamp(0.0, MAX_RADIATION);
                                    14.0
                                } else {
                                    11.0
                                };
                                if pstate.vitals.apply_damage(dmg) {
                                    pstate.vitals = Vitals::default();
                                    pstate.inventory = Inventory::starter();
                                    pstate.mounted_vehicle = None;
                                    pstate.notice = Some(format!("Вас розірвав {}!", kind.label()));
                                    if !defeated.contains(&target_id) {
                                        defeated.push(target_id);
                                    }
                                } else {
                                    pstate.notice =
                                        Some(format!("Укус: {} (-{dmg:.0} ХП)", kind.label()));
                                }
                            }
                        }
                    }
                }
            }

            let snapshots: Vec<_> = player_list
                .into_iter()
                .map(|(id, sprinting, pos)| (id, sprinting, self.environment_at(tick, pos)))
                .collect();
            for (id, sprinting, env) in snapshots {
                if let Some(player) = self.players.get_mut(&id) {
                    let effective_sprint = sprinting && player.mounted_vehicle.is_none();
                    player.vitals.step_full(
                        effective_sprint,
                        env.ambient_temp_c,
                        env.biome.hydration_multiplier(),
                        env.near_campfire,
                        env.in_radtown,
                        dt,
                    );
                    if player.vitals.health <= 0.0 && !defeated.contains(&id) {
                        if let Some(vid) = player.mounted_vehicle.take()
                            && let Some(v) = self.vehicles.iter_mut().find(|v| v.id == vid)
                        {
                            v.driver = None;
                        }
                        player.vitals = Vitals::default();
                        player.inventory = Inventory::starter();
                        player.notice = Some("Ви загинули й відродилися заново".into());
                        defeated.push(id);
                    }
                }
            }
        }
        (respawned_nodes, defeated)
    }

    /// Raycast from the player's eye along look direction to find the closest
    /// resource node within `max_distance`.
    pub fn targeted_node(
        &self,
        player_position: [f32; 3],
        yaw: f32,
        pitch: f32,
        max_distance: f32,
    ) -> Option<&ResourceNode> {
        let origin = [
            player_position[0],
            player_position[1] + EYE_OFFSET,
            player_position[2],
        ];
        let dir = look_direction(yaw, pitch);
        let mut best: Option<(f32, &ResourceNode)> = None;
        for node in &self.nodes {
            if !node.is_available() {
                continue;
            }
            let min = [
                node.position[0] - node.kind.radius(),
                node.position[1],
                node.position[2] - node.kind.radius(),
            ];
            let max = [
                node.position[0] + node.kind.radius(),
                node.position[1] + node.kind.height(),
                node.position[2] + node.kind.radius(),
            ];
            if let Some(distance) = ray_aabb(origin, dir, min, max, max_distance)
                && best.is_none_or(|(d, _)| distance < d)
            {
                best = Some((distance, node));
            }
        }
        best.map(|(_, node)| node)
    }

    fn interact_monument(
        &mut self,
        actor: PlayerId,
        hotspot: MonumentHotspot,
    ) -> Result<ActionOutcome, &'static str> {
        let state = self.players.get_mut(&actor).ok_or("Гравця не знайдено")?;
        match hotspot {
            MonumentHotspot::SupplyCrate => {
                if self.monument.supply_crate_cooldown > 0 {
                    return Err("Ящик постачання РТ ще відновлюється");
                }
                self.monument.supply_crate_cooldown = 60 * 30;
                state.inventory.add_item(ItemKind::ElectricFuse, 1);
                state.inventory.add_item(ItemKind::GreenKeycard, 1);
                state.inventory.add_item(ItemKind::AntiRadPills, 2);
                state.inventory.add_item(ItemKind::LowGradeFuel, 30);
                state.inventory.add_item(ItemKind::Scrap, 25);
                state.inventory.add_item(ItemKind::MetalFragments, 80);
                state.notice = Some(
                    "РТ Ящик: +Запобіжник, +Зелена картка, +30 Паливо, +25 Скрап, +80 Метал".into(),
                );
                Ok(ActionOutcome::MonumentInteracted(self.monument))
            }
            MonumentHotspot::Recycler => {
                if state.inventory.remove_item(ItemKind::Stone, 50) {
                    state.inventory.add_item(ItemKind::MetalFragments, 40);
                    state.inventory.add_item(ItemKind::Scrap, 10);
                    state.notice = Some(
                        "Переробник РТ: -50 Каміння -> +40 Металеві фрагменти, +10 Скрап".into(),
                    );
                    Ok(ActionOutcome::MonumentInteracted(self.monument))
                } else if state.inventory.remove_item(ItemKind::Wood, 50) {
                    state.inventory.add_item(ItemKind::MetalFragments, 25);
                    state.inventory.add_item(ItemKind::Scrap, 5);
                    state.notice = Some(
                        "Переробник РТ: -50 Деревини -> +25 Металеві фрагменти, +5 Скрап".into(),
                    );
                    Ok(ActionOutcome::MonumentInteracted(self.monument))
                } else {
                    Err("Переробник РТ: потрібно 50 Каміння або 50 Деревини")
                }
            }
            MonumentHotspot::FuseBox => {
                if self.monument.fuse_ticks_remaining == 0 {
                    if !state.inventory.remove_item(ItemKind::ElectricFuse, 1) {
                        return Err(
                            "Потрібен Електричний запобіжник (візьміть у ящику РТ або скрафтіть)",
                        );
                    }
                    self.monument.fuse_ticks_remaining = 60 * 60;
                    self.monument.switch_on = true;
                    state.notice = Some(
                        "Вставлено запобіжник та увімкнено рубильник! Живлення РТ подано (60с)"
                            .into(),
                    );
                } else {
                    self.monument.switch_on = !self.monument.switch_on;
                    state.notice = Some(if self.monument.switch_on {
                        "Рубильник генератора РТ: УВІМКНЕНО (живлення активне)".into()
                    } else {
                        "Рубильник генератора РТ: ВИМКНЕНО".into()
                    });
                }
                Ok(ActionOutcome::MonumentInteracted(self.monument))
            }
            MonumentHotspot::CardReader => {
                if !self.monument.is_powered() {
                    return Err("Зчитувач знеструмлено! Спочатку вставте запобіжник у щиток РТ");
                }
                if self.monument.is_door_open() {
                    state.notice = Some("Бронедвері бункера РТ вже відчинені!".into());
                    return Ok(ActionOutcome::MonumentInteracted(self.monument));
                }
                if state.inventory.count_item(ItemKind::GreenKeycard) == 0 {
                    return Err("Потрібна Зелена картка доступу для відкриття бункера РТ");
                }
                self.monument.door_open_ticks = 60 * 45;
                state.notice =
                    Some("Зелена картка прийнята! Бронедвері бункера РТ відчинено на 45с".into());
                Ok(ActionOutcome::MonumentInteracted(self.monument))
            }
            MonumentHotspot::MilitaryCrate => {
                if !self.monument.is_door_open() {
                    return Err("Бункер РТ зачинено! Подайте живлення і проведіть Зеленою карткою");
                }
                if self.monument.military_crate_cooldown > 0 {
                    return Err("Військовий ящик бункера РТ ще порожній");
                }
                self.monument.military_crate_cooldown = 60 * 45;
                state.inventory.add_item(ItemKind::AssaultRifle, 1);
                state.inventory.add_item(ItemKind::RifleAmmo, 30);
                state.inventory.add_item(ItemKind::Revolver, 1);
                state.inventory.add_item(ItemKind::PistolAmmo, 24);
                state.inventory.add_item(ItemKind::C4Explosive, 1);
                state.inventory.add_item(ItemKind::LowGradeFuel, 40);
                state.inventory.add_item(ItemKind::Scrap, 50);
                state.inventory.add_item(ItemKind::MetalFragments, 150);
                state.notice = Some(
                    "Військовий ящик РТ: +Штурмова гвинтівка АК, +30 5.56мм, +1 C4, +50 Скрап!"
                        .into(),
                );
                Ok(ActionOutcome::MonumentInteracted(self.monument))
            }
            MonumentHotspot::NuclearSiloButton => {
                if !self.monument.is_door_open() {
                    return Err(
                        "Доступ до ядерного пульта заблоковано! Спочатку відкрийте бункер РТ",
                    );
                }
                if !self.nuclear.launch() {
                    return Err("Ядерна боєголовка вже активована або шахта перезаряджається");
                }
                state.notice = Some(
                    "УВАГА! ЗАПУЩЕНО ЯДЕРНУ БОЄГОЛОВКУ! Сирена тривоги — покиньте червону зону на карті (M)!"
                        .into(),
                );
                Ok(ActionOutcome::MonumentInteracted(self.monument))
            }
            MonumentHotspot::FalloutCrate => {
                if self.nuclear.phase != NuclearPhase::Fallout || self.nuclear.fallout_crate_looted
                {
                    return Err("Елітний радіоактивний контейнер недоступний");
                }
                self.nuclear.fallout_crate_looted = true;
                state.inventory.add_item(ItemKind::C4Explosive, 2);
                state.inventory.add_item(ItemKind::AssaultRifle, 1);
                state.inventory.add_item(ItemKind::RifleAmmo, 60);
                state.inventory.add_item(ItemKind::AntiRadPills, 3);
                state.inventory.add_item(ItemKind::Scrap, 120);
                state.inventory.add_item(ItemKind::MetalFragments, 250);
                state.notice = Some(
                    "Елітний контейнер кратера: +2 C4, +АК-47, +60 5.56мм, +120 Скрап, +250 Метал!"
                        .into(),
                );
                Ok(ActionOutcome::MonumentInteracted(self.monument))
            }
        }
    }

    fn apply_npc_damage(
        &mut self,
        actor: PlayerId,
        tick: ServerTick,
        npc_id: u16,
        damage: f32,
        headshot: bool,
    ) -> bool {
        let Some(npc) = self
            .npcs
            .iter_mut()
            .find(|n| n.id == npc_id && n.is_alive())
        else {
            return false;
        };
        let dmg_u16 = (damage.round() as u16).max(1);
        npc.health = npc.health.saturating_sub(dmg_u16);
        let defeated = npc.health == 0;
        let kind = npc.kind;
        let remaining_hp = npc.health;
        let max_hp = npc.max_health;
        if defeated {
            npc.respawn_at = Some(ServerTick(tick.0 + 60 * 45));
        }
        if let Some(actor_state) = self.players.get_mut(&actor) {
            if defeated {
                match kind {
                    NpcKind::ZombieWalker => {
                        actor_state.inventory.add_item(ItemKind::Scrap, 15);
                        actor_state.inventory.add_item(ItemKind::Bandage, 1);
                        actor_state.inventory.add_item(ItemKind::LowGradeFuel, 15);
                        actor_state.notice = Some(
                            "Ліквідовано: Зомбі-блукач (+15 Скрап, +1 Бинт, +15 Паливо)".into(),
                        );
                    }
                    NpcKind::RadZombie => {
                        actor_state.inventory.add_item(ItemKind::Scrap, 25);
                        actor_state.inventory.add_item(ItemKind::AntiRadPills, 1);
                        actor_state.inventory.add_item(ItemKind::LowGradeFuel, 25);
                        actor_state.notice = Some(
                            "Ліквідовано: Радіоактивний зомбі (+25 Скрап, +1 Антирад, +25 Паливо)"
                                .into(),
                        );
                    }
                    NpcKind::ScientistNpc => {
                        actor_state.inventory.add_item(ItemKind::Scrap, 35);
                        actor_state.inventory.add_item(ItemKind::PistolAmmo, 16);
                        actor_state.inventory.add_item(ItemKind::LowGradeFuel, 20);
                        actor_state.notice = Some(
                            "Ліквідовано: Вчений РТ (+35 Скрап, +16 Набої 9мм, +20 Паливо)".into(),
                        );
                    }
                    NpcKind::BradleyApc => {
                        actor_state.inventory.add_item(ItemKind::Scrap, 120);
                        actor_state
                            .inventory
                            .add_item(ItemKind::MetalFragments, 200);
                        actor_state.inventory.add_item(ItemKind::C4Explosive, 1);
                        actor_state.inventory.add_item(ItemKind::RifleAmmo, 40);
                        actor_state.notice = Some(
                            "ЗНИЩЕНО БОСА БТР «БРЕДЛІ»! (+120 Скрап, +200 Метал, +1 C4, +40 5.56мм)"
                                .into(),
                        );
                    }
                    NpcKind::PatrolHeli => {
                        actor_state.inventory.add_item(ItemKind::Scrap, 100);
                        actor_state.inventory.add_item(ItemKind::AssaultRifle, 1);
                        actor_state.inventory.add_item(ItemKind::RifleAmmo, 40);
                        actor_state.inventory.add_item(ItemKind::LowGradeFuel, 40);
                        actor_state.notice = Some(
                            "ЗБИТО ПАТРУЛЬНИЙ ВЕРТОЛІТ! (+100 Скрап, +АК-47, +40 5.56мм, +40 Паливо)"
                                .into(),
                        );
                    }
                }
            } else {
                actor_state.notice = Some(if headshot {
                    format!(
                        "ХЕДШОТ по {}: -{dmg_u16} ХП ({remaining_hp}/{max_hp})",
                        kind.label()
                    )
                } else {
                    format!(
                        "Влучання по {}: -{dmg_u16} ХП ({remaining_hp}/{max_hp})",
                        kind.label()
                    )
                });
            }
        }
        defeated
    }

    pub fn primary_action(
        &mut self,
        actor: PlayerId,
        tick: ServerTick,
        actor_position: [f32; 3],
        yaw: f32,
        pitch: f32,
        other_players: &[(PlayerId, [f32; 3])],
    ) -> Result<ActionOutcome, &'static str> {
        if self.source_rules_pending {
            return Err("Source gameplay configuration is not loaded yet.");
        }
        if !yaw.is_finite() || !pitch.is_finite() {
            return Err("Некоректний кут дії");
        }
        let Some(state) = self.players.get_mut(&actor) else {
            return Err("Гравця не знайдено");
        };
        if state.last_action_tick.0 > 0
            && tick.0.saturating_sub(state.last_action_tick.0) < ACTION_COOLDOWN_TICKS
        {
            return Err("Зачекайте перед наступною дією");
        }
        state.last_action_tick = tick;

        let active = state.inventory.active_item().map(|s| s.kind);
        match active {
            Some(ItemKind::Bandage) => {
                if state.vitals.health >= MAX_HEALTH {
                    return Err("Здоров'я вже повне");
                }
                state.inventory.consume_active();
                state.vitals.health = (state.vitals.health + 15.0).min(MAX_HEALTH);
                state.notice = Some("Використано бинт (+15 ХП)".into());
                return Ok(ActionOutcome::Consumed {
                    item: ItemKind::Bandage,
                });
            }
            Some(ItemKind::WaterBottle) => {
                if state.vitals.hydration >= MAX_HYDRATION {
                    return Err("Спраги немає");
                }
                state.inventory.consume_active();
                state.vitals.hydration = (state.vitals.hydration + 75.0).min(MAX_HYDRATION);
                state.notice = Some("Випито воду (+75 Води)".into());
                return Ok(ActionOutcome::Consumed {
                    item: ItemKind::WaterBottle,
                });
            }
            Some(ItemKind::Ration) => {
                if state.vitals.calories >= MAX_CALORIES && state.vitals.health >= MAX_HEALTH {
                    return Err("Ви не голодні");
                }
                state.inventory.consume_active();
                state.vitals.calories = (state.vitals.calories + 100.0).min(MAX_CALORIES);
                state.vitals.health = (state.vitals.health + 5.0).min(MAX_HEALTH);
                state.notice = Some("З'їдено сухпайок (+100 Їжі, +5 ХП)".into());
                return Ok(ActionOutcome::Consumed {
                    item: ItemKind::Ration,
                });
            }
            Some(ItemKind::AntiRadPills) => {
                if state.vitals.radiation <= 0.0 && state.vitals.health >= MAX_HEALTH {
                    return Err("Рівень радіації нульовий");
                }
                state.inventory.consume_active();
                state.vitals.radiation = (state.vitals.radiation - 50.0).max(0.0);
                state.vitals.health = (state.vitals.health + 10.0).min(MAX_HEALTH);
                state.notice = Some("Прийнято антирадіаційні таблетки (-50 Рад, +10 ХП)".into());
                return Ok(ActionOutcome::Consumed {
                    item: ItemKind::AntiRadPills,
                });
            }
            _ => {}
        }

        // Check Radtown Monument interaction if the player is looking at or standing next to a monument hotspot
        // (unless holding BuildingPlan).
        if active != Some(ItemKind::BuildingPlan)
            && let Some(hotspot) = self.targeted_monument_hotspot(actor_position, yaw, pitch, 3.6)
        {
            return self.interact_monument(actor, hotspot);
        }

        if active == Some(ItemKind::TeleportGun) {
            let dir = look_direction(yaw, pitch);
            let end = [
                actor_position[0] + dir[0] * 500.0,
                actor_position[1] + dir[1] * 500.0,
                actor_position[2] + dir[2] * 500.0,
            ];
            if let Some(player) = self.players.get_mut(&actor) {
                player.notice = Some("Телепорт активовано!".into());
            }
            return Ok(ActionOutcome::FiredWeapon {
                weapon: ItemKind::TeleportGun,
                start: actor_position,
                end,
                hit_player: None,
                headshot: false,
                damage: 0.0,
            });
        }

        match active {
            Some(ItemKind::LowGradeFuel) => {
                let Some(vid) = self
                    .targeted_vehicle(actor_position, yaw, pitch, 3.8)
                    .map(|v| v.id)
                else {
                    return Err("Наведіть паливо на транспорт поблизу для заправки");
                };
                let v_idx = self.vehicles.iter().position(|v| v.id == vid).unwrap();
                if !self.vehicles[v_idx].kind.uses_fuel() {
                    return Err("Велосипед не потребує палива");
                }
                if self.vehicles[v_idx].fuel >= 250 {
                    return Err("Паливний бак транспорту вже повний");
                }
                let state = self.players.get_mut(&actor).unwrap();
                let take = state
                    .inventory
                    .count_item(ItemKind::LowGradeFuel)
                    .min(25)
                    .min((250 - self.vehicles[v_idx].fuel) as u32)
                    as u16;
                if take == 0 || !state.inventory.remove_item(ItemKind::LowGradeFuel, take) {
                    return Err("Недостатньо палива в інвентарі");
                }
                self.vehicles[v_idx].fuel += take;
                let total_fuel = self.vehicles[v_idx].fuel;
                let v_label = self.vehicles[v_idx].kind.label();
                state.notice = Some(format!(
                    "Заправлено {v_label}: +{take} Паливо (Бак: {total_fuel})"
                ));
                return Ok(ActionOutcome::Consumed {
                    item: ItemKind::LowGradeFuel,
                });
            }
            Some(ItemKind::C4Explosive) => {
                if let Some(npc_id) = self
                    .targeted_npc(actor_position, yaw, pitch, 5.5)
                    .map(|n| n.id)
                {
                    let state = self.players.get_mut(&actor).unwrap();
                    state.inventory.consume_active();
                    let defeated = self.apply_npc_damage(actor, tick, npc_id, 320.0, false);
                    return Ok(ActionOutcome::DamagedNpc {
                        npc_id,
                        damage: 320.0,
                        defeated,
                    });
                }
                if let Some(structure_id) = self
                    .targeted_structure(actor_position, yaw, pitch, 4.2)
                    .map(|s| s.id)
                {
                    let state = self.players.get_mut(&actor).unwrap();
                    state.inventory.consume_active();
                    let s_idx = self
                        .structures
                        .iter()
                        .position(|s| s.id == structure_id)
                        .unwrap();
                    self.structures[s_idx].health =
                        self.structures[s_idx].health.saturating_sub(320);
                    let destroyed = self.structures[s_idx].health == 0;
                    let s_label = self.structures[s_idx].kind.label();
                    if destroyed {
                        self.structures.remove(s_idx);
                        if let Some(st) = self.players.get_mut(&actor) {
                            st.notice = Some(format!("ПІДРИВ C4: зруйновано {s_label}! (-320 ХП)"));
                        }
                    } else {
                        let rem = self.structures[s_idx].health;
                        if let Some(st) = self.players.get_mut(&actor) {
                            st.notice = Some(format!(
                                "ПІДРИВ C4 по {s_label}: -320 ХП (залишилось {rem} ХП)"
                            ));
                        }
                    }
                    return Ok(ActionOutcome::DamagedStructure {
                        id: structure_id,
                        destroyed,
                    });
                }
                return Err("Наведіть C4 на споруду або БТР «Бредлі» для підриву");
            }
            _ => {}
        }

        if active == Some(ItemKind::BuildingPlan) {
            let state_ref = self.players.get(&actor).unwrap();
            let kind = state_ref.selected_structure;
            let build_rotation = state_ref.build_rotation;
            let cost = kind.build_cost_wood();
            let (target_pos, rotation) = preview_socket_placement(
                actor_position,
                yaw,
                pitch,
                kind,
                build_rotation,
                &self.structures,
            );
            if !self.has_building_privilege(actor, target_pos) {
                return Err("Чужа територія: будівництво заблоковано Шафою (TC)");
            }
            if self.structures.len() >= 256 {
                return Err("Досягнуто ліміту споруд на полігоні");
            }
            if self.structures.iter().any(|s| {
                s.kind == kind
                    && (s.position[0] - target_pos[0]).abs() < 0.2
                    && (s.position[1] - target_pos[1]).abs() < 0.2
                    && (s.position[2] - target_pos[2]).abs() < 0.2
            }) {
                return Err("На цьому сокеті вже стоїть така споруда");
            }
            let state = self.players.get_mut(&actor).unwrap();
            if !state.inventory.remove_item(ItemKind::Wood, cost) {
                return Err("Потрібно більше деревини для будівництва");
            }
            let tier = if matches!(kind, StructureKind::ToolCupboard | StructureKind::Campfire) {
                BuildingTier::Wood
            } else {
                BuildingTier::Twig
            };
            let placed = PlacedStructure {
                id: self.next_structure_id,
                owner: actor,
                kind,
                tier,
                position: target_pos,
                rotation,
                health: tier.max_health(),
                max_health: tier.max_health(),
            };
            self.next_structure_id += 1;
            self.structures.push(placed);
            state.notice = Some(format!(
                "Збудовано: {} [{}] ({}, -{} Деревина)",
                kind.label(),
                tier.label(),
                socket_edge_label(rotation),
                cost
            ));
            return Ok(ActionOutcome::BuiltStructure(placed));
        }

        if active == Some(ItemKind::Hammer) {
            if let Some(vid) = self
                .targeted_vehicle(actor_position, yaw, pitch, ItemKind::Hammer.reach_meters())
                .filter(|v| v.health < v.max_health)
                .map(|v| v.id)
            {
                let v_idx = self.vehicles.iter().position(|v| v.id == vid).unwrap();
                let (mat_kind, cost) = if self.vehicles[v_idx].kind == VehicleKind::Bicycle {
                    (ItemKind::Wood, 10)
                } else {
                    (ItemKind::MetalFragments, 15)
                };
                let state = self.players.get_mut(&actor).unwrap();
                if !state.inventory.remove_item(mat_kind, cost) {
                    return Err("Недостатньо ресурсів для ремонту транспорту");
                }
                self.vehicles[v_idx].health =
                    (self.vehicles[v_idx].health + 80).min(self.vehicles[v_idx].max_health);
                let hp = self.vehicles[v_idx].health;
                let max_hp = self.vehicles[v_idx].max_health;
                let v_label = self.vehicles[v_idx].kind.label();
                state.notice = Some(format!("Відремонтовано {v_label} ({hp}/{max_hp} ХП)"));
                return Ok(ActionOutcome::SwungEmpty);
            }

            let Some(target_id) = self
                .targeted_structure(actor_position, yaw, pitch, ItemKind::Hammer.reach_meters())
                .map(|s| s.id)
            else {
                return Err("Наведіть киянку на споруду або пошкоджений транспорт для ремонту");
            };
            let structure = *self.structures.iter().find(|s| s.id == target_id).unwrap();
            if !self.has_building_privilege(actor, structure.position) {
                return Err("Чужа територія: потрібен доступ до Шафи (TC)");
            }
            if structure.health < structure.max_health {
                let state = self.players.get_mut(&actor).unwrap();
                if !state.inventory.remove_item(ItemKind::Wood, 10) {
                    return Err("Потрібно 10 деревини для ремонту");
                }
                let updated = self
                    .structures
                    .iter_mut()
                    .find(|s| s.id == target_id)
                    .unwrap();
                updated.health = updated.max_health;
                let copy = *updated;
                state.notice = Some(format!("Відремонтовано: {}", copy.kind.label()));
                return Ok(ActionOutcome::UpgradedStructure(copy));
            }
            let Some((next_tier, mat_kind, cost)) = structure.tier.next_upgrade() else {
                return Err("Споруда вже максимального рівня");
            };
            let state = self.players.get_mut(&actor).unwrap();
            if !state.inventory.remove_item(mat_kind, cost) {
                return Err("Недостатньо ресурсів для покращення споруди");
            }
            let updated = self
                .structures
                .iter_mut()
                .find(|s| s.id == target_id)
                .unwrap();
            updated.tier = next_tier;
            updated.max_health = next_tier.max_health();
            updated.health = updated.max_health;
            let copy = *updated;
            state.notice = Some(format!(
                "Покращено до [{}]: {} (-{} {})",
                next_tier.label(),
                copy.kind.label(),
                cost,
                mat_kind.label()
            ));
            return Ok(ActionOutcome::UpgradedStructure(copy));
        }

        // Phase 4 & 5: Hybrid ballistics for ranged weapons (HuntingBow, Revolver, AssaultRifle).
        if let Some(weapon) = active
            && let Some((ammo_kind, base_damage, muzzle_speed, max_range)) = weapon.ranged_spec()
        {
            let state = self.players.get_mut(&actor).unwrap();
            if !state.inventory.remove_item(ammo_kind, 1) {
                return Err("Немає набоїв або стріл у рюкзаку");
            }
            let start = [
                actor_position[0],
                actor_position[1] + EYE_OFFSET,
                actor_position[2],
            ];
            let dir = look_direction(yaw, pitch);
            let (end, hit_player, hit_npc, headshot) = self.trace_hybrid_ballistics(
                actor,
                start,
                dir,
                muzzle_speed,
                max_range,
                other_players,
            );
            let damage = if headshot {
                base_damage * 2.0
            } else {
                base_damage
            };
            if let Some(npc_id) = hit_npc {
                self.apply_npc_damage(actor, tick, npc_id, damage, headshot);
            } else if let Some(target_id) = hit_player
                && let Some(target_state) = self.players.get_mut(&target_id)
            {
                let defeated = target_state.vitals.apply_damage(damage);
                if defeated {
                    target_state.vitals = Vitals::default();
                    target_state.inventory = Inventory::starter();
                    target_state.mounted_vehicle = None;
                    target_state.notice = Some(format!("Вас ліквідовано з {}", weapon.label()));
                } else {
                    target_state.notice = Some(format!(
                        "Вогнепальне поранення з {} (-{damage:.0} ХП)",
                        weapon.label()
                    ));
                }
                if let Some(actor_state) = self.players.get_mut(&actor) {
                    actor_state.notice = Some(if headshot {
                        format!("ХЕДШОТ з {}! (-{damage:.0} ХП)", weapon.label())
                    } else {
                        format!("Влучання з {} (-{damage:.0} ХП)", weapon.label())
                    });
                }
            } else if let Some(actor_state) = self.players.get_mut(&actor) {
                actor_state.notice = Some(format!(
                    "Постріл: {} (-1 {})",
                    weapon.label(),
                    ammo_kind.label()
                ));
            }
            return Ok(ActionOutcome::FiredWeapon {
                weapon,
                start,
                end,
                hit_player,
                headshot,
                damage: if hit_player.is_some() || hit_npc.is_some() {
                    damage
                } else {
                    0.0
                },
            });
        }

        let tool = active.unwrap_or(ItemKind::Rock);
        let reach = tool.reach_meters();

        // Check if aiming at a resource node first.
        let hit_node_id = self
            .targeted_node(actor_position, yaw, pitch, reach)
            .map(|n| n.id);
        if let Some(node_id) = hit_node_id
            && let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id)
        {
            let harvest_amount = tool.harvest_yield(node.kind).min(node.health);
            if harvest_amount > 0 {
                node.health -= harvest_amount;
                if node.health == 0 {
                    node.respawn_at = Some(ServerTick(tick.0 + NODE_RESPAWN_TICKS));
                }
                let resource_item = node.kind.item();
                let node_health = node.health;
                let state = self.players.get_mut(&actor).unwrap();
                let leftover = state.inventory.add_item(resource_item, harvest_amount);
                let added = harvest_amount - leftover;
                state.notice = Some(format!("+{added} {}", resource_item.label()));
                return Ok(ActionOutcome::Harvested {
                    node_id,
                    item: resource_item,
                    amount: added,
                    node_health,
                });
            }
        }

        // Check melee hit against an NPC / Zombie / Boss.
        if let Some(npc_id) = self
            .targeted_npc(actor_position, yaw, pitch, reach + 0.5)
            .map(|n| n.id)
        {
            let damage = tool.melee_damage();
            let defeated = self.apply_npc_damage(actor, tick, npc_id, damage, false);
            return Ok(ActionOutcome::DamagedNpc {
                npc_id,
                damage,
                defeated,
            });
        }

        // Check melee hit against another player capsule AABB.
        let origin = [
            actor_position[0],
            actor_position[1] + EYE_OFFSET,
            actor_position[2],
        ];
        let dir = look_direction(yaw, pitch);
        let mut closest_player: Option<(f32, PlayerId)> = None;
        for &(other_id, pos) in other_players {
            if other_id == actor {
                continue;
            }
            let min = [pos[0] - 0.45, pos[1] - 0.92, pos[2] - 0.45];
            let max = [pos[0] + 0.45, pos[1] + 0.92, pos[2] + 0.45];
            if let Some(distance) = ray_aabb(origin, dir, min, max, reach)
                && closest_player.is_none_or(|(d, _)| distance < d)
            {
                closest_player = Some((distance, other_id));
            }
        }
        if let Some((_, target)) = closest_player {
            let damage = tool.melee_damage();
            if let Some(target_state) = self.players.get_mut(&target) {
                let defeated = target_state.vitals.apply_damage(damage);
                if defeated {
                    target_state.vitals = Vitals::default();
                    target_state.inventory = Inventory::starter();
                    target_state.mounted_vehicle = None;
                    target_state.notice = Some("Вас здолали в ближньому бою".into());
                } else {
                    target_state.notice = Some(format!("Отримано удар (-{damage:.0} ХП)"));
                }
                if let Some(actor_state) = self.players.get_mut(&actor) {
                    actor_state.notice = Some(format!("Влучання по гравцю (-{damage:.0} ХП)"));
                }
                return Ok(ActionOutcome::DamagedPlayer {
                    target,
                    damage,
                    defeated,
                });
            }
        }

        Ok(ActionOutcome::SwungEmpty)
    }

    /// Hybrid ballistics: instant hitscan up to 25m, then stepped ballistic trajectory
    /// with gravity drop and aerodynamic drag up to `max_range`.
    #[allow(clippy::type_complexity)]
    fn trace_hybrid_ballistics(
        &self,
        actor: PlayerId,
        start: [f32; 3],
        dir: [f32; 3],
        muzzle_speed: f32,
        max_range: f32,
        other_players: &[(PlayerId, [f32; 3])],
    ) -> ([f32; 3], Option<PlayerId>, Option<u16>, bool) {
        let hitscan_dist = 25.0_f32.min(max_range);
        let player_hit =
            self.raycast_segment_players(actor, start, dir, hitscan_dist, other_players);
        let npc_hit = self.raycast_segment_npcs(start, dir, hitscan_dist);
        match (player_hit, npc_hit) {
            (Some((pd, target, phs)), Some((nd, npc_id, nhs))) => {
                if nd <= pd {
                    return (
                        [
                            start[0] + dir[0] * nd,
                            start[1] + dir[1] * nd,
                            start[2] + dir[2] * nd,
                        ],
                        None,
                        Some(npc_id),
                        nhs,
                    );
                } else {
                    return (
                        [
                            start[0] + dir[0] * pd,
                            start[1] + dir[1] * pd,
                            start[2] + dir[2] * pd,
                        ],
                        Some(target),
                        None,
                        phs,
                    );
                }
            }
            (Some((pd, target, phs)), None) => {
                return (
                    [
                        start[0] + dir[0] * pd,
                        start[1] + dir[1] * pd,
                        start[2] + dir[2] * pd,
                    ],
                    Some(target),
                    None,
                    phs,
                );
            }
            (None, Some((nd, npc_id, nhs))) => {
                return (
                    [
                        start[0] + dir[0] * nd,
                        start[1] + dir[1] * nd,
                        start[2] + dir[2] * nd,
                    ],
                    None,
                    Some(npc_id),
                    nhs,
                );
            }
            (None, None) => {}
        }

        let mut pos = [
            start[0] + dir[0] * hitscan_dist,
            start[1] + dir[1] * hitscan_dist,
            start[2] + dir[2] * hitscan_dist,
        ];
        if pos[1] <= 0.05 {
            pos[1] = 0.05;
            return (pos, None, None, false);
        }
        let mut vel = [
            dir[0] * muzzle_speed,
            dir[1] * muzzle_speed,
            dir[2] * muzzle_speed,
        ];
        let mut traveled = hitscan_dist;
        let dt = 1.0 / 120.0;
        while traveled < max_range && pos[1] > 0.05 {
            let next = [
                pos[0] + vel[0] * dt,
                pos[1] + vel[1] * dt,
                pos[2] + vel[2] * dt,
            ];
            let seg = [next[0] - pos[0], next[1] - pos[1], next[2] - pos[2]];
            let seg_len = (seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2]).sqrt();
            if seg_len > 1e-5 {
                let seg_dir = [seg[0] / seg_len, seg[1] / seg_len, seg[2] / seg_len];
                if let Some((nd, npc_id, nhs)) = self.raycast_segment_npcs(pos, seg_dir, seg_len) {
                    return (
                        [
                            pos[0] + seg_dir[0] * nd,
                            pos[1] + seg_dir[1] * nd,
                            pos[2] + seg_dir[2] * nd,
                        ],
                        None,
                        Some(npc_id),
                        nhs,
                    );
                }
                if let Some((dist, target, headshot)) =
                    self.raycast_segment_players(actor, pos, seg_dir, seg_len, other_players)
                {
                    return (
                        [
                            pos[0] + seg_dir[0] * dist,
                            pos[1] + seg_dir[1] * dist,
                            pos[2] + seg_dir[2] * dist,
                        ],
                        Some(target),
                        None,
                        headshot,
                    );
                }
            }
            pos = next;
            traveled += seg_len;
            // Gravity + air drag
            vel[1] -= 9.81 * dt;
            let drag = (1.0 - 0.08 * dt).max(0.0);
            vel[0] *= drag;
            vel[1] *= drag;
            vel[2] *= drag;
        }
        pos[1] = pos[1].max(0.05);
        (pos, None, None, false)
    }

    fn raycast_segment_npcs(
        &self,
        origin: [f32; 3],
        dir: [f32; 3],
        max_dist: f32,
    ) -> Option<(f32, u16, bool)> {
        let mut best: Option<(f32, u16, bool)> = None;
        for npc in &self.npcs {
            if !npc.is_alive() {
                continue;
            }
            let pos = npc.position;
            if npc.kind.is_humanoid() {
                let head_min = [pos[0] - 0.34, pos[1] + 0.35, pos[2] - 0.34];
                let head_max = [pos[0] + 0.34, pos[1] + 0.96, pos[2] + 0.34];
                if let Some(d) = ray_aabb(origin, dir, head_min, head_max, max_dist)
                    && best.is_none_or(|(best_d, _, _)| d < best_d)
                {
                    best = Some((d, npc.id, true));
                    continue;
                }
            }
            let ext = npc.kind.half_extents();
            let body_min = [pos[0] - ext[0], pos[1] - ext[1], pos[2] - ext[2]];
            let body_max = [pos[0] + ext[0], pos[1] + ext[1], pos[2] + ext[2]];
            if let Some(d) = ray_aabb(origin, dir, body_min, body_max, max_dist)
                && best.is_none_or(|(best_d, _, _)| d < best_d)
            {
                best = Some((d, npc.id, false));
            }
        }
        best
    }

    fn raycast_segment_players(
        &self,
        actor: PlayerId,
        origin: [f32; 3],
        dir: [f32; 3],
        max_dist: f32,
        other_players: &[(PlayerId, [f32; 3])],
    ) -> Option<(f32, PlayerId, bool)> {
        let mut best: Option<(f32, PlayerId, bool)> = None;
        for &(other_id, pos) in other_players {
            if other_id == actor {
                continue;
            }
            let head_min = [pos[0] - 0.32, pos[1] + 0.35, pos[2] - 0.32];
            let head_max = [pos[0] + 0.32, pos[1] + 0.95, pos[2] + 0.32];
            if let Some(d) = ray_aabb(origin, dir, head_min, head_max, max_dist)
                && best.is_none_or(|(best_d, _, _)| d < best_d)
            {
                best = Some((d, other_id, true));
                continue;
            }
            let body_min = [pos[0] - 0.45, pos[1] - 0.92, pos[2] - 0.45];
            let body_max = [pos[0] + 0.45, pos[1] + 0.35, pos[2] + 0.45];
            if let Some(d) = ray_aabb(origin, dir, body_min, body_max, max_dist)
                && best.is_none_or(|(best_d, _, _)| d < best_d)
            {
                best = Some((d, other_id, false));
            }
        }
        best
    }
}

pub fn look_direction(yaw: f32, pitch: f32) -> [f32; 3] {
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    [-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch]
}

fn ray_aabb(
    origin: [f32; 3],
    dir: [f32; 3],
    min: [f32; 3],
    max: [f32; 3],
    max_distance: f32,
) -> Option<f32> {
    let mut tmin = 0.0_f32;
    let mut tmax = max_distance;
    for axis in 0..3 {
        if dir[axis].abs() < 1e-6 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
        } else {
            let inv = 1.0 / dir[axis];
            let mut t1 = (min[axis] - origin[axis]) * inv;
            let mut t2 = (max[axis] - origin[axis]) * inv;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return None;
            }
        }
    }
    Some(tmin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_inventory_stacks_crafts_and_swaps_slots() {
        let mut inv = Inventory::starter();
        assert_eq!(inv.active_item().unwrap().kind, ItemKind::Rock);
        assert_eq!(inv.add_item(ItemKind::Wood, 120), 0);
        assert_eq!(inv.add_item(ItemKind::Stone, 60), 0);
        assert!(inv.can_craft(RecipeId::StoneHatchet));
        assert_eq!(
            inv.craft(RecipeId::StoneHatchet).unwrap(),
            ItemKind::StoneHatchet
        );
        assert_eq!(inv.count_item(ItemKind::Wood), 60);
        assert_eq!(inv.count_item(ItemKind::Stone), 30);
        assert_eq!(inv.count_item(ItemKind::StoneHatchet), 1);

        // Move crafted hatchet from Backpack(2) to Hotbar(0) (swapping with Rock).
        assert!(inv.move_slot(SlotIndex::Backpack(2), SlotIndex::Hotbar(0)));
        assert_eq!(inv.active_item().unwrap().kind, ItemKind::StoneHatchet);
    }

    #[test]
    fn building_plan_places_structures_hammer_upgrades_and_tc_protects_territory() {
        let mut world = SurvivalWorld::new(MapId::Polygon);
        world.join(PlayerId(1));
        world.join(PlayerId(2));
        world
            .player_mut(PlayerId(1))
            .unwrap()
            .inventory
            .add_item(ItemKind::Wood, 200);
        world
            .player_mut(PlayerId(1))
            .unwrap()
            .inventory
            .select_slot(4); // BuildingPlan
        let built = world
            .primary_action(
                PlayerId(1),
                ServerTick(50),
                [0.0, 0.92, 12.0],
                0.0,
                0.0,
                &[],
            )
            .unwrap();
        let ActionOutcome::BuiltStructure(foundation) = built else {
            panic!("expected built structure");
        };
        assert_eq!(foundation.kind, StructureKind::Foundation);
        assert_eq!(foundation.tier, BuildingTier::Twig);

        // Upgrade foundation from Twig to Wood using Hammer (slot 5).
        world
            .player_mut(PlayerId(1))
            .unwrap()
            .inventory
            .select_slot(5);
        let upgraded = world
            .primary_action(
                PlayerId(1),
                ServerTick(100),
                [0.0, 0.92, 12.0],
                0.0,
                -0.5,
                &[],
            )
            .unwrap();
        let ActionOutcome::UpgradedStructure(upgraded_foundation) = upgraded else {
            panic!("expected upgraded structure");
        };
        assert_eq!(upgraded_foundation.tier, BuildingTier::Wood);

        // Place a Tool Cupboard (TC) owned by PlayerId(1) and verify PlayerId(2) is blocked within 30m.
        let p1 = world.player_mut(PlayerId(1)).unwrap();
        p1.inventory.select_slot(4);
        p1.selected_structure = StructureKind::ToolCupboard;
        world
            .primary_action(
                PlayerId(1),
                ServerTick(150),
                [3.0, 0.92, 12.0],
                0.0,
                0.0,
                &[],
            )
            .unwrap();
        assert!(world.has_building_privilege(PlayerId(1), [4.0, 0.92, 12.0]));
        assert!(!world.has_building_privilege(PlayerId(2), [4.0, 0.92, 12.0]));
    }

    #[test]
    fn biomes_weather_day_night_and_campfire_regulate_body_temperature() {
        let mut world = SurvivalWorld::new(MapId::Polygon);
        world.join(PlayerId(1));
        assert_eq!(Biome::at([0.0, 0.0, 0.0]), Biome::Temperate);
        assert_eq!(Biome::at([0.0, 0.0, -1500.0]), Biome::Winter);
        assert_eq!(Biome::at([0.0, 0.0, 1500.0]), Biome::Desert);
        assert_eq!(Biome::at([2000.0, 0.0, 0.0]), Biome::Jungle);

        let winter_pos = [0.0, 1.0, -1500.0];
        let env_cold = world.environment_at(ServerTick(15), winter_pos);
        assert_eq!(env_cold.biome, Biome::Winter);
        assert!(env_cold.ambient_temp_c < 0.0);
        assert!(!env_cold.near_campfire);

        for step in 1..=8 {
            world.step_with_positions(ServerTick(step * 15), [(PlayerId(1), false, winter_pos)]);
        }
        let cold_body_temp = world.player(PlayerId(1)).unwrap().vitals.body_temp_c;
        assert!(cold_body_temp < NORMAL_BODY_TEMP_C);

        // Build a Campfire next to the player in the Winter biome and verify it warms them up.
        let p1 = world.player_mut(PlayerId(1)).unwrap();
        p1.inventory.add_item(ItemKind::Wood, 60);
        p1.inventory.select_slot(4);
        p1.selected_structure = StructureKind::Campfire;
        world
            .primary_action(PlayerId(1), ServerTick(200), winter_pos, 0.0, 0.0, &[])
            .unwrap();

        let env_warmed = world.environment_at(ServerTick(210), winter_pos);
        assert!(env_warmed.near_campfire);
        assert!(env_warmed.ambient_temp_c > env_cold.ambient_temp_c + 15.0);

        for step in 15..=30 {
            world.step_with_positions(ServerTick(step * 15), [(PlayerId(1), false, winter_pos)]);
        }
        let warmed_player = world.player(PlayerId(1)).unwrap();
        assert!(warmed_player.vitals.body_temp_c > cold_body_temp);
        assert!((warmed_player.vitals.comfort - 100.0).abs() < 1e-4);
    }

    #[test]
    fn harvesting_tree_and_rock_yields_resources_and_schedules_respawn() {
        let mut world = SurvivalWorld::new(MapId::Polygon);
        world.join(PlayerId(1));
        let tree = world.nodes[0];
        assert_eq!(tree.kind, ResourceKind::Tree);
        let player_pos = [tree.position[0], 0.92, tree.position[2] + 1.8];
        let outcome = world
            .primary_action(PlayerId(1), ServerTick(40), player_pos, 0.0, 0.0, &[])
            .unwrap();
        assert_eq!(
            outcome,
            ActionOutcome::Harvested {
                node_id: tree.id,
                item: ItemKind::Wood,
                amount: 10,
                node_health: tree.max_health - 10,
            }
        );
        assert_eq!(world.player(PlayerId(1)).unwrap()._inventory_wood(), 10);
    }

    #[test]
    fn consumables_restore_vitals_and_low_hydration_blocks_sprint() {
        let mut world = SurvivalWorld::new(MapId::Polygon);
        let state = world.join(PlayerId(1));
        assert!(state.vitals.can_sprint());
        let state = world.player_mut(PlayerId(1)).unwrap();
        state.vitals.health = 60.0;
        state.vitals.hydration = 30.0;
        state.inventory.select_slot(1); // Bandage
        assert!(!world.can_sprint(PlayerId(1)));
        world
            .primary_action(PlayerId(1), ServerTick(50), [0.0, 1.0, 0.0], 0.0, 0.0, &[])
            .unwrap();
        assert!((world.player(PlayerId(1)).unwrap().vitals.health - 75.0).abs() < 1e-4);

        world
            .player_mut(PlayerId(1))
            .unwrap()
            .inventory
            .select_slot(2); // WaterBottle
        world
            .primary_action(
                PlayerId(1),
                ServerTick(50 + ACTION_COOLDOWN_TICKS),
                [0.0, 1.0, 0.0],
                0.0,
                0.0,
                &[],
            )
            .unwrap();
        assert!(world.can_sprint(PlayerId(1)));
    }

    #[test]
    fn wall_socket_snapping_and_90_degree_rotation_across_foundation_edges() {
        let mut world = SurvivalWorld::new(MapId::Polygon);
        world.join(PlayerId(1));
        {
            let p1 = world.player_mut(PlayerId(1)).unwrap();
            p1.inventory.add_item(ItemKind::Wood, 200);
            p1.inventory.select_slot(4); // BuildingPlan
            p1.selected_structure = StructureKind::Foundation;
        }
        let ActionOutcome::BuiltStructure(foundation) = world
            .primary_action(
                PlayerId(1),
                ServerTick(20),
                [0.0, 0.92, 12.0],
                0.0,
                0.0,
                &[],
            )
            .unwrap()
        else {
            panic!("expected foundation");
        };
        assert_eq!(foundation.position, [0.0, 0.10, 9.0]);

        // Select Wall and rotate through all 4 socket edges (0 = North, 1 = East, 2 = South, 3 = West).
        world.player_mut(PlayerId(1)).unwrap().selected_structure = StructureKind::Wall;
        let (pos_n, rot_n) = preview_socket_placement(
            [0.0, 0.92, 12.0],
            0.0,
            0.0,
            StructureKind::Wall,
            0,
            &world.structures,
        );
        assert_eq!(rot_n, 0);
        assert_eq!(pos_n, [0.0, 1.45, 9.0 - 1.35]);

        let rot_1 = world.player_mut(PlayerId(1)).unwrap().rotate_structure();
        assert_eq!(rot_1, 1);
        let (pos_e, rot_e) = preview_socket_placement(
            [0.0, 0.92, 12.0],
            0.0,
            0.0,
            StructureKind::Wall,
            rot_1,
            &world.structures,
        );
        assert_eq!(rot_e, 1);
        assert_eq!(pos_e, [0.0 + 1.35, 1.45, 9.0]);
        assert_eq!(
            StructureKind::Wall.oriented_half_extents(rot_e),
            [0.15, 1.25, 1.5]
        );

        let ActionOutcome::BuiltStructure(east_wall) = world
            .primary_action(
                PlayerId(1),
                ServerTick(60),
                [0.0, 0.92, 12.0],
                0.0,
                0.0,
                &[],
            )
            .unwrap()
        else {
            panic!("expected east wall");
        };
        assert_eq!(east_wall.rotation, 1);
        assert_eq!(east_wall.position, [1.35, 1.45, 9.0]);
    }

    #[test]
    fn radtown_radiation_fuse_keycard_puzzle_and_recycler_workflow() {
        let mut world = SurvivalWorld::new(MapId::Polygon);
        world.join(PlayerId(1));

        // 1. Loot Supply Crate outside the bunker -> receive Fuse, GreenKeycard, AntiRadPills, Scrap, Metal
        let supply_pos = [
            RADTOWN_SUPPLY_CRATE_POS[0],
            0.92,
            RADTOWN_SUPPLY_CRATE_POS[2] + 1.2,
        ];
        world
            .primary_action(PlayerId(1), ServerTick(30), supply_pos, 0.0, -0.3, &[])
            .unwrap();
        assert_eq!(
            world
                .player(PlayerId(1))
                .unwrap()
                .inventory
                .count_item(ItemKind::ElectricFuse),
            1
        );
        assert_eq!(
            world
                .player(PlayerId(1))
                .unwrap()
                .inventory
                .count_item(ItemKind::GreenKeycard),
            1
        );

        // 2. Insert Fuse & turn on generator switch at FuseBox
        let fusebox_pos = [RADTOWN_FUSEBOX_POS[0], 0.92, RADTOWN_FUSEBOX_POS[2] + 1.1];
        world
            .primary_action(PlayerId(1), ServerTick(70), fusebox_pos, 0.0, 0.0, &[])
            .unwrap();
        assert!(world.monument.is_powered());

        // 3. Swipe GreenKeycard at CardReader -> opens bunker door
        let reader_pos = [RADTOWN_READER_POS[0], 0.92, RADTOWN_READER_POS[2] + 1.1];
        world
            .primary_action(PlayerId(1), ServerTick(110), reader_pos, 0.0, 0.0, &[])
            .unwrap();
        assert!(world.monument.is_door_open());

        // 4. Loot MilitaryCrate inside the bunker -> receive AssaultRifle + Revolver + Ammo
        let mil_pos = [
            RADTOWN_MILITARY_CRATE_POS[0],
            0.92,
            RADTOWN_MILITARY_CRATE_POS[2] + 1.1,
        ];
        world
            .primary_action(PlayerId(1), ServerTick(150), mil_pos, 0.0, -0.3, &[])
            .unwrap();
        assert_eq!(
            world
                .player(PlayerId(1))
                .unwrap()
                .inventory
                .count_item(ItemKind::AssaultRifle),
            1
        );
        assert_eq!(
            world
                .player(PlayerId(1))
                .unwrap()
                .inventory
                .count_item(ItemKind::RifleAmmo),
            30
        );
    }

    #[test]
    fn hybrid_ballistics_ranged_weapons_consume_ammo_and_score_headshot_damage() {
        let mut world = SurvivalWorld::new(MapId::Polygon);
        world.join(PlayerId(1));
        world.join(PlayerId(2));
        {
            let p1 = world.player_mut(PlayerId(1)).unwrap();
            p1.inventory.hotbar[0] = ItemStack::new(ItemKind::AssaultRifle, 1);
            p1.inventory.add_item(ItemKind::RifleAmmo, 10);
            p1.inventory.select_slot(0);
        }
        // Target PlayerId(2) 18 meters ahead (-Z), aiming slightly up at head height
        let target_pos = [0.0, 0.92, -18.0];
        let outcome = world
            .primary_action(
                PlayerId(1),
                ServerTick(50),
                [0.0, 0.92, 0.0],
                0.0,
                0.0,
                &[(PlayerId(2), target_pos)],
            )
            .unwrap();
        let ActionOutcome::FiredWeapon {
            weapon,
            hit_player,
            headshot,
            damage,
            ..
        } = outcome
        else {
            panic!("expected FiredWeapon");
        };
        assert_eq!(weapon, ItemKind::AssaultRifle);
        assert_eq!(hit_player, Some(PlayerId(2)));
        assert!(headshot);
        assert!((damage - 92.0).abs() < 1e-4);
        assert_eq!(
            world
                .player(PlayerId(1))
                .unwrap()
                .inventory
                .count_item(ItemKind::RifleAmmo),
            9
        );
    }

    #[test]
    fn phase5_vehicles_mount_drive_refuel_and_patrol_heli_aggression_rule() {
        let mut world = SurvivalWorld::new(MapId::Polygon);
        world.join(PlayerId(1));
        world.join(PlayerId(2));

        // 1. Starter player (Rock, Bandage, BuildingPlan, Hammer) does NOT trigger PatrolHeli.
        assert!(!player_triggers_patrol_heli(
            &world.player(PlayerId(1)).unwrap().inventory
        ));
        // Equipping a HuntingBow (primitive weapon) also does NOT trigger PatrolHeli.
        world.player_mut(PlayerId(1)).unwrap().inventory.hotbar[0] =
            ItemStack::new(ItemKind::HuntingBow, 1);
        assert!(!player_triggers_patrol_heli(
            &world.player(PlayerId(1)).unwrap().inventory
        ));
        // Equipping an AssaultRifle or Revolver in hotbar DOES trigger PatrolHeli.
        world.player_mut(PlayerId(2)).unwrap().inventory.hotbar[0] =
            ItemStack::new(ItemKind::AssaultRifle, 1);
        assert!(player_triggers_patrol_heli(
            &world.player(PlayerId(2)).unwrap().inventory
        ));

        // 2. Step AI at tick 60 with both players under the PatrolHeli orbit in the southern/eastern clear zone:
        // PlayerId(1) (Bow only) is ignored (100 HP), while PlayerId(2) (AssaultRifle) is attacked!
        world.step_with_positions(
            ServerTick(60),
            [
                (PlayerId(1), false, [14.0, 0.92, 8.0]),
                (PlayerId(2), false, [15.0, 0.92, 8.0]),
            ],
        );
        assert!(
            (world.player(PlayerId(1)).unwrap().vitals.health - MAX_HEALTH).abs() < 1e-3,
            "primitive player should be ignored by PatrolHeli"
        );
        assert!(
            world.player(PlayerId(2)).unwrap().vitals.health < MAX_HEALTH - 10.0,
            "firearm-carrying player should be attacked by PatrolHeli"
        );

        // 3. Mount the DuneBuggy near [7.2, 0.0, 13.5], drive it, and verify position & fuel sync.
        let buggy_pos = world
            .vehicles
            .iter()
            .find(|v| v.kind == VehicleKind::DuneBuggy)
            .unwrap()
            .position;
        let mounted_id = world
            .toggle_mount_vehicle(
                PlayerId(1),
                [buggy_pos[0], 0.92, buggy_pos[2] + 1.2],
                0.0,
                0.0,
            )
            .unwrap();
        assert!(mounted_id.is_some());
        let initial_fuel = world
            .vehicles
            .iter()
            .find(|v| v.id == mounted_id.unwrap())
            .unwrap()
            .fuel;
        world.step_with_positions(ServerTick(120), [(PlayerId(1), true, [10.0, 0.92, 14.0])]);
        let updated_buggy = world
            .vehicles
            .iter()
            .find(|v| v.id == mounted_id.unwrap())
            .unwrap();
        assert_eq!(updated_buggy.position, [10.0, 0.0, 14.0]);
        assert_eq!(updated_buggy.fuel, initial_fuel - 1);
    }

    #[test]
    fn phase5_c4_destroys_bradley_apc_and_nuclear_strike_shockwave_levels_twig_structures() {
        let mut world = SurvivalWorld::new(MapId::Polygon);
        world.join(PlayerId(1));

        // 1. Destroy Bradley APC using 2 C4Explosives (600 HP total, 320 damage per C4)
        let bradley_pos = world
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::BradleyApc)
            .unwrap()
            .position;
        {
            let p1 = world.player_mut(PlayerId(1)).unwrap();
            p1.inventory.hotbar[0] = ItemStack::new(ItemKind::C4Explosive, 2);
            p1.inventory.select_slot(0);
        }
        let actor_pos = [bradley_pos[0], 0.92, bradley_pos[2] + 3.2];
        let first_hit = world
            .primary_action(PlayerId(1), ServerTick(40), actor_pos, 0.0, 0.0, &[])
            .unwrap();
        assert!(matches!(
            first_hit,
            ActionOutcome::DamagedNpc {
                defeated: false,
                ..
            }
        ));
        let second_hit = world
            .primary_action(PlayerId(1), ServerTick(90), actor_pos, 0.0, 0.0, &[])
            .unwrap();
        assert!(matches!(
            second_hit,
            ActionOutcome::DamagedNpc { defeated: true, .. }
        ));
        // Defeating Bradley APC awards boss loot (+120 Scrap, +200 MetalFragments)
        assert!(
            world
                .player(PlayerId(1))
                .unwrap()
                .inventory
                .count_item(ItemKind::Scrap)
                >= 120
        );

        // 2. Place a Twig Foundation inside the Nuclear Strike blast zone ([8.0, 0.0, 14.0])
        world.structures.push(PlacedStructure {
            id: 99,
            owner: PlayerId(1),
            kind: StructureKind::Foundation,
            tier: BuildingTier::Twig,
            position: [8.0, 0.10, 14.0],
            rotation: 0,
            health: 50,
            max_health: 50,
        });

        // 3. Open Radtown bunker and press NuclearSiloButton -> launches SirenWarning
        world.monument.door_open_ticks = 600;
        let silo_pos = [
            RADTOWN_NUCLEAR_BUTTON_POS[0],
            0.92,
            RADTOWN_NUCLEAR_BUTTON_POS[2] + 1.0,
        ];
        world
            .primary_action(PlayerId(1), ServerTick(150), silo_pos, 0.0, 0.0, &[])
            .unwrap();
        assert_eq!(world.nuclear.phase, NuclearPhase::SirenWarning);

        // Advance to DetonationShockwave transition and verify the Twig structure in the blast zone is obliterated
        world.nuclear.ticks_remaining = 1;
        world.step_with_positions(
            ServerTick(200),
            [(PlayerId(1), false, NUCLEAR_STRIKE_CENTER)],
        );
        assert_eq!(world.nuclear.phase, NuclearPhase::DetonationShockwave);
        assert!(world.structures.iter().all(|s| s.id != 99));
        assert!(world.player(PlayerId(1)).unwrap().vitals.radiation >= 35.0);
    }

    impl PlayerSurvival {
        fn _inventory_wood(&self) -> u32 {
            self.inventory.count_item(ItemKind::Wood)
        }
    }
}
