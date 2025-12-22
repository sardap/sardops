use glam::{IVec2, usize};

use crate::{
    Button, HEIGHT,
    assets::{self, Image, StaticImage},
    display::{CENTER_X, CENTER_X_I32, ComplexRenderOption, GameDisplay},
    geo::RectIVec2,
    scene::{
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, mg_doge_em::MgDogeEmScene,
        mg_link_four::MgLinkFourScene, mg_tic_tac_toe::MgTicTacToeScene,
        mg_weight_lift::MgWeightLift,
    },
};

enum MiniGame {
    TicTacToe,
    DogeEm,
    LinkFour,
    WeightLift,
}

impl MiniGame {
    pub fn image(&self) -> &'static StaticImage {
        match self {
            MiniGame::TicTacToe => &assets::IMAGE_MG_TIC_TAC_TOE_ICON,
            MiniGame::DogeEm => &assets::IMAGE_MG_DOGE_ICON,
            MiniGame::LinkFour => &assets::IMAGE_MG_LINK_FOUR_ICON,
            MiniGame::WeightLift => &assets::IMAGE_MG_WEIGHT_LIFT_ICON,
        }
    }
}

const MINIGAMES: &[MiniGame] = &[
    MiniGame::TicTacToe,
    MiniGame::DogeEm,
    MiniGame::LinkFour,
    MiniGame::WeightLift,
];

pub struct GameSelectScene {
    active_minigames: &'static [MiniGame],
    selected: i32,
}

impl Default for GameSelectScene {
    fn default() -> Self {
        Self::new()
    }
}

impl GameSelectScene {
    pub fn new() -> Self {
        Self {
            active_minigames: MINIGAMES,
            selected: 0,
        }
    }
}

pub fn get_pos(i: i32) -> IVec2 {
    const X_OFFSET: i32 = 20;
    const Y_OFFSET: i32 = 30;
    const X_GAP: i32 = 4;
    const Y_GAP: i32 = 6;
    let x = X_OFFSET + ((i % 2) * (assets::IMAGE_MG_DOGE_ICON.isize.x + X_GAP));
    let y = Y_OFFSET
        + (libm::floorf(i as f32 / 2.) as i32 * (assets::IMAGE_MG_DOGE_ICON.isize.y + Y_GAP));
    IVec2::new(x, y)
}

impl Scene for GameSelectScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        let mut change = 0;
        if args.input.pressed(Button::Left) {
            change = -1;
        }

        if args.input.pressed(Button::Right) {
            change = 1;
        }

        self.selected =
            ((self.selected + change) % (self.active_minigames.len() + 1) as i32).max(0);

        if args.input.pressed(Button::Middle) {
            if self.selected == self.active_minigames.len() as i32 {
                output.set_home();
                return;
            }

            output.set(match self.active_minigames[self.selected as usize] {
                MiniGame::TicTacToe => SceneEnum::MgTicTacToe(MgTicTacToeScene::new()),
                MiniGame::DogeEm => {
                    SceneEnum::MgDogeEm(MgDogeEmScene::new(args.game_ctx.pet.def_id))
                }
                MiniGame::LinkFour => SceneEnum::MgTicLinkFour(MgLinkFourScene::new()),
                MiniGame::WeightLift => {
                    SceneEnum::MgWeightLift(MgWeightLift::new(args.game_ctx.pet.def_id))
                }
            });
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        for (i, minigame) in self.active_minigames.iter().enumerate() {
            let pos = get_pos(i as i32);
            display.render_image_complex(
                pos.x as i32,
                pos.y as i32,
                minigame.image(),
                ComplexRenderOption::new().with_white().with_center(),
            );
        }

        const BACK_Y: i32 = HEIGHT as i32 - assets::IMAGE_BACK_SYMBOL.size.y as i32 / 2 - 15;
        display.render_image_center(CENTER_X as i32, BACK_Y, &assets::IMAGE_BACK_SYMBOL);

        if self.selected as usize == self.active_minigames.len() {
            const RECT: RectIVec2 = RectIVec2::new_center(
                IVec2::new(CENTER_X_I32, BACK_Y),
                IVec2::new(
                    assets::IMAGE_BACK_SYMBOL.isize.x,
                    assets::IMAGE_BACK_SYMBOL.isize.y + 1,
                ),
            );

            display.render_rect_outline(&RECT, true);
        } else {
            let rect =
                RectIVec2::new_center(get_pos(self.selected), assets::IMAGE_MG_DOGE_ICON.isize)
                    .grow(2);
            display.render_rect_outline(&rect, true);
        }
    }
}
