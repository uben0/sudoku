# Sudoku solver and generator

It can solve or generate sudoku grids of size 4×4, 9×9, 16×16, 25×25, 36×36 and 49×49 in the blink of an eye.

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
| 7    |  49×49 |

Don't forget to run in `release` mode for instantaneous solving and generation.

## TODO

Add the following remove cascade:

- When for a given row or column, a value is only possible in one square, all other cells of the square can't have this value.
- When for a given square, a value is only possible in one row or column, all other cells of the row or column can't have this value.
