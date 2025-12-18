use const_for::const_for;
use core::{slice::Iter, time::Duration};
use heapless::Vec;

use bincode::{Decode, Encode};

use crate::{
    assets::StaticImage,
    game_context::GameContext,
    items::{BOOK_COUNT, ITEM_COUNT, Inventory, ItemKind},
    pet::definition::{PET_BRAINO_ID, PetDefinitionId},
};

pub struct BookInfo {
    pub item: ItemKind,
    pub length: Duration,
    pub chapters: u8,
    pub open_book: &'static StaticImage,
    pub word_bank: &'static [&'static str],
}

impl BookInfo {
    pub fn chapter_length(&self, def_id: PetDefinitionId) -> Duration {
        Duration::from_micros(
            (self.length.as_micros() as u64)
                .checked_div(self.chapters as u64)
                .unwrap_or(0),
        )
        .mul_f32(match def_id {
            PET_BRAINO_ID => 2.,
            _ => 1.,
        })
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct BookRead {
    item: ItemKind,
    chapters: u8,
}

impl BookRead {
    pub fn started(&self) -> bool {
        self.chapters > 0 && !self.completed()
    }

    pub fn chapters(&self) -> u8 {
        self.chapters
    }

    pub fn completed(&self) -> bool {
        let info = self.item.book_info();
        self.chapters >= info.chapters
    }

    pub fn complete_chapter(&mut self) {
        self.chapters = self.chapters.checked_add(1).unwrap_or_default();
    }

    pub fn item(&self) -> ItemKind {
        self.item
    }
}

pub fn on_book_completed(game_ctx: &mut GameContext, book: ItemKind) {
    match book {
        ItemKind::BookVic19811992 => {
            game_ctx
                .inventory
                .add_item(ItemKind::MapFlagstaffStation, 1);
        }
        ItemKind::BookCProgramming => {
            game_ctx.inventory.add_item(ItemKind::MapCyberspace, 1);
        }
        _ => {}
    }

    game_ctx.pet.explore.bonus_skill += book.skill();
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

    pub fn has_book_to_read(&self, inventory: &Inventory) -> bool {
        for read in &self.books {
            if inventory.has_item(read.item) && !read.completed() {
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
        inventory: &Inventory,
    ) -> Option<ItemKind> {
        let mut books: Vec<ItemKind, BOOK_COUNT> = Vec::new();
        for read in &self.books {
            if !read.completed()
                && inventory.has_item(read.item)
                && inventory.get_entry(read.item).item_extra.enabled
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
