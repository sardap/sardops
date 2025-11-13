use core::time::Duration;

use fixedstr::{str32, str_format};
use glam::Vec2;

use crate::{
    display::{ComplexRenderOption, GameDisplay, CENTER_VEC, CENTER_X},
    money::Money,
    pet::{
        definition::{PetAnimationSet, PetDefinitionId},
        render::PetRender,
    },
    scene::{home_scene::HomeScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
    sounds::{SongPlayOptions, SONG_FAN_FARE, SONG_LOST},
    Timestamp,
};

enum State {
    Intro,
    ShowingTotal,
}

pub struct MgFanFareScene {
    won: bool,
    money: Money,
    pet_render: PetRender,
    start_time: Timestamp,
    state: State,
    show_earned: bool,
    flash_duration: Duration,
}

impl MgFanFareScene {
    pub fn new(won: bool, money: Money, pet_def_id: PetDefinitionId) -> Self {
        Self {
            won,
            money,
            pet_render: PetRender::new(pet_def_id),
            start_time: Timestamp::default(),
            state: State::Intro,
            show_earned: true,
            flash_duration: Duration::ZERO,
        }
    }
}

impl Scene for MgFanFareScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.set_animation(if self.won {
            PetAnimationSet::Happy
        } else {
            PetAnimationSet::Sad
        });
        self.pet_render.pos = CENTER_VEC;
        self.start_time = args.timestamp;

        args.game_ctx.sound_system.push_song(
            if self.won { SONG_FAN_FARE } else { SONG_LOST },
            SongPlayOptions::new().with_effect(),
        );
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
        args.game_ctx.money += self.money;
        args.game_ctx.pet.played_game();
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.pet_render.tick(args.delta);

        match self.state {
            State::Intro => {
                let elapsed = args.timestamp - self.start_time;
                if elapsed > Duration::from_secs(3) {
                    self.start_time = args.timestamp;
                    self.state = State::ShowingTotal;
                }
            }
            State::ShowingTotal => {
                let elapsed = args.timestamp - self.start_time;
                if elapsed > Duration::from_secs(2) {
                    return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
                }
            }
        }

        self.flash_duration += args.delta;
        if self.show_earned {
            if self.flash_duration > Duration::from_millis(300) {
                self.show_earned = false;
                self.flash_duration = Duration::ZERO;
            }
        } else if self.flash_duration > Duration::from_millis(300) {
            self.show_earned = true;
            self.flash_duration = Duration::ZERO;
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        display.render_sprite(&self.pet_render);

        match self.state {
            State::Intro => {
                let total = str_format!(str32, "${}", args.game_ctx.money);
                display.render_text_complex(
                    Vec2::new(10., 10.),
                    &total,
                    ComplexRenderOption::new().with_white(),
                );
                if self.show_earned {
                    let winnings = str_format!(str32, "+${}", self.money);
                    display.render_text_complex(
                        Vec2::new(10., 20.),
                        &winnings,
                        ComplexRenderOption::new().with_white(),
                    );
                }
            }
            State::ShowingTotal => {
                let winnings = str_format!(str32, "${}", args.game_ctx.money + self.money);
                display.render_text_complex(
                    Vec2::new(CENTER_X, 10.),
                    &winnings,
                    ComplexRenderOption::new().with_white().with_center(),
                );
            }
        }
    }
}
