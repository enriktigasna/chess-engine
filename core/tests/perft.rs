use core::{
    board::{
        board::Board,
        defs::{Pieces, Sides, START_POS},
    },
    movegen::movegen::MoveGen,
};

#[test]
fn test_from_start() {
    let mut board = Board::from_fen(START_POS).unwrap();
    let mg = MoveGen;
    assert_eq!(test_perft_nodes_v2(1, &mg, &mut board), 20);
    assert_eq!(test_perft_nodes_v2(2, &mg, &mut board), 400);
    assert_eq!(test_perft_nodes_v2(3, &mg, &mut board), 8902);
    assert_eq!(test_perft_nodes_v2(4, &mg, &mut board), 197281);
    assert_eq!(test_perft_nodes_v2(5, &mg, &mut board), 4865609);
    // Just for getting a rough performance measurement, can be removed for faster testing
    // assert_eq!(test_perft_nodes_v2(6, &mg, &mut board), 119060324);
}

#[test]
fn test_from_kiwipete() {
    let mg = MoveGen;
    let mut kiwipete =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ")
            .unwrap();
    assert_eq!(test_perft_nodes_v2(1, &mg, &mut kiwipete), 48);
    assert_eq!(test_perft_nodes_v2(2, &mg, &mut kiwipete), 2039);
    assert_eq!(test_perft_nodes_v2(3, &mg, &mut kiwipete), 97862);
    assert_eq!(test_perft_nodes_v2(4, &mg, &mut kiwipete), 4085603);
}

#[test]
fn test_from_petrovic() {
    let mg = MoveGen;
    let mut board =
        Board::from_fen("R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1").unwrap();
    assert_eq!(test_perft_nodes_v2(1, &mg, &mut board), 218);
    assert_eq!(test_perft_nodes_v2(2, &mg, &mut board), 99);
    assert_eq!(test_perft_nodes_v2(3, &mg, &mut board), 19073);
    assert_eq!(test_perft_nodes_v2(4, &mg, &mut board), 85043);
}

#[test]
fn test_from_pos3() {
    let mg = MoveGen;
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();
    assert_eq!(test_perft_nodes_v2(1, &mg, &mut board), 14);
    assert_eq!(test_perft_nodes_v2(2, &mg, &mut board), 191);
    assert_eq!(test_perft_nodes_v2(3, &mg, &mut board), 2812);
    assert_eq!(test_perft_nodes_v2(4, &mg, &mut board), 43238);
}

fn test_perft_nodes_v2(depth: usize, mg: &MoveGen, board: &mut Board) -> usize {
    let mut count: usize = 0;

    if depth == 1 {
        return mg.gen_legal_moves(board).index;
    }

    let moves = mg.gen_moves(board);

    for _move in moves.iter() {
        board.do_move(&_move);
        if mg.in_check(board, board.them()) {
            board.undo_move(&_move);
            continue;
        }
        count += test_perft_nodes_v2(depth - 1, mg, board);
        board.undo_move(&_move);
    }

    count
}
