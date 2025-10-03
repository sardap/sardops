use bincode::{Decode, Encode};

use crate::{
    Timestamp,
    items::{COMMON_ITEMS, ItemCategory, ItemKind, RARE_ITEMS},
};

const MAX_SHOP_ITEMS: usize = 20;

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
        let mut top = 0;

        let counts = [
            (
                &[ItemCategory::Furniture, ItemCategory::Usable] as &[ItemCategory],
                2,
            ),
            (
                &[
                    ItemCategory::PlayThing,
                    ItemCategory::Book,
                    ItemCategory::Software,
                ] as &[ItemCategory],
                2,
            ),
            (&[ItemCategory::Food] as &[ItemCategory], 1),
        ];

        for (catas, count) in counts {
            // Flattening iter isn't working just do it myself
            let mut added = 0;
            let total: usize = catas.iter().map(|i| i.items().len()).sum();
            for _ in 0..total {
                let next = rng.usize(0..total);

                let mut max = 0;
                for cata in catas {
                    if next < max + cata.items().len() {
                        let item = cata.items()[next - max];
                        if !result.contains(&item) {
                            result[top] = item;
                            top += 1;
                            added += 1;
                        }
                        break;
                    }
                    max += cata.items().len();
                }

                if added >= count {
                    break;
                }
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
