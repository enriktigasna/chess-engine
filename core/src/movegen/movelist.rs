use super::moves::Move;
pub const MAX_LEGAL_MOVES: usize = 219;

#[derive(Clone)]
pub struct MoveList {
    pub moves: [Move; MAX_LEGAL_MOVES],
    pub index: usize
}

impl MoveList {
    pub fn push(&mut self, mv: Move) {
        self.moves[self.index] = mv;
        self.index += 1;
    }

    pub fn new() -> Self {
        Self {
            moves: [Move(0); MAX_LEGAL_MOVES],
            index: 0
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.index].iter()
    }
}