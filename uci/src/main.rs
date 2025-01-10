use core::{board::{board::Board, defs::START_POS}, movegen::{movegen::MoveGen, moves::Move}, search::{search::Search, ttable::TranspositionTable}};
use std::{io::{stdin, Read}, process::exit};

pub struct Engine{
    board: Board,
    search: Search,
    movegen: MoveGen
}

fn main() {
    let mut engine = Engine {
        board: Board::from_fen(START_POS).unwrap(),
        search: Search {
            transposition_table: TranspositionTable::new(10000000)},
        movegen: MoveGen
    };

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed input");
        input = input.trim().to_string();

        let parts: Vec<&str> = input.split(" ").collect();

        match parts[0] {
            "uci" => println!("uciok"),
            "ucinewgame" => (),
            "isready" => println!("readyok"),
            "position" => {
                if parts.len() == 0 { continue; }
                if parts[1] == "fen" {
                    engine.board = Board::from_fen(&input[13..]).unwrap()
                }
            },
            _ => ()
        }
    }
}
