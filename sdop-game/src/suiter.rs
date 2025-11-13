use core::time::Duration;

use bincode::{Decode, Encode};

use crate::{
    death::passed_threshold_chance,
    game_consts::{SUITER_LEAVE_ODDS, SUITER_SHOW_UP_ODDS_THRESHOLD},
    pet::{
        definition::{PetDefinitionId, PET_ADULTS},
        gen_pid, random_name, PetInstance, PetName, UniquePetId,
    },
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Default)]
pub struct Suiter {
    pub pet_def_id: PetDefinitionId,
    pub upid: UniquePetId,
    pub name: PetName,
    pub waiting: Duration,
}

impl Suiter {
    pub fn new_random(rng: &mut fastrand::Rng) -> Self {
        Self {
            pet_def_id: rng.choice(PET_ADULTS.iter()).cloned().unwrap(),
            upid: gen_pid(rng),
            name: random_name(rng),
            waiting: Duration::ZERO,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Default)]
pub struct SuiterSystem {
    pub suiter: Option<Suiter>,
    pub waiting_for_suiter: Duration,
}

impl SuiterSystem {
    pub fn suiter_waiting(&self) -> bool {
        self.suiter.is_some()
    }

    pub fn clear_suiter(&mut self) {
        self.suiter = None;
    }

    pub fn sim_tick(
        &mut self,
        delta: Duration,
        rng: &mut fastrand::Rng,
        pet: &PetInstance,
        sleeping: bool,
    ) {
        if let Some(suiter) = &mut self.suiter {
            suiter.waiting += delta;
        }

        if sleeping
            || (self.suiter.is_some() && (!pet.should_breed() || rng.f32() < SUITER_LEAVE_ODDS))
        {
            self.suiter = None;
        }

        if sleeping {
            return;
        }

        if pet.should_breed() && self.suiter.is_none() {
            self.waiting_for_suiter += delta;
            if passed_threshold_chance(rng, SUITER_SHOW_UP_ODDS_THRESHOLD, self.waiting_for_suiter)
            {
                self.suiter = Some(Suiter::new_random(rng));
            }
        }
    }
}
