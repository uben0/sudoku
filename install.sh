#!/bin/sh

mkdir -p static
cargo build -p sudoku_wasm --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/sudoku_web.wasm static/sudoku.wasm
cd sudoku_web; gleam run -m lustre/dev -- build --outdir=../static;
