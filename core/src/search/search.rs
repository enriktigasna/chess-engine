use std::usize;

use crate::{board::board::Board, movegen::{movegen::MoveGen, moves::Move}};

pub struct Search;
impl Search {
    pub fn find_best_move(&self, board: &mut Board, mg: &MoveGen, depth: usize) -> Option<Move> {
        let moves = mg.gen_legal_moves_no_rep(board);

        if moves.len() == 0 {
            return None
        }
        let mut best_move: Move = moves[0].clone();
        let mut max: i32 = i32::MIN;
        for _move in moves {
            board.do_move(&_move);
            let eval = self.alpha_beta_search(board, i32::MAX, i32::MIN, depth);
            if eval > max {
                max = eval;
                best_move = _move.clone();
            }
            board.undo_move(&_move);
        }

        return Some(best_move.clone());
    }
    pub fn alpha_beta_search(&self, board: &mut Board, alpha: i32, beta: i32, depth: usize) -> i32 {
        if (depth == 0) {
            return 0;
        }
        return self.alpha_beta_search(board, alpha, beta, depth - 1);
    }
}