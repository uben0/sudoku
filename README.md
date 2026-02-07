# Sudoku solver and generator

It can solve or generate sudoku grids of size 4×4, 9×9, 16×16, 25×25 and 36×36 in the blink of an eye.

To solve a sudoku, use the `solve` subcommand and provide your grid in a text file.

```
cargo run --release solve grid-3-a.txt
```

To generate a sudoku, use the `generate` subcommand, provide a size and optionally a seed.

```
cargo run --release generate 3
```

| size | grid   |
|-----:|:------:|
| 1    |  1×1   |
| 2    |  4×4   |
| 3    |  9×9   |
| 4    |  16×16 |
| 5    |  25×25 |
| 6    |  36×36 |

Don't forget to run in `release` mode for instantaneous solving and generation.

## TODO

- [ ] deterministic pre-fill, then remap values at end when option provided

- [ ] Investigate why stuck on solve `grid-6-a.txt` with `seed=8`
- [ ] Investigate why recursion level is so high
  - use a hashmap to postpone function collapse
  - instead of hashmap, combine a vec and a grid with booleans
- [ ] Benchmark.
- [x] Remove generic parameter `R`.
- [x] Remove generic parameter `NN`.
- [ ] Remove `Game` trait.
- [x] Fix invalid moves being available.
- [x] Optimize.
- [ ] Maybe avoid recursion by using a stack.
- [x] Better random choices.
- [ ] Character mapping starting at 0.
- [x] Remove duplicated code.
- [ ] Simplify rewinder.
- [ ] Wasm friendly.
- [x] Error handling
