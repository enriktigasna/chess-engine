
use core::{board::{board::Board, defs::START_POS}, movegen::movegen::MoveGen};

#[test]
fn test_from_start() {
    let mut board = Board::from_fen(START_POS).unwrap();
    let mg = MoveGen;
    assert_eq!(test_perft_nodes(1, &mg, &mut board), 20);
    assert_eq!(test_perft_nodes(2, &mg, &mut board), 400);
    assert_eq!(test_perft_nodes(3, &mg, &mut board), 8902);
    assert_eq!(test_perft_nodes(4, &mg, &mut board), 197281);
    assert_eq!(test_perft_nodes(5, &mg, &mut board), 4865609);
    // Uncomment if you want to do more time expensive test
    //assert_eq!(test_perft_nodes(6, &mg, &mut board), 119060324);

    let mut kiwipete  = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ").unwrap();
    assert_eq!(test_perft_nodes(1, &mg, &mut kiwipete), 48);
    //assert_eq!(test_perft_nodes(2, &mg, &mut kiwipete), 2039);
    //assert_eq!(test_perft_nodes(3, &mg, &mut kiwipete), 97862);

    assert_eq!(test_perft_captures(1, &mg, &mut kiwipete), 8);
    assert_eq!(test_perft_captures(2, &mg, &mut kiwipete), 351);
}

fn test_perft_nodes(depth: usize, mg: &MoveGen, board: &mut Board) -> usize {
    let mut count: usize = 0;
    let moves = mg.gen_legal_moves(board);
    
    if depth == 1 {
        return moves.len();
    }

    for _move in moves {
        board.do_move(&_move);
        count += test_perft_nodes(depth - 1, mg, board);
        board.undo_move(&_move);
    }

    count 
}

fn test_perft_captures(depth: usize, mg: &MoveGen, board: &mut Board) -> usize {
    let mut count: usize = 0;
    let moves = mg.gen_legal_moves(board);
    
    if depth == 1 {
        for _move in moves.into_iter() {
            if _move.capture().is_some() {
                count += 1;
            }
        }
        return count;
    }

    for _move in moves {
        board.do_move(&_move);
        count += test_perft_captures(depth - 1, mg, board);
        board.undo_move(&_move);
    }

    count 
}