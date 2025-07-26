use glam::{Vec2, usize};

use crate::{
    Button,
    assets::{self, StaticImage},
    display::{ComplexRenderOption, GameDisplay},
    geo::Rect,
    scene::{
        Scene, SceneEnum, SceneOutput, SceneTickArgs, mg_doge_em::MgDogeEmScene,
        mg_tic_tac_toe::MgTicTacToeScene,
    },
};

enum MiniGame {
    TicTacToe,
    DogeEm,
}

impl MiniGame {
    pub fn image(&self) -> &'static StaticImage {
        match self {
            MiniGame::TicTacToe => &assets::IMAGE_MG_TIC_TAC_TOE_ICON,
            MiniGame::DogeEm => &assets::IMAGE_MG_DOGE_ICON,
        }
    }
}

const MINIGAMES: &[MiniGame] = &[MiniGame::TicTacToe, MiniGame::DogeEm];

pub struct GameSelectScene {
    active_minigames: &'static [MiniGame],
    selected: i32,
}

impl GameSelectScene {
    pub fn new() -> Self {
        Self {
            active_minigames: MINIGAMES,
            selected: 0,
        }
    }
}

pub fn get_pos(i: usize) -> Vec2 {
    let x = match i % 3 {
        0 => 10,
        1 => 33,
        2 => 44,
        _ => unreachable!(),
    } + 2;
    let y = (libm::floorf(i as f32 / 3.) * 17.) + 30.;
    Vec2::new(x as f32, y)
}

impl Scene for GameSelectScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        if args.input.pressed(Button::Left) {
            self.selected -= 1;
            if self.selected < 0 {
                self.selected = (self.active_minigames.len() - 1) as i32;
            }
        }

        if args.input.pressed(Button::Right) {
            self.selected += 1;
            if self.selected as usize >= self.active_minigames.len() {
                self.selected = 0;
            }
        }

        if args.input.pressed(Button::Middle) {
            return SceneOutput::new(match self.active_minigames[self.selected as usize] {
                MiniGame::TicTacToe => SceneEnum::MgTicTacToe(MgTicTacToeScene::new()),
                MiniGame::DogeEm => {
                    SceneEnum::MgDogeEm(MgDogeEmScene::new(args.game_ctx.pet.def_id))
                }
            });
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut SceneTickArgs) {
        for (i, minigame) in self.active_minigames.iter().enumerate() {
            let pos = get_pos(i);
            display.render_image_complex(
                pos.x as i32,
                pos.y as i32,
                minigame.image(),
                ComplexRenderOption::default().with_white().with_center(),
            );
        }

        let rect = Rect::new_center(get_pos(self.selected as usize), Vec2::new(24., 24.));
        display.render_rect_outline(rect, true);
    }
}
