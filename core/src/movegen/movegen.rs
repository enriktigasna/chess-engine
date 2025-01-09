use std::vec;


// Low hanging fruit:
// replace bitscan_forward with squares_in iterator
// PRE_ALLOCATE STACK SLICES INSTEAD OF VEC

use crate::board::{board::Board, defs::{Bitboard, Pieces, Side, Sides}};

use super::moves::{LeapingMagics, Move};

pub fn bitscan_forward(bb: u64) -> Option<usize> {
    if bb == 0 {
        None
    } else {
        Some(bb.trailing_zeros() as usize)
    }
}


pub struct MoveGen;
impl MoveGen {
    pub fn gen_pawn_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(20);
        let us = board.us();
        let them = board.them();

        let our_pawns = board.get_pieces(us, Pieces::PAWN);

        let (forward, attacks, start_rank, promotion_rank) = match us {
            Sides::WHITE => (-8, &[-9, -7], 6, 0),
            Sides::BLACK => (8, &[7, 9], 1, 7),
            _ => panic!("Invalid board state"),
        };

        // Easier and more efficient to have a lookup table for pawn attacks and forward moves
        let mut pawns = our_pawns;
        while let Some(square) = bitscan_forward(pawns) {
            pawns &= pawns - 1;
            let rank = square / 8 ;
            let steps = if rank == start_rank { 2 } else { 1 };

            for i in 1..=steps {
                let target = (square as isize + i * forward) as usize;
                let target_rank = target / 8;

                if board.is_occupied(Sides::BOTH, target) {
                    break;
                }

                if target_rank == promotion_rank {
                    moves.push(Move::new_promotion(square, target, board, Pieces::QUEEN));
                    moves.push(Move::new_promotion(square, target, board, Pieces::ROOK));
                    moves.push(Move::new_promotion(square, target, board, Pieces::BISHOP));
                    moves.push(Move::new_promotion(square, target, board, Pieces::KNIGHT));
                } else {
                    moves.push(Move::new(square, target, board));
                }
            }

            for &offset in attacks {
                if (square % 8 < 1 && offset == attacks[0]) || (square % 8 > 6 && offset == attacks[1]) {
                    continue;
                }

                let target = (square as isize + offset) as usize;
                let target_rank = target / 8;

                if (square as isize / 8 - target as isize / 8).abs() == 1 && board.is_occupied(them, target) {
                    if target_rank == promotion_rank {
                        moves.push(Move::new_promotion(square, target, board, Pieces::QUEEN));
                        moves.push(Move::new_promotion(square, target, board, Pieces::ROOK));
                        moves.push(Move::new_promotion(square, target, board, Pieces::BISHOP));
                        moves.push(Move::new_promotion(square, target, board, Pieces::KNIGHT));
                    } else {
                        moves.push(Move::new(square, target, board));
                    }
                }
                
                if (square as isize / 8 - target as isize / 8).abs() == 1 && board.is_enpassant(target) {
                    moves.push(Move::new_enpassant(square, target, board));
                }

            }
        }

        moves
    }


    pub fn gen_pawn_attacks(&self, board: &Board) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(20);
        let us = board.us();

        let our_pawns = board.get_pieces(us, Pieces::PAWN);

        let attacks = match us {
            Sides::WHITE => &[-9, -7],
            Sides::BLACK => &[7, 9],
            _ => panic!("Invalid board state"),
        };

        // Easier and more efficient to have a lookup table for pawn attacks and forward moves
        let mut pawns = our_pawns;
        while let Some(square) = bitscan_forward(pawns) {
            pawns &= pawns - 1;
            for &offset in attacks {
                if (square % 8 < 1 && offset == attacks[0]) || (square % 8 > 6 && offset == attacks[1]) {
                    continue;
                }
                let target = (square as isize + offset) as usize;

                if (square as isize / 8 - target as isize / 8).abs() == 1 {
                    moves.push(Move::new(square, target, board));
                }
                
                if (square as isize / 8 - target as isize / 8).abs() == 1 && board.is_enpassant(target) {
                    moves.push(Move::new_enpassant(square, target, board));
                }

            }
        }

        moves
    }

    pub fn gen_knight_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(20);
        let us = board.us();

        let mut our_knights = board.get_pieces(us, Pieces::KNIGHT);
        while let Some(square) = bitscan_forward(our_knights) {
            our_knights &= our_knights - 1;

            let mut target_attack_board = LeapingMagics::KNIGHT[square];
            while let Some(target) = bitscan_forward(target_attack_board as u64) {
                target_attack_board &= target_attack_board - 1;

                if board.is_occupied(us, target) { continue; }
                moves.push(Move::new(square, target, board));
            }

        }

        moves
    }

    pub fn gen_rook_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(20);
        let us = board.us();
        let them = board.them();

        let directions: &[isize; 4] = &[1, -1, 8, -8];

        let mut our_rooks = board.get_pieces(us, Pieces::ROOK);
        while let Some(square) = bitscan_forward(our_rooks) {
            our_rooks &= our_rooks - 1;
            let square_rank = square / 8;
            let square_file = square % 8;

            for &offset in directions {
                let mut target = square;
                for _ in 0..7 {
                    target = (target as isize + offset) as usize;
                    let target_rank = target / 8;
                    let target_file = target % 8;
                    
                    if target > 63 { break; }
                    

                    // Break if it went out of bounds
                    match offset.abs() {
                        1 => if square_rank != target_rank { break }
                        8 => if square_file != target_file { break }
                        _ => ()
                    }

                    if board.is_occupied(us, target) { break; }
                    moves.push(Move::new(square, target, board));
                    if board.is_occupied(them, target) { break; }
                }
            }
        }

        moves
    }

    pub fn gen_bishop_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(20);
        let us = board.us();
        let them = board.them();
        let directions: &[isize; 4] = &[9, -9, 7, -7];
        let mut our_bishops = board.get_pieces(us, Pieces::BISHOP);

        while let Some(square) = bitscan_forward(our_bishops) {
            our_bishops &= our_bishops - 1;

            let square_rank = square / 8;
            let square_file = square % 8;
            for &offset in directions {
                let mut target = square;
                for _ in 0..7 {
                    target = (target as isize + offset) as usize;
                    let target_rank = target / 8;
                    let target_file = target % 8;
                    
                    if target > 63 { break; }
                    

                    // Break if it went out of bounds
                    // Casts go crazy
                    match offset.abs() {
                        7 => if target_rank + target_file != square_rank + square_file {break},
                        9 => if target_rank as isize - target_file as isize != square_rank as isize - square_file as isize {break},
                        _ => ()
                    }

                    if board.is_occupied(us, target) { break; }
                    moves.push(Move::new(square, target, board));
                    if board.is_occupied(them, target) { break; }
                }
            }
        }

        moves
    }

    pub fn gen_queen_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(20);
        let us = board.us();
        let them = board.them();
        let directions: &[isize; 8] = &[9, -9, 7, -7, 1, -1, 8, -8];
        let mut our_queens = board.get_pieces(us, Pieces::QUEEN);

        while let Some(square) = bitscan_forward(our_queens) {
            our_queens &= our_queens - 1;

            let square_rank = square / 8;
            let square_file = square % 8;
            for &offset in directions {
                let mut target = square;
                for _ in 0..7 {
                    target = (target as isize + offset) as usize;
                    let target_rank = target / 8;
                    let target_file = target % 8;
                    
                    if target > 63 { break; }
                    

                    // Break if it went out of bounds
                    // Casts go crazy
                    match offset.abs() {
                        7 => if target_rank + target_file != square_rank + square_file {break},
                        9 => if target_rank as isize - target_file as isize != square_rank as isize - square_file as isize {break},
                        1 => if square_rank != target_rank { break }
                        8 => if square_file != target_file { break }
                        _ => ()
                    }

                    if board.is_occupied(us, target) { break; }
                    moves.push(Move::new(square, target, board));
                    if board.is_occupied(them, target) { break; }
                }
            }
        }

        moves
    }

    pub fn gen_king_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(8);
        let us = board.us();

        let mut our_king = board.get_pieces(us, Pieces::KING);
        while let Some(square) = bitscan_forward(our_king) {
            our_king &= our_king - 1;

            let mut target_attack_board = LeapingMagics::KING[square];
            while let Some(target) = bitscan_forward(target_attack_board as u64) {
                target_attack_board &= target_attack_board - 1;

                if board.is_occupied(us, target) { continue; }
                moves.push(Move::new(square, target, board));
            }

        }

        moves
    }

    pub fn gen_castle_moves(&self, board: &mut Board) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(20);
        let us = board.us();
        let them = board.them();

        let our_king = bitscan_forward(board.get_pieces(us, Pieces::KING)).expect("King not found");
        
        let attack_bb = self.gen_attack_bitboard(board, them);

        // Short castle
        if board.game_state.can_castle(us, false) {
            if board.get_piece_at(our_king + 1).is_none() && board.get_piece_at(our_king + 2).is_none() && !self.in_check(board, us) {
                if (1 << our_king & attack_bb) == 0 && (1 << our_king + 1 & attack_bb) == 0 {
                    moves.push(Move::new_castle(our_king, our_king + 2, board));
                }
            }
        }

        if board.game_state.can_castle(us, true) {
            if board.get_piece_at(our_king - 1).is_none() && board.get_piece_at(our_king - 2).is_none() && board.get_piece_at(our_king - 3).is_none() && !self.in_check(board, us){
                if (1 << our_king & attack_bb) == 0 && (1 << our_king - 1 & attack_bb) == 0 {
                    moves.push(Move::new_castle(our_king, our_king - 2, board));
                }
            }
        }

        moves
    }

    // NEED TO REDO THIS ENTIRE THING
    pub fn gen_attack_bitboard(&self, board: &mut Board, side: Side) -> Bitboard {
        let mut bitboard: Bitboard = 0;

        let backup = board.game_state.active_color;
        board.game_state.active_color = side;
        let mut moves = Vec::with_capacity(218);
        
        moves.append(&mut self.gen_pawn_attacks(board));
        moves.append(&mut self.gen_knight_moves(board));
        moves.append(&mut self.gen_rook_moves(board));
        moves.append(&mut self.gen_bishop_moves(board));
        moves.append(&mut self.gen_queen_moves(board));
        moves.append(&mut self.gen_king_moves(board));

        for _move in moves {
            bitboard |=  1 << _move.to();
        }

        
        board.game_state.active_color = backup;

        bitboard
    }
    
    pub fn in_check(&self, board: &mut Board, side: Side) -> bool {
        let king = board.get_pieces(side, Pieces::KING);
        king & self.gen_attack_bitboard(board, side ^ 1) != 0
    }

    pub fn gen_moves(&self, board: &mut Board) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        moves.append(&mut self.gen_pawn_moves(board));
        moves.append(&mut self.gen_knight_moves(board));
        moves.append(&mut self.gen_rook_moves(board));
        moves.append(&mut self.gen_bishop_moves(board));
        moves.append(&mut self.gen_queen_moves(board));
        moves.append(&mut self.gen_king_moves(board));
        moves.append(&mut self.gen_castle_moves(board));

        moves
    }
    
    pub fn gen_legal_moves_no_rep(&self, board: &mut Board) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        
        let count = board.history.count_hash(board.zobrist_hash());
        if count >= 3 {
            return moves;
        }

        moves.append(&mut self.gen_pawn_moves(board));
        moves.append(&mut self.gen_knight_moves(board));
        moves.append(&mut self.gen_rook_moves(board));
        moves.append(&mut self.gen_bishop_moves(board));
        moves.append(&mut self.gen_queen_moves(board));
        moves.append(&mut self.gen_king_moves(board));
        moves.append(&mut self.gen_castle_moves(board));

        moves
    }

    pub fn gen_legal_moves(&self, board: &mut Board) -> Vec<Move> {
        let mut pseudo_legal = self.gen_moves(board);
        let us = board.us();

        pseudo_legal.retain(|_move| {
            board.do_move(_move);
            let king_in_check = self.in_check(board, us);
            board.undo_move(_move);
            !king_in_check
        });

        pseudo_legal
    }
}
