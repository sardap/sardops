use bincode::{Decode, Encode};
use const_for::const_for;
use glam::usize;
use strum_macros::{EnumCount, EnumIter, FromRepr};

use crate::{
    food::STARTING_FOOD,
    game_context::GameContext,
    scene::{fishing_scene, SceneEnum},
};

include!(concat!(env!("OUT_DIR"), "/dist_items.rs"));

pub const ITEM_COUNT: usize = core::mem::variant_count::<ItemKind>();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRarity {
    None,
    Common,
    Rare,
}

impl ItemKind {
    pub const fn is_none(&self) -> bool {
        matches!(self, ItemKind::None)
    }

    pub const fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_usable(&self) -> bool {
        ALL_USEABLE_ITEMS.iter().any(|i| *self == i.item)
    }

    pub fn use_item(&self, game_ctx: &mut GameContext) -> Option<UseItemOutput> {
        for usable in ALL_USEABLE_ITEMS {
            if usable.item == *self {
                return Some(usable.use_item(game_ctx));
            }
        }

        None
    }
}

impl Default for ItemKind {
    fn default() -> Self {
        Self::None
    }
}

// Can use casey but can't figure it out
macro_rules! items_for_rarity {
    ($name_lower:tt, $name_upper:tt, $variant:ident) => {
        paste::paste! {
            const fn [<count_items_for_rarity_ $name_lower >]() -> usize {
                let mut result = 0;
                const_for!(i in 0..ITEM_COUNT => {
                    let item = ItemKind::from_repr(i).unwrap();
                    if matches!(item.rarity(), ItemRarity::$variant) {
                        result += 1;
                    }
                });
                result
            }

            const fn [< items_for_rarity_ $name_lower >]<const T: usize>() -> [ItemKind; T] {
                let mut result = [ItemKind::None; T];
                let mut top = 0;
                const_for!(i in 0..ITEM_COUNT => {
                    let item = ItemKind::from_repr(i).unwrap();
                    if matches!(item.rarity(), ItemRarity::$variant) {
                        result[top] = item;
                        top += 1;
                    }
                });
                result
            }

            const [< $name_upper _ITEM_COUNT>]: usize = [< count_items_for_rarity_ $name_lower>]();
            pub const [< $name_upper _ITEMS>]: [ItemKind; [< $name_upper _ITEM_COUNT>]] = [< items_for_rarity_ $name_lower>]();
        }
    };
}

items_for_rarity!("common", "COMMON", Common);
items_for_rarity!("rare", "RARE", Rare);

const fn all_items_gen() -> [ItemKind; ITEM_COUNT] {
    let mut result = [ItemKind::None; ITEM_COUNT];
    const_for!(i in 0..ITEM_COUNT => {
        let item = ItemKind::from_repr(i).unwrap();
        result[i] = item;
    });
    result
}

#[allow(dead_code)]
pub const ALL_ITEMS: [ItemKind; ITEM_COUNT] = all_items_gen();

pub const MAX_OWNED: i32 = 1000000;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct ItemExtra {
    uses: i32,
}

impl ItemExtra {
    pub const fn new() -> Self {
        Self { uses: 0 }
    }

    pub const fn with_uses(mut self, uses: i32) -> Self {
        self.uses = uses;
        self
    }

    pub const fn new_from_kind(kind: ItemKind) -> Self {
        match kind {
            ItemKind::FishingRod => Self::new().with_uses(5),
            _ => Self::new(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct InventoryEntry {
    pub owned: u32,
    pub item_extra: ItemExtra,
}

impl Default for InventoryEntry {
    fn default() -> Self {
        Self {
            owned: 0,
            item_extra: ItemExtra::new(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct Inventory {
    contents: [InventoryEntry; ITEM_COUNT],
}

impl Inventory {
    pub fn item_count(&self, item: ItemKind) -> u32 {
        self.contents[item as usize].owned
    }

    pub fn has_item(&self, item: ItemKind) -> bool {
        item != ItemKind::None && self.item_count(item) > 0
    }

    pub fn get_entry_mut(&mut self, item: ItemKind) -> &mut InventoryEntry {
        &mut self.contents[item as usize]
    }

    pub fn add_item(&mut self, item: ItemKind, qty: i32) {
        let entry = self.get_entry_mut(item);
        if entry.owned <= 0 && qty > 0 {
            entry.item_extra = ItemExtra::new_from_kind(item);
        }
        let mut updated = entry.owned as i32 + qty;
        if updated > MAX_OWNED {
            updated = MAX_OWNED
        } else if updated < 0 {
            updated = 0;
        }
        entry.owned = updated as u32;
    }
}

impl Default for Inventory {
    fn default() -> Self {
        let mut result = Self {
            contents: Default::default(),
        };

        for food in STARTING_FOOD {
            result.add_item(ItemKind::from_food(food.id), 1);
        }

        result
    }
}

pub struct UseItemOutput {
    pub new_scene: Option<SceneEnum>,
    pub consumed: bool,
}

impl UseItemOutput {
    pub const fn new() -> Self {
        Self {
            new_scene: None,
            consumed: false,
        }
    }

    pub const fn with_consumed(mut self) -> Self {
        self.consumed = true;
        self
    }

    pub const fn with_scene(mut self, scene: SceneEnum) -> Self {
        self.new_scene = Some(scene);
        self
    }
}

impl Default for UseItemOutput {
    fn default() -> Self {
        Self {
            new_scene: Default::default(),
            consumed: false,
        }
    }
}

pub type UseItemFn = fn(game_ctx: &mut GameContext) -> UseItemOutput;

pub struct UsableItem {
    item: ItemKind,
    function: UseItemFn,
}

impl UsableItem {
    pub fn use_item(&self, game_ctx: &mut GameContext) -> UseItemOutput {
        let output = (self.function)(game_ctx);

        if output.consumed {
            game_ctx.inventory.add_item(self.item, -1);
        }

        output
    }
}

const USE_SHOP_UPGRADE: UsableItem = UsableItem {
    item: ItemKind::ShopUpgrade,
    function: |game_ctx| {
        game_ctx
            .shop
            .set_item_count(game_ctx.shop.get_item_count() + 1);

        UseItemOutput::new().with_consumed()
    },
};

const USE_FISHING_ROD: UsableItem = UsableItem {
    item: ItemKind::FishingRod,
    function: |game_ctx| {
        let mut result =
            UseItemOutput::new().with_scene(SceneEnum::Fishing(fishing_scene::FishingScene::new()));
        let entry = game_ctx.inventory.get_entry_mut(ItemKind::FishingRod);
        entry.item_extra.uses -= 1;
        if entry.item_extra.uses <= 0 {
            entry.item_extra = ItemExtra::new_from_kind(ItemKind::FishingRod);
            result = result.with_consumed();
        }

        result
    },
};

const ALL_USEABLE_ITEMS: &[UsableItem] = &[USE_SHOP_UPGRADE, USE_FISHING_ROD];

pub struct ItemChance {
    kind: ItemKind,
    odds: f32,
}

impl ItemChance {
    pub const fn new(kind: ItemKind, odds: f32) -> Self {
        ItemChance { kind, odds }
    }
}

pub fn pick_item_from_set(val: f32, chance_set: &[ItemChance]) -> ItemKind {
    if chance_set.len() == 0 {
        return ItemKind::None;
    }

    for entry in chance_set {
        if val < entry.odds {
            return entry.kind;
        }
    }

    ItemKind::None
}
