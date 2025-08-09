use bincode::{Decode, Encode};
use const_for::const_for;
use glam::usize;
use strum_macros::{EnumCount, EnumIter, FromRepr};

use crate::food::STARTING_FOOD;

include!(concat!(env!("OUT_DIR"), "/dist_items.rs"));

pub const ITEM_COUNT: usize = core::mem::variant_count::<Item>();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRarity {
    None,
    Common,
    Rare,
}

impl Item {
    pub const fn is_none(&self) -> bool {
        matches!(self, Item::None)
    }

    pub const fn is_some(&self) -> bool {
        !self.is_none()
    }
}

impl Default for Item {
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
                    let item = Item::from_repr(i).unwrap();
                    if matches!(item.rarity(), ItemRarity::$variant) {
                        result += 1;
                    }
                });
                result
            }

            const fn [< items_for_rarity_ $name_lower >]<const T: usize>() -> [Item; T] {
                let mut result = [Item::None; T];
                let mut top = 0;
                const_for!(i in 0..ITEM_COUNT => {
                    let item = Item::from_repr(i).unwrap();
                    if matches!(item.rarity(), ItemRarity::$variant) {
                        result[top] = item;
                        top += 1;
                    }
                });
                result
            }

            const [< $name_upper _ITEM_COUNT>]: usize = [< count_items_for_rarity_ $name_lower>]();
            pub const [< $name_upper _ITEMS>]: [Item; [< $name_upper _ITEM_COUNT>]] = [< items_for_rarity_ $name_lower>]();
        }
    };
}

items_for_rarity!("common", "COMMON", Common);
items_for_rarity!("rare", "RARE", Rare);

const fn all_items_gen() -> [Item; ITEM_COUNT] {
    let mut result = [Item::None; ITEM_COUNT];
    const_for!(i in 0..ITEM_COUNT => {
        let item = Item::from_repr(i).unwrap();
        result[i] = item;
    });
    result
}

pub const ALL_ITEMS: [Item; ITEM_COUNT] = all_items_gen();

pub const MAX_OWNED: i32 = 1000000;

#[derive(Clone, Copy, Encode, Decode)]
pub struct InventoryEntry {
    pub owned: u32,
}

impl Default for InventoryEntry {
    fn default() -> Self {
        Self { owned: 0 }
    }
}

#[derive(Clone, Copy, Encode, Decode)]
pub struct Inventory {
    contents: [InventoryEntry; ITEM_COUNT],
}

impl Inventory {
    pub fn item_count(&self, item: Item) -> u32 {
        self.contents[item as usize].owned
    }

    pub fn has_item(&self, item: Item) -> bool {
        self.item_count(item) > 0
    }

    pub fn get_entry_mut(&mut self, item: Item) -> &mut InventoryEntry {
        &mut self.contents[item as usize]
    }

    pub fn add_item(&mut self, item: Item, qty: i32) {
        let entry = self.get_entry_mut(item);
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
            result.add_item(Item::from_food(food.id), 1);
        }

        result
    }
}
