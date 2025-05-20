use std::{
    io::{Read, Write},
    str::FromStr,
};

use rand::{Rng, seq::SliceRandom};

use super::{Game, Resulting, cell::Cell};

/// The sudoku grid with perfomed moves
///
/// `R` is the range of values, i.e. the MAX+1
/// `N` The size of a square
/// `NN` The size of the grid, `N*N`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sudoku<const R: u32, const N: usize, const NN: usize> {
    /// Two dimensional array of Cell
    grid: [[Cell<R>; NN]; NN],
    /// Remember the performed move in a stack
    /// `(removed_possiblity, [line, column])`
    moves: Vec<(u32, [usize; 2])>,
}

// This is a macro, it is expanded at compile time. '$' means macro variable.
// This allow to easily iterate over the correlated cells of one cell
// We call correlated cells the one in the same line, column or square
macro_rules! correlated_cells {
    (
        $n:expr, // The size of a square (N)
        $nn:expr, // The size of a square squared (N * N)
        [$y:expr, $x:expr], // The targated cell
        [$iy:ident, $ix:ident], // The name of the variable succesively storing the correlated cell
        $exec:tt // The block of code to execute for each correlated cell
    ) => {
        // line
        for $ix in 0..$nn {
            if $ix != $x {
                let $iy = $y;
                $exec
            }
        }
        // column
        for $iy in 0..$nn {
            if $iy != $y {
                let $ix = $x;
                $exec
            }
        }
        // square
        for $iy in ($y - $y % $n..).take($n) {
            for $ix in ($x - $x % $n..).take($n) {
                if $iy != $y && $ix != $x {
                    let $ix = $ix;
                    let $iy = $iy;
                    $exec
                }
            }
        }
    };
}

// Implementation of associated methods to Sudoku
impl<const R: u32, const N: usize, const NN: usize> Sudoku<R, N, NN> {
    pub fn generate(rng: &mut impl Rng) -> Self {
        loop {
            let mut grid = Self::default();
            let mut line: Vec<_> = (0..NN as u32).collect();
            line.shuffle(rng);
            for x in 0..NN {
                let Some(_) = grid.place_number(line[x], [0, x]) else {
                    unreachable!()
                };
            }
            // found experimentally
            let mut tickets = 6_400;
            if let Some(solved) = grid.brute_force(&mut tickets) {
                return solved;
            }
        }
    }

    /// Remove all possibilities which are not the desired `value`.
    ///
    /// It has a cascading effect on correlated cells.
    /// It returns the number of pushed moves.
    /// It may fail if the grid turns out to be inconsistent, in that case
    /// no move is pushed and it returns `None`
    #[inline]
    #[must_use]
    fn place_number(&mut self, value: u32, [y, x]: [usize; 2]) -> Option<usize> {
        // self.debug_print();
        // dbg!(x, y, value);
        debug_assert!(self.grid[y][x].contains(value));
        let mut count = 0;
        for iv in self.grid[y][x] {
            if iv != value {
                if self.grid[y][x].contains(iv) {
                    if let Some(n) = self.remove(iv, [y, x]) {
                        count += n;
                    } else {
                        self.pop_n_moves(count);
                        return None;
                    }
                }
            }
        }
        Some(count)
    }

    /// Remove one possiblitity of a cell, with a cascading effect.
    ///
    /// The effect is recursively cascading on correlated cells.
    /// It returns the number of moves it has pushed.
    /// It may fail if the grid turns out to be inconsistent, in that case
    /// no moves are pushed and it returns `None`
    fn remove(&mut self, value: u32, [y, x]: [usize; 2]) -> Option<usize> {
        if !self.grid[y][x].remove(value) {
            return None;
        }
        self.moves.push((value, [y, x]));
        let mut count = 1; // we count the number of pushed moves

        // if the current cell as a unic possiblity
        // all correlated cells can't have it
        if let Some(value) = self.grid[y][x].get_value() {
            correlated_cells!(N, NN, [y, x], [iy, ix], {
                if self.grid[iy][ix].contains(value) {
                    if let Some(n) = self.remove(value, [iy, ix]) {
                        count += n;
                    } else {
                        // cascading on a coherent grid should not fail
                        // if it does, we know the grid is incoherent
                        // we imediatelly buble up the error
                        self.pop_n_moves(count);
                        return None;
                    }
                }
            });
        }

        // Now that we removed the `value` possibility of the cell `[y, x]`
        // Maybe a correlated cell now is the only one with it in its correlated neigbourhood
        // If it is the case, it become its only possibility, and we cascade the effect
        correlated_cells!(N, NN, [y, x], [iy, ix], {
            let unics = [
                self.unic_on_row([iy, ix]),
                self.unic_on_column([iy, ix]),
                self.unic_on_square([iy, ix]),
            ];
            for unic in unics {
                // multiple forced values means incoherent grid
                if unic.len() > 1 {
                    self.pop_n_moves(count);
                    return None;
                }
            }
            // The line, the column and the square may all enforce a unic value
            // We must check if it is the same, otherwise, it means the grid is incoherent
            let unics = unics.map(Cell::get_value);
            let mut iter = unics.into_iter().flatten();
            let unic = match [iter.next(), iter.next(), iter.next()] {
                [Some(a), Some(b), Some(c)] => {
                    if a == b && a == c {
                        Some(a)
                    } else {
                        self.pop_n_moves(count);
                        return None;
                    }
                }
                [Some(a), Some(b), _] => {
                    if a == b {
                        Some(a)
                    } else {
                        self.pop_n_moves(count);
                        return None;
                    }
                }
                [v, _, _] => v,
            };
            // if there is an enforced value
            if let Some(value) = unic {
                // we remove all other possibilities
                if let Some(n) = self.place_number(value, [iy, ix]) {
                    count += n;
                } else {
                    // it may fail if the grid is incoherent
                    self.pop_n_moves(count);
                    return None;
                }
            }
        });
        Some(count)
    }

    // For a given cell, returns all possibilities of the cell
    // which are not present in the other one of its line (row).
    // If there is more than one, its incoherent, it means there
    // is more than one enforced value.
    fn unic_on_row(&self, [y, x]: [usize; 2]) -> Cell<R> {
        let mut possibles = Cell::EMPTY;
        for ix in 0..NN {
            if ix != x {
                possibles |= self.grid[y][ix];
            }
        }
        !possibles & self.grid[y][x]
    }
    // For a given cell, returns all possibilities of the cell
    // which are not present in the other one of its column.
    // If there is more than one, its incoherent, it means there
    // is more than one enforced value.
    fn unic_on_column(&self, [y, x]: [usize; 2]) -> Cell<R> {
        let mut possibles = Cell::EMPTY;
        for iy in 0..NN {
            if iy != y {
                possibles |= self.grid[iy][x];
            }
        }
        !possibles & self.grid[y][x]
    }
    // For a given cell, returns all possibilities of the cell
    // which are not present in the other one of its square.
    // If there is more than one, its incoherent, it means there
    // is more than one enforced value.
    fn unic_on_square(&self, [y, x]: [usize; 2]) -> Cell<R> {
        let mut possibles = Cell::EMPTY;
        for iy in (y - y % N..).take(N) {
            for ix in (x - x % N..).take(N) {
                if iy != y || ix != x {
                    possibles |= self.grid[iy][ix];
                }
            }
        }
        !possibles & self.grid[y][x]
    }

    // pub fn debug_print(&self) {
    //     for y in 0..NN {
    //         for x in 0..NN {
    //             self.grid[y][x].debug_print();
    //             print!(" ");
    //         }
    //         println!();
    //     }
    //     println!();
    // }

    fn pop_n_moves(&mut self, n: usize) {
        for _ in 0..n {
            let (value, [y, x]) = self.moves.pop().unwrap();
            self.grid[y][x] |= Cell::from_value(value);
        }
    }
    fn brute_force(&mut self, tickets: &mut u32) -> Option<Self> {
        // let mut game = self.trivial_moves();
        if self.is_accepting() {
            return Some(self.clone());
        }
        if *tickets == 0 {
            return None;
        }
        *tickets -= 1;

        let mut min = None;
        // for all cells
        for y in 0..NN {
            for x in 0..NN {
                // how many possibilities
                let len = self.grid[y][x].len();
                match (len, min) {
                    // less than 2
                    (0 | 1, _) => {}
                    // more than 2, no previous minimum
                    (_, None) => {
                        min = Some((len, [y, x]));
                    }
                    // more than 2, compare to previous minimum
                    (_, Some((v, _))) if len < v => {
                        min = Some((len, [y, x]));
                    }
                    _ => {}
                }
            }
        }
        // TODO: if available move is impossible, we may miss other cells
        // if we found a cell
        if let Some((_, [y, x])) = min {
            // for each possibilities
            for value in self.grid[y][x] {
                // push the move and call the callback function
                // TODO: investigate why available moves can be invalid
                // let Some(mut result) = game.push_move((value, [y, x])) else {
                //     game.print(&mut std::io::stdout());
                //     println!("y={} x={}  {}", y, x, value);
                //     unreachable!()
                // };
                if let Some(mut result) = self.push_move((value, [y, x])) {
                    if let Some(found) = result.brute_force(tickets) {
                        return Some(found);
                    }
                }
            }
        }
        return None;
    }
    pub fn save(&self, mut writer: impl Write) {
        for yn in 0..N {
            for y in 0..N {
                for xn in 0..N {
                    for x in 0..N {
                        write!(writer, "{} ", self.grid[yn * N + y][xn * N + x].to_char()).unwrap();
                    }
                    write!(writer, " ").unwrap();
                }
                writeln!(writer).unwrap();
            }
            writeln!(writer).unwrap();
        }
    }
}

#[derive(Debug, Clone)]
pub enum SudokuAny {
    Sudoku1(Sudoku<1, 1, 1>),
    Sudoku2(Sudoku<4, 2, 4>),
    Sudoku3(Sudoku<9, 3, 9>),
    Sudoku4(Sudoku<16, 4, 16>),
    Sudoku5(Sudoku<25, 5, 25>),
    Sudoku6(Sudoku<36, 6, 36>),
}
impl SudokuAny {
    pub fn brute_force(&mut self, tickets: &mut u32) -> Option<Self> {
        match self {
            SudokuAny::Sudoku1(sudoku) => (sudoku.brute_force(tickets)).map(SudokuAny::Sudoku1),
            SudokuAny::Sudoku2(sudoku) => (sudoku.brute_force(tickets)).map(SudokuAny::Sudoku2),
            SudokuAny::Sudoku3(sudoku) => (sudoku.brute_force(tickets)).map(SudokuAny::Sudoku3),
            SudokuAny::Sudoku4(sudoku) => (sudoku.brute_force(tickets)).map(SudokuAny::Sudoku4),
            SudokuAny::Sudoku5(sudoku) => (sudoku.brute_force(tickets)).map(SudokuAny::Sudoku5),
            SudokuAny::Sudoku6(sudoku) => (sudoku.brute_force(tickets)).map(SudokuAny::Sudoku6),
        }
    }
    pub fn print(&self, writer: &mut impl Write) {
        match self {
            SudokuAny::Sudoku1(sudoku) => sudoku.print(writer),
            SudokuAny::Sudoku2(sudoku) => sudoku.print(writer),
            SudokuAny::Sudoku3(sudoku) => sudoku.print(writer),
            SudokuAny::Sudoku4(sudoku) => sudoku.print(writer),
            SudokuAny::Sudoku5(sudoku) => sudoku.print(writer),
            SudokuAny::Sudoku6(sudoku) => sudoku.print(writer),
        }
    }
    pub fn generate(size: u32, rng: &mut impl Rng) -> Self {
        match size {
            1 => Self::Sudoku1(Sudoku::generate(rng)),
            2 => Self::Sudoku2(Sudoku::generate(rng)),
            3 => Self::Sudoku3(Sudoku::generate(rng)),
            4 => Self::Sudoku4(Sudoku::generate(rng)),
            5 => Self::Sudoku5(Sudoku::generate(rng)),
            6 => Self::Sudoku6(Sudoku::generate(rng)),
            _ => panic!("invalid sudoku size"),
        }
    }
}

impl<const R: u32, const N: usize, const NN: usize> Default for Sudoku<R, N, NN> {
    fn default() -> Self {
        Self {
            grid: [[Cell::FULL; NN]; NN],
            moves: Vec::new(),
        }
    }
}

impl FromStr for SudokuAny {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n = s.chars().filter(|c| !c.is_whitespace()).count();
        Ok(match n {
            1 => Self::Sudoku1(s.parse()?),
            16 => Self::Sudoku2(s.parse()?),
            81 => Self::Sudoku3(s.parse()?),
            256 => Self::Sudoku4(s.parse()?),
            625 => Self::Sudoku5(s.parse()?),
            1296 => Self::Sudoku6(s.parse()?),
            _ => panic!("invalid grid size {}", n),
        })
    }
}

impl<const R: u32, const N: usize, const NN: usize> FromStr for Sudoku<R, N, NN> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cells: Vec<_> = s
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(Cell::<R>::from_char)
            .collect();
        assert_eq!(cells.len(), NN * NN);
        let mut game = Self::default();
        let mut cells = cells.iter();
        for y in 0..NN {
            for x in 0..NN {
                if let Some(value) = cells.next().unwrap().get_value() {
                    let Some(_) = game.place_number(value, [y, x]) else {
                        panic!("inconscistent grid value {:?} at [{}, {}]", value, y, x);
                    };
                }
            }
        }
        Ok(game)
    }
}
fn rewind<const R: u32, const N: usize, const NN: usize>(game: &mut Sudoku<R, N, NN>, n: &usize) {
    game.pop_n_moves(*n);
}

// The Sudoku struct implements the Game trait (interface)
impl<const R: u32, const N: usize, const NN: usize> Game for Sudoku<R, N, NN> {
    // The removed possiblity at given coords
    type Move = (u32, [usize; 2]);
    // The number of moves to pop
    type RewindData = usize;

    fn push_move(&mut self, (value, [y, x]): Self::Move) -> Option<Resulting<Self, usize>> {
        let count = self.remove(value, [y, x])?;
        Some(Resulting {
            game: self,
            data: count,
            rewind,
        })
    }

    fn print(&self, mut writer: impl Write) {
        fn print_line_sep(
            mut writer: impl Write,
            n: usize,
            nn: usize,
            left: char,
            right: char,
            line: char,
            cross_thin: char,
            cross_bold: char,
        ) {
            write!(writer, "{left}{line}{line}{line}").unwrap();
            for x in 1..nn {
                if x % n == 0 {
                    write!(writer, "{cross_bold}{line}{line}{line}").unwrap();
                } else {
                    write!(writer, "{cross_thin}{line}{line}{line}").unwrap();
                }
            }
            writeln!(writer, "{right}").unwrap();
        }
        print_line_sep(&mut writer, N, NN, '┏', '┓', '━', '┯', '┳');
        for y in 0..NN {
            if y > 0 {
                if y % N == 0 {
                    print_line_sep(&mut writer, N, NN, '┣', '┫', '━', '┿', '╋');
                } else {
                    print_line_sep(&mut writer, N, NN, '┠', '┨', '─', '┼', '╂');
                }
            }
            for x in 0..NN {
                write!(writer, "{}", if x % N == 0 { '┃' } else { '│' }).unwrap();
                let c = match self.grid[y][x].to_char() {
                    '_' => ' ',
                    c => c,
                };
                write!(writer, " {} ", c).unwrap();
            }
            writeln!(writer, "┃").unwrap();
        }
        print_line_sep(&mut writer, N, NN, '┗', '┛', '━', '┷', '┻');
    }

    // Because of the way moves are pushed, it enforces that the grid
    // remains coherent. We only have to check how many moves were pushed.
    fn is_accepting(&self) -> bool {
        self.moves.len() == NN * NN * (NN - 1)
    }
}
