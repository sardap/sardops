use chrono::Timelike;

use crate::{
    assets::{self, Frame},
    food::Food,
    pet::LifeStage,
    Timestamp,
};

include!(concat!(env!("OUT_DIR"), "/dist_pets.rs"));

pub type PetDefinitionId = i32;

pub struct PetDefinition {
    pub id: PetDefinitionId,
    pub name: &'static str,
    pub life_stage: LifeStage,
    pub images: PetImageSet,
    pub stomach_size: f32,
    pub base_weight: f32,
}

impl PetDefinition {
    pub const fn new(
        id: PetDefinitionId,
        name: &'static str,
        life_stage: LifeStage,
        stomach_size: f32,
        base_weight: f32,
        images: PetImageSet,
    ) -> Self {
        Self {
            id,
            name,
            life_stage,
            images,
            stomach_size,
            base_weight,
        }
    }
}

impl PetDefinition {
    pub fn food_multiplier(&self, _food: &Food) -> f32 {
        match self.id {
            _ => 1.,
        }
    }

    pub fn poop_time_multiplier(&self) -> f32 {
        1.
    }

    pub fn should_be_sleeping(&self, timestamp: &Timestamp) -> bool {
        if timestamp.inner().hour() >= 22 || timestamp.inner().hour() < 6 {
            return true;
        }

        false
    }

    pub fn get_by_id(id: PetDefinitionId) -> &'static PetDefinition {
        let id = id as usize;
        if id >= PET_DEFINITIONS.len() {
            return &PET_DEFINITIONS[0];
        }
        return &PET_DEFINITIONS[id];
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PetAnimationSet {
    Normal,
    Happy,
    Sad,
    Eat,
    Sleeping,
}

impl Default for PetAnimationSet {
    fn default() -> Self {
        PetAnimationSet::Normal
    }
}

pub struct PetImageSet {
    pub width: i32,
    pub height: i32,
    pub normal: &'static [Frame],
    pub happy: Option<&'static [Frame]>,
    pub sad: Option<&'static [Frame]>,
    pub eat: Option<&'static [Frame]>,
    pub sleep: Option<&'static [Frame]>,
}

impl PetImageSet {
    pub const fn new(normal: &'static [Frame], width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            normal,
            happy: None,
            sad: None,
            eat: None,
            sleep: None,
        }
    }

    pub const fn with_happy(mut self, happy: &'static [Frame]) -> Self {
        self.happy = Some(happy);
        self
    }

    pub const fn with_sad(mut self, sad: &'static [Frame]) -> Self {
        self.sad = Some(sad);
        self
    }

    pub const fn with_eat(mut self, eat: &'static [Frame]) -> Self {
        self.eat = Some(eat);
        self
    }

    pub const fn with_sleep(mut self, sleep: &'static [Frame]) -> Self {
        self.sleep = Some(sleep);
        self
    }

    pub fn frames(&self, mood: PetAnimationSet) -> &[Frame] {
        match mood {
            PetAnimationSet::Normal => return self.normal,
            PetAnimationSet::Happy => {
                if let Some(happy) = self.happy {
                    return happy;
                }
            }
            PetAnimationSet::Sad => {
                if let Some(sad) = self.sad {
                    return sad;
                }
            }
            PetAnimationSet::Eat => {
                if let Some(eat) = self.eat {
                    return eat;
                }
            }
            PetAnimationSet::Sleeping => {
                if let Some(sleep) = self.sleep {
                    return sleep;
                }
            }
        }

        return self.normal;
    }
}
