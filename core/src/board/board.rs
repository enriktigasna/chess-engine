use crate::movegen::defs::Move;

use super::defs::{Bitboard, InvalidFenError, NrOf, Piece, Pieces, Side, Sides, Square, BB_SQUARES, EMPTY};

fn algebraic_to_square(alg: &str) -> usize {
    let chars: Vec<char> = alg.chars().collect();
    if chars.len() != 2 {
        panic!("Algebraic notation must be exactly 2 characters, e.g. 'a1'. Got: {}", alg);
    }

    let file_char = chars[0];
    let rank_char = chars[1];

    let file = (file_char as u8)
        .checked_sub(b'a')
        .expect("Invalid file character");

    let rank = (rank_char as u8)
        .checked_sub(b'1')
        .expect("Invalid rank character");

    if file > 7 || rank > 7 {
        panic!("File '{}' or rank '{}' out of valid range", file_char, rank_char);
    }

    (rank as usize) * 8 + file as usize
}

pub struct Board {
    pub bb_pieces: [[Bitboard; NrOf::PIECE_TYPES]; 2],
    pub bb_side: [Bitboard; 3],
    pub game_state: GameState,
    pub history: History,
    // pub piece_list: [Piece; NrOf::SQUARES],
}

#[derive(Clone)]
pub struct GameState {
    pub active_color: Side,
    pub castling_permissions: u8,
    pub enpassant_piece: Option<Square>
}

impl GameState {
    pub fn disable_castle(&mut self, side: Side, long_castle: bool) {
        let mut shift = side*2;
        if long_castle {
            shift += 1;
        }
        self.castling_permissions &= !(1 << shift);
    }

    pub fn clear_ep(&mut self) {
        self.enpassant_piece = None;
    }

    pub fn can_castle(&self, side: Side, long_castle: bool) -> bool {
        let mut shift = side*2;
        if long_castle {
            shift += 1;
        }

        if self.castling_permissions >> shift & 1 == 1 {
            true
        } else {
            false
        }
    }
}

pub struct History {
    stack: Vec<GameState>
}
impl History {
    pub fn new() -> History {
        History { stack: vec![] }
    }
    pub fn add_entry(&mut self, game_state: &GameState) {
        self.stack.push(game_state.clone());
    }

    pub fn pop_entry(&mut self) -> GameState{
        self.stack.pop().expect("Don't pop an empty history!")
    }
}

impl Board {
    pub fn do_move(&mut self, _move: &Move) {
        self.history.add_entry(&self.game_state);
        self.remove_piece(_move.to());
        self.remove_piece(_move.from());

        if _move.is_promotion() {
            self.add_piece(_move.to(), _move.promotion_piece(), self.us());
        } else {
            self.add_piece(_move.to(), _move.piece(), self.us());
        }

        if _move.is_castle() {
            // Remove rook that is one past
            let long_castle: bool = match _move.to() as isize - _move.from() as isize {
                -2 => true,
                2 => false,
                _ => panic!("Invalid castle"),
            };

            let rook_remove_offset = if long_castle {-2} else {1};
            let rook_add_offset = if long_castle {-1} else {1};


            self.remove_piece((_move.to() as isize + rook_remove_offset) as usize);
            self.add_piece((_move.to() as isize - rook_add_offset) as usize, Pieces::ROOK, self.us());
            
            // Disable long and short castle
            self.game_state.disable_castle(self.us(), true);
            self.game_state.disable_castle(self.us(), false);
        };

        self.game_state.enpassant_piece = None;
        if _move.is_double_hop() {
            match self.us() {
                Sides::WHITE => self.game_state.enpassant_piece = Some(_move.to() + 8),
                Sides::BLACK => self.game_state.enpassant_piece = Some(_move.to() - 8),
                _ => ()
            }
        }

        if _move.is_enpassant() {
            match self.us() {
                Sides::WHITE => self.remove_piece(_move.to() + 8),
                Sides::BLACK => self.remove_piece(_move.to() - 8),
                _ => ()
            }
        }

        // Remove castling permission if rook is moved or captured

        self.game_state.active_color = self.them();
    }

    pub fn undo_move(&mut self, _move: &Move) {
        self.game_state = self.history.pop_entry();

        self.remove_piece(_move.to());

        // Non-enpassant capture
        if let Some(capture) = _move.capture() {
            if !_move.is_enpassant() {
                self.add_piece(_move.to(), capture, self.them());
            }
        }

        if _move.is_enpassant() {
            match self.game_state.active_color {
                Sides::WHITE => {
                    let cap_square = _move.to() + 8;
                    self.add_piece(cap_square, Pieces::PAWN, Sides::BLACK);
                },
                Sides::BLACK => {
                    let cap_square = _move.to() - 8;
                    self.add_piece(cap_square, Pieces::PAWN, Sides::WHITE);
                },
                _ => {}
            }
        }

        if _move.is_castle() {
            let delta = _move.to() as isize - _move.from() as isize;
            let long_castle = match delta {
                -2 => true,
                 2 => false,
                 _ => panic!("Invalid castle in undo_move"),
            };

            let rook_from = if long_castle {
                (_move.to() as isize - 2) as usize
            } else {
                (_move.to() as isize + 1) as usize
            };

            let rook_to = if long_castle {
                (_move.to() as isize + 1) as usize
            } else {
                (_move.to() as isize - 1) as usize
            };

            self.remove_piece(rook_to);
            self.add_piece(rook_from, Pieces::ROOK, self.us());
        }

        // weird
        if _move.is_promotion() {
            self.add_piece(_move.from(), Pieces::PAWN, self.us());
        } else {
            self.add_piece(_move.from(), _move.piece(), self.us());
        }
    }

    pub fn add_piece(&mut self, square: Square, piece: Piece, side: Side) {
        self.bb_pieces[side][piece] |= 1 << square;
        self.bb_side[side] |= 1 << square;
        self.bb_side[Sides::BOTH] |= 1 << square;
    }

    pub fn remove_piece(&mut self, square: Square) {
        for side in 0..2 {
            for piece in 0..NrOf::PIECE_TYPES {
                self.bb_pieces[side][piece] = self.bb_pieces[side][piece] & !(1 << square);
            }
        }

        for side in 0..3 {
            self.bb_side[side] = self.bb_side[side] & !(1 << square);
        }
    }

    pub fn get_pieces(&self, side: Side, piece: Piece) -> Bitboard {
        self.bb_pieces[side][piece]
    }

    pub fn get_all_pieces(&self, side: Side) -> Bitboard {
        self.bb_side[side]
    }

    pub fn get_piece_at(&self, square: Square) -> Option<Piece> {
        for piece in 0..NrOf::PIECE_TYPES {
            if self.bb_pieces[Sides::WHITE][piece] >> square & 1 == 1 {
                return Some(piece);
            }

            if self.bb_pieces[Sides::BLACK][piece] >> square & 1 == 1 {
                return Some(piece);
            }
        }
        None
    }

    pub fn us(&self) -> Side {
        self.game_state.active_color
    }
    
    pub fn them(&self) -> Side {
        self.game_state.active_color ^ 1
    }

    pub fn is_occupied(&self, side: Side, square: Square) -> bool {
        self.get_all_pieces(side) >> square & 1 == 1
    }
    
    pub fn is_enpassant(&self, square: Square) -> bool {
        match self.game_state.enpassant_piece {
            None => false,
            Some(ep_square) => {square == ep_square}
        }
    }


    pub fn from_fen(fen_string: &str) -> Result<Board, InvalidFenError> {
        let mut bb_pieces = [[EMPTY; NrOf::PIECE_TYPES]; 2];
        let parts: Vec<&str> = fen_string.split(' ').collect();

        let ranks: Vec<&str> = parts[0].split('/').collect();
        if ranks.len() != 8 {
            return Err(InvalidFenError::InvalidRankCount)
        }

        let active_color = match parts[1] {
            "w" => Sides::WHITE,
            "b" => Sides::WHITE,
            _ => return Err(InvalidFenError::InvalidActiveColor)
        };

        for rank in 0..NrOf::RANKS {
            let mut file: usize = 0;
            // For each rank
            for c in ranks[rank].chars() {
                let square_index = file + (rank*8);

                if c.is_digit(10) {
                    file += c.to_digit(10).unwrap() as usize;
                    continue;
                }
                match c {
                    'P' => bb_pieces[Sides::WHITE][Pieces::PAWN] |= BB_SQUARES[square_index],
                    'B' => bb_pieces[Sides::WHITE][Pieces::BISHOP] |= BB_SQUARES[square_index],
                    'N' => bb_pieces[Sides::WHITE][Pieces::KNIGHT] |= BB_SQUARES[square_index],
                    'R' => bb_pieces[Sides::WHITE][Pieces::ROOK] |= BB_SQUARES[square_index],
                    'Q' => bb_pieces[Sides::WHITE][Pieces::QUEEN] |= BB_SQUARES[square_index],
                    'K' => bb_pieces[Sides::WHITE][Pieces::KING] |= BB_SQUARES[square_index],
                    'p' => bb_pieces[Sides::BLACK][Pieces::PAWN] |= BB_SQUARES[square_index],
                    'b' => bb_pieces[Sides::BLACK][Pieces::BISHOP] |= BB_SQUARES[square_index],
                    'n' => bb_pieces[Sides::BLACK][Pieces::KNIGHT] |= BB_SQUARES[square_index],
                    'r' => bb_pieces[Sides::BLACK][Pieces::ROOK] |= BB_SQUARES[square_index],
                    'q' => bb_pieces[Sides::BLACK][Pieces::QUEEN] |= BB_SQUARES[square_index],
                    'k' => bb_pieces[Sides::BLACK][Pieces::KING] |= BB_SQUARES[square_index],
                    _ => (),
                }

                file += 1;
            }
        }

        // Fold all white and black pieces with bitwise AND
        // This will be the aggregate bitboard for white and black pieces
        let bb_side: [Bitboard; 3] = [
            bb_pieces[Sides::WHITE].iter().fold(0, |a, b| a | b),
            bb_pieces[Sides::BLACK].iter().fold(0, |a, b| a | b),
            bb_pieces.into_iter().flatten().fold(0, |a, b| a | b)
        ];

        let mut castling_permissions = 0;
        for c in parts[2].chars() {
            match c {
                'K' => castling_permissions |= 1 << 0,
                'Q' => castling_permissions |= 1 << 1,
                'k' => castling_permissions |= 1 << 2,
                'q' => castling_permissions |= 1 << 3,
                '-' => (),
                _ => return Err(InvalidFenError::InvalidCastlingPermission)
            }
        }

        let mut enpassant_piece = None;
        if parts[3] != "-" {
            enpassant_piece = Some(algebraic_to_square(parts[3]));
        }
        

        let game_state: GameState = GameState {
            active_color,
            castling_permissions,
            enpassant_piece
        };

        let history = History::new(); // Empty history, FEN doesn't give history information
    
        Ok(Board { bb_pieces, bb_side, game_state, history })
    }
}