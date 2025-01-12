use core::{board::{board::Board, defs::{Pieces, Sides, START_POS}}, movegen::{movegen::MoveGen, moves::Move}, search::{search::Search, ttable::TranspositionTable}};
use std::{io::stdin, panic, time::Duration};

pub struct Engine{
    board: Board,
    search: Search,
    movegen: MoveGen,
    white_time: usize,
    black_time: usize,
}

fn main() {
    let mut engine = Engine {
        board: Board::from_fen(START_POS).unwrap(),
        search: Search {
            transposition_table: TranspositionTable::new(10000000)},
        movegen: MoveGen,
        black_time: 1000*60,
        white_time: 1000*60
    };

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed input");
        input = input.trim().to_string();

        let parts: Vec<&str> = input.split(" ").collect();

        match parts[0] {
            "uci" => println!("uciok"),
            "ucinewgame" => {
                engine.board = Board::from_fen(START_POS).unwrap();
            },
            "isready" => {
                println!("readyok");
            },
            "position" => {
                let mut fen_index = 0;
                let mut moves_index = 0;

                // Default to startpos if not otherwise specified.
                // If user typed "position startpos"...
                if parts.len() > 1 && parts[1] == "startpos" {
                    engine.board = Board::from_fen(START_POS).unwrap();
                    moves_index = 2;
                }
                // If user typed "position fen <something>" ...
                else if parts.len() > 1 && parts[1] == "fen" {
                    fen_index = 2;
                    // We need to collect all parts until we hit "moves" or run out.
                    let mut fen_args = Vec::new();
                    while fen_index < parts.len() && parts[fen_index] != "moves" {
                        fen_args.push(parts[fen_index]);
                        fen_index += 1;
                    }
                    let fen_string = fen_args.join(" ");
                    // Attempt to parse FEN
                    engine.board = Board::from_fen(&fen_string).unwrap();
                    moves_index = fen_index;
                }

                if moves_index < parts.len() && parts[moves_index] == "moves" {
                    moves_index += 1;
                    for mv_str in &parts[moves_index..] {
                        let moves = engine.movegen.gen_legal_moves(&mut engine.board);
                        if mv_str.len() < 4 {
                            continue;
                        }
                        let from_sq = algebraic_to_square(&mv_str[0..2]);
                        let to_sq = algebraic_to_square(&mv_str[2..4]);
                        
                        for mv in moves {
                            if mv.is_promotion() {
                                // If parts[move_index] is 4, it will promote to queen
                                if mv_str.len() == 4 {
                                    if mv.promotion_piece() != Pieces::QUEEN { continue; }
                                } else {
                                    match mv_str.chars().nth(4).unwrap_or('q') {
                                        'n' => { 
                                            if mv.promotion_piece() != Pieces::KNIGHT { continue; }
                                        }
                                        'r' => { 
                                            if mv.promotion_piece() != Pieces::ROOK { continue; }
                                        }
                                        'b' => { 
                                            if mv.promotion_piece() != Pieces::BISHOP { continue; }
                                        }
                                        _ => { 
                                            if mv.promotion_piece() != Pieces::QUEEN { continue; }
                                        }
                                    }
                                }
                            }

                            if mv.from() == from_sq && mv.to() == to_sq {
                                engine.board.do_move(&mv);
                                break;
                            }
                        }
                    }
                }
            },
            "go" => {
                parse_time_parameters(&parts, &mut engine);


                let time_left_ms = match engine.board.us() {
                    Sides::WHITE => engine.white_time,
                    Sides::BLACK => engine.black_time,
                    _ => panic!("Corrupted board color"),
                };
                let ten_percent = (time_left_ms as f64 * 0.1).round() as u64;
                let ten_percent_duration = Duration::from_millis(ten_percent);

                let mut search_duration = Duration::from_secs(2);
                if ten_percent_duration < search_duration {
                    search_duration = ten_percent_duration;
                }

                let best_move = engine.search.find_best_move_iter(
                    &mut engine.board,
                    &engine.movegen,
                    8,
                    search_duration,
                );
                match best_move {
                    Some(mv) => {
                        if let Some(entry) = engine.search.transposition_table.get(engine.board.zobrist_hash()) {
                            println!("info score {} depth {}", entry.eval, entry.depth);
                        }
                        println!(
                            "bestmove {}",
                            move_to_alg(&mv)
                        );
                    }
                    None => {
                        println!("bestmove 0000");
                    }
                }

                if let Some(entry) = engine.search.transposition_table.get(engine.board.zobrist_hash()) {
                    println!("info score {} depth {}", entry.eval, entry.depth);
                }
            },
            "info" => {
                if let Some(entry) = engine.search.transposition_table.get(engine.board.zobrist_hash()) {
                    println!("info score {} depth {}", entry.eval, entry.depth);
                }
            }
            "quit" => {
                break;
            }
            _ => ()
        }
    }
}


fn square_to_algebraic(square: usize) -> String {
    let rank = 8 - (square / 8);
    let file = square % 8;

    // Convert file to 'a'..'h'
    let file_char = (b'a' + file as u8) as char;
    // Convert rank to '1'..'8'
    let rank_char = (b'0' + rank as u8) as char;

    format!("{}{}", file_char, rank_char)
}

pub fn algebraic_to_square(alg: &str) -> usize {
    let chars: Vec<char> = alg.chars().collect();
    if chars.len() != 2 {
        panic!("Algebraic notation must be exactly 2 characters, e.g. 'a1'. Got: {}", alg);
    }

    let file_char = chars[0];
    let rank_char = chars[1];

    let file = (file_char as u8)
        .checked_sub(b'a')
        .expect("Invalid file character");

    let rank = (rank_char as u8)
        .checked_sub(b'1')
        .expect("Invalid rank character");

    if file > 7 || rank > 7 {
        panic!("File '{}' or rank '{}' out of valid range", file_char, rank_char);
    }

    ((7-rank) as usize) * 8 + file as usize
}

fn parse_time_parameters(parts: &[&str], engine: &mut Engine) {
    if let Some(idx) = parts.iter().position(|&p| p == "wtime") {
        if idx + 1 < parts.len() {
            if let Ok(val) = parts[idx + 1].parse::<usize>() {
                engine.white_time = val;
            }
        }
    }
    if let Some(idx) = parts.iter().position(|&p| p == "btime") {
        if idx + 1 < parts.len() {
            if let Ok(val) = parts[idx + 1].parse::<usize>() {
                engine.black_time = val;
            }
        }
    }
}

fn move_to_alg(_move: &Move) -> String {
    let mut alg = format!("{}{}", square_to_algebraic(_move.from()), square_to_algebraic(_move.to()));
    if _move.is_promotion() {
        match _move.promotion_piece() {
            Pieces::BISHOP => alg += "b",
            Pieces::KNIGHT => alg += "n",
            Pieces::ROOK => alg += "r",
            Pieces::QUEEN => alg += "q",
            _ => panic!("Invalid promotion")
        }
    }

    alg.to_string()
}