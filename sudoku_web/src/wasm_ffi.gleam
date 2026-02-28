import gleam/dynamic.{type Dynamic}
import gleam/dynamic/decode
import gleam/javascript/promise.{type Promise}
import gleam/list
import gleam/option.{type Option, None, Some}

@external(javascript, "./wasm_ffi.js", "loadWasm")
pub fn load_wasm() -> Promise(Nil)

@external(javascript, "./wasm_ffi.js", "sudokuFill")
fn sudoku_fill_ffi(
  size: Int,
  seed: Int,
  grid: Dynamic,
  retry: Bool,
  sparse: Bool,
) -> Dynamic

@external(javascript, "./wasm_ffi.js", "valueToChar")
pub fn value_to_char(value: Int) -> String

pub fn sudoku_fill(
  size: Int,
  seed: Int,
  grid: List(Option(Int)),
  retry retry: Bool,
  sparse sparse: Bool,
) -> Option(List(Option(Int))) {
  sudoku_fill_ffi(size, seed, encode_grid(grid), retry, sparse) |> decode_grid
}

fn decode_grid(grid: Dynamic) -> Option(List(Option(Int))) {
  let assert Ok(grid) =
    grid
    |> decode.run(decode.optional(decode.list(decode.int)))
  use grid <- option.map(grid)
  use cell <- list.map(grid)
  case cell {
    255 -> None
    value -> Some(value)
  }
}

fn encode_grid(grid: List(Option(Int))) -> Dynamic {
  dynamic.array({
    use cell <- list.map(grid)
    dynamic.int(case cell {
      None -> 255
      Some(value) -> value
    })
  })
}
