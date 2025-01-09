use crate::board::{board::Board, defs::{Piece, Pieces, Square}};
use std::fmt;

#[derive(Clone)]
pub struct Move(pub usize);


// 6 bits From
// 6 bits To
// 3 bits Piece
// 3 bits Piece capture
// 1 bit castle flag
// 1 bit en-passant flag
// 3 bit promotion flag (first bit for enabled/disabled) (2 bits for piece)
// More for promotions, enpassant, move order etc. for future..
pub struct Shift;
impl Shift {
    pub const FROM: u64 = 0;
    pub const TO: u64 = 6;
    pub const PIECE: u64 = 12;
    pub const CAPTURE: u64 = 15;
    pub const CASTLE: u64 = 18;
    pub const ENPASSANT: u64 = 19;
    pub const PROMOTION: u64 = 20;
    pub const PROMOTION_PIECE: u64 = 21;
    // ..
}

impl Move {
    // TODO: Make other functions just use this then extend it
    pub fn new(from: usize, to: usize, board: &Board) -> Move {
        let mut data: usize = 0;
        let capture = board.get_piece_at(to);
        // This should NEVER happen
        let piece = board.get_piece_at(from).expect("Passed move from empty board piece");

        data |= from << Shift::FROM;
        data |= to << Shift::TO;
        data |= piece << Shift::PIECE;

        // None is represented as 7 in this structure
        match capture {
            None => data |= 7 << Shift::CAPTURE,
            Some(cap) => data |= cap << Shift::CAPTURE
        }

        Move(data)
    }

    pub fn new_enpassant(from: usize, to: usize, board: &Board) -> Move {
        let mut data: usize = 0;
        let capture = board.get_piece_at(to);
        // This should NEVER happen
        let piece = Pieces::PAWN;

        data |= from << Shift::FROM;
        data |= to << Shift::TO;
        data |= piece << Shift::PIECE;
        data |= 1 << Shift::ENPASSANT;

        // None is represented as 7 in this structure
        match capture {
            None => data |= 7 << Shift::CAPTURE,
            Some(cap) => data |= cap << Shift::CAPTURE
        }

        Move(data)
    }

    pub fn new_castle(from: usize, to: usize, board: &Board) -> Move {
        let mut data: usize = 0;
        let capture = board.get_piece_at(to);
        // This should NEVER happen
        let piece = board.get_piece_at(from).expect("Passed move from empty board piece");

        data |= from << Shift::FROM;
        data |= to << Shift::TO;
        data |= piece << Shift::PIECE;
        data |= 1 << Shift::CASTLE;

        // None is represented as 7 in this structure
        match capture {
            None => data |= 7 << Shift::CAPTURE,
            Some(cap) => data |= cap << Shift::CAPTURE
        }

        Move(data)
    }

    pub fn new_promotion(from: usize, to: usize, board: &Board, promotion_piece: Piece) -> Move {
        let mut data: usize = 0;
        let capture = board.get_piece_at(to);
        // This should NEVER happen
        let piece = board.get_piece_at(from).expect("Passed move from empty board piece");

        data |= from << Shift::FROM;
        data |= to << Shift::TO;
        data |= piece << Shift::PIECE;
        data |= 1 << Shift::PROMOTION;
        data |= (promotion_piece - 1) << Shift::PROMOTION_PIECE;

        // None is represented as 7 in this structure
        match capture {
            None => data |= 7 << Shift::CAPTURE,
            Some(cap) => data |= cap << Shift::CAPTURE
        }

        Move(data)
    }

    pub fn is_castle(&self) -> bool {
        self.0 >> Shift::CASTLE & 1 == 1
    }

    pub fn is_promotion(&self) -> bool {
        self.0 >> Shift::PROMOTION & 1 == 1
    }
    
    pub fn promotion_piece(&self) -> Piece {
        (self.0 >> Shift::PROMOTION_PIECE & 0b11) + 1
    }

    pub fn from(&self) -> Square {
        self.0 >> Shift::FROM & 0x3f
    }

    pub fn to(&self) -> Square {
        self.0 >> Shift::TO & 0x3f
    }

    pub fn piece(&self) -> Piece {
        self.0 >> Shift::PIECE & 7
    }
    
    pub fn capture(&self) -> Option<Piece> {
        let piece = self.0 >> Shift::CAPTURE & 7;
        if piece == 7 {
            None
        } else {
            Some(piece)
        }
    }

    pub fn is_double_hop(&self) -> bool {
        if (self.from() as isize - self.to() as isize).abs() == 16{
            true
        } else {
            false
        }
    }

    pub fn is_enpassant(&self) -> bool {
        self.0 >> Shift::ENPASSANT & 1 == 1
    }
}

pub struct LeapingMagics;
impl LeapingMagics {
    pub const KNIGHT: [usize; 64] = 
    [0x20400, 0x50800, 0xa1100, 0x142200, 0x284400, 0x508800, 0xa01000, 0x402000,
    0x2040004, 0x5080008, 0xa110011, 0x14220022, 0x28440044, 0x50880088, 0xa0100010, 0x40200020,
    0x204000402, 0x508000805, 0xa1100110a, 0x1422002214, 0x2844004428, 0x5088008850, 0xa0100010a0, 0x4020002040,
    0x20400040200, 0x50800080500, 0xa1100110a00, 0x142200221400, 0x284400442800, 0x508800885000, 0xa0100010a000, 0x402000204000,
    0x2040004020000, 0x5080008050000, 0xa1100110a0000, 0x14220022140000, 0x28440044280000, 0x50880088500000, 0xa0100010a00000, 0x40200020400000,
    0x204000402000000, 0x508000805000000, 0xa1100110a000000, 0x1422002214000000, 0x2844004428000000, 0x5088008850000000, 0xa0100010a0000000, 0x4020002040000000,
    0x400040200000000, 0x800080500000000, 0x1100110a00000000, 0x2200221400000000, 0x4400442800000000, 0x8800885000000000, 0x100010a000000000, 0x2000204000000000,
    0x4020000000000, 0x8050000000000, 0x110a0000000000, 0x22140000000000, 0x44280000000000, 0x88500000000000, 0x10a00000000000, 0x20400000000000];
    pub const KING: [usize; 64] = [0x302, 0x705, 0xe0a, 0x1c14, 0x3828, 0x7050, 0xe0a0, 0xc040, 0x30203, 0x70507, 0xe0a0e, 0x1c141c, 0x382838, 0x705070, 0xe0a0e0, 0xc040c0, 0x3020300, 0x7050700, 0xe0a0e00, 0x1c141c00, 0x38283800, 0x70507000, 0xe0a0e000, 0xc040c000, 0x302030000, 0x705070000, 0xe0a0e0000, 0x1c141c0000, 0x3828380000, 0x7050700000, 0xe0a0e00000, 0xc040c00000, 0x30203000000, 0x70507000000, 0xe0a0e000000, 0x1c141c000000, 0x382838000000, 0x705070000000, 0xe0a0e0000000, 0xc040c0000000, 0x3020300000000, 0x7050700000000, 0xe0a0e00000000, 0x1c141c00000000, 0x38283800000000, 0x70507000000000, 0xe0a0e000000000, 0xc040c000000000, 0x302030000000000, 0x705070000000000, 0xe0a0e0000000000, 0x1c141c0000000000, 0x3828380000000000, 0x7050700000000000, 0xe0a0e00000000000, 0xc040c00000000000, 0x203000000000000, 0x507000000000000, 0xa0e000000000000, 0x141c000000000000, 0x2838000000000000, 0x5070000000000000, 0xa0e0000000000000, 0x40c0000000000000];
}


impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Move {{ \
               from: {}, \
               to: {}, \
               piece: {}, \
               capture: {:?}, \
               castle: {}, \
               en_passant: {}, \
               double_hop: {}, \
               promotion: {}, \
               promotion_piece: {:?}, \
               raw_data: 0x{:x} \
            }}",
            self.from(),
            self.to(),
            self.piece(),
            self.capture(),
            self.is_castle(),
            self.is_enpassant(),
            self.is_double_hop(),
            self.is_promotion(),
            // Only show the promotion_piece if it’s actually a promotion
            if self.is_promotion() {
                Some(self.promotion_piece())
            } else {
                None
            },
            self.0
        )
    }
}