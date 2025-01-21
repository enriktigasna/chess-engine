use std::{
    i32,
    time::{Duration, Instant},
    usize,
};

use crate::{
    board::{
        board::Board,
        defs::{Pieces, Sides},
    },
    movegen::{
        movegen::{bitscan_forward, MoveGen},
        moves::Move,
    },
};

use super::{
    defs::{PieceTables, INF},
    sorting::{retain_captures, sort_moves},
    ttable::{MoveType, TranspositionEntry, TranspositionTable},
};

#[rustfmt::skip]
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
    pub transposition_table: TranspositionTable,
    pub best_move: Option<Move>,
    pub psqt_cache: Box<[[[i32; 64]; 6]; 257]>,
}
impl Search {
    // Refactor into a factory constructor! (cache shouldnt be public too)
    pub fn init_psqt_cache(&mut self) {
        for weight in 0..257 {
            self.psqt_cache[weight] = self.gen_psqt_set(weight as i32);
        }
    }

    pub fn find_best_move_iter(
        &mut self,
        board: &mut Board,
        mg: &MoveGen,
        max_depth: usize,
        duration: Duration,
    ) -> Option<Move> {
        let mut moves = mg.gen_legal_moves_no_rep(board);
        if moves.index == 0 {
            return None;
        }

        let mut best_move = moves.moves[0].clone();
        let start_time = Instant::now();

        // Incase search can't even reach 1 depth (wtf)
        sort_moves(&mut moves, None);
        self.best_move = Some(moves.moves[0].clone());
        for depth in 1..max_depth {
            let current_best = self.find_best_move(board, mg, depth, start_time, duration);

            if start_time.elapsed() > duration {
                break;
            }

            // We already established it's not empty
            best_move = current_best.unwrap();
        }

        Some(best_move)
    }

    pub fn find_best_move(
        &mut self,
        board: &mut Board,
        mg: &MoveGen,
        depth: usize,
        start_time: Instant,
        duration: Duration,
    ) -> Option<Move> {
        let moves = mg.gen_legal_moves_no_rep(board);

        if moves.index == 0 {
            return None;
        }

        let score = self.negascout(
            board,
            mg,
            start_time,
            duration,
            -INF,
            INF,
            depth,
            0,
        );

        if start_time.elapsed() < duration {
            println!("info score cp {} depth {}", score, depth);
        }
        return self.best_move.clone();
    }

    pub fn negascout(
        &mut self,
        board: &mut Board,
        mg: &MoveGen,
        start_time: Instant,
        duration: Duration,
        mut alpha: i32,
        beta: i32,
        mut depth: usize,
        ply: usize,
    ) -> i32 {
        if start_time.elapsed() > duration {
            return 0;
        }

        if depth == 0 {
            if mg.in_check(board, board.us()) && ply < 10 {
                depth += 1
            } else {
                return self.quiesce(board, mg, alpha, beta, 10);
            }
        }

        // Check for draw or checkmate
        let mut moves = mg.gen_legal_moves_no_rep(board);
        if moves.index == 0 {
            if mg.in_check(board, board.us()) {
                return -INF + (ply as i32);
            }
            return 0;
        }

        // Probe transposition table for principal move, or an existing evaluation
        let hash = board.zobrist_hash();
        let tt_entry = self.transposition_table.get(hash);
        let hash_move: Option<Move> = tt_entry.as_ref().map(|entry| entry.best_move.clone());

        let estimation = match tt_entry {
            // If depth is better try to prune instantly
            Some(entry) if entry.depth >= depth => match entry.move_type {
                MoveType::Exact => return entry.eval,
                MoveType::Minimum if entry.eval >= beta => return entry.eval,
                MoveType::Maximum if entry.eval <= alpha => return entry.eval,
                _ => entry.eval,
            },
            // Otherwise just pick static eval
            _ => self.static_eval(board),
        };

        let in_check = mg.in_check(board, board.us());
        // Reverse futility pruning
        if depth >= 3 && beta.abs() < 1000000 && !in_check {
            let margin: i32 = 150 * (depth as i32);

            if estimation >= beta + margin {
                return estimation;
            }
        }

        // Null move pruning
        // Check if we are in check/only have king/pawns
        let r = 3;
        if !in_check
        // Try - 50
            && estimation >= beta
            && depth >= 3
            && ply > 0
            && board.game_state.can_nullmove
            && alpha == beta - 1
            && board.bb_side[board.us()]
                != board.bb_pieces[board.us()][Pieces::KING]
                    | board.bb_pieces[board.us()][Pieces::PAWN]
        {
            let old_state = board.game_state.clone();

            board.game_state.active_color ^= 1;
            board.game_state.enpassant_piece = None;
            board.game_state.can_nullmove = false;

            let v = -self.negascout(
                board,
                mg,
                start_time,
                duration,
                -beta,
                -(beta - 1),
                depth - r,
                ply + 1,
            );

            board.game_state = old_state;

            if v >= beta {
                let verify_score = -self.negascout(
                    board,
                    mg,
                    start_time,
                    duration,
                    -beta,
                    -(beta - 1),
                    depth - r + 1,
                    ply + 1,
                );
                if verify_score >= beta {
                    return verify_score;
                }
            }
        }

        sort_moves(&mut moves, hash_move.clone());
        let mut best_score = -INF;
        let mut best_move = moves.moves[0].clone();

        let original_can_nullmove = board.game_state.can_nullmove;

        let mut move_count = 0;
        for mv in moves.iter() {
            board.do_move(&mv);
            board.game_state.can_nullmove = true;

            // Late move reduction
            let can_reduce = 
                depth >= 3 &&
                move_count >= 3 &&
                !in_check &&
                mv.capture().is_none() &&
                !mv.is_promotion() &&
                alpha > -INF &&
                beta < INF;
            
            // Magic LMR Formula
            let mut reduction = if can_reduce {
                pub const CONSTANT: f64 = 2.78;
                pub const FACTOR: f64 = 0.40;
                (CONSTANT + (depth.min(32) as f64).ln() * (move_count.min(32) as f64).ln() * FACTOR) as usize
            } else {
                0
            };

            // Make it not be larger than depth - 1 (otherwise it tries to subtract with overflow)
            reduction = reduction.min(depth - 1);
            
            let mut score;
            if move_count == 0 {
                score = -self.negascout(
                    board,
                    mg,
                    start_time,
                    duration,
                    -beta,
                    -alpha,
                    depth - 1,
                    ply + 1,
                );
            } else {
                // Null window search
                score = -self.negascout(
                    board,
                    mg,
                    start_time,
                    duration,
                    -(alpha + 1),
                    -alpha,
                    depth - 1 - reduction,
                    ply + 1,
                );

                // Search entire window if null window failed
                if score > alpha {
                    score = -self.negascout(
                        board,
                        mg,
                        start_time,
                        duration,
                        -beta,
                        -alpha,
                        depth - 1,
                        ply + 1,
                    );
                }
            }

            board.undo_move(&mv);
            board.game_state.can_nullmove = original_can_nullmove;

            if score > best_score {
                best_score = score;
                best_move = mv.clone();
                if ply == 0 {
                    self.best_move = Some(*mv);
                }

                if score > alpha {
                    alpha = score;
                    if alpha >= beta {
                        break;
                    }
                }
            }

            move_count += 1;
        }
        self.transposition_table.insert(TranspositionEntry {
            depth,
            key: hash,
            best_move: best_move.clone(),
            eval: best_score,
            move_type: if best_score <= alpha {
                MoveType::Maximum
            } else if best_score >= beta {
                MoveType::Minimum
            } else {
                MoveType::Exact
            },
        });

        if ply == 0 {
            self.best_move = Some(best_move);
        }

        best_score
    }

    pub fn quiesce(
        &mut self,
        board: &mut Board,
        mg: &MoveGen,
        mut alpha: i32,
        beta: i32,
        depth: usize,
    ) -> i32 {
        let stand_pat = self.static_eval(board);
        if stand_pat >= beta {
            return stand_pat;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        if depth == 0 {
            return self.static_eval(board);
        }

        let moves = mg.gen_legal_moves_no_rep(board);
        if moves.index == 0 {
            if mg.in_check(board, Sides::WHITE) || mg.in_check(board, Sides::BLACK) {
                return -INF;
            }
            return 0;
        }

        let mut captures = retain_captures(moves);

        if captures.index == 0 {
            return self.static_eval(board);
        }

        // TODO: Check if we should grab hash move from tt
        sort_moves(&mut captures, None);

        let mut best_value = -INF;
        for mv in captures.iter() {
            board.do_move(&mv);
            let score = -self.quiesce(board, mg, -beta, -alpha, depth - 1);
            board.undo_move(&mv);

            if score > best_value {
                best_value = score;
                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                return best_value;
            }
        }

        best_value
    }

    pub fn static_eval(&self, board: &mut Board) -> i32 {
        // add all white pieces
        let mut eval: i32 = 0;

        let phase = self.get_phase(board);
        let psqt_set = self.get_psqt_set(phase);

        eval += self.apply_psqt(board, psqt_set);

        let side2move = match board.us() {
            Sides::WHITE => 1,
            _ => -1,
        };
        eval * side2move
    }

    pub fn get_psqt_set(&self, phase: i32) -> [[i32; 64]; 6] {
        self.psqt_cache[phase as usize]
    }

    pub fn gen_psqt_set(&self, weight: i32) -> [[i32; 64]; 6] {
        [
            self.apply_weight(PieceTables::EARLY_PAWN, PieceTables::LATE_PAWN, weight),
            self.apply_weight(PieceTables::EARLY_BISHOP, PieceTables::LATE_BISHOP, weight),
            self.apply_weight(PieceTables::EARLY_KNIGHT, PieceTables::LATE_KNIGHT, weight),
            self.apply_weight(PieceTables::EARLY_ROOK, PieceTables::LATE_ROOK, weight),
            self.apply_weight(PieceTables::EARLY_QUEEN, PieceTables::LATE_QUEEN, weight),
            self.apply_weight(PieceTables::EARLY_KING, PieceTables::LATE_KING, weight),
        ]
    }

    pub fn apply_weight(&self, midgame: [i32; 64], endgame: [i32; 64], weight: i32) -> [i32; 64] {
        let mut out = [0; 64];

        for square in 0..64 {
            out[square] = (midgame[square] * (256 - weight) + endgame[square] * weight) / 256
        }

        out
    }

    pub fn get_phase(&self, board: &Board) -> i32 {
        let phase_values = [0, 1, 1, 2, 4, 0];
        let mut phase: i32 = 24;

        for side in 0..2 {
            for piece in 0..6 {
                phase -= phase_values[piece] * (board.bb_pieces[side][piece].count_ones() as i32);
            }
        }

        ((phase * 256) / 24).max(0)
    }

    pub fn apply_psqt(&self, board: &Board, psqt: [[i32; 64]; 6]) -> i32 {
        let mut eval: i32 = 0;
        let mut white = board.bb_side[Sides::WHITE];

        let scores = [100, 320, 300, 500, 900, 0];
        while let Some(square) = bitscan_forward(white) {
            white &= white - 1;
            if let Some(piece) = board.get_piece_at(square) {
                eval += psqt[piece][square] + scores[piece];
            }
        }

        let mut black = board.bb_side[Sides::BLACK];
        while let Some(square) = bitscan_forward(black) {
            black &= black - 1;
            if let Some(piece) = board.get_piece_at(square) {
                eval -= psqt[piece][FLIP[square]] + scores[piece];
            }
        }

        eval
    }
}
