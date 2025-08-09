use glam::{usize, Vec2};

use crate::{
    assets::{self, Image, StaticImage},
    display::{ComplexRenderOption, GameDisplay},
    geo::Rect,
    scene::{
        mg_doge_em::MgDogeEmScene, mg_link_four::MgLinkFourScene, mg_tic_tac_toe::MgTicTacToeScene,
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
    },
    Button,
};

enum MiniGame {
    TicTacToe,
    DogeEm,
    LinkFour,
}

impl MiniGame {
    pub fn image(&self) -> &'static StaticImage {
        match self {
            MiniGame::TicTacToe => &assets::IMAGE_MG_TIC_TAC_TOE_ICON,
            MiniGame::DogeEm => &assets::IMAGE_MG_DOGE_ICON,
            MiniGame::LinkFour => &assets::IMAGE_MG_LINK_FOUR_ICON,
        }
    }
}

const MINIGAMES: &[MiniGame] = &[MiniGame::TicTacToe, MiniGame::DogeEm, MiniGame::LinkFour];

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
    const X_OFFSET: f32 = 20.;
    const Y_OFFSET: f32 = 30.;
    const X_GAP: f32 = 4.;
    const Y_GAP: f32 = 6.;
    let x = X_OFFSET + ((i % 2) as f32 * (assets::IMAGE_MG_DOGE_ICON.size.x as f32 + X_GAP));
    let y = Y_OFFSET
        + (libm::floorf(i as f32 / 2.) * (assets::IMAGE_MG_DOGE_ICON.size.y as f32 + Y_GAP));
    Vec2::new(x, y)
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
                MiniGame::LinkFour => SceneEnum::MgTicLinkFour(MgLinkFourScene::new()),
            });
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        for (i, minigame) in self.active_minigames.iter().enumerate() {
            let pos = get_pos(i);
            display.render_image_complex(
                pos.x as i32,
                pos.y as i32,
                minigame.image(),
                ComplexRenderOption::new().with_white().with_center(),
            );
        }

        let rect = Rect::new_center(
            get_pos(self.selected as usize),
            assets::IMAGE_MG_DOGE_ICON.size_vec2(),
        )
        .grow(2.);
        display.render_rect_outline(rect, true);
    }
}
