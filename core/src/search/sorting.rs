use crate::movegen::{movelist::MoveList, moves::Move};

use super::defs::INF;

pub fn sort_moves(moves: &mut MoveList, hash_move: Option<Move>) {
    let points = [1, 3, 3, 5, 9, 0];

    moves.moves[..moves.index].sort_unstable_by_key(|mv| {
        if Some(*mv) == hash_move {
            return -INF;
        }

        if mv.capture().is_none() { return 0 };
        
        -points[mv.capture().unwrap()]*10 - points[mv.piece()]
    });
}

pub fn retain_captures(moves: MoveList) -> MoveList {
    let mut captures = MoveList::new();
    for mv in moves.iter() {
        if mv.capture().is_some() {
            captures.push(*mv);
        };
    }

    captures
}