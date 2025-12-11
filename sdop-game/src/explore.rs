use core::{ops::Range, time::Duration};

use crate::{
    Timestamp,
    assets::{self, StaticImage},
    items::{Inventory, ItemKind},
    money::Money,
    pet::PetInstance,
};
use bincode::{Decode, Encode};

include!(concat!(env!("OUT_DIR"), "/dist_locations.rs"));

pub type ExploreSkill = i32;

const MAX_REWARD_ITEMS_LOCATION: usize = 10;
const CHECK_INTERVAL: Duration = Duration::from_millis(100);
const PASSED_THRESHOLD: f32 = 0.5;

pub struct ItemReward {
    item: ItemKind,
    odds: f32,
}
impl ItemReward {
    pub const fn new(item: ItemKind, odds: f32) -> Self {
        Self { item, odds }
    }
}

pub struct LocationRewards {
    money: Range<Money>,
    items: &'static [ItemReward],
}

impl LocationRewards {
    pub const fn new(money: Range<Money>, items: &'static [ItemReward]) -> Self {
        Self { money, items }
    }
}

pub struct Location {
    pub id: usize,
    pub name: &'static str,
    pub length: Duration,
    pub cooldown: Duration,
    pub difficulty: ExploreSkill,
    pub rewards: LocationRewards,
    pub cover: StaticImage,
    pub activities: &'static [&'static str],
}

impl Location {
    pub const fn new(
        id: usize,
        name: &'static str,
        length: Duration,
        cooldown: Duration,
        difficulty: ExploreSkill,
        rewards: LocationRewards,
        cover: StaticImage,
        activities: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            name,
            cooldown,
            length,
            difficulty,
            rewards,
            cover,
            activities,
        }
    }

    pub const fn total_checks(&self) -> u64 {
        (self.length.as_millis() / CHECK_INTERVAL.as_millis()) as u64
    }
}

pub const LOCATION_UNKNOWN: Location = Location::new(
    0,
    "UNKNOWN",
    Duration::ZERO,
    Duration::from_days(9999),
    9999,
    LocationRewards::new(0..1, &[]),
    assets::IMAGE_LOCATION_UNKOWN,
    &[],
);

pub const fn get_location(id: usize) -> &'static Location {
    if id >= LOCATIONS.len() {
        return &LOCATIONS[0];
    }
    LOCATIONS[id]
}

pub struct LocationHistoryIter<'a> {
    current: usize,
    history: &'a ExploreHistory,
}

impl<'a> LocationHistoryIter<'a> {
    pub fn new(current: usize, history: &'a ExploreHistory) -> Self {
        Self { current, history }
    }
}

impl<'a> Iterator for LocationHistoryIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        for i in (self.current + 1)..LOCATION_COUNT {
            if self.history.location_history[i].unlocked {
                self.current = i;
                return Some(self.current);
            }
        }

        None
    }
}

impl<'a> DoubleEndedIterator for LocationHistoryIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        for i in (0..self.current).rev() {
            if self.history.location_history[i].unlocked {
                self.current = i;
                return Some(self.current);
            }
        }

        None
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct LocationHistory {
    pub unlocked: bool,
    pub runs: u16,
    pub successful: u16,
    pub last_ran: Timestamp,
}

impl Default for LocationHistory {
    fn default() -> Self {
        Self {
            unlocked: true,
            runs: 0,
            successful: 0,
            last_ran: Timestamp::default(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct ExploreHistory {
    pub location_history: [LocationHistory; LOCATION_COUNT],
    pub skill: ExploreSkill,
}

impl Default for ExploreHistory {
    fn default() -> Self {
        let mut result = Self {
            location_history: Default::default(),
            skill: 100,
        };

        result.location_history[1].unlocked = true;

        result
    }
}

impl ExploreHistory {
    pub fn get_mut_by_id(&mut self, id: usize) -> &mut LocationHistory {
        &mut self.location_history[id]
    }

    pub fn get_by_id(&self, id: usize) -> &LocationHistory {
        &self.location_history[id]
    }

    pub fn unlocked(&self, id: usize) -> bool {
        return id != 0 && self.get_by_id(id).unlocked;
    }
}

pub struct ExploreDetailedResult {
    pub location: &'static Location,
    pub passed: u32,
    pub earnings: Money,
    pub items: heapless::Vec<ItemKind, MAX_REWARD_ITEMS_LOCATION>,
}

impl Default for ExploreDetailedResult {
    fn default() -> Self {
        Self::new(&LOCATION_UNKNOWN, 0)
    }
}

impl ExploreDetailedResult {
    pub fn new(location: &'static Location, passed: u32) -> Self {
        Self {
            location,
            passed,
            earnings: Default::default(),
            items: Default::default(),
        }
    }

    pub const fn percent_passed(&self) -> f32 {
        let x = self.location.total_checks() as f32;
        self.passed as f32 / x
    }

    pub const fn completed(&self) -> bool {
        self.percent_passed() > PASSED_THRESHOLD
    }
}

const PLACEHOLDER_ACTIVTY: &'static str = "????";

pub struct ExploreSystem {
    current: Option<&'static Location>,
    current_activity: &'static str,
    elapsed: Duration,
    passes: u32,
    last_result: ExploreDetailedResult,
}

impl Default for ExploreSystem {
    fn default() -> Self {
        Self {
            current: None,
            current_activity: PLACEHOLDER_ACTIVTY,
            elapsed: Duration::ZERO,
            passes: 0,
            last_result: Default::default(),
        }
    }
}

impl ExploreSystem {
    pub fn sim_tick(
        &mut self,
        delta: Duration,
        timestamp: &Timestamp,
        rng: &mut fastrand::Rng,
        pet: &mut PetInstance,
        inventory: &mut Inventory,
        wallet: &mut Money,
    ) {
        let current = match self.current {
            Some(current) => current,
            None => return,
        };

        if self.current_activity == PLACEHOLDER_ACTIVTY {
            self.current_activity = rng
                .choice(self.current_location().activities)
                .unwrap_or(&PLACEHOLDER_ACTIVTY);
        }

        self.elapsed += delta;
        if self.elapsed > current.length {
            let mut result = ExploreDetailedResult::new(current, self.passes);
            if result.completed() {
                let percent_passed = result.percent_passed();

                for reward in current.rewards.items {
                    if rng.f32() < reward.odds * percent_passed {
                        let _ = result.items.push(reward.item);
                        inventory.add_item(reward.item, 1);
                    }
                }

                result.earnings = (rng.i32(current.rewards.money.start..current.rewards.money.end)
                    as f32
                    * percent_passed) as Money;
                *wallet += result.earnings;
            }

            {
                let history = pet.explore.get_mut_by_id(current.id);
                history.last_ran = timestamp.clone();
                history.runs += 1;
                history.successful += if result.completed() { 1 } else { 0 };
            }

            self.last_result = result;
            self.passes = 0;
            self.elapsed = Duration::ZERO;
            self.current = None;
        } else {
            let since_check = Duration::from_millis(
                (self.elapsed.as_millis() % CHECK_INTERVAL.as_millis()) as u64,
            );
            if since_check == Duration::ZERO {
                self.current_activity = rng
                    .choice(self.current_location().activities)
                    .unwrap_or(&PLACEHOLDER_ACTIVTY);

                let skill = pet.explore_skill();
                let odds = rng.i32((skill / 2)..=skill);
                let location_odds = rng.i32(0..current.difficulty);
                if odds > location_odds {
                    self.passes += 1;
                }
            }
        }
    }

    pub fn current_percent_passed(&self) -> f32 {
        if self.elapsed < CHECK_INTERVAL {
            return 1.;
        }
        let total_odds = (self.elapsed.as_millis() / CHECK_INTERVAL.as_millis()) as f32;
        self.passes as f32 / total_odds
    }

    pub fn start_exploring(&mut self, location_id: usize) {
        self.current = Some(get_location(location_id));
    }

    pub fn currently_exploring(&self) -> bool {
        self.current.is_some()
    }

    pub fn current_location(&self) -> &Location {
        self.current.unwrap_or(&LOCATIONS[0])
    }

    pub fn current_activity(&self) -> &'static str {
        self.current_activity
    }

    pub fn percent_complete(&self) -> f32 {
        match self.current {
            Some(location) => self.elapsed.as_millis_f32() / location.length.as_millis_f32(),
            None => 0.,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn current_check(&self) -> u32 {
        (self.elapsed.as_millis() / CHECK_INTERVAL.as_millis()) as u32
    }

    pub fn last_result(&self) -> &ExploreDetailedResult {
        &self.last_result
    }

    pub fn save(&self) -> ExploreSystemSave {
        ExploreSystemSave {
            current: self.current.map(|i| i.id),
            elapsed: self.elapsed,
            passes: self.passes,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct ExploreSystemSave {
    current: Option<usize>,
    elapsed: Duration,
    passes: u32,
}

impl Default for ExploreSystemSave {
    fn default() -> Self {
        Self {
            current: Default::default(),
            elapsed: Default::default(),
            passes: Default::default(),
        }
    }
}

impl From<ExploreSystemSave> for ExploreSystem {
    fn from(value: ExploreSystemSave) -> Self {
        ExploreSystem {
            current: value
                .current
                .map(|i| LOCATIONS.get(i).unwrap_or(&LOCATIONS[0]))
                .copied(),
            current_activity: PLACEHOLDER_ACTIVTY,
            elapsed: value.elapsed,
            passes: value.passes,
            last_result: Default::default(),
        }
    }
}
