# Chess engine
To run:
```
git clone https://github.com/enriktigasna/chess-engine
cd chess-engine
# Make sure to set target cpu in env variables before running
# Windows: $env:RUSTFLAGS="-Ctarget-cpu=native"
# Linux: export RUSTFLAGS="-Ctarget-cpu=native"
cargo run frontend --release
cargo build uci --release # Use UCI however you wish
```

Read more [here](https://mackan.dev/posts/Bitboard%20Chess%20Engine%20in%20Rust)
# **Engine features**
- Bitboards
- Move generation
- SIMD Optimizations
- Material evaluation
- Minimax/Negamax
- Alpha-Beta pruning
- Move ordering
- Piece-Square tables
- Quiescence Search
- Principal Variation Search
- Nullmove pruning/heuristic

# **Program features**
- Rust macroquad frontend
- UCI Integration
- SPRT testing
