use std::vec;

use crate::{board::defs::ZobristHash, movegen::moves::Move};

#[derive(Clone, PartialEq)]
pub enum MoveType {
    Minimum,
    Exact,
}

#[derive(Clone)]
// 37 Byte entry
// TODO: Remove age, depth as u16
pub struct TranspositionEntry {
    pub key: ZobristHash,
    pub best_move: Move,
    pub eval: i32,
    pub depth: usize,
    pub age: usize,
    pub move_type: MoveType,
}

pub struct TranspositionTable {
    table: Vec<TranspositionEntry>,
    max_size: usize,
    pub age: usize,
}

impl TranspositionTable {
    pub fn new(max_size: usize) -> Self {
        let table = vec![
            TranspositionEntry {
                key: 0,
                best_move: Move(0),
                eval: 0,
                depth: 0,
                age: 0,
                move_type: MoveType::Exact
            };
            max_size
        ];

        TranspositionTable {
            table,
            max_size,
            age: 0,
        }
    }

    pub fn get(&self, hash: ZobristHash) -> Option<TranspositionEntry> {
        let index = self.index(hash);
        let entry = &self.table[index];

        if entry.key == hash {
            Some(entry.clone())
        } else {
            None
        }
    }

    pub fn insert(&mut self, entry: TranspositionEntry) {
        let index = self.index(entry.key);
        self.table[index] = entry;
    }

    pub fn increment_age(&mut self) {
        self.age += 1
    }

    fn index(&self, key: ZobristHash) -> usize {
        (key as usize) % self.max_size
    }
}
