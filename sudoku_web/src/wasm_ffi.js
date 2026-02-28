const WASM_PATH = "./sudoku.wasm";
const NOT_FOUND = 1;

let wasm = null;

function len(size) {
  return size * size * size * size;
}

function grid_view(size) {
  return new Uint8Array(
    wasm.memory.buffer,
    wasm.sudoku_ptr(),
    len(size)
  );
}

function encode_grid(size, grid) {
  let view = grid_view(size);
  for (let i = 0; i < len(size); i++) {
    view[i] = grid[i]
  }
}

function decode_grid(size) {
  return Array.from(grid_view(size));
}

export async function loadWasm() {
  const { instance } = await WebAssembly.instantiateStreaming(fetch(WASM_PATH), {});
  wasm = instance.exports;
}

export function sudokuFill(size, seed, grid, retry, sparse) {
  encode_grid(size, grid);
  while (wasm.sudoku_fill(size, seed, sparse) == NOT_FOUND) {
    if (!retry) return null;
    seed += 1;
  }
  return decode_grid(size);
}

export function valueToChar(value) {
  return String.fromCodePoint(wasm.value_to_char(value));
}
