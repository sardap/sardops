use core::time::Duration;

use bincode::{Decode, Encode};

use crate::{
    Timestamp,
    death::DeathCause,
    pet::{
        LifeStageHistory, PetInstance, PetName, PetParents, UniquePetId,
        definition::{PetDefinition, PetDefinitionId},
    },
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct PetRecord {
    pub upid: UniquePetId,
    pub def_id: PetDefinitionId,
    pub name: PetName,
    pub born: Timestamp,
    pub death: Timestamp,
    pub extra_weight: f32,
    pub died_of: DeathCause,
    pub parents: Option<PetParents>,
    pub stage_history: LifeStageHistory,
}

impl PetRecord {
    pub fn from_pet_instance(
        pet_instance: &PetInstance,
        time_of_death: Timestamp,
        cause_of_death: DeathCause,
    ) -> Self {
        Self {
            upid: pet_instance.upid,
            def_id: pet_instance.def_id,
            name: pet_instance.name,
            born: pet_instance.born,
            death: time_of_death,
            extra_weight: pet_instance.extra_weight,
            died_of: cause_of_death,
            parents: pet_instance.parents,
            stage_history: pet_instance.life_stage_history,
        }
    }

    pub fn age(&self) -> Duration {
        self.death - self.born
    }

    pub fn weight(&self) -> f32 {
        self.extra_weight + PetDefinition::get_by_id(self.def_id).base_weight
    }
}

pub const PET_HISTORY_ENTRIES: usize = 20;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode, Default)]
pub struct PetHistory {
    top: usize,
    entires: [Option<PetRecord>; PET_HISTORY_ENTRIES],
}

impl PetHistory {
    pub fn add(&mut self, entry: PetRecord) {
        self.entires[self.top] = Some(entry);
        self.top += 1;
    }

    pub fn get_by_index(&self, index: usize) -> Option<&PetRecord> {
        self.entires.get(index)?.as_ref()
    }

    pub fn get_by_upid(&self, upid: UniquePetId) -> Option<&PetRecord> {
        for entry in &self.entires {
            if let Some(entry) = entry {
                if entry.upid == upid {
                    return Some(entry);
                }
            }
        }

        None
    }

    pub fn count(&self) -> usize {
        self.top
    }
}
