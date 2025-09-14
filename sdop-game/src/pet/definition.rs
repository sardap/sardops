use chrono::Timelike;

use crate::{
    Timestamp,
    assets::{self, MaskedFramesSet},
    food::Food,
    pet::LifeStage,
};
use const_for::const_for;

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
    pub normal: MaskedFramesSet,
    pub happy: Option<MaskedFramesSet>,
    pub sad: Option<MaskedFramesSet>,
    pub eat: Option<MaskedFramesSet>,
    pub sleep: Option<MaskedFramesSet>,
}

impl PetImageSet {
    pub const fn new(normal: MaskedFramesSet, width: i32, height: i32) -> Self {
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

    pub const fn with_happy(mut self, happy: MaskedFramesSet) -> Self {
        self.happy = Some(happy);
        self
    }

    pub const fn with_sad(mut self, sad: MaskedFramesSet) -> Self {
        self.sad = Some(sad);
        self
    }

    pub const fn with_eat(mut self, eat: MaskedFramesSet) -> Self {
        self.eat = Some(eat);
        self
    }

    pub const fn with_sleep(mut self, sleep: MaskedFramesSet) -> Self {
        self.sleep = Some(sleep);
        self
    }

    pub fn frames(&self, mood: PetAnimationSet) -> MaskedFramesSet {
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

pub const fn get_count_from_stage(stage: LifeStage) -> usize {
    let mut result = 0;
    const_for!(i in 0..PET_DEFINITIONS.len() => {
        let def = PET_DEFINITIONS[i];
        if def.life_stage as u8 == stage as u8 {
            result += 1;
        }
    });
    result
}

pub const fn get_pets_from_stage<const N: usize>(stage: LifeStage) -> [PetDefinitionId; N] {
    let mut top = 0;
    let mut result: [PetDefinitionId; N] = [0; N];
    const_for!(i in 0..PET_DEFINITIONS.len() => {
        let def = PET_DEFINITIONS[i];
        if def.life_stage as u8 == stage as u8 {
            result[top] = def.id;
            top += 1;
        }
    });
    result
}

pub const PET_BABY_ID_COUNT: usize = get_count_from_stage(LifeStage::Baby);
pub const PET_BABIES: [PetDefinitionId; PET_BABY_ID_COUNT] =
    get_pets_from_stage::<PET_BABY_ID_COUNT>(LifeStage::Baby);

pub const PET_CHILD_ID_COUNT: usize = get_count_from_stage(LifeStage::Child);
pub const PET_CHILDS: [PetDefinitionId; PET_CHILD_ID_COUNT] =
    get_pets_from_stage::<PET_CHILD_ID_COUNT>(LifeStage::Child);

pub const PET_ADULT_ID_COUNT: usize = get_count_from_stage(LifeStage::Adult);
pub const PET_ADULTS: [PetDefinitionId; PET_ADULT_ID_COUNT] =
    get_pets_from_stage::<PET_ADULT_ID_COUNT>(LifeStage::Adult);
