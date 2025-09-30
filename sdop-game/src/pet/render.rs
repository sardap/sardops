use glam::Vec2;

use crate::{
    anime::Anime,
    assets::Image,
    display::PostionMode,
    pet::definition::{PET_BLOB_ID, PetAnimationSet, PetDefinition, PetDefinitionId},
    sprite::{Sprite, SpriteMask, SpritePostionMode},
};

#[derive(Clone, Copy)]
pub struct PetRender {
    def_id: PetDefinitionId,
    pub pos: Vec2,
    set: PetAnimationSet,
    pub anime: Anime,
    pub pos_mode: PostionMode,
}

fn create_pet_anime(def_id: PetDefinitionId, set: PetAnimationSet) -> Anime {
    Anime::new(PetDefinition::get_by_id(def_id).images.frames(set).frame)
        .with_mask(PetDefinition::get_by_id(def_id).images.frames(set).masked)
}

impl PetRender {
    pub fn new(def_id: PetDefinitionId) -> Self {
        Self {
            def_id,
            anime: create_pet_anime(def_id, Default::default()),
            ..Default::default()
        }
    }

    pub fn with_pos_mode(mut self, pos_mode: PostionMode) -> Self {
        self.pos_mode = pos_mode;
        self
    }

    pub fn def_id(&self) -> PetDefinitionId {
        self.def_id
    }

    fn reset_anime(&mut self) {
        self.anime = create_pet_anime(self.def_id, self.set);
    }

    pub fn set_def_id(&mut self, def_id: PetDefinitionId) {
        if def_id != self.def_id {
            self.def_id = def_id;
            self.reset_anime();
        }
    }

    pub fn set_animation(&mut self, set: PetAnimationSet) {
        if set != self.set {
            self.set = set;
            self.reset_anime();
        }
    }

    pub fn tick(&mut self, delta: core::time::Duration) {
        self.anime.tick(delta);
    }
}

impl Sprite for PetRender {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl Image {
        self.anime.current_frame()
    }
}

impl SpriteMask for PetRender {
    fn image_mask(&self) -> &impl Image {
        self.anime.current_frame_mask().unwrap()
    }
}

impl Default for PetRender {
    fn default() -> Self {
        Self {
            def_id: Default::default(),
            pos: Vec2::default(),
            set: PetAnimationSet::default(),
            anime: create_pet_anime(PET_BLOB_ID, Default::default()),
            pos_mode: PostionMode::Center,
        }
    }
}

impl SpritePostionMode for PetRender {
    fn sprite_postion_mode(&self) -> PostionMode {
        self.pos_mode
    }
}
