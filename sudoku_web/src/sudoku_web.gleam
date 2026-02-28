import gleam/int
import gleam/javascript/promise
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/result
import iterators
import lustre
import lustre/attribute.{
  type Attribute, class, disabled, placeholder, selected, step, type_, value,
}
import lustre/effect.{type Effect}
import lustre/element.{type Element, text}
import lustre/element/html.{button, div, input, option, p, select, table, td, tr}
import lustre/event.{on_change, on_click, on_input}
import wasm_ffi.{load_wasm, sudoku_fill, value_to_char}

type Model {
  WasmLoading
  WasmError
  Model(
    seed: Option(Int),
    size: Int,
    brush: Option(Int),
    grid: List(Option(Int)),
  )
}

type Msg {
  WasmLoaded
  WasmFailed
  Generate
  SetSize(String)
  SetSeed(String)
  Clear
  Solve
  SetBrush(Option(Int))
  PlaceValue(y: Int, x: Int)
}

fn init(_: Nil) -> #(Model, Effect(Msg)) {
  #(WasmLoading, load_wasm_effect())
}

fn update(model: Model, msg: Msg) -> #(Model, Effect(Msg)) {
  #(
    case model {
      WasmLoading ->
        case msg {
          WasmFailed -> WasmError
          WasmLoaded -> {
            let size = 3
            Model(seed: None, size:, brush: None, grid: empty(size))
          }
          _ -> model
        }
      WasmError -> WasmError
      Model(seed:, size:, grid:, brush:) ->
        case msg {
          WasmLoaded -> model
          WasmFailed -> model
          Generate ->
            Model(..model, grid: case generate_sudoku(size, seed) {
              Some(grid) -> grid
              None -> empty(size)
            })
          Clear -> {
            Model(..model, seed: None, grid: empty(size))
          }
          Solve ->
            Model(..model, grid: case solve_sudoku(size, seed, grid) {
              Some(grid) -> grid
              None -> grid
            })
          SetSize(size) -> {
            let size = int.parse(size) |> result.unwrap(0)
            Model(..model, size:, brush: None, grid: empty(size))
          }
          SetBrush(brush) -> Model(..model, brush:)
          SetSeed(input) ->
            Model(..model, seed: case input {
              "" -> None
              digits ->
                int.parse(digits) |> result.map(Some) |> result.unwrap(seed)
            })
          PlaceValue(y:, x:) -> {
            Model(
              ..model,
              grid: grid
                |> list.index_map(fn(cell, i) {
                  case i == y * size * size + x {
                    True -> brush
                    False -> cell
                  }
                }),
            )
          }
        }
    },
    effect.none(),
  )
}

const sizes = [
  "0x0",
  "1x1",
  "4x4",
  "9x9",
  "16x16",
  "25x25",
  "36x36",
  "49x49",
  "64x64",
]

fn view(model: Model) -> Element(Msg) {
  case model {
    WasmLoading -> p([], [text("loading...")])
    WasmError ->
      div([], [
        p([], [
          text("Failed to load "),
          html.code([], [text("./sudoku.wasm")]),
        ]),
        p([], [text(" Make sure the wasm file is available at the root.")]),
      ])
    Model(seed:, size:, grid:, brush:) ->
      div([], [
        view_select_size(size),
        input([
          type_("number"),
          step("1"),
          placeholder("random"),
          on_input(SetSeed),
          case seed {
            None -> value("")
            Some(seed) -> value(int.to_string(seed))
          },
        ]),
        button([on_click(Generate)], [text("generate")]),
        button([on_click(Clear)], [text("clear")]),
        button([on_click(Solve)], [text("solve")]),
        view_filler_select(size, brush),
        view_grid(size, grid),
      ])
  }
}

fn view_filler_select(size: Int, brush: Option(Int)) -> Element(Msg) {
  div([], [
    button([on_click(SetBrush(None)), disabled(brush == None)], [
      text("_"),
    ]),
    ..iterators.naturals(0)
    |> iterators.take(size * size)
    |> iterators.to_list
    |> list.map(fn(value) {
      button([on_click(SetBrush(Some(value))), disabled(brush == Some(value))], [
        text(value_to_char(value)),
      ])
    })
  ])
}

fn view_grid(size: Int, grid: List(Option(Int))) -> Element(Msg) {
  assert size * size * size * size == list.length(grid)
  table(
    [],
    list.sized_chunk(grid, size * size)
      |> list.index_map(fn(row, y) {
        tr(
          [view_delimiter(y, size)],
          row
            |> list.index_map(fn(cell, x) {
              td(
                [
                  view_delimiter(x, size),
                  view_empty(cell),
                  on_click(PlaceValue(y:, x:)),
                ],
                [view_cell(cell)],
              )
            }),
        )
      }),
  )
}

fn view_delimiter(i: Int, size: Int) -> Attribute(Msg) {
  case i % size {
    0 -> class("delimiter")
    _ -> attribute.none()
  }
}

fn view_empty(cell: Option(Int)) -> Attribute(Msg) {
  case cell {
    Some(_) -> attribute.none()
    None -> class("empty")
  }
}

fn view_cell(cell: Option(Int)) -> Element(Msg) {
  cell
  |> option.map(value_to_char)
  |> option.map(text)
  |> option.unwrap(element.none())
}

fn view_select_size(selected_size: Int) -> Element(Msg) {
  select([on_change(SetSize)], {
    use label, size <- list.index_map(sizes)
    option([selected(size == selected_size), value(int.to_string(size))], label)
  })
}

pub fn main() -> Nil {
  let assert Ok(_) =
    lustre.application(init, update, view)
    |> lustre.start(onto: "#app", with: Nil)
  Nil
}

pub fn generate_sudoku(
  size: Int,
  seed: Option(Int),
) -> Option(List(Option(Int))) {
  sudoku_fill(
    size,
    seed_or_random(seed),
    empty(size),
    retry: True,
    sparse: True,
  )
}

fn seed_or_random(seed: Option(Int)) -> Int {
  seed |> option.lazy_unwrap(fn() { int.random(max_seed) })
}

const max_seed = 4_294_967_295

fn empty(size: Int) -> List(Option(Int)) {
  list.repeat(None, size * size * size * size)
}

pub fn solve_sudoku(
  size: Int,
  seed: Option(Int),
  grid: List(Option(Int)),
) -> Option(List(Option(Int))) {
  sudoku_fill(size, seed_or_random(seed), grid, retry: False, sparse: False)
}

fn load_wasm_effect() -> Effect(Msg) {
  use dispatch <- effect.from
  load_wasm()
  |> promise.map(fn(_) { dispatch(WasmLoaded) })
  |> promise.rescue(fn(_) { dispatch(WasmFailed) })
  Nil
}
