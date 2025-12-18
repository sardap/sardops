use core::time::Duration;

use bincode::{Decode, Encode};
use const_for::const_for;
use glam::usize;
use sdop_common::ItemCategory;
use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter, FromRepr};

use crate::{
    assets::{self, StaticImage},
    book::BookInfo,
    explore::LOCATIONS,
    furniture::HomeFurnitureKind,
    game_consts::STARTING_ITEMS,
    game_context::GameContext,
    items_use::ALL_USEABLE_ITEMS,
    pc::Program,
    scene::SceneEnum,
};

include!(concat!(env!("OUT_DIR"), "/dist_items.rs"));

pub const ITEM_COUNT: usize = core::mem::variant_count::<ItemKind>();

pub const fn items_for_cata(cata: &ItemCategory) -> &'static [ItemKind] {
    match cata {
        ItemCategory::Misc => &[],
        ItemCategory::Furniture => &FURNITURE_ITEMS,
        ItemCategory::PlayThing => &PLAYTHING_ITEMS,
        ItemCategory::Usable => &USABLE_ITEMS,
        ItemCategory::Book => &BOOK_ITEMS,
        ItemCategory::Software => &SOFTWARE_ITEMS,
        ItemCategory::Food => &FOOD_ITEMS,
        ItemCategory::Map => &MAP_ITEMS,
    }
}

pub const fn icon_for_cata(cata: &ItemCategory) -> &'static StaticImage {
    match cata {
        ItemCategory::Misc => &assets::IMAGE_BAG_ICON_FURNITURE,
        ItemCategory::Furniture => &assets::IMAGE_BAG_ICON_FURNITURE,
        ItemCategory::PlayThing => &assets::IMAGE_BAG_ICON_PLAYTHING,
        ItemCategory::Usable => &assets::IMAGE_BAG_ICON_USEABLE,
        ItemCategory::Book => &assets::IMAGE_BAG_ICON_BOOK,
        ItemCategory::Software => &assets::IMAGE_BAG_ICON_SOFTWARE,
        ItemCategory::Food => &assets::IMAGE_BAG_ICON_FOOD,
        ItemCategory::Map => &assets::IMAGE_BAG_ICON_FOOD,
    }
}

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

    // This should be done at compile time
    pub fn is_usable(&self, game_ctx: &mut GameContext) -> bool {
        for usable in ALL_USEABLE_ITEMS {
            if usable.item == *self && (usable.usable_fn)(game_ctx) {
                return true;
            }
        }

        false
    }

    pub fn use_item(&self, game_ctx: &mut GameContext) -> Option<UseItemOutput> {
        for usable in ALL_USEABLE_ITEMS {
            if usable.item == *self {
                return Some(usable.use_item(game_ctx));
            }
        }

        None
    }

    pub const fn program(&self) -> Option<Program> {
        Some(match self {
            ItemKind::ProgramTicTacToe => &assets::FRAMES_PC_PROGRAM_TIC_TAC_TOE,
            ItemKind::ProgramDopCraft => &assets::FRAMES_PC_PROGRAM_RTS,
            ItemKind::ProgramCCompiler => &assets::FRAMES_PC_PROGRAM_C_COMPILER,
            ItemKind::ProgramWwwSurfer => &assets::FRAMES_PC_PROGRAM_WWW,
            ItemKind::ProgramSardips => &assets::FRAMES_PC_PROGRAM_SARDIPS,
            _ => return None,
        })
    }

    pub const fn toggleable(&self) -> bool {
        self.is_book()
    }

    pub const fn is_book(&self) -> bool {
        self.book_info().chapters > 0
    }

    pub const fn book_info(&self) -> &'static BookInfo {
        const DEFAULT: BookInfo = BookInfo {
            item: ItemKind::None,
            length: Duration::ZERO,
            chapters: 0,
            open_book: &assets::IMAGE_BOOK_0_OPEN,
            word_bank: &[],
        };

        const VIC: BookInfo = BookInfo {
            item: ItemKind::BookVic19811992,
            length: Duration::from_hours(2),
            chapters: 9,
            open_book: &assets::IMAGE_BOOK_0_OPEN,
            word_bank: &[
                "John", "Cain", "Jr", "Debt", "Bundoora", "Union", "City", "Loop", "Dock", "Lands",
                "Trams", "Metcard",
            ],
        };

        const WRAN: BookInfo = BookInfo {
            item: ItemKind::BookNevileWran,
            length: Duration::from_hours(3),
            chapters: 24,
            open_book: &assets::IMAGE_BOOK_WRAN_OPEN,
            word_bank: &[
                "Neville",
                "Wran",
                "Wranslide",
                "Health",
                "Train",
                "Strike",
                "Economy",
                "Women",
                "Reform",
                "Shelter",
                "Vote",
                "One",
                "Person",
                "Rainforest",
            ],
        };

        const C_PROGRAMMING: BookInfo = BookInfo {
            item: ItemKind::BookCProgramming,
            length: Duration::from_hours(4),
            chapters: 17,
            open_book: &assets::IMAGE_BOOK_C_OPEN,
            word_bank: &[
                "Segfault",
                "Recursion",
                "Stack",
                "Malloc",
                "Null",
                "float*",
                "float**",
                "void****",
                "&top",
                "Pointer",
                "Pointer-Pointer",
                "Array",
                "Struct",
                "Typedef",
                "Macro",
                "Static",
            ],
        };

        const DRACULA: BookInfo = BookInfo {
            item: ItemKind::BookDracula,
            length: Duration::from_hours(1),
            chapters: 27,
            open_book: &assets::IMAGE_BOOK_DRACULA_OPEN,
            word_bank: &[
                "Dracula",
                "vampire",
                "Transylvania",
                "Count",
                "Blood",
                "Undead",
                "Fangs",
                "Night",
                "Castle",
                "Ghoul",
                "Renfield",
                "Lucy",
                "Mina",
                "Jonathan",
                "Van Helsing",
                "Stake",
                "Garlic",
                "Cross",
                "Coffin",
            ],
        };

        const GATSBY: BookInfo = BookInfo {
            item: ItemKind::BookGreatGatsby,
            length: Duration::from_mins(45),
            chapters: 9,
            open_book: &assets::IMAGE_BOOK_GREAT_GATSBY_OPEN,
            word_bank: &[
                "Gatsby",
                "Daisy",
                "Nick",
                "Tom",
                "Jordan",
                "Myrtle",
                "Valley of Ashes",
                "Green Light",
                "American Dream",
                "Parties",
                "Bootlegging",
                "Affair",
                "Infidelity",
                "Class",
                "Wealth",
                "Illusion",
                "Hope",
                "Tragedy",
                "WW1",
                "Veteran",
            ],
        };

        const GILGAMESH: BookInfo = BookInfo {
            item: ItemKind::BookEpicOfGilgamesh,
            length: Duration::from_mins(45),
            chapters: 12,
            open_book: &assets::IMAGE_BOOK_GILGAMESH_OPEN,
            word_bank: &[
                "Enkidu",
                "Uruk",
                "Immortality",
                "Death",
                "Friendship",
                "Gods",
                "Humbaba",
                "Ishtar",
                "Eanna",
                "Council",
                "Journey",
                "Flood",
                "Utnapishtim",
                "Wild man",
                "Temple",
                "Cuneiform",
                "Epic",
                "Hero",
                "Quest",
            ],
        };

        const ODYSSEY: BookInfo = BookInfo {
            item: ItemKind::BookHomersOdyssey,
            length: Duration::from_hours(5),
            chapters: 24,
            open_book: &assets::IMAGE_BOOK_ODYSSEY_OPEN,
            word_bank: &[
                "Homer",
                "Odysseus",
                "Ithaca",
                "Penelope",
                "Telemachus",
                "Athena",
                "Poseidon",
                "Zeus",
                "Calypso",
                "Circe",
                "Cyclops",
                "Polyphemus",
                "Sirens",
                "Scylla",
                "Charybdis",
                "Trojan",
                "War",
                "Suitors",
                "Revenge",
                "Homecoming",
            ],
        };

        match self {
            ItemKind::BookVic19811992 => &VIC,
            ItemKind::BookNevileWran => &WRAN,
            ItemKind::BookCProgramming => &C_PROGRAMMING,
            ItemKind::BookDracula => &DRACULA,
            ItemKind::BookGreatGatsby => &GATSBY,
            ItemKind::BookEpicOfGilgamesh => &GILGAMESH,
            ItemKind::BookHomersOdyssey => &ODYSSEY,
            _ => &DEFAULT,
        }
    }
}

impl From<HomeFurnitureKind> for ItemKind {
    fn from(value: HomeFurnitureKind) -> Self {
        match value {
            HomeFurnitureKind::None => Self::None,
            HomeFurnitureKind::DigitalClock => Self::DigitalClock,
            HomeFurnitureKind::AnalogueClock => Self::AnalogueClock,
            HomeFurnitureKind::Alarm => Self::Alarm,
            HomeFurnitureKind::ThermometerMercury => Self::ThermometerMercury,
            HomeFurnitureKind::ThermometerDigital => Self::ThermometerDigital,
            HomeFurnitureKind::SpaceHeater => Self::SpaceHeater,
            HomeFurnitureKind::AirCon => Self::AirConditioner,
            HomeFurnitureKind::FishTank => Self::FishTank,
            HomeFurnitureKind::InvertroLight => Self::InvetroLight,
            HomeFurnitureKind::Calendar => Self::Calendar,
            HomeFurnitureKind::PaintingBranch => Self::PaintingBranch,
            HomeFurnitureKind::PaintingDude => Self::PaintingDude,
            HomeFurnitureKind::PaintingMan => Self::PaintingMan,
            HomeFurnitureKind::PaintingPc => Self::PaintingPc,
            HomeFurnitureKind::PaintingSun => Self::PaintingSun,
            HomeFurnitureKind::PaintingMallsBalls => Self::PaintingMallsBalls,
        }
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

macro_rules! items_for_category {
    ($name_lower:tt, $name_upper:tt, $variant:ident) => {
        paste::paste! {
            const fn [<count_items_for_category_ $name_lower >]() -> usize {
                let mut result = 0;
                const_for!(i in 0..ITEM_COUNT => {
                    let item = ItemKind::from_repr(i).unwrap();
                    if matches!(item.category(), ItemCategory::$variant) {
                        result += 1;
                    }
                });
                result
            }

            const fn [< items_for_category_ $name_lower >]<const T: usize>() -> [ItemKind; T] {
                let mut result = [ItemKind::None; T];
                let mut top = 0;
                const_for!(i in 0..ITEM_COUNT => {
                    let item = ItemKind::from_repr(i).unwrap();
                    if matches!(item.category(), ItemCategory::$variant) {
                        result[top] = item;
                        top += 1;
                    }
                });
                result
            }

            const [< $name_upper _ITEM_COUNT>]: usize = [< count_items_for_category_ $name_lower>]();
            pub const [< $name_upper _ITEMS>]: [ItemKind; [< $name_upper _ITEM_COUNT>]] = [< items_for_category_ $name_lower>]();
        }
    };
}

items_for_category!("furniture", "FURNITURE", Furniture);
items_for_category!("plaything", "PLAYTHING", PlayThing);
items_for_category!("usable", "USABLE", Usable);
items_for_category!("book", "BOOK", Book);
items_for_category!("software", "SOFTWARE", Software);
items_for_category!("food", "FOOD", Food);
items_for_category!("map", "MAP", Map);

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

const fn program_count() -> usize {
    let mut count = 0;
    const_for!(i in 0..ITEM_COUNT => {
        let item = ItemKind::from_repr(i).unwrap();
        if item.program().is_some() {
            count += 1;
        }
    });
    count
}

pub const PROGRAM_COUNT: usize = program_count();

const fn get_all_programs() -> [ItemKind; PROGRAM_COUNT] {
    let mut result: [ItemKind; PROGRAM_COUNT] = [ItemKind::None; PROGRAM_COUNT];
    let mut top = 0;
    const_for!(i in 0..ITEM_COUNT => {
        let item = ItemKind::from_repr(i).unwrap();
        if item.program().is_some() {
            result[top] = item;
            top += 1;
        }
    });
    result
}

pub const ALL_PROGRAMS: [ItemKind; PROGRAM_COUNT] = get_all_programs();

pub const MAX_OWNED: i32 = 1000000;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct ItemExtra {
    pub uses: i8,
    pub enabled: bool,
}

impl Default for ItemExtra {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemExtra {
    pub const fn new() -> Self {
        Self {
            uses: 0,
            enabled: true,
        }
    }

    pub const fn with_uses(mut self, uses: i8) -> Self {
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
#[derive(Clone, Copy, Encode, Decode, Default)]
pub struct InventoryEntry {
    pub owned: u32,
    pub item_extra: ItemExtra,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct Inventory {
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    contents: [InventoryEntry; ITEM_COUNT],
}

impl Inventory {
    pub fn item_count(&self, item: ItemKind) -> u32 {
        self.contents[item as usize].owned
    }

    pub fn has_any_item(&self) -> bool {
        ItemKind::iter().any(|i| self.has_item(i))
    }

    pub fn has_any_furniture(&self) -> bool {
        FURNITURE_ITEMS.iter().any(|i| self.has_item(*i))
    }

    pub fn has_item(&self, item: ItemKind) -> bool {
        item != ItemKind::None && self.item_count(item) > 0
    }

    pub fn get_entry_mut(&mut self, item: ItemKind) -> &mut InventoryEntry {
        &mut self.contents[item as usize]
    }

    pub fn get_entry(&self, item: ItemKind) -> &InventoryEntry {
        &self.contents[item as usize]
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

    pub fn clear_item(&mut self, item: ItemKind) {
        let entry = self.get_entry_mut(item);
        if entry.owned <= 0 {
            return;
        }
        entry.owned = 0;
    }

    pub fn has_any_enabled_book(&self) -> bool {
        BOOKS
            .iter()
            .any(|item| self.has_item(*item) && self.get_entry(*item).item_extra.enabled)
    }

    pub fn has_any_map(&self) -> bool {
        LOCATIONS.iter().any(|loc| self.has_item(loc.item))
    }
}

impl Default for Inventory {
    fn default() -> Self {
        let mut result = Self {
            contents: core::array::from_fn(|_| InventoryEntry::default()),
        };

        for item in STARTING_ITEMS {
            result.add_item(*item, 1);
        }

        result
    }
}

#[derive(Default)]
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

    pub fn with_consumed(mut self) -> Self {
        self.consumed = true;
        self
    }

    pub fn with_scene(mut self, scene: SceneEnum) -> Self {
        self.new_scene = Some(scene);
        self
    }
}

pub type UseItemFn = fn(game_ctx: &mut GameContext) -> UseItemOutput;
pub type IsUseableItemFn = fn(game_ctx: &mut GameContext) -> bool;

pub struct UsableItem {
    item: ItemKind,
    use_fn: UseItemFn,
    usable_fn: IsUseableItemFn,
}

impl UsableItem {
    pub const fn new(item: ItemKind, use_fn: UseItemFn) -> Self {
        Self {
            item,
            use_fn,
            usable_fn: |_| true,
        }
    }

    pub const fn with_is_usable_fn(mut self, usable_fn: IsUseableItemFn) -> Self {
        self.usable_fn = usable_fn;
        self
    }

    pub fn use_item(&self, game_ctx: &mut GameContext) -> UseItemOutput {
        let output = (self.use_fn)(game_ctx);

        if output.consumed {
            game_ctx.inventory.add_item(self.item, -1);
        }

        output
    }
}

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
    if chance_set.is_empty() {
        return ItemKind::None;
    }

    for entry in chance_set {
        if val < entry.odds {
            return entry.kind;
        }
    }

    ItemKind::None
}

pub const fn get_book_count() -> usize {
    let mut result = 0;
    const_for!(i in 0..ITEM_COUNT => {
        let item = ItemKind::from_repr(i).unwrap();
        if item.is_book() {
            result += 1;
        }
    });
    result
}

pub const BOOK_COUNT: usize = get_book_count();

pub const fn get_books() -> [ItemKind; BOOK_COUNT] {
    let mut result = [ItemKind::None; BOOK_COUNT];
    let mut top = 0;
    const_for!(i in 0..ITEM_COUNT => {
        let item = ItemKind::from_repr(i).unwrap();
        if item.is_book() {
            result[top] = item;
            top += 1;
        }
    });
    result
}

pub const BOOKS: [ItemKind; BOOK_COUNT] = get_books();
