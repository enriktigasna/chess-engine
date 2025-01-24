pub type Bitboard = u64;
pub type ZobristHash = u64;
pub type Piece = usize;
pub type Side = usize;
pub type Square = usize;

pub const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const EMPTY: Bitboard = 0;
pub const BB_SQUARES: [u64; 64] = bb_squares();

const fn bb_squares() -> [u64; 64] {
    let mut squares = [EMPTY; 64];
    let mut i = 0;
    while i < 64 {
        squares[i] = 1 << i;
        i += 1;
    }

    squares
}

pub struct Pieces;
impl Pieces {
    pub const PAWN: Piece = 0;
    pub const BISHOP: Piece = 1;
    pub const KNIGHT: Piece = 2;
    pub const ROOK: Piece = 3;
    pub const QUEEN: Piece = 4;
    pub const KING: Piece = 5;
}

pub struct Sides;
impl Sides {
    pub const WHITE: Side = 0;
    pub const BLACK: Side = 1;
    pub const BOTH: Side = 2;
}

#[derive(Debug)]
pub enum InvalidFenError {
    InvalidPartCount,
    InvalidRankCount,
    InvalidActiveColor,
    InvalidCastlingPermission,
}
