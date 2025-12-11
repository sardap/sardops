use bincode::{Decode, Encode};
use sdop_common::ItemCategory;

use crate::{
    Timestamp,
    items::{ItemKind, items_for_cata},
};

const MAX_SHOP_ITEMS: usize = 20;

pub type ShopItemSet = [ItemKind; MAX_SHOP_ITEMS];

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct Shop {
    cata_0: u32,
    cata_1: u32,
    cata_2: u32,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            cata_0: 2,
            cata_1: 2,
            cata_2: 1,
        }
    }
}

impl Shop {
    pub fn item_set(&self, timestamp: Timestamp) -> ShopItemSet {
        let mut rng = fastrand::Rng::with_seed(timestamp.date_seed());

        let mut result = [ItemKind::None; MAX_SHOP_ITEMS];
        let mut top = 0;

        let counts = [
            (
                &[ItemCategory::Furniture, ItemCategory::Usable] as &[ItemCategory],
                self.cata_0,
            ),
            (
                &[
                    ItemCategory::PlayThing,
                    ItemCategory::Book,
                    ItemCategory::Software,
                ] as &[ItemCategory],
                self.cata_1,
            ),
            (&[ItemCategory::Food] as &[ItemCategory], self.cata_2),
        ];

        for (catas, count) in counts {
            // Flattening iter isn't working just do it myself
            let mut added = 0;
            let total: usize = catas.iter().map(|i| items_for_cata(i).len()).sum();
            for _ in 0..total {
                let next = rng.usize(0..total);

                let mut max = 0;
                for cata in catas {
                    let items = items_for_cata(cata);
                    if next < max + items.len() {
                        let item = items[next - max];
                        if !result.contains(&item) && item != ItemKind::RecipeBiscuit {
                            result[top] = item;
                            top += 1;
                            added += 1;
                        }
                        break;
                    }
                    max += items.len();
                }

                if added >= count {
                    break;
                }
            }
        }

        result
    }

    pub fn get_item_count(&self) -> u32 {
        self.cata_0
    }

    pub fn set_item_count(&mut self, item_count: u32) {
        self.cata_0 = item_count;
    }
}
