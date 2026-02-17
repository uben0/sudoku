static: target/wasm32-unknown-unknown/release/sudoku_web.wasm
	mkdir -p static
	cp sudoku-web/src/index.html static/index.html
	cp sudoku-web/src/index.js   static/index.js
	cp sudoku-web/src/style.css  static/style.css
	cp target/wasm32-unknown-unknown/release/sudoku_web.wasm static/sudoku.wasm

target/wasm32-unknown-unknown/release/sudoku_web.wasm:
	cargo build -p sudoku-web --release --target wasm32-unknown-unknown
