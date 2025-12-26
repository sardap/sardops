use bincode::{Decode, Encode};
use chrono::NaiveDate;
use glam::{IVec2, Vec2};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    ROOM_TEMPTURE,
    alarm::AlarmRender,
    anime::HasAnime,
    assets::{self, Image},
    calendar::CalendarRender,
    clock::{AnalogueClockKind, AnalogueRenderClock, DigitalClockRender},
    display::{CENTER_X, ComplexRender, HEIGHT_F32, WIDTH_F32},
    fish_tank::FishTankRender,
    geo::ivec_to_vec2,
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
    Alarm,
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
    PaintingMallsBalls,
}

impl HomeFurnitureKind {
    pub fn size(&self) -> IVec2 {
        match self {
            Self::None => IVec2::ZERO,
            Self::DigitalClock => DigitalClockRender::size(),
            Self::AnalogueClock => AnalogueClockKind::Clock21.size(),
            Self::Alarm => AlarmRender::size(),
            Self::ThermometerMercury => RenderThermometerMercury::size(),
            Self::ThermometerDigital => RenderThermometerDigital::size(),
            Self::SpaceHeater => assets::IMAGE_SPACE_HEATER.isize,
            Self::AirCon => assets::IMAGE_AIR_CONDITIONER.isize,
            Self::FishTank => FishTankRender::size(),
            Self::InvertroLight => InvetroLightRender::size(),
            Self::Calendar => CalendarRender::size(),
            Self::PaintingBranch => assets::IMAGE_PAINTING_BRANCH.isize,
            Self::PaintingDude => assets::IMAGE_PAINTING_DUDE.isize,
            Self::PaintingMan => assets::IMAGE_PAINTING_MAN.isize,
            Self::PaintingPc => assets::IMAGE_PAINTING_PC.isize,
            Self::PaintingSun => assets::IMAGE_PAINTING_SUN.isize,
            Self::PaintingMallsBalls => assets::IMAGE_PAINTING_MALLS_BALLS.isize,
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
            HomeFurnitureLocation::Top => {
                Vec2::new(CENTER_X, HOME_SCENE_TOP_BORDER_RECT.y2() as f32)
            }
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
    Alarm(AlarmRender),
    ThermometerMercury(RenderThermometerMercury),
    ThermometerDigital(RenderThermometerDigital),
    FishTank(FishTankRender),
    InvetroLight(InvetroLightRender),
    Calendar(CalendarRender),
    Sprite(BasicSprite),
}

impl HomeFurnitureRender {
    pub type Kind = HomeFurnitureKind;

    pub fn new(location: HomeFurnitureLocation, kind: Self::Kind) -> Self {
        let pos = location.pos()
            + ivec_to_vec2(match location {
                HomeFurnitureLocation::Top => IVec2::new(0, kind.size().y / 2),
                HomeFurnitureLocation::Left => IVec2::new(kind.size().x / 2 + 1, 0),
                HomeFurnitureLocation::Right => IVec2::new(-(kind.size().x / 2 + 1), 0),
            });

        match kind {
            Self::Kind::None => HomeFurnitureRender::None,
            Self::Kind::DigitalClock => {
                HomeFurnitureRender::DigitalClock(DigitalClockRender::new(pos, Default::default()))
            }
            Self::Kind::AnalogueClock => HomeFurnitureRender::AnalogueClock(
                AnalogueRenderClock::new(AnalogueClockKind::Clock21, pos, Default::default()),
            ),
            Self::Kind::Alarm => HomeFurnitureRender::Alarm(AlarmRender::new(pos)),
            Self::Kind::ThermometerMercury => HomeFurnitureRender::ThermometerMercury(
                RenderThermometerMercury::new(pos, ROOM_TEMPTURE),
            ),
            Self::Kind::ThermometerDigital => HomeFurnitureRender::ThermometerDigital(
                RenderThermometerDigital::new(pos, ROOM_TEMPTURE),
            ),
            Self::Kind::SpaceHeater => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_SPACE_HEATER))
            }
            Self::Kind::AirCon => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_AIR_CONDITIONER))
            }
            Self::Kind::FishTank => HomeFurnitureRender::FishTank(FishTankRender::new(pos)),
            Self::Kind::InvertroLight => {
                HomeFurnitureRender::InvetroLight(InvetroLightRender::new(pos, 50, location))
            }
            Self::Kind::Calendar => {
                HomeFurnitureRender::Calendar(CalendarRender::new(pos, NaiveDate::default()))
            }
            Self::Kind::PaintingBranch => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_BRANCH))
            }
            Self::Kind::PaintingDude => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_DUDE))
            }
            Self::Kind::PaintingMan => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_MAN))
            }
            Self::Kind::PaintingPc => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_PC))
            }
            Self::Kind::PaintingSun => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_SUN))
            }
            Self::Kind::PaintingMallsBalls => HomeFurnitureRender::Sprite(BasicSprite::new(
                pos,
                &assets::IMAGE_PAINTING_MALLS_BALLS,
            )),
        }
    }

    pub fn size(&self) -> IVec2 {
        match self {
            Self::None => IVec2::ZERO,
            Self::DigitalClock(_) => Self::Kind::DigitalClock.size(),
            Self::AnalogueClock(_) => Self::Kind::AnalogueClock.size(),
            Self::Alarm(_) => AlarmRender::size(),
            Self::ThermometerMercury(_) => Self::Kind::ThermometerMercury.size(),
            Self::ThermometerDigital(_) => Self::Kind::ThermometerDigital.size(),
            Self::FishTank(_) => Self::Kind::FishTank.size(),
            Self::Calendar(_) => CalendarRender::size(),
            Self::InvetroLight(_) => Self::Kind::InvertroLight.size(),
            Self::Sprite(basic_sprite) => basic_sprite.image.size_ivec2(),
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
            Self::Alarm(render) => {
                render.anime().tick(args.delta);
                render.set_rining(args.game_ctx.alarm.should_be_rining());
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
            Self::Alarm(render) => {
                display.render_sprite(render);
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
