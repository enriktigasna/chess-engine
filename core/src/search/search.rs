use core::panic;
use std::usize;

use crate::{board::{board::Board, defs::Sides}, movegen::{movegen::{bitscan_forward, MoveGen}, moves::Move}};

pub struct Search;
impl Search {
    pub fn find_best_move(&self, board: &mut Board, mg: &MoveGen, depth: usize) -> Option<Move> {
        let moves = mg.gen_legal_moves_no_rep(board);

        if moves.len() == 0 {
            return None
        }
        let mut best_move: Move = moves[0].clone();
        let mut max: f32 = f32::MIN;
        let mut min: f32 = f32::MAX;

        for _move in moves {
            board.do_move(&_move);

            match board.game_state.active_color {
                Sides::WHITE => {
                    let eval = self.search_max(board, f32::MIN, f32::MAX, mg, depth-1);
                    if eval > max {
                        max = eval;
                        best_move = _move.clone();
                    }
                }
                Sides::BLACK => {
                    let eval = self.search_min(board, f32::MIN, f32::MAX, mg, depth-1);
                    if eval < min {
                        min = eval;
                        best_move = _move.clone();
                    }
                }
                _ => panic!("Invalid game state")
            }
            board.undo_move(&_move);
        }

        return Some(best_move.clone());
    }

    
    pub fn search_max(&self, board: &mut Board, mut alpha: f32, mut beta: f32, mg: &MoveGen, depth: usize) -> f32 {
        if depth == 0 {
            return self.static_eval(board, mg);
        }

        let mut max_value = f32::MIN;

        let moves = mg.gen_legal_moves_no_rep(board);
        if moves.len() == 0 {
            if mg.in_check(board, Sides::WHITE) {
                return f32::MIN
            }
            if mg.in_check(board, Sides::BLACK) {
                return f32::MAX
            }

            return 0.0
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

        let moves = mg.gen_legal_moves_no_rep(board);
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

        if mg.gen_legal_moves(board).len() == 0 {
            if mg.in_check(board, Sides::WHITE) {
                return f32::MIN
            }
            if mg.in_check(board, Sides::BLACK) {
                return f32::MAX
            }
            return 0.0
        }

        for side in 0..2 {
            for piece in 0..6 {
                let mut current = board.bb_pieces[side][piece];
                while let Some(_) = bitscan_forward(current) {
                    current &= current - 1;
                    match side {
                        Sides::WHITE => eval += pieces[piece],
                        Sides::BLACK => eval -= pieces[piece],
                        _ => ()
                    }
                }
            }
        }

        eval
    }
}