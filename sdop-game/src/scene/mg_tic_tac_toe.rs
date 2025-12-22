use core::time::Duration;

use glam::Vec2;

use crate::{
    Button, Timestamp,
    display::{CENTER_X, ComplexRenderOption, GameDisplay},
    geo::RectVec2,
    pet::{definition::PetAnimationSet, render::PetRender},
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, mg_fanfare::MgFanFareScene},
    tic_tac_toe::{
        BestMoveSearch, BoardStatus, Side, Square, TIC_TAC_TOE_OPPONENT, TicTacToeGame,
        TicTacToeOpponent,
    },
};

enum State {
    Playing,
    PostGame,
}

pub struct MgTicTacToeScene {
    game: TicTacToeGame,
    selected: i8,
    flash_state: bool,
    flash_duration: Duration,
    player_side: Side,
    thinking_end: Timestamp,
    state: State,
    first_move: bool,
    best_move_search: BestMoveSearch,
    player_pet_render: PetRender,
    opponent: &'static TicTacToeOpponent,
    opponent_pet_render: PetRender,
    post_game_start: Timestamp,
}

impl Default for MgTicTacToeScene {
    fn default() -> Self {
        Self::new()
    }
}

impl MgTicTacToeScene {
    pub fn new() -> Self {
        Self {
            game: TicTacToeGame::default(),
            selected: 0,
            flash_state: false,
            flash_duration: Duration::ZERO,
            player_side: Side::O,
            thinking_end: Timestamp::default(),
            state: State::Playing,
            first_move: false,
            best_move_search: Default::default(),
            player_pet_render: PetRender::default(),
            opponent: &TIC_TAC_TOE_OPPONENT[0],
            opponent_pet_render: PetRender::default(),
            post_game_start: Default::default(),
        }
    }

    pub fn change_animations(&mut self) {
        if self.game.board().side_to_move() == self.player_side {
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

const SQUARE_SIZE: Vec2 = Vec2::new(20., 20.);
const SQUARE_X_OFFSET: f32 = 1.;
const SQUARE_Y_OFFSET: f32 = 30.;
const COL_COUNT: usize = 3;
const ROW_COUNT: usize = 3;
const BOARD_SIZE: usize = COL_COUNT * ROW_COUNT;

const fn generate_board_rects() -> [RectVec2; BOARD_SIZE] {
    let mut rects = [RectVec2::new(); BOARD_SIZE];
    let mut i = 0;
    while i < BOARD_SIZE {
        let row = i / ROW_COUNT;
        let col = i % COL_COUNT;

        let x = SQUARE_X_OFFSET + SQUARE_SIZE.x * col as f32;
        let y = SQUARE_Y_OFFSET + SQUARE_SIZE.y * row as f32;

        rects[i] = RectVec2::new_top_left(Vec2::new(x, y), SQUARE_SIZE);
        i += 1;
    }
    rects
}

const RECTANGLES: [RectVec2; BOARD_SIZE] = generate_board_rects();

impl Scene for MgTicTacToeScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.player_side = if args.game_ctx.rng.bool() {
            Side::O
        } else {
            Side::X
        };

        self.opponent = args
            .game_ctx
            .rng
            .choice(TIC_TAC_TOE_OPPONENT.iter())
            .unwrap();

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

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.player_pet_render.tick(args.delta);
        self.opponent_pet_render.tick(args.delta);

        match self.state {
            State::Playing => match self.game.board().status() {
                BoardStatus::InProgress => {
                    if self.game.board().side_to_move() == self.player_side {
                        // Check current select is valid
                        let moves = self.game.board().possible_moves();

                        if args.input.pressed(Button::Left) {
                            self.selected -= 1;
                            if self.selected < 0 {
                                self.selected = self.game.board().size() as i8;
                            }
                        }

                        if args.input.pressed(Button::Right) {
                            self.selected += 1;
                            if self.selected >= self.game.board().size() as i8 {
                                self.selected = 0;
                            }
                        }

                        if moves.get_bit(self.selected as usize)
                            && args.input.pressed(Button::Middle)
                        {
                            self.game.make_move(Square::new(self.selected as u8));
                            let thinking_time = args.game_ctx.rng.i32(1000..1500);
                            self.best_move_search = BestMoveSearch::new(*self.game.board(), 5);
                            self.thinking_end =
                                args.timestamp + Duration::from_millis(thinking_time as u64);
                            self.change_animations();
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
                            self.game.board().possible_moves()
                        } else if self.best_move_search.best_moves().is_empty() {
                            self.game.board().possible_moves()
                        } else {
                            *self.best_move_search.best_moves()
                        };
                        self.game.make_move(Square::new(
                            moves.random_set_bit(&mut args.game_ctx.rng).unwrap() as u8,
                        ));
                        self.change_animations();
                    }

                    self.first_move = false
                }
                _ => {
                    self.selected = -1;
                    self.state = State::PostGame;
                    self.post_game_start = args.timestamp;
                    match self.game.board().status() {
                        BoardStatus::InProgress => unreachable!(),
                        BoardStatus::Draw => {
                            self.player_pet_render.set_animation(PetAnimationSet::Sad);
                            self.opponent_pet_render.set_animation(PetAnimationSet::Sad);
                        }
                        BoardStatus::Win(win_info) => {
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
            State::PostGame => {
                self.flash_duration += args.delta;
                if self.flash_duration > Duration::from_millis(300) {
                    self.flash_state = !self.flash_state;
                    self.flash_duration = Duration::ZERO;
                }

                if args.timestamp - self.post_game_start > Duration::from_secs(4) {
                    let (won, amount) = if let BoardStatus::Win(win) = self.game.board().status() {
                        if win.side == self.player_side {
                            (true, 5000)
                        } else {
                            (false, 1000)
                        }
                    } else {
                        (false, 200)
                    };

                    output.set(SceneEnum::MgFanFare(MgFanFareScene::new(
                        won,
                        amount,
                        args.game_ctx.pet.def_id,
                    )));
                    return;
                }
            }
        }
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

        for i in 0..self.game.board().size() {
            let rect = RECTANGLES[i as usize];
            if self.selected == i as i8 && self.game.board().side_to_move() == self.player_side {
                if self.flash_state {
                    display.render_rect_outline_dashed(rect, true, 2);
                }
            } else {
                display.render_rect_outline(rect, true);
            }
            if let Some(side) = self.game.board().get_square(Square::new(i)) {
                display.render_image_center(rect.pos.x as i32, rect.pos.y as i32, side.get_image());
            }
        }

        if let BoardStatus::Win(win) = self.game.board().status() {
            for i in 0..9 {
                if (*win.win_board >> i) & 1 == 1 {
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
                        self.game
                            .board()
                            .get_square(Square::new(i as u8))
                            .unwrap()
                            .get_image(),
                        options,
                    );
                }
            }
        }
    }
}
