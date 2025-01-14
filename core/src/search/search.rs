use std::{
    i32,
    time::{Duration, Instant},
    usize,
};

use crate::{
    board::{board::Board, defs::Sides},
    movegen::{
        movegen::{bitscan_forward, MoveGen},
        moves::Move,
    },
};

use super::{
    defs::PieceTables,
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
    pub psqt_cache: Box<[[[i32; 64]; 6]; 257]>
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
        let moves = mg.gen_legal_moves_no_rep(board);
        if moves.len() == 0 {
            return None;
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

    pub fn find_best_move(
        &mut self,
        board: &mut Board,
        mg: &MoveGen,
        depth: usize,
        start_time: Instant,
        duration: Duration,
    ) -> Option<Move> {
        let moves = mg.gen_legal_moves_no_rep(board);

        if moves.len() == 0 {
            return None;
        }

        self.best_move = Some(moves[0].clone());
        let score= self.negamax(
            board,
            mg,
            start_time,
            duration,
            i32::MIN + 1,
            i32::MAX - 1,
            depth,
            0,
        );

        if start_time.elapsed() < duration {
            println!("info score cp {} depth {}", score, depth);
        }
        return self.best_move.clone();
    }

    pub fn negamax(
        &mut self,
        board: &mut Board,
        mg: &MoveGen,
        start_time: Instant,
        duration: Duration,
        mut alpha: i32,
        beta: i32,
        depth: usize,
        ply: usize,
    ) -> i32 {
        let hash = board.zobrist_hash();

        if depth == 0 {
            //return self.quiesce(board, mg, -beta, -alpha, 3)
            return self.quiesce(board, mg, alpha, beta, 10);
        }

        if start_time.elapsed() > duration {
            return 0;
        }

        let mut best_value = i32::MIN + 1;

        let mut moves = mg.gen_legal_moves_no_rep(board);
        if moves.len() == 0 {
            if mg.in_check(board, Sides::WHITE) || mg.in_check(board, Sides::BLACK) {
                return i32::MIN + 1;
            }
            return 0;
        }

        if let Some(entry) = self.transposition_table.get(hash) {
            self.put_move_first(&mut moves, &entry.best_move);
            if entry.depth > depth && entry.move_type == MoveType::Exact {
                return entry.eval;
            }
        }

        let mut best_move = moves[0].clone();
        for mv in moves {
            board.do_move(&mv);
            let score = -self.negamax(
                board,
                mg,
                start_time,
                duration,
                -beta,
                -alpha,
                depth - 1,
                ply + 1,
            );
            board.undo_move(&mv);
            if score > best_value {
                best_value = score;
                best_move = mv.clone();
                if score > alpha {
                    alpha = score;
                    if ply == 0 {
                        self.best_move = Some(mv);
                    }
                }
            }

            if score >= beta {
                self.transposition_table.insert(TranspositionEntry {
                    key: hash,
                    best_move,
                    eval: best_value,
                    depth,
                    age: self.transposition_table.age,
                    move_type: MoveType::Minimum,
                });
                return best_value;
            }
        }

        self.transposition_table.insert(TranspositionEntry {
            key: hash,
            best_move,
            eval: best_value,
            depth,
            age: self.transposition_table.age,
            move_type: MoveType::Exact,
        });

        best_value
    }

    pub fn quiesce(
        &mut self,
        board: &mut Board,
        mg: &MoveGen,
        mut alpha: i32,
        beta: i32,
        depth: usize,
    ) -> i32 {
        if depth == 0 {
            return self.static_eval(board);
        }

        let mut best_value = i32::MIN + 1;

        let mut moves = mg.gen_legal_moves_no_rep(board);
        if moves.len() == 0 {
            if mg.in_check(board, Sides::WHITE) || mg.in_check(board, Sides::BLACK) {
                return i32::MIN + 1;
            }
            return 0;
        }

        moves.retain(|mv| mv.capture().is_some());
        if moves.len() == 0 {
            return self.static_eval(board);
        }

        for mv in moves {
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

    pub fn put_move_first(&self, moves: &mut Vec<Move>, _move: &Move) {
        if let Some(index) = moves.iter().position(|m| m.0 == _move.0) {
            // Remove the move from its current position
            let move_to_front = moves.remove(index);
            // Insert the move at the beginning
            moves.insert(0, move_to_front);
        }
    }

    pub fn static_eval(&self, board: &mut Board) -> i32 {
        // add all white pieces
        let mut eval: i32 = 0;

        let psqt_set = self.get_psqt_set(board);
        eval += self.apply_psqt(board, psqt_set);
        let side2move = match board.us() {
            Sides::WHITE => 1,
            Sides::BLACK => -1,
            _ => panic!("Fake"),
        };

        eval * side2move
    }

    pub fn get_psqt_set(&self, board: &Board) -> [[i32; 64]; 6] {
        let weight = self.get_phase(board);

        self.psqt_cache[weight as usize]
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
            out[square] = (midgame[square]*(256-weight) + endgame[square]*weight) / 256
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

        let scores = [100, 300, 300, 500, 900, 0];
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
