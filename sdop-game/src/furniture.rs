use bincode::{Decode, Encode};
use chrono::NaiveDate;
use glam::Vec2;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    ROOM_TEMPTURE,
    assets::{self, Image},
    calendar::CalendarRender,
    clock::{AnalogueClockKind, AnalogueRenderClock, DigitalClockRender},
    display::{CENTER_X, ComplexRender, HEIGHT_F32, WIDTH_F32},
    fish_tank::FishTankRender,
    invetro_light::InvetroLightRender,
    scene::{SceneTickArgs, home_scene::HOME_SCENE_TOP_BORDER_RECT},
    sprite::BasicSprite,
    thermometer::{RenderThermometerDigital, RenderThermometerMercury},
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, EnumIter, Default)]
pub enum HomeFurnitureKind {
    #[default]
    None,
    DigitalClock,
    AnalogueClock,
    ThermometerMercury,
    ThermometerDigital,
    SpaceHeater,
    AirCon,
    FishTank,
    InvertroLight,
    Calendar,
    PaintingBranch,
    PaintingDude,
    PaintingMan,
    PaintingPc,
    PaintingSun,
}

impl HomeFurnitureKind {
    pub fn size(&self) -> Vec2 {
        match self {
            Self::None => Vec2::ZERO,
            Self::DigitalClock => DigitalClockRender::size(),
            Self::AnalogueClock => AnalogueClockKind::Clock21.size(),
            Self::ThermometerMercury => RenderThermometerMercury::size(),
            Self::ThermometerDigital => RenderThermometerDigital::size(),
            Self::SpaceHeater => assets::IMAGE_SPACE_HEATER.size_vec2(),
            Self::AirCon => assets::IMAGE_AIR_CONDITIONER.size_vec2(),
            Self::FishTank => FishTankRender::size(),
            Self::InvertroLight => InvetroLightRender::size(),
            Self::Calendar => CalendarRender::size(),
            Self::PaintingBranch => assets::IMAGE_PAINTING_BRANCH.size_vec2(),
            Self::PaintingDude => assets::IMAGE_PAINTING_DUDE.size_vec2(),
            Self::PaintingMan => assets::IMAGE_PAINTING_MAN.size_vec2(),
            Self::PaintingPc => assets::IMAGE_PAINTING_PC.size_vec2(),
            Self::PaintingSun => assets::IMAGE_PAINTING_SUN.size_vec2(),
        }
    }

    pub fn change(&self, change: isize) -> Self {
        let len = Self::iter().count() as isize;
        let current = Self::iter().position(|i| i == *self).unwrap_or(0) as isize;
        let next = (current + change).rem_euclid(len);

        Self::iter().nth(next as usize).unwrap_or_default()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct HomeLayout {
    pub top: HomeFurnitureKind,
    pub left: HomeFurnitureKind,
    pub right: HomeFurnitureKind,
}

impl HomeLayout {
    pub fn place(&mut self, location: HomeFurnitureLocation, kind: HomeFurnitureKind) {
        if self.top == kind {
            self.top = HomeFurnitureKind::None;
        }

        if self.left == kind {
            self.left = HomeFurnitureKind::None;
        }

        if self.right == kind {
            self.right = HomeFurnitureKind::None;
        }

        match location {
            HomeFurnitureLocation::Top => self.top = kind,
            HomeFurnitureLocation::Left => self.left = kind,
            HomeFurnitureLocation::Right => self.right = kind,
        }
    }

    pub fn furniture_present(&self, kind: HomeFurnitureKind) -> bool {
        [self.left, self.top, self.right].contains(&kind)
    }
}

impl Default for HomeLayout {
    fn default() -> Self {
        Self {
            top: HomeFurnitureKind::None,
            left: HomeFurnitureKind::None,
            right: HomeFurnitureKind::None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum HomeFurnitureLocation {
    #[default]
    Top,
    Left,
    Right,
}

impl HomeFurnitureLocation {
    pub const fn pos(&self) -> Vec2 {
        match self {
            HomeFurnitureLocation::Top => Vec2::new(CENTER_X, HOME_SCENE_TOP_BORDER_RECT.y2()),
            HomeFurnitureLocation::Left => Vec2::new(0., HEIGHT_F32 / 2.),
            HomeFurnitureLocation::Right => Vec2::new(WIDTH_F32, HEIGHT_F32 / 2.),
        }
    }

    pub const fn index(&self) -> usize {
        match self {
            HomeFurnitureLocation::Top => 0,
            HomeFurnitureLocation::Left => 1,
            HomeFurnitureLocation::Right => 2,
        }
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(HomeFurnitureLocation::Top),
            1 => Some(HomeFurnitureLocation::Left),
            2 => Some(HomeFurnitureLocation::Right),
            _ => None,
        }
    }
}

#[derive(Default)]
pub enum HomeFurnitureRender {
    #[default]
    None,
    DigitalClock(DigitalClockRender),
    AnalogueClock(AnalogueRenderClock),
    ThermometerMercury(RenderThermometerMercury),
    ThermometerDigital(RenderThermometerDigital),
    FishTank(FishTankRender),
    InvetroLight(InvetroLightRender),
    Calendar(CalendarRender),
    Sprite(BasicSprite),
}

impl HomeFurnitureRender {
    pub fn new(location: HomeFurnitureLocation, kind: HomeFurnitureKind) -> Self {
        let pos = location.pos()
            + match location {
                HomeFurnitureLocation::Top => Vec2::new(0., kind.size().y / 2.),
                HomeFurnitureLocation::Left => Vec2::new(kind.size().x / 2. + 1., 0.),
                HomeFurnitureLocation::Right => Vec2::new(-(kind.size().x / 2. + 1.), 0.),
            };

        match kind {
            HomeFurnitureKind::None => HomeFurnitureRender::None,
            HomeFurnitureKind::DigitalClock => {
                HomeFurnitureRender::DigitalClock(DigitalClockRender::new(pos, Default::default()))
            }
            HomeFurnitureKind::AnalogueClock => HomeFurnitureRender::AnalogueClock(
                AnalogueRenderClock::new(AnalogueClockKind::Clock21, pos, Default::default()),
            ),
            HomeFurnitureKind::ThermometerMercury => HomeFurnitureRender::ThermometerMercury(
                RenderThermometerMercury::new(pos, ROOM_TEMPTURE),
            ),
            HomeFurnitureKind::ThermometerDigital => HomeFurnitureRender::ThermometerDigital(
                RenderThermometerDigital::new(pos, ROOM_TEMPTURE),
            ),
            HomeFurnitureKind::SpaceHeater => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_SPACE_HEATER))
            }
            HomeFurnitureKind::AirCon => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_AIR_CONDITIONER))
            }
            HomeFurnitureKind::FishTank => HomeFurnitureRender::FishTank(FishTankRender::new(pos)),
            HomeFurnitureKind::InvertroLight => {
                HomeFurnitureRender::InvetroLight(InvetroLightRender::new(pos, 50, location))
            }
            HomeFurnitureKind::Calendar => {
                HomeFurnitureRender::Calendar(CalendarRender::new(pos, NaiveDate::default()))
            }
            HomeFurnitureKind::PaintingBranch => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_BRANCH))
            }
            HomeFurnitureKind::PaintingDude => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_DUDE))
            }
            HomeFurnitureKind::PaintingMan => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_MAN))
            }
            HomeFurnitureKind::PaintingPc => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_PC))
            }
            HomeFurnitureKind::PaintingSun => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_SUN))
            }
        }
    }

    pub fn size(&self) -> Vec2 {
        match self {
            Self::None => Vec2::ZERO,
            Self::DigitalClock(_) => HomeFurnitureKind::DigitalClock.size(),
            Self::AnalogueClock(_) => HomeFurnitureKind::AnalogueClock.size(),
            Self::ThermometerMercury(_) => HomeFurnitureKind::ThermometerMercury.size(),
            Self::ThermometerDigital(_) => HomeFurnitureKind::ThermometerDigital.size(),
            Self::FishTank(_) => HomeFurnitureKind::FishTank.size(),
            Self::Calendar(_) => CalendarRender::size(),
            Self::InvetroLight(_) => HomeFurnitureKind::InvertroLight.size(),
            Self::Sprite(basic_sprite) => basic_sprite.image.size_vec2(),
        }
    }

    pub fn tick(&mut self, args: &mut SceneTickArgs) {
        match self {
            Self::None => {}
            Self::DigitalClock(digital_clock_render) => {
                digital_clock_render.update_time(&args.timestamp.inner().time());
            }
            Self::AnalogueClock(analogue_render_clock) => {
                analogue_render_clock.update_time(&args.timestamp.inner().time());
            }
            Self::ThermometerMercury(render) => {
                render.temperature = args.input.temperature();
            }
            Self::ThermometerDigital(render) => {
                render.temperature = args.input.temperature();
            }
            Self::FishTank(fishtank_render) => {
                while fishtank_render.fish_count() < args.game_ctx.home_fish_tank.count() {
                    fishtank_render.add_fish(
                        &mut args.game_ctx.rng,
                        args.game_ctx.home_fish_tank.fish[fishtank_render.fish_count()] as f32,
                    );
                }

                fishtank_render.tick(args.delta, &mut args.game_ctx.rng);
            }
            Self::Calendar(calendar) => {
                calendar.set_date(args.timestamp.inner().date());
            }
            Self::InvetroLight(_) => {}
            Self::Sprite(_) => {}
        }
    }
}

impl ComplexRender for HomeFurnitureRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        match self {
            Self::None => {}
            Self::DigitalClock(digital_clock_render) => {
                display.render_complex(digital_clock_render)
            }
            Self::AnalogueClock(analogue_render_clock) => {
                display.render_complex(analogue_render_clock)
            }
            Self::ThermometerMercury(thermometer_mercury_render) => {
                display.render_complex(thermometer_mercury_render);
            }
            Self::ThermometerDigital(thermometer_digital_render) => {
                display.render_complex(thermometer_digital_render);
            }
            Self::FishTank(fishtank_render) => display.render_complex(fishtank_render),
            Self::Calendar(calendar_render) => display.render_complex(calendar_render),
            Self::Sprite(basic_sprite) => display.render_sprite(basic_sprite),
            // We want these to render later so this is a hack
            Self::InvetroLight(_) => {}
        }
    }
}
