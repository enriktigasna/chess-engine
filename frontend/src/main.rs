use core::{
    board::{
        board::Board,
        defs::{Bitboard, Pieces, Sides, Square, START_POS},
    },
    movegen::{movegen::MoveGen, moves::Move},
    search::{search::Search, ttable::TranspositionTable},
};
use std::time::{Duration, Instant};

use macroquad::{
    color::{Color, WHITE},
    file::set_pc_assets_folder,
    input::{is_mouse_button_down, mouse_position, MouseButton},
    math::vec2,
    shapes::{draw_circle, draw_rectangle},
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{next_frame, screen_height, screen_width, Conf},
};

pub struct PieceAssets {
    pub wp: Texture2D,
    pub wb: Texture2D,
    pub wn: Texture2D,
    pub wr: Texture2D,
    pub wq: Texture2D,
    pub wk: Texture2D,
    pub bp: Texture2D,
    pub bb: Texture2D,
    pub bn: Texture2D,
    pub br: Texture2D,
    pub bq: Texture2D,
    pub bk: Texture2D,
}

impl PieceAssets {
    pub async fn new() -> PieceAssets {
        PieceAssets {
            wp: load_texture("assets/wp.png").await.unwrap(),
            wb: load_texture("assets/wb.png").await.unwrap(),
            wn: load_texture("assets/wn.png").await.unwrap(),
            wr: load_texture("assets/wr.png").await.unwrap(),
            wq: load_texture("assets/wq.png").await.unwrap(),
            wk: load_texture("assets/wk.png").await.unwrap(),
            bp: load_texture("assets/bp.png").await.unwrap(),
            bb: load_texture("assets/bb.png").await.unwrap(),
            bn: load_texture("assets/bn.png").await.unwrap(),
            br: load_texture("assets/br.png").await.unwrap(),
            bq: load_texture("assets/bq.png").await.unwrap(),
            bk: load_texture("assets/bk.png").await.unwrap(),
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Chess".to_owned(),
        window_height: 480,
        window_width: 480,
        ..Default::default()
    }
}

fn draw_bitboard_pieces(bitboard: Bitboard, texture: &Texture2D, square_size: f32) {
    // A common pattern: iterate from 0..64 for each bit in the u64
    // If the bit is set, draw that piece
    for square_index in 0..64 {
        if bitboard >> square_index & 1 == 1 {
            let x = square_size * (square_index % 8) as f32;
            let y = square_size * (square_index / 8) as f32;
            draw_texture_ex(
                texture,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(square_size, square_size)),
                    ..Default::default()
                },
            );
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    set_pc_assets_folder("frontend");

    let board_colors = [Color::from_hex(0x4b7399), Color::from_hex(0xeae9d2)];
    let active_colors = [Color::from_hex(0x258ccc), Color::from_hex(0x75c7e9)];
    let piece_assets = PieceAssets::new().await;

    let mg = MoveGen;
    let mut board = Board::from_fen(START_POS).expect("Invalid FEN");
    let mut search = Search {
        transposition_table: TranspositionTable::new(1361702),
        best_move: None,
        psqt_cache: Box::new([[[0; 64]; 6]; 257]),
    };

    search.init_psqt_cache();

    //let mut board = Board::from_fen("8/3r2p1/1P2k3/p2p3p/3R2p1/4P3/1PP3PP/2K2R2 b - - 0 33").expect("Invalid FEN");
    //let mut board = Board::from_fen("8/8/1Kpp4/1P5r/1R3p1k/4P3/6P1/8 b - - 1 2").unwrap();

    let mut moves = mg.gen_legal_moves(&mut board);
    let mut move_hints: Vec<Square> = vec![];
    let mut active_square: Option<Square> = None;

    let white_lookup = [
        (Pieces::PAWN, &piece_assets.wp),
        (Pieces::BISHOP, &piece_assets.wb),
        (Pieces::KNIGHT, &piece_assets.wn),
        (Pieces::ROOK, &piece_assets.wr),
        (Pieces::QUEEN, &piece_assets.wq),
        (Pieces::KING, &piece_assets.wk),
    ];

    let black_lookup = [
        (Pieces::PAWN, &piece_assets.bp),
        (Pieces::BISHOP, &piece_assets.bb),
        (Pieces::KNIGHT, &piece_assets.bn),
        (Pieces::ROOK, &piece_assets.br),
        (Pieces::QUEEN, &piece_assets.bq),
        (Pieces::KING, &piece_assets.bk),
    ];

    loop {
        let board_size = screen_height().min(screen_width());
        let square_size = board_size / 8.0;

        for i in 0..64 {
            let x = square_size * ((i % 8) as f32);
            let y = square_size * ((i / 8) as f32);
            draw_rectangle(
                x,
                y,
                square_size,
                square_size,
                board_colors[((i / 8) + (i % 8)) % 2],
            );
        }

        // Highlight active square
        if let Some(square) = active_square {
            let x = square_size * ((square % 8) as f32);
            let y = square_size * ((square / 8) as f32);
            draw_rectangle(
                x,
                y,
                square_size,
                square_size,
                active_colors[((square / 8) + (square % 8)) % 2],
            );
        }

        // Draw white pieces
        for (piece_type, texture) in white_lookup {
            let bitboard = board.get_pieces(Sides::WHITE, piece_type);
            draw_bitboard_pieces(bitboard, texture, square_size);
        }

        // Draw black pieces
        for (piece_type, texture) in black_lookup {
            let bitboard = board.get_pieces(Sides::BLACK, piece_type);
            draw_bitboard_pieces(bitboard, texture, square_size);
        }

        if let Some(square_index) = active_square {
            let moves: Vec<Move> = moves
                .clone()
                .into_iter()
                .filter(|x| x.from() == square_index)
                .collect();

            move_hints.clear();
            for _move in moves.into_iter() {
                if _move.from() == square_index {
                    move_hints.push(_move.to());

                    let x = square_size * (_move.to() % 8) as f32;
                    let y = square_size * (_move.to() / 8) as f32;

                    draw_circle(
                        x + 0.5 * square_size,
                        y + 0.5 * square_size,
                        square_size / 5.0,
                        Color::from_rgba(0, 0, 0, (255.0 * 0.14) as u8),
                    );
                }
            }
        }

        // Draw check
        let us = board.us();
        if mg.in_check(&mut board, us) {
            let king = board.get_pieces(board.us(), Pieces::KING).trailing_zeros();
            let x = square_size * (king % 8) as f32;
            let y = square_size * (king / 8) as f32;

            draw_circle(
                x + 0.5 * square_size,
                y + 0.5 * square_size,
                square_size / 2.0,
                Color::from_rgba(255, 0, 0, (255.0 * 0.14) as u8),
            );
        }

        // EN PASSANT
        if let Some(square) = board.game_state.enpassant_piece {
            let x = square_size * (square % 8) as f32;
            let y = square_size * (square / 8) as f32;

            draw_circle(
                x + 0.5 * square_size,
                y + 0.5 * square_size,
                square_size / 5.0,
                Color::from_rgba(255, 0, 0, (255.0 * 0.14) as u8),
            );
        }

        // TODO: Add mouse right deselect
        if is_mouse_button_down(MouseButton::Left) {
            let pos = mouse_position();
            let x = pos.0 / square_size;
            let y = pos.1 / square_size;

            if x > 8.0 || y > 8.0 {
                active_square = None;
            } else {
                let target = (y as usize) * 8 + (x as usize);

                if move_hints.contains(&target) {
                    for _move in moves.clone() {
                        if _move.from() == active_square.expect("Corrupted board state")
                            && _move.to() == target
                        {
                            // Force promote to queen
                            if _move.is_promotion() && _move.promotion_piece() != Pieces::QUEEN {
                                continue;
                            }

                            board.do_move(&_move);

                            if let Some(best_move) =
                                search.find_best_move_iter(&mut board, &mg, 20, Duration::new(1, 0))
                            {
                                board.do_move(&best_move);
                            }

                            moves = mg.gen_legal_moves_no_rep(&mut board);

                            break;
                        }
                    }
                }
                active_square = Some((y as usize) * 8 + (x as usize));
            }
        }

        next_frame().await
    }
}
