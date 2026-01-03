use core::{ops::Range, time::Duration};

use crate::{
    Timestamp,
    assets::{self, StaticImage},
    game_consts::EXPLORE_MONEY_RESET_TIME,
    items::{Inventory, ItemKind},
    money::Money,
    pet::{LifeStage, PetInstance},
};
use bincode::{Decode, Encode};
use sdop_common::LifeStageMask;

include!(concat!(env!("OUT_DIR"), "/dist_locations.rs"));

pub type ExploreSkill = i32;

const MAX_REWARD_ITEMS_LOCATION: usize = 10;
const PHRASE_UPDATE_INTERVAL: Duration = Duration::from_secs(60);
const PASSED_THRESHOLD: f32 = 0.5;
const CHECKS_PER_LOCATION: u32 = 10;

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
    pub item: ItemKind,
    pub ls_mask: LifeStageMask,
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
        item: ItemKind,
        life_stages: &'static [LifeStage],
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
            item,
            ls_mask: LifeStage::create_bitmask(life_stages),
        }
    }

    pub const fn total_checks(&self) -> u32 {
        CHECKS_PER_LOCATION
    }

    pub const fn check_interval(&self) -> Duration {
        Duration::from_millis((self.length.as_millis() as u32 / CHECKS_PER_LOCATION) as u64)
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
    ItemKind::None,
    &[],
);

pub const fn get_location(id: usize) -> &'static Location {
    if id >= LOCATIONS.len() {
        return &LOCATION_UNKNOWN;
    }
    LOCATIONS[id]
}

pub struct LocationHistoryIter<'a> {
    current: usize,
    history: &'a ExploreHistory,
    inventory: &'a Inventory,
}

impl<'a> LocationHistoryIter<'a> {
    pub type Item = usize;

    pub fn new(current: usize, history: &'a ExploreHistory, inventory: &'a Inventory) -> Self {
        Self {
            current,
            history,
            inventory,
        }
    }

    pub fn first(&mut self) -> Option<Self::Item> {
        if self.inventory.has_item(get_location(self.current).item) {
            return Some(self.current);
        }

        loop {
            let next = self.next();
            if let Some(next) = next {
                return Some(next);
            } else {
                return None;
            }
        }
    }
}

impl<'a> Iterator for LocationHistoryIter<'a> {
    type Item = LocationHistoryIter<'a>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        for i in (self.current + 1)..LOCATION_COUNT {
            if self.inventory.has_item(get_location(i).item) {
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
            if self.inventory.has_item(get_location(i).item) {
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
    pub runs: u16,
    pub successful: u16,
    pub last_ran: Timestamp,
    pub last_money_run: Timestamp,
    pub running_money_earned: Money,
}

impl Default for LocationHistory {
    fn default() -> Self {
        Self {
            runs: 0,
            successful: 0,
            last_ran: Timestamp::default(),
            last_money_run: Timestamp::default(),
            running_money_earned: 0,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct ExploreHistory {
    pub location_history: [LocationHistory; LOCATION_COUNT],
    pub bonus_skill: ExploreSkill,
}

impl Default for ExploreHistory {
    fn default() -> Self {
        Self {
            location_history: Default::default(),
            bonus_skill: 0,
        }
    }
}

impl ExploreHistory {
    pub fn get_mut_by_id(&mut self, id: usize) -> &mut LocationHistory {
        &mut self.location_history[id]
    }

    pub fn get_by_id(&self, id: usize) -> &LocationHistory {
        &self.location_history[id]
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
    until_check: Duration,
    last_result: ExploreDetailedResult,
}

impl Default for ExploreSystem {
    fn default() -> Self {
        Self {
            current: None,
            current_activity: PLACEHOLDER_ACTIVTY,
            elapsed: Duration::ZERO,
            passes: 0,
            until_check: Duration::ZERO,
            last_result: Default::default(),
        }
    }
}

impl ExploreSystem {
    pub fn sim_tick(
        &mut self,
        delta: Duration,
        now: &Timestamp,
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
            // Update money run
            {
                let history = pet.explore.get_mut_by_id(current.id);
                if (*now - history.last_money_run) > EXPLORE_MONEY_RESET_TIME {
                    history.last_money_run = *now;
                    history.running_money_earned = 0;
                }
            }

            if result.completed() {
                let percent_passed = result.percent_passed();

                for reward in current.rewards.items {
                    if rng.f32() < reward.odds * percent_passed {
                        if inventory.add_item(reward.item, 1) {
                            let _ = result.items.push(reward.item);
                        }
                    }
                }

                result.earnings = {
                    let raw = (rng.i32(current.rewards.money.start..current.rewards.money.end)
                        as f32
                        * percent_passed) as Money;

                    let history = pet.explore.get_by_id(current.id);

                    let max_earn =
                        (current.rewards.money.end - history.running_money_earned).max(0);

                    raw.min(max_earn)
                };
                *wallet += result.earnings;
            }

            {
                let history = pet.explore.get_mut_by_id(current.id);
                history.last_ran = now.clone();
                history.runs += 1;
                history.successful += if result.completed() { 1 } else { 0 };
                history.running_money_earned += result.earnings;
            }

            self.last_result = result;
            self.passes = 0;
            self.until_check = Duration::ZERO;
            self.elapsed = Duration::ZERO;
            self.current = None;
        } else {
            self.until_check += delta;

            if self.until_check > current.check_interval() {
                // Get left overs
                self.until_check = self.until_check - current.check_interval();
                let skill = pet.explore_skill();
                let odds = rng.i32((skill / 4)..=skill);
                let location_odds = rng.i32(0..current.difficulty);
                if odds > location_odds {
                    self.passes += 1;
                }
            }

            let since_phrase = Duration::from_millis(
                (self.elapsed.as_millis() % PHRASE_UPDATE_INTERVAL.as_millis()) as u64,
            );
            if since_phrase == Duration::ZERO {
                self.current_activity = rng
                    .choice(self.current_location().activities)
                    .unwrap_or(&PLACEHOLDER_ACTIVTY);
            }
        }
    }

    pub fn current_percent_passed(&self) -> f32 {
        let check_interval = self.current.unwrap_or(&LOCATION_UNKNOWN).check_interval();
        if self.elapsed < check_interval {
            return 1.;
        }

        let total_odds = (self.elapsed.as_millis() / check_interval.as_millis()) as f32;
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
        (self.elapsed.as_millis()
            / self
                .current
                .unwrap_or(&LOCATION_UNKNOWN)
                .check_interval()
                .as_millis()) as u32
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
            until_check: Duration::ZERO,
            last_result: Default::default(),
        }
    }
}
