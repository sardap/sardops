use core::time::Duration;

use glam::Vec2;

use crate::{
    Button, Timestamp,
    display::{CENTER_X, ComplexRenderOption, GameDisplay},
    geo::Rect,
    link_four::{BestMoveSearch, COLUMNS, Game, GameStatus, Side},
    pet::{
        definition::{PetAnimationSet, PetDefinitionId},
        render::PetRender,
    },
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, mg_fanfare::MgFanFareScene},
};

enum State {
    Playing,
    Dropping { dur: Duration, col: usize },
    PostGame,
}

pub struct LinkFourOpponent {
    pub pet_def_id: PetDefinitionId,
    pub strength: i32,
}

impl LinkFourOpponent {
    pub const fn new(pet: PetDefinitionId, strength: i32) -> Self {
        Self {
            pet_def_id: pet,
            strength,
        }
    }
}

pub const OPPONENTS: &[LinkFourOpponent] = &[
    LinkFourOpponent::new(crate::pet::definition::PET_BLOB_ID, 60),
    LinkFourOpponent::new(crate::pet::definition::PET_PAWN_WHITE_ID, 70),
    LinkFourOpponent::new(crate::pet::definition::PET_BALLOTEE_ID, 70),
    LinkFourOpponent::new(crate::pet::definition::PET_COMPUTIE_ID, 80),
];

const DROP_TIME: Duration = Duration::from_millis(500);

pub struct MgLinkFourScene {
    game: Game,
    selected: isize,
    flash_state: bool,
    flash_duration: Duration,
    player_side: Side,
    thinking_end: Timestamp,
    state: State,
    first_move: bool,
    best_move_search: BestMoveSearch,
    player_pet_render: PetRender,
    opponent: &'static LinkFourOpponent,
    opponent_pet_render: PetRender,
    post_game_start: Timestamp,
}

impl Default for MgLinkFourScene {
    fn default() -> Self {
        Self::new()
    }
}

impl MgLinkFourScene {
    pub fn new() -> Self {
        Self {
            game: Game::default(),
            selected: 0,
            flash_state: false,
            flash_duration: Duration::ZERO,
            player_side: Side::Red,
            thinking_end: Timestamp::default(),
            state: State::Playing,
            first_move: false,
            best_move_search: Default::default(),
            player_pet_render: PetRender::default(),
            opponent: &OPPONENTS[0],
            opponent_pet_render: PetRender::default(),
            post_game_start: Default::default(),
        }
    }

    pub fn change_animations(&mut self) {
        if self.game.side_to_move() == self.player_side {
            self.player_pet_render.set_animation(PetAnimationSet::Happy);
            self.opponent_pet_render
                .set_animation(PetAnimationSet::Normal);
        } else {
            self.player_pet_render
                .set_animation(PetAnimationSet::Normal);
            self.opponent_pet_render
                .set_animation(PetAnimationSet::Happy);
        }
    }
}

const SQUARE_SIZE: Vec2 = Vec2::new(8., 8.);
const SQUARE_X_OFFSET: f32 = 4.;
const SQUARE_Y_OFFSET: f32 = 30.;
const COL_COUNT: usize = 7;
const ROW_COUNT: usize = 6;
const BOARD_SIZE: usize = COL_COUNT * ROW_COUNT;

const fn generate_board_rects() -> [Rect; BOARD_SIZE] {
    let mut rects = [Rect::new(); BOARD_SIZE];
    let mut row = 0;
    while row < ROW_COUNT {
        let mut col = 0;
        while col < COL_COUNT {
            let index = row * COL_COUNT + col;

            let x = SQUARE_X_OFFSET + SQUARE_SIZE.x * col as f32;
            let y = SQUARE_Y_OFFSET + SQUARE_SIZE.y * row as f32;

            rects[index] = Rect::new_top_left(Vec2::new(x, y), SQUARE_SIZE);

            col += 1;
        }
        row += 1;
    }

    rects
}

const RECTANGLES: [Rect; BOARD_SIZE] = generate_board_rects();

impl Scene for MgLinkFourScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.player_side = if args.game_ctx.rng.bool() {
            Side::Yellow
        } else {
            Side::Red
        };

        self.opponent = args.game_ctx.rng.choice(OPPONENTS.iter()).unwrap();

        self.player_pet_render.pos = Vec2::new(CENTER_X, 100.);
        self.player_pet_render.set_def_id(args.game_ctx.pet.def_id);

        self.opponent_pet_render.pos = Vec2::new(CENTER_X, 15.);
        self.opponent_pet_render
            .set_def_id(self.opponent.pet_def_id);

        self.change_animations();
        // Tick by random amount
        self.opponent_pet_render
            .tick(Duration::from_millis(args.game_ctx.rng.u64(0..2000)));
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.player_pet_render.tick(args.delta);
        self.opponent_pet_render.tick(args.delta);

        match &mut self.state {
            State::Playing => match self.game.status() {
                GameStatus::InProgress => {
                    if self.game.side_to_move() == self.player_side {
                        // Check current select is valid
                        let moves = self.game.possible_moves();

                        if args.input.pressed(Button::Left) {
                            self.selected -= 1;
                            if self.selected < 0 {
                                self.selected = (COL_COUNT - 1) as isize;
                            }
                        }

                        if args.input.pressed(Button::Right) {
                            self.selected += 1;
                            if self.selected >= COL_COUNT as isize {
                                self.selected = 0;
                            }
                        }

                        if moves.get_bit(self.selected as usize)
                            && args.input.pressed(Button::Middle)
                        {
                            self.state = State::Dropping {
                                dur: Duration::ZERO,
                                col: self.selected as usize,
                            };
                            // should start thinking as soon as it's dropping
                            let thinking_time = args.game_ctx.rng.i32(2000..4000);
                            self.best_move_search = BestMoveSearch::new(self.game, 3);
                            self.thinking_end =
                                args.timestamp + Duration::from_millis(thinking_time as u64);
                        }

                        self.flash_duration += args.delta;
                        if self.flash_duration > Duration::from_millis(300) {
                            self.flash_state = !self.flash_state;
                            self.flash_duration = Duration::ZERO;
                        }
                    } else if args.timestamp < self.thinking_end {
                        self.best_move_search.step(1);
                    } else {
                        let moves = if args.game_ctx.rng.i32(0..100) > self.opponent.strength {
                            self.game.possible_moves()
                        } else if self.best_move_search.best_moves().is_empty() {
                            self.game.possible_moves()
                        } else {
                            *self.best_move_search.best_moves()
                        };
                        self.state = State::Dropping {
                            dur: Duration::ZERO,
                            col: moves.random_set_bit(&mut args.game_ctx.rng).unwrap(),
                        };
                    }

                    self.first_move = false
                }
                _ => {
                    self.selected = -1;
                    self.state = State::PostGame;
                    self.post_game_start = args.timestamp;
                    match self.game.status() {
                        GameStatus::InProgress => unreachable!(),
                        GameStatus::Draw => {
                            self.player_pet_render.set_animation(PetAnimationSet::Sad);
                            self.opponent_pet_render.set_animation(PetAnimationSet::Sad);
                        }
                        GameStatus::Win(win_info) => {
                            if win_info.side == self.player_side {
                                self.player_pet_render.set_animation(PetAnimationSet::Happy);
                                self.opponent_pet_render.set_animation(PetAnimationSet::Sad);
                            } else {
                                self.player_pet_render.set_animation(PetAnimationSet::Sad);
                                self.opponent_pet_render
                                    .set_animation(PetAnimationSet::Happy);
                            }
                        }
                    }
                }
            },
            State::Dropping { dur, col } => {
                *dur += args.delta;
                self.flash_state = true;
                let row_count = self.game.get_empty_row(*col).unwrap();
                let drop_time = DROP_TIME.mul_f32(row_count as f32 / ROW_COUNT as f32);
                if *dur > drop_time {
                    self.game.make_move(*col);
                    self.state = State::Playing;
                    self.change_animations();
                }
            }
            State::PostGame => {
                self.flash_duration += args.delta;
                if self.flash_duration > Duration::from_millis(300) {
                    self.flash_state = !self.flash_state;
                    self.flash_duration = Duration::ZERO;
                }

                if args.timestamp - self.post_game_start > Duration::from_secs(5) {
                    let (won, amount) = if let GameStatus::Win(win) = self.game.status() {
                        if win.side == self.player_side {
                            (true, 5000)
                        } else {
                            (false, 1000)
                        }
                    } else {
                        (false, 200)
                    };

                    return SceneOutput::new(SceneEnum::MgFanFare(MgFanFareScene::new(
                        won,
                        amount,
                        args.game_ctx.pet.def_id,
                    )));
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        display.render_image_center(
            10,
            self.player_pet_render.pos.y as i32,
            self.player_side.get_image(),
        );
        display.render_sprite(&self.player_pet_render);
        display.render_image_center(
            10,
            self.opponent_pet_render.pos.y as i32,
            self.player_side.other().get_image(),
        );
        display.render_sprite(&self.opponent_pet_render);

        for i in 0..self.game.size() {
            let rect = RECTANGLES[i];
            if self.selected == i as isize
                && self.game.side_to_move() == self.player_side
                && !matches!(self.state, State::Dropping { dur: _, col: _ })
            {
                if self.flash_state {
                    display.render_rect_outline_dashed(rect, true, 2);
                }
            } else {
                display.render_rect_outline(rect, true);
            }
            if let Some(side) = self.game.get(i) {
                display.render_image_center(rect.pos.x as i32, rect.pos.y as i32, side.get_image());
            }
        }
        if let State::Dropping { dur, col } = self.state {
            let row_count = self.game.get_empty_row(col).unwrap();
            let drop_time = DROP_TIME.mul_f32(row_count as f32 / ROW_COUNT as f32);
            let percent = dur.as_secs_f32() / drop_time.as_secs_f32();
            let target = RECTANGLES[row_count * COLUMNS + col];
            let start = RECTANGLES[col];
            let diff = target.pos.y - start.pos.y;
            let y = diff * percent + start.pos.y;
            display.render_image_center(
                target.pos.x as i32,
                y as i32,
                self.game.side_to_move().get_image(),
            );
        }

        if let GameStatus::Win(win) = self.game.status() {
            for i in 0..self.game.size() {
                if (win.line >> i) & 1 == 1 {
                    let options = if self.flash_state {
                        ComplexRenderOption::new()
                            .with_black()
                            .with_center()
                            .with_white()
                    } else {
                        ComplexRenderOption::new()
                            .with_flip()
                            .with_black()
                            .with_center()
                    };
                    display.render_image_complex(
                        RECTANGLES[i].pos.x as i32,
                        RECTANGLES[i].pos.y as i32,
                        self.game.get(i).unwrap().get_image(),
                        options,
                    );
                }
            }
        }
    }
}
