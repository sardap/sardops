use core::time::Duration;

use bincode::{Decode, Encode};
use const_for::const_for;
use glam::usize;
use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter, FromRepr};

use crate::{
    assets::{self, StaticImage},
    book::BookInfo,
    food::STARTING_FOOD,
    furniture::HomeFurnitureKind,
    game_context::GameContext,
    pc::Program,
    scene::{SceneEnum, fishing_scene, star_gazing_scene},
};

include!(concat!(env!("OUT_DIR"), "/dist_items.rs"));

pub const ITEM_COUNT: usize = core::mem::variant_count::<ItemKind>();

// MAKE ITEMS
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum ItemCategory {
    Misc,
    Furniture,
    PlayThing,
    Usable,
    Book,
    Software,
    Food,
}

impl ItemCategory {
    pub const fn items(&self) -> &'static [ItemKind] {
        match self {
            ItemCategory::Misc => &[],
            ItemCategory::Furniture => &FURNITURE_ITEMS,
            ItemCategory::PlayThing => &PLAYTHING_ITEMS,
            ItemCategory::Usable => &USABLE_ITEMS,
            ItemCategory::Book => &BOOK_ITEMS,
            ItemCategory::Software => &SOFTWARE_ITEMS,
            ItemCategory::Food => &FOOD_ITEMS,
        }
    }

    pub const fn icon(&self) -> &'static StaticImage {
        match self {
            ItemCategory::Misc => &assets::IMAGE_BAG_ICON_FURNITURE,
            ItemCategory::Furniture => &assets::IMAGE_BAG_ICON_FURNITURE,
            ItemCategory::PlayThing => &assets::IMAGE_BAG_ICON_PLAYTHING,
            ItemCategory::Usable => &assets::IMAGE_BAG_ICON_USEABLE,
            ItemCategory::Book => &assets::IMAGE_BAG_ICON_BOOK,
            ItemCategory::Software => &assets::IMAGE_BAG_ICON_SOFTWARE,
            ItemCategory::Food => &assets::IMAGE_BAG_ICON_FOOD,
        }
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

    pub const fn furniture(&self) -> Option<HomeFurnitureKind> {
        Some(match self {
            ItemKind::AnalogueClock => HomeFurnitureKind::AnalogueClock,
            ItemKind::DigitalClock => HomeFurnitureKind::DigitalClock,
            ItemKind::FishTank => HomeFurnitureKind::FishTank,
            ItemKind::PaintingBranch => HomeFurnitureKind::PaintingBranch,
            ItemKind::PaintingDude => HomeFurnitureKind::PaintingDude,
            ItemKind::PaintingMan => HomeFurnitureKind::PaintingMan,
            ItemKind::PaintingPc => HomeFurnitureKind::PaintingPc,
            ItemKind::PaintingSun => HomeFurnitureKind::PaintingSun,
            ItemKind::InvetroLight => HomeFurnitureKind::InvertroLight,
            _ => return None,
        })
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
    uses: i8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub enabled: bool,
}

impl ItemExtra {
    pub const fn new() -> Self {
        Self {
            uses: 0,
            enabled: false,
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
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    contents: [InventoryEntry; ITEM_COUNT],
}

impl Inventory {
    pub fn item_count(&self, item: ItemKind) -> u32 {
        self.contents[item as usize].owned
    }

    pub fn has_any_item(&self) -> bool {
        for item in ItemKind::iter() {
            if item != ItemKind::None && self.has_item(item) {
                return true;
            }
        }

        false
    }

    pub fn has_any_furniture(&self) -> bool {
        for item in ItemKind::iter() {
            if item.furniture().is_some() && self.has_item(item) {
                return true;
            }
        }

        false
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

    // SLOW POINT this is called really rarely
    pub fn has_any_enabled_book(&self) -> bool {
        for item in ItemKind::iter() {
            if item.is_book() && self.has_item(item) {
                return true;
            }
        }

        false
    }
}

impl Default for Inventory {
    fn default() -> Self {
        let mut result = Self {
            contents: core::array::from_fn(|_| InventoryEntry::default()),
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

    pub fn with_consumed(mut self) -> Self {
        self.consumed = true;
        self
    }

    pub fn with_scene(mut self, scene: SceneEnum) -> Self {
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
pub type IsUseableItemFn = fn(game_ctx: &mut GameContext) -> bool;

pub struct UsableItem {
    item: ItemKind,
    use_fn: UseItemFn,
    usable_fn: IsUseableItemFn,
}

impl UsableItem {
    pub const fn new(item: ItemKind, use_fn: UseItemFn) -> Self {
        Self {
            item: item,
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

const USE_SHOP_UPGRADE: UsableItem = UsableItem::new(ItemKind::ShopUpgrade, |game_ctx| {
    game_ctx
        .shop
        .set_item_count(game_ctx.shop.get_item_count() + 1);

    UseItemOutput::new().with_consumed()
});

const USE_FISHING_ROD: UsableItem = UsableItem::new(ItemKind::FishingRod, |game_ctx| {
    let mut result =
        UseItemOutput::new().with_scene(SceneEnum::Fishing(fishing_scene::FishingScene::new()));
    let entry = game_ctx.inventory.get_entry_mut(ItemKind::FishingRod);
    entry.item_extra.uses -= 1;
    if entry.item_extra.uses <= 0 {
        entry.item_extra = ItemExtra::new_from_kind(ItemKind::FishingRod);
        result = result.with_consumed();
    }

    result
});

const USE_FISH: UsableItem = UsableItem::new(ItemKind::Fish, |game_ctx| {
    game_ctx.home_fish_tank.add(&mut game_ctx.rng);
    UseItemOutput::new().with_consumed()
})
.with_is_usable_fn(|game_ctx| game_ctx.inventory.has_item(ItemKind::FishTank));

const USE_TELESCOPE: UsableItem = UsableItem::new(ItemKind::Telescope, |_| {
    UseItemOutput::new().with_scene(SceneEnum::StarGazing(
        star_gazing_scene::StarGazingScene::new(),
    ))
})
.with_is_usable_fn(|game_ctx| game_ctx.inventory.has_item(ItemKind::FishTank));

const ALL_USEABLE_ITEMS: &[UsableItem] =
    &[USE_SHOP_UPGRADE, USE_FISHING_ROD, USE_FISH, USE_TELESCOPE];

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
