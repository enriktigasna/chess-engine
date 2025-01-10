use std::{collections::HashMap, vec};

use crate::{board::defs::ZobristHash, movegen::moves::Move};

#[derive(Clone)]
enum MoveType {
    Alpha,
    Beta
}

#[derive(Clone)]
pub struct TranspositionEntry {
    key: ZobristHash,
    best_move: Move,
    move_type: MoveType,
    evaluation: f32,
    depth: usize,
    age: usize
}

pub struct TranspositionTable {
    table: Vec<TranspositionEntry>,
    max_size: usize,
    age: usize
}

impl TranspositionTable {
    fn new(max_size: usize) -> Self {
        let table = vec![
            TranspositionEntry {
                key: 0,
                best_move: Move(0),
                move_type: MoveType::Alpha,
                evaluation: 0.0,
                depth: 0,
                age: 0
            };
            max_size
        ];

        TranspositionTable {
            table, max_size, age: 0
        }
    }
    fn index(&self, key: ZobristHash) -> usize {
        (key as usize) % self.max_size
    }
}