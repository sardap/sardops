use bincode::{Decode, Encode};
use chrono::Datelike;

use crate::{
    Timestamp,
    items::{COMMON_ITEMS, ItemKind, RARE_ITEMS},
};

const MAX_SHOP_ITEMS: usize = 10;

pub type ShopItemSet = [ItemKind; MAX_SHOP_ITEMS];

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct Shop {
    item_count: u32,
}

impl Shop {
    pub fn item_set(&self, timestamp: Timestamp) -> ShopItemSet {
        let mut rng = fastrand::Rng::with_seed(timestamp.date_seed());

        let mut result = [ItemKind::None; MAX_SHOP_ITEMS];
        let mut count = 0;
        while count < self.item_count {
            let set: &[ItemKind] = if rng.f32() < 0.9 {
                &COMMON_ITEMS
            } else {
                &RARE_ITEMS
            };

            let item = *rng.choice(set).unwrap();
            if result.iter().all(|i| *i != item) {
                result[count as usize] = item;
                count += 1;
            }
        }

        result
    }

    pub fn get_item_count(&self) -> u32 {
        self.item_count
    }

    pub fn set_item_count(&mut self, item_count: u32) {
        self.item_count = item_count;
    }
}

impl Default for Shop {
    fn default() -> Self {
        Self { item_count: 2 }
    }
}
