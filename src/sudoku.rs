use std::{
    io::Write,
    ops::{Index, IndexMut},
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
pub struct Sudoku<const N: usize> {
    /// Two dimensional array of Cell
    grid: [[[[Cell<N>; N]; N]; N]; N],
    /// Remember the performed move in a stack
    /// `(removed_possiblity, [line, column])`
    moves: Vec<(u32, Pos)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    x_1: u8,
    x_2: u8,
    y_1: u8,
    y_2: u8,
}

// This is a macro, it is expanded at compile time. '$' means macro variable.
// This allow to easily iterate over the correlated cells of one cell
// We call correlated cells the one in the same line, column or square
macro_rules! correlated_cells {
    (
        $n:expr, // The size of a square (N)
        $pos:expr, // The targated cell
        $ipos:ident, // The name of the variable succesively storing the correlated cell
        { $($exec:tt)* } // The block of code to execute for each correlated cell
    ) => {
        // line (without square)
        for ix_1 in 0..$n {
            if ix_1 != $pos.x_1 {
                for ix_2 in 0..$n {
                    let $ipos = Pos {
                        x_1: ix_1,
                        x_2: ix_2,
                        ..$pos
                    };
                    $($exec)*
                }
            }
        }
        // column (without square)
        for iy_1 in 0..$n {
            if iy_1 != $pos.y_1 {
                for iy_2 in 0..$n {
                    let $ipos = Pos {
                        y_1: iy_1,
                        y_2: iy_2,
                        ..$pos
                    };
                    $($exec)*
                }
            }
        }
        // square (full)
        for iy_2 in 0..$n {
            for ix_2 in 0..$n {
                if iy_2 != $pos.y_2 || ix_2 != $pos.x_2 {
                    let $ipos = Pos {
                        y_2: iy_2,
                        x_2: ix_2,
                        ..$pos
                    };
                    $($exec)*
                }
            }
        }
    };
}

impl<const N: usize> Index<Pos> for [[[[Cell<N>; N]; N]; N]; N] {
    type Output = Cell<N>;

    #[inline]
    fn index(&self, index: Pos) -> &Self::Output {
        &self[index.y_1 as usize][index.y_2 as usize][index.x_1 as usize][index.x_2 as usize]
    }
}
impl<const N: usize> IndexMut<Pos> for [[[[Cell<N>; N]; N]; N]; N] {
    #[inline]
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self[index.y_1 as usize][index.y_2 as usize][index.x_1 as usize][index.x_2 as usize]
    }
}

// Implementation of associated methods to Sudoku
impl<const N: usize> Sudoku<N> {
    pub fn generate(rng: &mut impl Rng) -> Self {
        loop {
            let mut grid = Self::default();
            let mut line: Vec<_> = (0..(N * N) as u32).collect();
            line.shuffle(rng);
            for x_1 in 0..N as u8 {
                for x_2 in 0..N as u8 {
                    let Some(_) = grid.place_number(
                        line.pop().unwrap(),
                        Pos {
                            x_1,
                            x_2,
                            y_1: 0,
                            y_2: 0,
                        },
                    ) else {
                        unreachable!()
                    };
                }
            }
            // found experimentally
            let mut tickets = 6_400;
            if let Some(solved) = grid.brute_force(&mut tickets, rng) {
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
    fn place_number(&mut self, value: u32, pos: Pos) -> Option<usize> {
        debug_assert!(self.grid[pos].contains(value));
        let mut count = 0;
        for iv in self.grid[pos] {
            if iv != value && self.grid[pos].contains(iv) {
                if let Some(n) = self.remove(iv, pos) {
                    count += n;
                } else {
                    self.pop_n_moves(count);
                    return None;
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
    fn remove(&mut self, value: u32, pos: Pos) -> Option<usize> {
        // TODO: why failing
        if !self.grid[pos].remove(value) {
            return None;
        }
        self.moves.push((value, pos));
        let mut count = 1; // we count the number of pushed moves

        // if the current cell has a unique possiblity
        // all correlated cells can't have it
        if let Some(value) = self.grid[pos].get_value() {
            correlated_cells!(N as u8, pos, ipos, {
                if self.grid[ipos].contains(value) {
                    if let Some(n) = self.remove(value, ipos) {
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
        correlated_cells!(N as u8, pos, ipos, {
            // A determine cell will always result in enforcing its value
            // It is already unique, so we don't have to do anything
            if self.grid[ipos].len() == 1 {
                continue;
            }
            let unics = [
                self.unic_on_row(ipos),
                self.unic_on_column(ipos),
                self.unic_on_square(ipos),
            ];
            for unic in unics {
                // multiple forced values means incoherent grid
                if unic.len() > 1 {
                    self.pop_n_moves(count);
                    return None;
                }
            }
            // TODO: just merge the bitsets
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
                if let Some(n) = self.place_number(value, ipos) {
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
    fn unic_on_row(&self, pos: Pos) -> Cell<N> {
        let mut possibles = Cell::EMPTY;
        for x_1 in 0..N as u8 {
            for x_2 in 0..N as u8 {
                if x_1 != pos.x_1 || x_2 != pos.x_2 {
                    possibles |= self.grid[Pos { x_1, x_2, ..pos }];
                }
            }
        }
        !possibles & self.grid[pos]
    }
    // For a given cell, returns all possibilities of the cell
    // which are not present in the other one of its column.
    // If there is more than one, its incoherent, it means there
    // is more than one enforced value.
    fn unic_on_column(&self, pos: Pos) -> Cell<N> {
        let mut possibles = Cell::EMPTY;
        for y_1 in 0..N as u8 {
            for y_2 in 0..N as u8 {
                if y_1 != pos.y_1 || y_2 != pos.y_2 {
                    possibles |= self.grid[Pos { y_1, y_2, ..pos }];
                }
            }
        }
        !possibles & self.grid[pos]
    }
    // For a given cell, returns all possibilities of the cell
    // which are not present in the other one of its square.
    // If there is more than one, its incoherent, it means there
    // is more than one enforced value.
    fn unic_on_square(&self, pos: Pos) -> Cell<N> {
        let mut possibles = Cell::EMPTY;
        for y_2 in 0..N as u8 {
            for x_2 in 0..N as u8 {
                if y_2 != pos.y_2 || x_2 != pos.x_2 {
                    possibles |= self.grid[Pos { y_2, x_2, ..pos }];
                }
            }
        }
        !possibles & self.grid[pos]
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
            let (value, pos) = self.moves.pop().unwrap();
            self.grid[pos] |= Cell::from_value(value);
        }
    }
    fn brute_force(&mut self, tickets: &mut u32, rng: &mut impl Rng) -> Option<Self> {
        if self.is_accepting() {
            return Some(self.clone());
        }
        *tickets = tickets.checked_sub(1)?;

        let mut min = None;
        // for all cells
        for y_1 in 0..N as u8 {
            for y_2 in 0..N as u8 {
                for x_1 in 0..N as u8 {
                    for x_2 in 0..N as u8 {
                        let pos = Pos { y_1, y_2, x_1, x_2 };
                        // how many possibilities
                        let len = self.grid[pos].len();
                        match (len, min) {
                            (0, _) => unreachable!(),
                            (1, _) => {}
                            // TODO: break loop when exactly 2
                            // at least 2, no previous minimum
                            (_, None) => {
                                min = Some((len, pos));
                            }
                            // at least 2, compare to previous minimum
                            (_, Some((v, _))) if len < v => {
                                min = Some((len, pos));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // if we found a cell
        let (_, pos) = min?;
        // for each possibilities
        let mut available = self.grid[pos];
        while let Some(value) = available.get_random(rng) {
            available.pop(value);
            if let Some(mut result) = self.push_move((value, pos)) {
                if let Some(found) = result.brute_force(tickets, rng) {
                    return Some(found);
                }
            }
        }
        // for value in self.grid[pos] {
        //     // push the move and call the callback function
        // }
        return None;
    }
    // pub fn save(&self, mut writer: impl Write) {
    //     for y_1 in 0..N {
    //         for y_2 in 0..N {
    //             for x_1 in 0..N {
    //                 for x_2 in 0..N {
    //                     write!(writer, "{} ", self.grid[y_1][y_2][x_1][x_2].to_char()).unwrap();
    //                 }
    //                 write!(writer, " ").unwrap();
    //             }
    //             writeln!(writer).unwrap();
    //         }
    //         writeln!(writer).unwrap();
    //     }
    // }
}

#[derive(Debug, Clone)]
pub enum SudokuAny {
    Sudoku1(Sudoku<1>),
    Sudoku2(Sudoku<2>),
    Sudoku3(Sudoku<3>),
    Sudoku4(Sudoku<4>),
    Sudoku5(Sudoku<5>),
    Sudoku6(Sudoku<6>),
}
impl SudokuAny {
    pub fn brute_force(&mut self, tickets: &mut u32, rng: &mut impl Rng) -> Option<Self> {
        match self {
            SudokuAny::Sudoku1(sudoku) => {
                (sudoku.brute_force(tickets, rng)).map(SudokuAny::Sudoku1)
            }
            SudokuAny::Sudoku2(sudoku) => {
                (sudoku.brute_force(tickets, rng)).map(SudokuAny::Sudoku2)
            }
            SudokuAny::Sudoku3(sudoku) => {
                (sudoku.brute_force(tickets, rng)).map(SudokuAny::Sudoku3)
            }
            SudokuAny::Sudoku4(sudoku) => {
                (sudoku.brute_force(tickets, rng)).map(SudokuAny::Sudoku4)
            }
            SudokuAny::Sudoku5(sudoku) => {
                (sudoku.brute_force(tickets, rng)).map(SudokuAny::Sudoku5)
            }
            SudokuAny::Sudoku6(sudoku) => {
                (sudoku.brute_force(tickets, rng)).map(SudokuAny::Sudoku6)
            }
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

impl<const N: usize> Default for Sudoku<N> {
    fn default() -> Self {
        Self {
            grid: [[[[Cell::FULL; N]; N]; N]; N],
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

impl<const N: usize> FromStr for Sudoku<N> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cells: Vec<_> = s
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(Cell::<N>::from_char)
            .collect();
        // TODO: don't collect into Vec
        assert_eq!(cells.len(), N * N * N * N);
        let mut game = Self::default();
        let mut cells = cells.iter();
        for y_1 in 0..N as u8 {
            for y_2 in 0..N as u8 {
                for x_1 in 0..N as u8 {
                    for x_2 in 0..N as u8 {
                        let pos = Pos { y_1, y_2, x_1, x_2 };
                        if let Some(value) = cells.next().unwrap().get_value() {
                            let Some(_) = game.place_number(value, pos) else {
                                game.print(std::io::stdout());
                                panic!(
                                    "inconscistent grid value {:?} at [{}, {}, {}, {}]",
                                    value, y_1, y_2, x_1, x_2
                                );
                            };
                        }
                    }
                }
            }
        }
        Ok(game)
    }
}
fn rewind<const N: usize>(game: &mut Sudoku<N>, n: &usize) {
    game.pop_n_moves(*n);
}

// The Sudoku struct implements the Game trait (interface)
impl<const N: usize> Game for Sudoku<N> {
    // The removed possiblity at given coords
    type Move = (u32, Pos);
    // The number of moves to pop
    type RewindData = usize;

    fn push_move(&mut self, (value, pos): Self::Move) -> Option<Resulting<Self, usize>> {
        let count = self.remove(value, pos)?;
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
            left: char,
            right: char,
            line: char,
            cross_thin: char,
            cross_bold: char,
        ) {
            let nn = n * n;
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
        print_line_sep(&mut writer, N, '┏', '┓', '━', '┯', '┳');
        for y_1 in 0..N {
            for y_2 in 0..N {
                if y_1 > 0 || y_2 > 0 {
                    if y_2 == 0 {
                        print_line_sep(&mut writer, N, '┣', '┫', '━', '┿', '╋');
                    } else {
                        print_line_sep(&mut writer, N, '┠', '┨', '─', '┼', '╂');
                    }
                }
                for x_1 in 0..N {
                    for x_2 in 0..N {
                        if x_2 == 0 {
                            write!(writer, "┃").unwrap();
                        } else {
                            write!(writer, "│").unwrap();
                        }
                        let c = match self.grid[y_1][y_2][x_1][x_2].to_char() {
                            '_' => ' ',
                            c => c,
                        };
                        write!(writer, " {} ", c).unwrap();
                    }
                }
                writeln!(writer, "┃").unwrap();
            }
        }
        print_line_sep(&mut writer, N, '┗', '┛', '━', '┷', '┻');
    }

    // Because of the way moves are pushed, it enforces that the grid
    // remains coherent. We only have to check how many moves were pushed.
    fn is_accepting(&self) -> bool {
        self.moves.len() == N * N * N * N * (N * N - 1)
    }
}
