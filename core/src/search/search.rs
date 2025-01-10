use std::{time::{Duration, Instant}, usize};

use crate::{board::{board::Board, defs::Sides}, movegen::{movegen::{bitscan_forward, MoveGen}, moves::Move}};

use super::{defs::PieceTables, ttable::{MoveType, TranspositionEntry, TranspositionTable}};

pub const FLIP: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63,
    48, 49, 50, 51, 52, 53, 54, 55,
    40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39,
    24, 25, 26, 27, 28, 29, 30, 31,
    16, 17, 18, 19, 20, 21, 22, 23,
     8,  9, 10, 11, 12, 13, 14, 15,
     0,  1,  2,  3,  4,  5,  6,  7,
];

pub struct Search {
    pub transposition_table: TranspositionTable
}
impl Search {
    pub fn find_best_move_iter(&mut self, board: &mut Board, mg: &MoveGen, max_depth: usize, duration: Duration) -> Option<Move> {
        let mut moves = mg.gen_legal_moves_no_rep(board);
        if moves.len() == 0 {
            return None
        }

        let mut best_move = moves[0].clone();
        let start_time = Instant::now();

        for depth in 1..max_depth {
            let current_best = self.find_best_move(board, mg, depth, start_time, duration);

            if start_time.elapsed() > duration {
                break;
            }

            // We already established it's not empty
            best_move = current_best.unwrap();
        }

        self.transposition_table.increment_age();

        Some(best_move)
    }

    pub fn find_best_move(&mut self, board: &mut Board, mg: &MoveGen, depth: usize, start_time: Instant, duration: Duration) -> Option<Move> {
        let moves = mg.gen_legal_moves_no_rep(board);

        if moves.len() == 0 {
            return None
        }
        let mut best_move: Move = moves[0].clone();
        let mut max: f32 = f32::MIN;
        let mut min: f32 = f32::MAX;

        let hash = board.zobrist_hash();
        if let Some(entry) = self.transposition_table.get(hash) {
            if entry.depth >= depth {
                return Some(entry.best_move)
            }
        }

        let us = board.us();

        let mut move_type = MoveType::Alpha;
        let mut final_eval = 0.0;

        for _move in moves {
            if start_time.elapsed() > duration {
                break;
            }

            board.do_move(&_move);
            match us {
                Sides::WHITE => {
                    let eval = self.search_max(board, f32::MIN, f32::MAX, mg, depth-1);
                    if eval > max {
                        max = eval;
                        final_eval = max;
                        best_move = _move.clone();
                        println!("Eval {max} Depth {depth}");
                        move_type = MoveType::Alpha;
                    }
                }
                Sides::BLACK => {
                    let eval = self.search_min(board, f32::MIN, f32::MAX, mg, depth-1);
                    if eval < min {
                        min = eval;
                        final_eval = min;
                        best_move = _move.clone();
                        println!("Eval {min} Depth {depth}");
                        move_type = MoveType::Beta;
                    }
                }
                _ => panic!("Invalid game state")
            }

            board.undo_move(&_move);
            self.transposition_table.insert(TranspositionEntry {
                key: hash,
                best_move: best_move.clone(),
                move_type: move_type.clone(),
                eval: final_eval,
                depth,
                age: self.transposition_table.age
            });
        }

        return Some(best_move.clone());
    }

    pub fn search_max(&self, board: &mut Board, mut alpha: f32, mut beta: f32, mg: &MoveGen, depth: usize) -> f32 {
        if depth == 0 {
            return self.static_eval(board, mg);
        }

        let mut max_value = f32::MIN;

        let mut moves = mg.gen_legal_moves_no_rep(board);

        if moves.len() == 0 {
            if mg.in_check(board, Sides::WHITE) {
                return f32::MIN
            }
            if mg.in_check(board, Sides::BLACK) {
                return f32::MAX
            }

            return 0.0
        }

        let hash = board.zobrist_hash();
        let existing_entry = self.transposition_table.get(hash);

        if let Some(entry) = existing_entry {
            let index = moves.iter()
                .enumerate()
                .find(|&r| r.1.0 == entry.best_move.0)
                .unwrap()
                .0;
            moves.swap(0, index);
        }

        for _move in moves {
            board.do_move(&_move);
            let score = self.search_min(board, alpha, beta, mg, depth-1);
            board.undo_move(&_move);

            if score > max_value {
                max_value = score;
            }

            if score > alpha {
                alpha = score;
            }

            if alpha >= beta {
                break;
            }
        }

        max_value
    }
    
    pub fn search_min(&self, board: &mut Board, mut alpha: f32, mut beta: f32, mg: &MoveGen, depth: usize) -> f32 {
        if depth == 0 {
            return self.static_eval(board, mg);
        }

        let mut moves = mg.gen_legal_moves_no_rep(board);
        let hash = board.zobrist_hash();

        if let Some(entry) = self.transposition_table.get(hash) {
            if entry.depth >= depth && entry.move_type == MoveType::Beta {
                return entry.eval
            }
            
            let index = moves.iter()
                .enumerate()
                .find(|&r| r.1.0 == entry.best_move.0)
                .unwrap()
                .0;
            moves.swap(0, index);
        }

        if moves.len() == 0 {
            if mg.in_check(board, Sides::WHITE) {
                return f32::MAX
            }
            if mg.in_check(board, Sides::BLACK) {
                return f32::MIN
            }
            return 0.0
        }

        let mut min_value = f32::MAX;
        for _move in moves {
            board.do_move(&_move);
            let score = self.search_max(board, alpha, beta, mg, depth-1);
            board.undo_move(&_move);

            if score < min_value {
                min_value = score;
            }

            if score < beta {
                beta = score;
            }

            if alpha >= beta {
                break;
            }
        }

        min_value
    }

    pub fn static_eval(&self, board: &mut Board, mg: &MoveGen) -> f32 {
        // add all white pieces
        let mut eval: f32 = 0.0;
        let pieces = [1.0, 3.0, 3.2, 5.0, 9.0, 0.0];

        let psqt_set = self.get_psqt_set(board);
        eval += self.apply_psqt(board, psqt_set);

        eval
    }
    
    pub fn get_psqt_set(&self, board: &Board) -> [[i32; 64]; 6] {
        [PieceTables::PAWN, PieceTables::BISHOP, PieceTables::KNIGHT, PieceTables::ROOK, PieceTables::QUEEN, PieceTables::EARLY_KING]
    }

    pub fn apply_psqt(&self, board: &Board, psqt: [[i32; 64]; 6]) -> f32 {
        let mut eval: i32 = 0;
        let mut white = board.bb_side[Sides::WHITE];
        while let Some(square) = bitscan_forward(white) {
            white &= white - 1;
            if let Some(piece) = board.get_piece_at(square) {
                eval += psqt[piece][square];
            }
        }
        
        let mut black = board.bb_side[Sides::BLACK];
        while let Some(square) = bitscan_forward(black) {
            black &= black - 1;
            if let Some(piece) = board.get_piece_at(square) {
                eval -= psqt[piece][FLIP[square]];
            }
        }

        (eval as f32) / 100.0
    }
}