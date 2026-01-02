use std::{path::PathBuf, str::FromStr};

use csv::Writer;
use sdop_build_common::{ITEMS_RON_PATH, ItemTemplate};
use sdop_common::ItemCategory;

#[derive(serde::Serialize)]
struct Row<'a> {
    id: usize,
    name: &'a str,
    category: ItemCategory,
    cost: i32,
    in_shop: bool,
    unique: bool,
}

fn main() {
    let locations_path = PathBuf::from_str(ITEMS_RON_PATH).unwrap();

    let contents = std::fs::read_to_string(locations_path).unwrap();
    let item_templates: Vec<ItemTemplate> = ron::from_str(&contents).unwrap();

    let mut writer = Writer::from_path("items.csv").unwrap();
    for (i, item) in item_templates.into_iter().enumerate() {
        writer
            .serialize(Row {
                id: i,
                name: &item.name,
                cost: item.cost,
                in_shop: item.in_shop,
                unique: item.unique,
                category: item.category,
            })
            .unwrap();
    }

    writer.flush().unwrap();

    println!("Wrote CSV!");
}
