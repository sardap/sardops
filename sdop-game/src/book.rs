use const_for::const_for;
use core::{ops::Div, slice::Iter, time::Duration};
use heapless::Vec;
use sdop_common::{LifeStage, LifeStageMask};

use bincode::{Decode, Encode};

use crate::{
    assets::{self, StaticImage},
    game_context::GameContext,
    items::{BOOK_COUNT, ITEM_COUNT, Inventory, ItemKind},
    pet::{
        PetInstance,
        definition::{PET_BRAINO_ID, PetDefinition, PetDefinitionId},
    },
};

pub struct BookInfo {
    pub item: ItemKind,
    pub length: Duration,
    pub chapters: u8,
    pub open_book: &'static StaticImage,
    pub ls_mask: LifeStageMask,
    pub word_bank: &'static [&'static str],
}

impl BookInfo {
    pub fn chapter_length(&self) -> Duration {
        Duration::from_micros(
            (self.length.as_micros() as u64)
                .checked_div(self.chapters as u64)
                .unwrap_or(0),
        )
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct BookRead {
    item: ItemKind,
    read_time: Duration,
    chapters: u8,
}

impl BookRead {
    pub fn tick_read(&mut self, delta: Duration, read_mul: f32) {
        let delta = Duration::from_millis((delta.as_millis_f32() * read_mul) as u64);
        self.read_time = (self.read_time + delta).min(self.book().length);
        self.chapters = libm::floorf(
            self.read_time
                .as_secs_f32()
                .div(self.book().chapter_length().as_secs_f32()),
        ) as u8;
    }

    pub fn book(&self) -> &BookInfo {
        self.item().book_info()
    }

    pub fn started(&self) -> bool {
        self.read_time > Duration::ZERO
    }

    pub fn chapters(&self) -> u8 {
        self.chapters
    }

    pub fn percent_of_next_complete(&self) -> f32 {
        let chapter_len = self.book().chapter_length();
        let completed =
            Duration::from_millis(self.chapters as u64 * chapter_len.as_millis() as u64);
        let left_over = self.read_time - completed;
        left_over.as_millis_f32() / chapter_len.as_millis_f32()
    }

    pub fn completed(&self) -> bool {
        let info = self.item.book_info();
        self.chapters >= info.chapters
    }

    pub fn item(&self) -> ItemKind {
        self.item
    }
}

pub fn on_book_completed(pet: &mut PetInstance, inventory: &mut Inventory, book: ItemKind) {
    match book {
        ItemKind::BookVic19811992 => {
            inventory.add_item(ItemKind::MapFlagstaffStation, 1);
        }
        ItemKind::BookCProgramming => {
            inventory.add_item(ItemKind::MapCyberspace, 1);
        }
        _ => {}
    }

    pet.explore.bonus_skill += book.skill();
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct BookHistory {
    books: [BookRead; BOOK_COUNT],
}

impl BookHistory {
    pub fn get_read(&self, item: ItemKind) -> &BookRead {
        &self.books[BOOK_INDEXES[item as usize]]
    }

    pub fn get_mut_read(&mut self, item: ItemKind) -> &mut BookRead {
        &mut self.books[BOOK_INDEXES[item as usize]]
    }

    pub fn has_book_to_read(&self, life_stage: LifeStage, inventory: &Inventory) -> bool {
        for read in &self.books {
            if inventory.has_item(read.item)
                && !read.completed()
                && inventory.get_entry(read.item).item_extra.enabled
                && read.item.book_info().ls_mask & life_stage.bitmask() > 0
            {
                return true;
            }
        }

        false
    }

    pub fn get_reading_book(&self, inventory: &Inventory) -> Option<ItemKind> {
        for read in &self.books {
            if read.started() && inventory.has_item(read.item) {
                return Some(read.item);
            }
        }

        None
    }

    pub fn pick_random_unread_book(
        &self,
        rng: &mut fastrand::Rng,
        life_stage: LifeStage,
        inventory: &Inventory,
    ) -> Option<ItemKind> {
        let mut books: Vec<ItemKind, BOOK_COUNT> = Vec::new();
        for read in &self.books {
            if !read.completed()
                && inventory.has_item(read.item)
                && inventory.get_entry(read.item).item_extra.enabled
                && read.item.book_info().ls_mask & life_stage.bitmask() > 0
            {
                let _ = books.push(read.item);
            }
        }

        rng.choice(books.iter()).cloned()
    }

    pub fn completed_count(&self) -> usize {
        self.books.iter().filter(|i| i.completed()).count()
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, BookRead> {
        self.books.iter()
    }
}

const fn create_book_indexes() -> [usize; ITEM_COUNT] {
    let mut result = [0; ITEM_COUNT];
    let mut top = 0;
    const_for!(i in 0..ITEM_COUNT => {
        let item = ItemKind::from_repr(i).unwrap();
        if item.is_book() {
            result[i] = top;
            top += 1;
        }
    });
    result
}

const BOOK_INDEXES: [usize; ITEM_COUNT] = create_book_indexes();

const fn create_book_history() -> [BookRead; BOOK_COUNT] {
    let mut result = [BookRead {
        item: ItemKind::None,
        read_time: Duration::ZERO,
        chapters: 0,
    }; BOOK_COUNT];
    let mut top = 0;
    const_for!(i in 0..ITEM_COUNT => {
        let item = ItemKind::from_repr(i).unwrap();
        if item.is_book() {
            result[top].item = item;
            top += 1;
        }
    });
    result
}

impl Default for BookHistory {
    fn default() -> Self {
        Self {
            books: create_book_history(),
        }
    }
}

pub const fn item_to_book(item: &ItemKind) -> &'static BookInfo {
    match item {
        ItemKind::BookForBabies => {
            const BABIES: BookInfo = BookInfo {
                item: ItemKind::BookMeditations,
                length: Duration::from_mins(30),
                chapters: 6,
                open_book: &assets::IMAGE_BOOK_FOR_BABIES_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Baby]),
                word_bank: &[
                    "cat", "bat", "hat", "rat", "mat", "pat", "fat", "dog", "log", "hog", "pig",
                    "wig", "dig", "fig", "bug", "hug", "mug", "rug", "bed", "red", "led", "fed",
                    "net", "pet", "sun", "fun", "run", "bun", "gun", "cup", "pup", "mud", "pen",
                    "hen", "ten", "men", "man", "fan", "pan", "tan", "box", "fox", "leg", "beg",
                    "peg", "jam", "ram", "ham", "yam", "top", "mop", "hop", "pop", "lip", "sip",
                    "tip", "dip", "web", "cob", "rob", "van", "can", "pan", "tan",
                ],
            };
            &BABIES
        }
        ItemKind::BookVic19811992 => {
            const VIC: BookInfo = BookInfo {
                item: ItemKind::BookVic19811992,
                length: Duration::from_hours(2),
                chapters: 9,
                open_book: &assets::IMAGE_BOOK_0_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
                word_bank: &[
                    "John", "Cain", "Jr", "Debt", "Bundoora", "Union", "City", "Loop", "Dock",
                    "Lands", "Trams", "Metcard",
                ],
            };
            &VIC
        }
        ItemKind::BookNevileWran => {
            const WRAN: BookInfo = BookInfo {
                item: ItemKind::BookNevileWran,
                length: Duration::from_hours(3),
                chapters: 24,
                open_book: &assets::IMAGE_BOOK_WRAN_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
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
            &WRAN
        }
        ItemKind::BookCProgramming => {
            const C_PROGRAMMING: BookInfo = BookInfo {
                item: ItemKind::BookCProgramming,
                length: Duration::from_hours(4),
                chapters: 17,
                open_book: &assets::IMAGE_BOOK_C_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
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
            &C_PROGRAMMING
        }
        ItemKind::BookDracula => {
            const DRACULA: BookInfo = BookInfo {
                item: ItemKind::BookDracula,
                length: Duration::from_hours(3),
                chapters: 27,
                open_book: &assets::IMAGE_BOOK_DRACULA_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
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
            &DRACULA
        }
        ItemKind::BookGreatGatsby => {
            const GATSBY: BookInfo = BookInfo {
                item: ItemKind::BookGreatGatsby,
                length: Duration::from_hours(3),
                chapters: 9,
                open_book: &assets::IMAGE_BOOK_GREAT_GATSBY_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
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
            &GATSBY
        }
        ItemKind::BookEpicOfGilgamesh => {
            const GILGAMESH: BookInfo = BookInfo {
                item: ItemKind::BookEpicOfGilgamesh,
                length: Duration::from_mins(45),
                chapters: 12,
                open_book: &assets::IMAGE_BOOK_GILGAMESH_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
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
            &GILGAMESH
        }
        ItemKind::BookHomersOdyssey => {
            const ODYSSEY: BookInfo = BookInfo {
                item: ItemKind::BookHomersOdyssey,
                length: Duration::from_hours(5),
                chapters: 24,
                open_book: &assets::IMAGE_BOOK_ODYSSEY_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
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
            &ODYSSEY
        }
        ItemKind::BookTheArtOfWar => {
            const ART_OF_WAR: BookInfo = BookInfo {
                item: ItemKind::BookTheArtOfWar,
                length: Duration::from_hours(5),
                chapters: 12,
                open_book: &assets::IMAGE_BOOK_ART_OF_WAR_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
                word_bank: &[
                    "weak",
                    "strong",
                    "no fight",
                    "plans",
                    "schemes",
                    "food",
                    "position",
                    "chaos",
                    "opportunity",
                    "deception",
                    "warrior",
                    "no battle",
                    "victory",
                    "long war",
                    "bad",
                ],
            };
            &ART_OF_WAR
        }
        ItemKind::BookHomersIliad => {
            const ILIAD: BookInfo = BookInfo {
                item: ItemKind::BookHomersIliad,
                length: Duration::from_hours(5),
                chapters: 18,
                open_book: &assets::IMAGE_BOOK_ILIAD_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
                word_bank: &[
                    "Homer",
                    "Achilles",
                    "Agamemnon",
                    "Hector",
                    "Helen",
                    "Menelaus",
                    "Patroclus",
                    "Odysseus",
                    "Ajax",
                    "Aeneas",
                    "Trojan",
                    "AchillesHeel",
                    "Wrath",
                    "Honor",
                    "Glory",
                    "Fate",
                    "Gods",
                    "Olympus",
                ],
            };
            &ILIAD
        }
        ItemKind::BookTheDivineComedy => {
            const DIVINE_COMEDY: BookInfo = BookInfo {
                item: ItemKind::BookTheDivineComedy,
                length: Duration::from_hours(4),
                chapters: 16,
                open_book: &assets::IMAGE_BOOK_DIVINE_COMEDY_OPEN,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
                word_bank: &[
                    "Dante",
                    "Virgil",
                    "Beatrice",
                    "Inferno",
                    "Purgatorio",
                    "Paradiso",
                    "Hell",
                    "Purgatory",
                    "Heaven",
                    "Sin",
                    "Virtue",
                    "Divine",
                    "God",
                    "Lucifer",
                    "Angels",
                    "Demons",
                    "Allegory",
                    "Journey",
                    "Souls",
                    "7",
                    "Layers",
                ],
            };
            &DIVINE_COMEDY
        }
        ItemKind::BookMeditations => {
            const MEDITATIONS: BookInfo = BookInfo {
                item: ItemKind::BookMeditations,
                length: Duration::from_hours(3),
                chapters: 12,
                open_book: &assets::IMAGE_BOOK_MEDITATIONS_CLOSED,
                ls_mask: LifeStage::create_bitmask(&[LifeStage::Child, LifeStage::Adult]),
                word_bank: &[
                    "Mind",
                    "Beauty",
                    "Happiness",
                    "Life",
                    "Fact",
                    "Good",
                    "Pain",
                    "Alive",
                    "Love",
                    "Soul",
                    "Thoughts",
                    "Thinking",
                ],
            };
            &MEDITATIONS
        }
        _ => {
            const DEFAULT: BookInfo = BookInfo {
                item: ItemKind::None,
                length: Duration::ZERO,
                chapters: 0,
                open_book: &assets::IMAGE_BOOK_0_OPEN,
                ls_mask: LifeStage::create_bitmask(&[]),
                word_bank: &[],
            };
            &DEFAULT
        }
    }
}
