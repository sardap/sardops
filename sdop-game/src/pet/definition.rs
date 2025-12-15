use core::{ops::Range, time::Duration};

use chrono::Timelike;

use crate::{
    Timestamp,
    assets::{self, MaskedFramesSet},
    explore::ExploreSkill,
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
        1.
    }

    pub const fn poop_interval_range(&self) -> Range<Duration> {
        match self.life_stage {
            LifeStage::Baby => (Duration::from_mins(30))..(Duration::from_mins(90)),
            LifeStage::Child => (Duration::from_mins(90))..(Duration::from_hours(3)),
            LifeStage::Adult => (Duration::from_mins(150))..(Duration::from_hours(4)),
        }
    }

    pub fn should_be_sleeping(&self, timestamp: &Timestamp) -> bool {
        return false;
        let hour = timestamp.inner().hour();
        match self.life_stage {
            LifeStage::Baby => false,
            LifeStage::Child => !(8..21).contains(&hour),
            LifeStage::Adult => !(7..22).contains(&hour),
        }
    }

    pub fn explore_skill(&self) -> ExploreSkill {
        match self.life_stage {
            LifeStage::Baby => 5,
            LifeStage::Child => 40,
            LifeStage::Adult => 100,
        }
    }

    pub fn get_by_id(id: PetDefinitionId) -> &'static PetDefinition {
        let id = id as usize;
        if id >= PET_DEFINITIONS.len() {
            return PET_DEFINITIONS[0];
        }
        PET_DEFINITIONS[id]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub enum PetAnimationSet {
    #[default]
    Normal,
    Happy,
    Sad,
    Eat,
    Sleeping,
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

        self.normal
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
