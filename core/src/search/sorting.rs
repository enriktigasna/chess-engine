use crate::movegen::moves::Move;

pub fn sort_moves(moves: &mut Vec<Move>, hash_move: Option<Move>) {
    let points = [1, 3, 3, 5, 8, 0];

    moves.sort_by(|a, b| {
        let score_a = a
            .capture()
            .map(|cap| points[cap] * 10 - points[a.piece()])
            .unwrap_or(0);

        let score_b = b
            .capture()
            .map(|cap| points[cap] * 10 - points[b.piece()])
            .unwrap_or(0);

        score_b.cmp(&score_a)
    });

    if let Some(mv) = hash_move {
        if let Some(index) = moves.iter().position(|m| m.0 == mv.0) {
            // Remove the move from its current position
            let move_to_front = moves.remove(index);
            // Insert the move at the beginning
            moves.insert(0, move_to_front);
        }
    }
}
