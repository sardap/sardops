use core::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use crate::{
    assets::{self, StaticImage},
    bit_array::{BitArray, bytes_for_bits},
    pet::definition::{PET_BLOB_ID, PET_PAWN_WHITE_ID, PetDefinitionId},
};

pub struct TicTacToeOpponent {
    pub pet_def_id: PetDefinitionId,
    pub strength: i32,
}

impl TicTacToeOpponent {
    pub const fn new(pet: PetDefinitionId, strength: i32) -> Self {
        Self {
            pet_def_id: pet,
            strength,
        }
    }
}

pub const TIC_TAC_TOE_OPPONENT: &[TicTacToeOpponent] = &[
    TicTacToeOpponent::new(PET_BLOB_ID, 40),
    TicTacToeOpponent::new(PET_PAWN_WHITE_ID, 60),
];

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(u16);

impl BitBoard {
    fn from_square(square: Square) -> Self {
        BitBoard(1 << *square)
    }
}

impl Deref for BitBoard {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BitBoard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Side {
    X,
    O,
}

impl Side {
    fn to_index(self) -> usize {
        match self {
            Side::X => 0,
            Side::O => 1,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => Side::X,
            1 => Side::O,
            _ => panic!("Invalid index"),
        }
    }

    pub fn get_image(&self) -> &'static StaticImage {
        match self {
            Side::X => &assets::IMAGE_TIC_TAC_TOE_CROSS,
            Side::O => &assets::IMAGE_TIC_TAC_TOE_CIRCLE,
        }
    }

    pub fn other(&self) -> Self {
        match self {
            Side::X => Side::O,
            Side::O => Side::X,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct Square(u8);

impl Square {
    pub fn new<T: Into<u8>>(val: T) -> Self {
        Self(val.into())
    }
}

#[allow(dead_code)]
impl Square {
    const A1: Square = Square(0);
    const B1: Square = Square(1);
    const C1: Square = Square(2);
    const A2: Square = Square(3);
    const B2: Square = Square(4);
    const C2: Square = Square(5);
    const A3: Square = Square(6);
    const B3: Square = Square(7);
    const C3: Square = Square(8);
}

impl Deref for Square {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct WinInfo {
    pub side: Side,
    pub win_board: BitBoard,
}

impl WinInfo {
    fn new(side: Side, win_board: BitBoard) -> Self {
        Self { side, win_board }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum BoardStatus {
    InProgress,
    Draw,
    Win(WinInfo),
}

const EMPTY: BitBoard = BitBoard(0);
const WINS: [BitBoard; 8] = [
    BitBoard(0b111000000),
    BitBoard(0b000111000),
    BitBoard(0b000000111),
    BitBoard(0b100100100),
    BitBoard(0b010010010),
    BitBoard(0b001001001),
    BitBoard(0b100010001),
    BitBoard(0b001010100),
];

#[derive(Copy, Clone)]
pub struct Board {
    size: u8,
    pieces: [BitBoard; 2],
    side_to_move: Side,
    status: BoardStatus,
}

const MAX_POSSIBLE_MOVES: usize = bytes_for_bits(9);
pub type PossibleMoves = BitArray<MAX_POSSIBLE_MOVES>;

impl Board {
    fn new() -> Self {
        Self {
            pieces: [EMPTY, EMPTY],
            side_to_move: Side::X,
            size: 9,
            status: BoardStatus::InProgress,
        }
    }

    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn side_to_move(&self) -> Side {
        self.side_to_move
    }

    fn make_move_new(&self, game_move: Square) -> Self {
        let mut result = *self;

        let bb = result.pieces[result.side_to_move.to_index()];

        result.pieces[result.side_to_move.to_index()] = BitBoard(*bb | (1 << *game_move));

        result.side_to_move = result.side_to_move.other();
        result.status = result.calc_status();

        result
    }

    pub fn possible_moves(&self) -> PossibleMoves {
        if self.win_position() {
            return PossibleMoves::default();
        }

        let mut moves = PossibleMoves::default();

        let mut filled = 0;
        for bb in &self.pieces {
            filled |= **bb;
        }

        for i in 0..self.size {
            if (filled >> i) & 1 == 0 {
                moves.set_bit(i as usize, true);
            }
        }

        moves
    }

    fn win_position(&self) -> bool {
        for bb in self.pieces.iter() {
            for win in WINS {
                if **bb & *win == *win {
                    return true;
                }
            }
        }

        false
    }

    pub fn status(&self) -> BoardStatus {
        self.status
    }

    fn calc_status(&self) -> BoardStatus {
        for (i, bb) in self.pieces.iter().enumerate() {
            for win in WINS {
                if **bb & *win == *win {
                    return BoardStatus::Win(WinInfo::new(Side::from_index(i), win));
                }
            }
        }

        if self.possible_moves().is_empty() {
            return BoardStatus::Draw;
        }

        BoardStatus::InProgress
    }

    pub fn get_square(&self, square: Square) -> Option<Side> {
        let sq_bb = BitBoard::from_square(square);

        for (i, bb) in self.pieces.iter().enumerate() {
            if **bb & *sq_bb == *sq_bb {
                return Some(Side::from_index(i));
            }
        }

        None
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TicTacToeGame {
    board: Board,
    status: BoardStatus,
}

impl TicTacToeGame {
    fn new() -> Self {
        Self {
            board: Board::new(),
            status: BoardStatus::InProgress,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn make_move(&mut self, game_move: Square) {
        self.board = self.board.make_move_new(game_move);
        self.status = self.board.calc_status();
    }
}

impl Default for TicTacToeGame {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct BestMoveSearch {
    board: Board,
    moves: PossibleMoves,
    current_index: usize,
    best_rating: i32,
    best_moves: PossibleMoves,
    depth: i32,
}

impl BestMoveSearch {
    pub fn new(board: Board, depth: i32) -> Self {
        let moves = board.possible_moves();
        Self {
            board,
            moves,
            current_index: 0,
            best_rating: i32::MIN,
            best_moves: PossibleMoves::default(),
            depth,
        }
    }

    pub fn best_moves(&self) -> &PossibleMoves {
        &self.best_moves
    }

    pub fn step(&mut self, chunk: usize) -> bool {
        let mut processed = 0;
        let total_bits = self.moves.bit_count();

        while processed < chunk && self.current_index < total_bits {
            match self.moves.next_set_bit(self.current_index) {
                Some(index) => {
                    let square = Square(index as u8);
                    let new_board = self.board.make_move_new(square);
                    let score = -nega_max(&new_board, self.depth);

                    match score.cmp(&self.best_rating) {
                        Ordering::Less => {}
                        Ordering::Equal => {
                            self.best_moves.set_bit(*square as usize, true);
                        }
                        Ordering::Greater => {
                            self.best_moves = PossibleMoves::default();
                            self.best_rating = score;
                            self.best_moves.set_bit(*square as usize, true);
                        }
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

fn nega_max(board: &Board, depth: i32) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let mut max = i32::MIN;
    let squares = board.possible_moves();
    if squares.is_empty() {
        return evaluate(board);
    }
    for square in &squares {
        let square = Square(square as u8);
        let new_board = board.make_move_new(square);
        let score = -nega_max(&new_board, depth - 1);
        if score > max {
            max = score;
        }
    }

    max
}

fn evaluate(board: &Board) -> i32 {
    match board.calc_status() {
        BoardStatus::InProgress | BoardStatus::Draw => 0,
        BoardStatus::Win(info) => {
            if info.side == board.side_to_move {
                1
            } else {
                -1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_possible_moves_blank_board() {
        let board: Board = Board::default();

        let moves = board.possible_moves();

        assert_eq!(moves.into_iter().count(), 9);
    }

    #[test]
    fn test_possible_moves_xox_x_xo_o() {
        let mut board: Board = Board::default();

        board.pieces[Side::X.to_index()] = BitBoard(0b101010000);
        board.pieces[Side::O.to_index()] = BitBoard(0b010000101);

        let moves = board.possible_moves();

        assert_eq!(moves.into_iter().count(), 3);
    }

    #[test]
    fn test_win_x() {
        let mut board: Board = Board::default();

        board.pieces[Side::X.to_index()] = BitBoard(0b111000000);
        board.pieces[Side::O.to_index()] = BitBoard(0b000101010);

        assert_eq!(
            board.calc_status(),
            BoardStatus::Win(WinInfo::new(Side::X, BitBoard(0b111000000)))
        );
    }

    #[test]
    fn test_win_o() {
        let mut board: Board = Board::default();

        board.pieces[Side::X.to_index()] = BitBoard(0b000101010);
        board.pieces[Side::O.to_index()] = BitBoard(0b111000000);

        assert_eq!(
            board.calc_status(),
            BoardStatus::Win(WinInfo::new(Side::O, BitBoard(0b111000000)))
        );
    }

    #[test]
    fn test_make_move() {
        let board: Board = Board::default();

        let updated_board = board.make_move_new(Square::A1);

        assert_eq!(
            updated_board.pieces[Side::X.to_index()],
            BitBoard(0b000000001)
        );
    }

    #[test]
    fn test_game_stalemate() {
        let mut game: TicTacToeGame = TicTacToeGame::new();

        const MOVES: [Square; 9] = [
            Square::A1,
            Square::B2,
            Square::C1,
            Square::B1,
            Square::B3,
            Square::A2,
            Square::C2,
            Square::C3,
            Square::A3,
        ];

        for (i, m) in MOVES.iter().enumerate() {
            game.make_move(*m);

            if i < 8 {
                assert_eq!(game.board.calc_status(), BoardStatus::InProgress);
            }
        }

        assert_eq!(game.board.calc_status(), BoardStatus::Draw);
    }

    #[test]
    fn test_get_square() {
        let mut board: Board = Board::default();

        board.pieces[Side::X.to_index()] = BitBoard(0b000000001);
        board.pieces[Side::O.to_index()] = BitBoard(0b000000010);

        assert_eq!(board.get_square(Square::A1), Some(Side::X));
        assert_eq!(board.get_square(Square::B1), Some(Side::O));
    }
}
