use chrono::Datelike;

use crate::{
    items::{Item, COMMON_ITEMS, RARE_ITEMS},
    Timestamp,
};

const MAX_SHOP_ITEMS: usize = 5;

// make the shop a struct that is stored in game context and saved
// Should have item count?
// Then figure out if items enum should be auto generated (Probably) so I can auto make recpie items

pub fn get_shop_items(timestamp: Timestamp) -> [Item; MAX_SHOP_ITEMS] {
    let day = timestamp.inner().day() as u8;
    let month = timestamp.inner().month() as u8;
    let year = timestamp.inner().year() as u16;
    let year_bytes = year.to_be_bytes();
    let year_left = year_bytes[0];
    let year_right = year_bytes[1];

    let seed = u32::from_be_bytes([day, month, year_left, year_right]) as u64;

    let mut rng = fastrand::Rng::with_seed(seed);

    let mut result = [Item::None; MAX_SHOP_ITEMS];
    let mut count = 0;
    while count < 4 {
        let set: &[Item] = if rng.f32() < 0.9 {
            &COMMON_ITEMS
        } else {
            &RARE_ITEMS
        };

        let item = *rng.choice(set).unwrap();
        if result.iter().all(|i| *i != item) {
            result[count] = item;
            count += 1;
        }
    }

    result
}
