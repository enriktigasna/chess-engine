use core::{board::board::Board, movegen::movegen::MoveGen};

pub fn main() {
    let mut board = Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ").unwrap();
    let mg = MoveGen;

    print_perft(&mut board, &mg, 4);

}

fn print_perft(board: &mut Board, mg: &MoveGen, depth: usize) {
    let moves = mg.gen_legal_moves(board);
    for _move in moves {
        board.do_move(&_move);
        println!("{}{} {}", square_to_algebraic(_move.from()), square_to_algebraic(_move.to()), test_perft_nodes(depth - 1, &mg, board));
        board.undo_move(&_move);

    }

    println!("Total {}", test_perft_nodes(depth, &mg, board));
}

fn test_perft_nodes(depth: usize, mg: &MoveGen, board: &mut Board) -> usize {
    let mut count: usize = 0;
    
    if depth == 1 {
        return mg.gen_legal_moves(board).len();
    }

    let moves = mg.gen_moves(board);

    for _move in moves {
        board.do_move(&_move);
        if mg.in_check(board, board.them()) { board.undo_move(&_move); continue; }
        count += test_perft_nodes(depth - 1, mg, board);
        board.undo_move(&_move);
    }

    count 
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