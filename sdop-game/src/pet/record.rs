use bincode::{Decode, Encode};

use crate::{
    death::DeathCause,
    pet::{definition::PetDefinitionId, PetInstance, PetName, UniquePetId},
    Timestamp,
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
        }
    }
}

pub const PET_HISTORY_ENTRIES: usize = 20;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct PetHistory {
    top: usize,
    entires: [Option<PetRecord>; PET_HISTORY_ENTRIES],
}

impl PetHistory {
    pub fn add(&mut self, entry: PetRecord) {
        self.entires[self.top] = Some(entry);
        self.top += 1;
    }
}

impl Default for PetHistory {
    fn default() -> Self {
        Self {
            top: 0,
            entires: Default::default(),
        }
    }
}
