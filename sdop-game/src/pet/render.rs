use glam::Vec2;

use crate::{
    anime::Anime,
    assets::Image,
    pet::definition::{PetAnimationSet, PetDefinition, PetDefinitionId},
    sprite::Sprite,
};

#[derive(Clone, Copy)]
pub struct PetRender {
    def_id: PetDefinitionId,
    pub pos: Vec2,
    set: PetAnimationSet,
    pub anime: Anime,
}

impl PetRender {
    pub fn new(def_id: PetDefinitionId) -> Self {
        Self {
            def_id,
            anime: Anime::new(
                PetDefinition::get_by_id(def_id)
                    .images
                    .frames(PetAnimationSet::default()),
            ),
            ..Default::default()
        }
    }

    pub fn definition(&self) -> &'static PetDefinition {
        PetDefinition::get_by_id(self.def_id)
    }

    pub fn def_id(&self) -> PetDefinitionId {
        self.def_id
    }

    fn reset_anime(&mut self) {
        self.anime = Anime::new(self.definition().images.frames(self.set));
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
    fn pos<'a>(&'a self) -> &'a Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl Image {
        self.anime.current_frame()
    }
}

impl Default for PetRender {
    fn default() -> Self {
        Self {
            def_id: Default::default(),
            pos: Vec2::default(),
            set: PetAnimationSet::default(),
            anime: Anime::new(
                PetDefinition::get_by_id(Default::default())
                    .images
                    .frames(PetAnimationSet::default()),
            ),
        }
    }
}
