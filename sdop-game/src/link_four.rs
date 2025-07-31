use crate::{
    assets::{self, StaticImage},
    bit_array::{BitArray, bytes_for_bits},
};

const ROWS: usize = 6;
pub const COLUMNS: usize = 7;

const BOARD_SIZE: usize = ROWS * COLUMNS;

type BitBoard = u64;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Side {
    Red,
    Yellow,
}

const ALL_PLAYERS: [Side; 2] = [Side::Red, Side::Yellow];

impl Side {
    fn to_index(self) -> usize {
        match self {
            Side::Red => 0,
            Side::Yellow => 1,
        }
    }

    pub fn other(&self) -> Self {
        match self {
            Side::Red => Side::Yellow,
            Side::Yellow => Side::Red,
        }
    }

    pub fn get_image(&self) -> &'static StaticImage {
        match self {
            Side::Red => &assets::IMAGE_MG_LINK_FOUR_RED,
            Side::Yellow => &assets::IMAGE_MG_LINK_FOUR_YELLOW,
        }
    }
}

const WIN_N: usize = 4;

const fn count_lines(dr: isize, dc: isize) -> usize {
    let mut count = 0;
    let mut row = 0;
    while row < ROWS {
        let mut col = 0;
        while col < COLUMNS {
            let mut valid = true;
            let mut i = 0;
            while i < WIN_N {
                let r = row as isize + dr * i as isize;
                let c = col as isize + dc * i as isize;

                if r < 0 || c < 0 || r >= ROWS as isize || c >= COLUMNS as isize {
                    valid = false;
                    break;
                }

                i += 1;
            }

            if valid {
                count += 1;
            }

            col += 1;
        }
        row += 1;
    }
    count
}

const fn calc_winning_combos() -> usize {
    count_lines(0, 1) + // Horizontal
    count_lines(1, 0) + // Vertical
    count_lines(1, 1) + // Diagonal
    count_lines(1, -1) // Anti-diagonal
}

const WINING_COMBO_COUNT: usize = calc_winning_combos();

const fn generate_win_combos() -> [BitBoard; WINING_COMBO_COUNT] {
    let mut top = 0;
    let mut lines = [0; WINING_COMBO_COUNT];

    let directions: [(isize, isize); 4] = [
        (0, 1),  // Horizontal
        (1, 0),  // Vertical
        (1, 1),  // Diagonal
        (1, -1), // Anti-diagonal
    ];

    let mut d = 0;
    while d < directions.len() {
        let (dr, dc) = directions[d];
        let mut row = 0;
        while row < ROWS {
            let mut col = 0;
            while col < COLUMNS {
                let mut valid = true;
                let mut i = 0;
                let mut line = 0;

                while i < WIN_N {
                    let r = row as isize + dr * i as isize;
                    let c = col as isize + dc * i as isize;

                    if r < 0 || r >= ROWS as isize || c < 0 || c >= COLUMNS as isize {
                        valid = false;
                        break;
                    }

                    line |= 1 << (r as usize * COLUMNS + c as usize);
                    i += 1;
                }

                if valid {
                    lines[top] = line;
                    top += 1;
                }

                col += 1;
            }
            row += 1;
        }
        d += 1;
    }

    lines
}

const WINNING_COMBINATIONS: [BitBoard; WINING_COMBO_COUNT] = generate_win_combos();

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WinInfo {
    pub side: Side,
    pub line: BitBoard,
}

impl WinInfo {
    fn new(player: Side, line: BitBoard) -> Self {
        Self { side: player, line }
    }
}

const POSSIBLE_BYTES: usize = bytes_for_bits(COLUMNS);

pub type PossibleMoves = BitArray<POSSIBLE_BYTES>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GameStatus {
    InProgress,
    Draw,
    Win(WinInfo),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Game {
    bb: [BitBoard; 2],
    side_to_move: Side,
    last_move: Option<(usize, usize)>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            bb: Default::default(),
            side_to_move: Side::Red,
            last_move: Default::default(),
        }
    }
}

impl Game {
    pub const fn size(&self) -> usize {
        return ROWS * COLUMNS;
    }

    pub fn side_to_move(&self) -> Side {
        self.side_to_move
    }

    pub fn get(&self, square: usize) -> Option<Side> {
        for player in ALL_PLAYERS.iter() {
            if self.bb[player.to_index()] & (1 << square) != 0 {
                return Some(*player);
            }
        }

        None
    }

    pub fn get_empty_row(&self, col: usize) -> Option<usize> {
        for row in 0..ROWS {
            if self.get(square_to_index(col, row)).is_some() {
                if row == 0 {
                    return None;
                }
                return Some(row - 1);
            }
        }

        Some(ROWS - 1)
    }

    fn make_move_new(&self, column: usize) -> Self {
        let mut result = *self;

        result.make_move(column);

        result
    }

    pub fn make_move(&mut self, column: usize) {
        let mut row = 0;
        while row < ROWS - 1 && self.get(square_to_index(column, row + 1)).is_none() {
            row += 1;
        }
        if row < ROWS {
            self.bb[self.side_to_move.to_index()] |= 1 << (row * COLUMNS + column);
        }

        self.last_move = Some((row, column));

        self.side_to_move = self.side_to_move.other();
    }

    pub fn complete_board(&self) -> BitBoard {
        let mut result = 0;

        for player in ALL_PLAYERS.iter() {
            result |= self.bb[player.to_index()];
        }

        result
    }

    pub fn possible_moves(&self) -> PossibleMoves {
        let mut moves = PossibleMoves::default();

        for column in 0..COLUMNS {
            if self.get(square_to_index(column, 0)).is_none() {
                moves.set_bit(column, true);
            }
        }

        moves
    }

    pub fn status(&self) -> GameStatus {
        for combination in WINNING_COMBINATIONS.iter() {
            for player in ALL_PLAYERS.iter() {
                if self.bb[player.to_index()] & combination == *combination {
                    return GameStatus::Win(WinInfo::new(*player, *combination));
                }
            }
        }

        if self.complete_board().count_ones() == BOARD_SIZE as u32 {
            return GameStatus::Draw;
        }

        GameStatus::InProgress
    }
}

const WIN_EVAL: f32 = f32::MAX;
const LOSE_EVAL: f32 = f32::MIN;
const UNKNOWN_EVAL: f32 = 0.;

fn evaluate(board: &Game) -> f32 {
    match board.status() {
        GameStatus::InProgress | GameStatus::Draw => UNKNOWN_EVAL,
        GameStatus::Win(info) => {
            if info.side == board.side_to_move {
                WIN_EVAL
            } else {
                LOSE_EVAL
            }
        }
    }
}

fn nega_max(board: &Game, depth: i32) -> f32 {
    let cols = board.possible_moves();
    if depth == 0 || cols.is_empty() {
        return evaluate(board);
    }

    let mut max = f32::NEG_INFINITY;

    for col in 0..COLUMNS {
        if cols.get_bit(col) {
            let new_board = board.make_move_new(col);
            let score = -nega_max(&new_board, depth - 1);
            if score > max {
                max = score;
            }
        }
    }

    max
}

fn best_moves(board: &Game) -> PossibleMoves {
    let mut result = PossibleMoves::default();
    // if board is empty go middle
    if board.bb.iter().all(|&bb| bb == 0) {
        result.set_bit(COLUMNS / 2, true);
        return result;
    }

    let mut max = f32::MIN;
    let moves = board.possible_moves();
    for col in 0..COLUMNS {
        if !moves.get_bit(col) {
            continue;
        }
        let new_board = board.make_move_new(col);
        let score = -nega_max(&new_board, 5);
        if score > max {
            max = score;
            result = PossibleMoves::default();
            result.set_bit(col, true);
        } else if score == max {
            result.set_bit(col, true);
        }
    }

    if result.is_empty() { moves } else { result }
}

pub fn square_to_index(column: usize, row: usize) -> usize {
    row * COLUMNS + column
}

pub struct BestMoveSearch {
    board: Game,
    moves: PossibleMoves,
    current_index: usize,
    best_rating: f32,
    best_moves: PossibleMoves,
    depth: i32,
}

impl Default for BestMoveSearch {
    fn default() -> Self {
        Self {
            board: Default::default(),
            moves: Default::default(),
            current_index: Default::default(),
            best_rating: Default::default(),
            best_moves: Default::default(),
            depth: Default::default(),
        }
    }
}

impl BestMoveSearch {
    pub fn new(board: Game, depth: i32) -> Self {
        let moves = board.possible_moves();
        Self {
            board,
            moves,
            current_index: 0,
            best_rating: f32::MIN,
            best_moves: PossibleMoves::default(),
            depth,
        }
    }

    pub fn best_moves<'a>(&'a self) -> &'a PossibleMoves {
        &self.best_moves
    }

    pub fn step(&mut self, chunk: usize) -> bool {
        let mut processed = 0;
        let total_bits = self.moves.bit_count();

        while processed < chunk && self.current_index < total_bits {
            match self.moves.next_set_bit(self.current_index) {
                Some(index) => {
                    let col = index;
                    let new_board = self.board.make_move_new(col);
                    let score = -nega_max(&new_board, self.depth);

                    if score > self.best_rating {
                        self.best_moves = PossibleMoves::default();
                        self.best_rating = score;
                    }
                    if score == self.best_rating {
                        self.best_moves.set_bit(col, true);
                    }

                    self.current_index = index + 1;
                    processed += 1;
                }
                None => return true, // No more bits set
            }
        }

        self.current_index >= total_bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_move() {
        let mut board = Game::default();

        board = board.make_move_new(0);

        assert_eq!(board.side_to_move, Side::Yellow);
        assert_eq!(board.get(square_to_index(0, 0)), Some(Side::Red));

        board = board.make_move_new(0);

        assert_eq!(board.side_to_move, Side::Red);
        assert_eq!(board.get(square_to_index(0, 1)), Some(Side::Yellow));
    }

    #[test]
    fn test_valid_moves() {
        let mut board = Game::default();

        assert_eq!(board.possible_moves().get_raw()[0], 0b01111111);

        for _ in 0..ROWS {
            board = board.make_move_new(0);
        }

        assert_eq!(board.possible_moves().into_iter().count(), COLUMNS - 1);
    }

    #[test]
    fn test_best_moves() {
        let mut board = Game::default();

        let moves = best_moves(&board);

        assert_eq!(moves.into_iter().next().unwrap(), 3);

        // Red
        board = board.make_move_new(3);
        // Yellow
        board = board.make_move_new(0);
        // Red
        board = board.make_move_new(3);
        // Yellow
        board = board.make_move_new(1);
        // Red
        board = board.make_move_new(3);
        // Yellow

        // Here yellow should be blocking red
        let moves = best_moves(&board);

        assert_eq!(moves.into_iter().next().unwrap(), 3);
    }

    #[test]
    fn test_draws() {
        let mut board = Game::default();

        while board.status() == GameStatus::InProgress {
            let moves = best_moves(&board);
            board = board.make_move_new(moves.into_iter().next().unwrap());
        }

        assert_eq!(board.status(), GameStatus::Draw);
    }
}
