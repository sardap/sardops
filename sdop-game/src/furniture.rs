use bincode::{Decode, Encode};
use glam::Vec2;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    assets::{self, Image},
    clock::{AnalogueClockKind, AnalogueRenderClock, DigitalClockRender},
    display::{CENTER_X, ComplexRender, HEIGHT_F32, WIDTH_F32},
    fish_tank::FishTankRender,
    invetro_light::InvetroLightRender,
    scene::{SceneTickArgs, home_scene::HOME_SCENE_TOP_BORDER_RECT},
    sprite::BasicSprite,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, EnumIter)]
pub enum HomeFurnitureKind {
    None,
    DigitalClock,
    AnalogueClock,
    FishTank,
    InvertroLight,
    PaintingBranch,
    PaintingDude,
    PaintingMan,
    PaintingPc,
    PaintingSun,
}

impl HomeFurnitureKind {
    pub fn size(&self) -> Vec2 {
        match self {
            HomeFurnitureKind::None => Vec2::ZERO,
            HomeFurnitureKind::DigitalClock => DigitalClockRender::size(),
            HomeFurnitureKind::AnalogueClock => AnalogueClockKind::Clock21.size(),
            HomeFurnitureKind::FishTank => FishTankRender::size(),
            HomeFurnitureKind::InvertroLight => InvetroLightRender::size(),
            HomeFurnitureKind::PaintingBranch => assets::IMAGE_PAINTING_BRANCH.size_vec2(),
            HomeFurnitureKind::PaintingDude => assets::IMAGE_PAINTING_DUDE.size_vec2(),
            HomeFurnitureKind::PaintingMan => assets::IMAGE_PAINTING_MAN.size_vec2(),
            HomeFurnitureKind::PaintingPc => assets::IMAGE_PAINTING_PC.size_vec2(),
            HomeFurnitureKind::PaintingSun => assets::IMAGE_PAINTING_SUN.size_vec2(),
        }
    }

    pub fn change(&self, change: isize) -> Self {
        let len = Self::iter().count() as isize;
        let current = Self::iter().position(|i| i == *self).unwrap_or(0) as isize;
        let next = (current + change).rem_euclid(len);

        Self::iter().nth(next as usize).unwrap_or_default()
    }
}

impl Default for HomeFurnitureKind {
    fn default() -> Self {
        Self::None
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HomeFurnitureLocation {
    Top,
    Left,
    Right,
}

impl Default for HomeFurnitureLocation {
    fn default() -> Self {
        Self::Top
    }
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

pub enum HomeFurnitureRender {
    None,
    DigitalClock(DigitalClockRender),
    AnalogueClock(AnalogueRenderClock),
    FishTank(FishTankRender),
    InvetroLight(InvetroLightRender),
    Sprite(BasicSprite),
}

impl Default for HomeFurnitureRender {
    fn default() -> Self {
        Self::None
    }
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
            HomeFurnitureKind::FishTank => HomeFurnitureRender::FishTank(FishTankRender::new(pos)),
            HomeFurnitureKind::InvertroLight => {
                HomeFurnitureRender::InvetroLight(InvetroLightRender::new(pos, 50, location))
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
            HomeFurnitureRender::None => Vec2::ZERO,
            HomeFurnitureRender::DigitalClock(_) => HomeFurnitureKind::DigitalClock.size(),
            HomeFurnitureRender::AnalogueClock(_) => HomeFurnitureKind::AnalogueClock.size(),
            HomeFurnitureRender::FishTank(_) => HomeFurnitureKind::FishTank.size(),
            HomeFurnitureRender::InvetroLight(_) => HomeFurnitureKind::InvertroLight.size(),
            HomeFurnitureRender::Sprite(basic_sprite) => basic_sprite.image.size_vec2(),
        }
    }

    pub fn tick(&mut self, args: &mut SceneTickArgs) {
        match self {
            HomeFurnitureRender::None => {}
            HomeFurnitureRender::DigitalClock(digital_clock_render) => {
                digital_clock_render.update_time(&args.timestamp.inner().time());
            }
            HomeFurnitureRender::AnalogueClock(analogue_render_clock) => {
                analogue_render_clock.update_time(&args.timestamp.inner().time());
            }
            HomeFurnitureRender::FishTank(fishtank_render) => {
                while fishtank_render.fish_count() < args.game_ctx.home_fish_tank.count() {
                    fishtank_render.add_fish(
                        &mut args.game_ctx.rng,
                        args.game_ctx.home_fish_tank.fish[fishtank_render.fish_count()] as f32,
                    );
                }

                fishtank_render.tick(args.delta, &mut args.game_ctx.rng);
            }
            HomeFurnitureRender::InvetroLight(_) => {}
            HomeFurnitureRender::Sprite(_) => {}
        }
    }
}

impl ComplexRender for HomeFurnitureRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        match self {
            HomeFurnitureRender::None => {}
            HomeFurnitureRender::DigitalClock(digital_clock_render) => {
                display.render_complex(digital_clock_render)
            }
            HomeFurnitureRender::AnalogueClock(analogue_render_clock) => {
                display.render_complex(analogue_render_clock)
            }
            HomeFurnitureRender::FishTank(fishtank_render) => {
                display.render_complex(fishtank_render)
            }
            // We want these to render later so this is a hack
            HomeFurnitureRender::InvetroLight(_) => {}
            HomeFurnitureRender::Sprite(basic_sprite) => display.render_sprite(basic_sprite),
        }
    }
}
