use bincode::{Decode, Encode};

use crate::{
    death::DeathCause,
    pet::{definition::PetDefinitionId, PetInstance, PetName, UniquePetId},
    Timestamp,
};

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
