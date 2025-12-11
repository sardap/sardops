use bincode::{Decode, Encode};
use chrono::{Datelike, NaiveDate};
use fixedstr::str12;
use glam::Vec2;

use crate::{
    assets,
    display::{CENTER_X, ComplexRender, ComplexRenderOption},
    fonts,
    pet::definition::{PetDefinition, PetDefinitionId},
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum DeathCause {
    LightingStrike,
    Starvation,
    OldAge,
    ToxicShock,
    Leaving,
    Illness,
    Hypothermia,
}

impl DeathCause {
    pub const fn name(&self) -> &'static str {
        match self {
            DeathCause::LightingStrike => "Lighting",
            DeathCause::Starvation => "Starvation",
            DeathCause::OldAge => "Old age",
            DeathCause::ToxicShock => "Toxic shock",
            DeathCause::Leaving => "Left",
            DeathCause::Illness => "Sickness",
            DeathCause::Hypothermia => "Hypothermia",
        }
    }
}

pub struct GraveStone {
    pub pos: Vec2,
    pub name: str12,
    pub def_id: PetDefinitionId,
    pub cause: DeathCause,
    pub born: NaiveDate,
    pub died: NaiveDate,
}

impl Default for GraveStone {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            name: Default::default(),
            def_id: Default::default(),
            cause: DeathCause::LightingStrike,
            born: Default::default(),
            died: Default::default(),
        }
    }
}

impl GraveStone {
    pub fn new(
        pos: Vec2,
        name: str12,
        def_id: PetDefinitionId,
        born: NaiveDate,
        died: NaiveDate,
        cause: DeathCause,
    ) -> Self {
        Self {
            pos,
            name,
            def_id,
            born,
            died,
            cause,
        }
    }
}

impl ComplexRender for GraveStone {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        let mut top = self.pos.y - assets::IMAGE_GRAVESTONE.size.y as f32 / 2. + 20.;
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            &assets::IMAGE_GRAVESTONE,
            ComplexRenderOption::new().with_white().with_center(),
        );

        display.render_text_complex(
            Vec2::new(CENTER_X, top),
            "HERE LIES",
            ComplexRenderOption::new()
                .with_flip()
                .with_black()
                .with_font(&fonts::FONT_VARIABLE_SMALL)
                .with_center(),
        );
        top += 7.;

        display.render_text_complex(
            Vec2::new(CENTER_X, top),
            &self.name,
            ComplexRenderOption::new()
                .with_flip()
                .with_black()
                .with_font(&fonts::FONT_VARIABLE_SMALL)
                .with_center(),
        );
        top += 7.;

        display.render_text_complex(
            Vec2::new(CENTER_X, top),
            PetDefinition::get_by_id(self.def_id).name,
            ComplexRenderOption::new()
                .with_flip()
                .with_black()
                .with_font(&fonts::FONT_VARIABLE_SMALL)
                .with_center(),
        );
        top += 10.;

        display.render_text_complex(
            Vec2::new(CENTER_X, top),
            "Died from",
            ComplexRenderOption::new()
                .with_flip()
                .with_black()
                .with_font(&fonts::FONT_VARIABLE_SMALL)
                .with_center(),
        );
        top += 7.;

        display.render_text_complex(
            Vec2::new(CENTER_X, top),
            self.cause.name(),
            ComplexRenderOption::new()
                .with_flip()
                .with_black()
                .with_font(&fonts::FONT_VARIABLE_SMALL)
                .with_center(),
        );
        top += 10.;

        let str = fixedstr::str_format!(
            fixedstr::str12,
            "B{}/{:0>2}/{:0>2}",
            self.born.year() % 100,
            self.born.month(),
            self.born.day()
        );
        display.render_text_complex(
            Vec2::new(CENTER_X, top),
            &str,
            ComplexRenderOption::new()
                .with_flip()
                .with_black()
                .with_font(&fonts::FONT_VARIABLE_SMALL)
                .with_center(),
        );
        top += 7.;

        let str = fixedstr::str_format!(
            fixedstr::str12,
            "D{}/{:0>2}/{:0>2}",
            self.died.year() % 100,
            self.died.month(),
            self.died.day()
        );
        display.render_text_complex(
            Vec2::new(CENTER_X, top),
            &str,
            ComplexRenderOption::new()
                .with_flip()
                .with_black()
                .with_font(&fonts::FONT_VARIABLE_SMALL)
                .with_center(),
        );
    }
}

pub struct Threshold<T> {
    pub value: T,
    pub odds: f32,
}

impl<T> Threshold<T> {
    pub const fn new(value: T, odds: f32) -> Self {
        Self { value, odds }
    }
}

pub fn get_threshold_odds<T>(values: &[Threshold<T>], value: T) -> f32
where
    T: Ord,
{
    for threashold in values {
        if value < threashold.value {
            return threashold.odds;
        }
    }

    values[values.len() - 1].odds
}

pub fn passed_threshold_chance<T>(
    rng: &mut fastrand::Rng,
    values: &[Threshold<T>],
    current: T,
) -> bool
where
    T: Ord,
{
    let odds = get_threshold_odds(values, current);
    rng.f32() < odds
}
