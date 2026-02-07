mod defer;

use crate::sudoku::defer::Defer;

use super::cell::Cell;
use rand::{Rng, seq::SliceRandom};
use std::{
    io::Write,
    ops::{Index, IndexMut},
    str::FromStr,
};

/// The sudoku grid with perfomed moves
///
/// `N` The size of a square
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sudoku<const N: usize> {
    /// Four dimensional array of Cell
    ///
    /// Refer to [Pos] for dimension order
    grid: [[[[Cell<N>; N]; N]; N]; N],
    /// Remember the performed move in a stack
    /// `(removed_possiblity, [line, column])`
    moves: Vec<(u32, Pos)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pos {
    /// Selects the row chunk
    x_1: u8,
    /// Selects the row line
    x_2: u8,
    /// Selects the column chunk
    y_1: u8,
    /// Selects the column line
    y_2: u8,
}
impl Pos {
    pub fn iter<const N: usize>() -> impl Iterator<Item = Pos> {
        gen {
            for y_1 in 0..N as u8 {
                for y_2 in 0..N as u8 {
                    for x_1 in 0..N as u8 {
                        for x_2 in 0..N as u8 {
                            yield Pos { y_1, y_2, x_1, x_2 };
                        }
                    }
                }
            }
        }
    }
}

// This allow to easily iterate over the correlated cells of one cell
// We call correlated cells the one in the same line, column or square
fn correlated<const N: usize>(pos: Pos) -> impl Iterator<Item = Pos> {
    gen move {
        let n = N as u8;
        // row (without square)
        for x_1 in 0..n {
            if x_1 != pos.x_1 {
                for x_2 in 0..n {
                    yield Pos { x_1, x_2, ..pos };
                }
            }
        }
        // column (without square)
        for y_1 in 0..n {
            if y_1 != pos.y_1 {
                for y_2 in 0..n {
                    yield Pos { y_1, y_2, ..pos };
                }
            }
        }
        // square (full)
        for y_2 in 0..n {
            for x_2 in 0..n {
                if y_2 != pos.y_2 || x_2 != pos.x_2 {
                    yield Pos { y_2, x_2, ..pos };
                }
            }
        }
    }
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
        let mut defer = Defer::<N>::new();
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
                        &mut defer,
                    ) else {
                        unreachable!()
                    };
                }
            }
            // found experimentally
            if let Some(solved) = grid.brute_force(rng) {
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
    fn place_number(&mut self, value: u32, pos: Pos, defer: &mut Defer<N>) -> Option<usize> {
        if !self.grid[pos].contains(value) {
            return None;
        }
        let mut count = 0;
        for iv in self.grid[pos] {
            if iv != value && self.grid[pos].contains(iv) {
                if let Some(n) = self.remove(iv, pos, defer) {
                    count += n;
                } else {
                    self.pop_n_moves(count);
                    return None;
                }
            }
        }
        Some(count)
    }

    fn remove(&mut self, value: u32, pos: Pos, defer: &mut Defer<N>) -> Option<usize> {
        defer.clear();
        defer.push(value, pos);
        let mut pushed = 0;

        while let Some((value, pos)) = defer.pop() {
            if !self.grid[pos].contains(value) {
                continue;
            }
            if self.grid[pos].len() <= 1 {
                self.pop_n_moves(pushed);
                return None;
            }

            self.grid[pos].remove(value);
            self.moves.push((value, pos));
            pushed += 1;

            // if the current cell has a unique possiblity
            // all correlated cells can't have it
            if let Some(value) = self.grid[pos].get_value() {
                for pos in correlated::<N>(pos) {
                    if self.grid[pos].contains(value) {
                        defer.push(value, pos);
                    }
                }
            }

            // Now that we removed the `value` possibility of the cell `[y, x]`
            // Maybe a correlated cell now is the only one with it in its correlated neigbourhood
            // If it is the case, it become its only possibility, and we cascade the effect
            for pos in correlated::<N>(pos) {
                // A determine cell will always result in enforcing its value
                // It is already unique, so we don't have to do anything
                if self.grid[pos].len() == 1 {
                    continue;
                }
                // TODO: investigate optimisation when a value can't be placed anymore, to short-circuit search
                let unic =
                    self.unic_on_row(pos) | self.unic_on_column(pos) | self.unic_on_square(pos);

                if unic.len() == 0 {
                    continue;
                }

                let Some(value) = unic.get_value() else {
                    // more than one value is enforce in the cell, leading to incoherence
                    self.pop_n_moves(pushed);
                    return None;
                };

                if !self.grid[pos].contains(value) {
                    self.pop_n_moves(pushed);
                    return None;
                }
                for iv in self.grid[pos] {
                    if iv != value && self.grid[pos].contains(iv) {
                        defer.push(iv, pos);
                    }
                }
            }
        }
        Some(pushed)
    }

    /// Remove one possiblitity of a cell, with a cascading effect.
    ///
    /// The effect is recursively cascading on correlated cells.
    /// It returns the number of moves it has pushed.
    /// It may fail if the grid turns out to be inconsistent, in that case
    /// no moves are pushed and it returns `None`
    // fn remove(&mut self, value: u32, pos: Pos) -> Option<usize> {
    //     if !self.grid[pos].contains(value) {
    //         return None;
    //     }
    //     if self.grid[pos].len() <= 1 {
    //         return None;
    //     }
    //     self.grid[pos].remove(value);
    //     self.moves.push((value, pos));
    //     let mut count = 1; // we count the number of pushed moves

    //     // if the current cell has a unique possiblity
    //     // all correlated cells can't have it
    //     if let Some(value) = self.grid[pos].get_value() {
    //         for pos in correlated::<N>(pos) {
    //             if self.grid[pos].contains(value) {
    //                 if let Some(n) = self.remove(value, pos) {
    //                     count += n;
    //                 } else {
    //                     // cascading on a coherent grid should not fail
    //                     // if it does, we know the grid is incoherent
    //                     // we imediatelly buble up the error
    //                     self.pop_n_moves(count);
    //                     return None;
    //                 }
    //             }
    //         }
    //     }

    //     // Now that we removed the `value` possibility of the cell `[y, x]`
    //     // Maybe a correlated cell now is the only one with it in its correlated neigbourhood
    //     // If it is the case, it become its only possibility, and we cascade the effect
    //     for pos in correlated::<N>(pos) {
    //         // A determine cell will always result in enforcing its value
    //         // It is already unique, so we don't have to do anything
    //         if self.grid[pos].len() == 1 {
    //             continue;
    //         }
    //         // TODO: investigate optimisation when a value can't be placed anymore, to short-circuit search
    //         let unic = self.unic_on_row(pos) | self.unic_on_column(pos) | self.unic_on_square(pos);

    //         if unic.len() == 0 {
    //             continue;
    //         }
    //         let Some(value) = unic.get_value() else {
    //             self.pop_n_moves(count);
    //             return None;
    //         };

    //         // we remove all other possibilities
    //         if let Some(n) = self.place_number(value, pos) {
    //             count += n;
    //         } else {
    //             // it may fail if the grid is incoherent
    //             self.pop_n_moves(count);
    //             return None;
    //         }
    //     }

    //     Some(count)
    // }

    // For a given cell, returns all possibilities of the cell
    // which are not present in the other one of its line (row).
    // If there is more than one, its incoherent, it means there
    // is more than one enforced value.
    #[must_use]
    fn unic_on_row(&self, pos: Pos) -> Cell<N> {
        let mut possibles = Cell::EMPTY;
        for x_1 in 0..N as u8 {
            for x_2 in 0..N as u8 {
                // TODO: replace if by aritmetic filter
                if x_1 != pos.x_1 || x_2 != pos.x_2 {
                    possibles |= self.grid[Pos { x_1, x_2, ..pos }];
                }
            }
            if possibles == Cell::FULL {
                return Cell::EMPTY;
            }
        }
        !possibles
    }
    // For a given cell, returns all possibilities of the cell
    // which are not present in the other one of its column.
    // If there is more than one, its incoherent, it means there
    // is more than one enforced value.
    #[must_use]
    fn unic_on_column(&self, pos: Pos) -> Cell<N> {
        let mut possibles = Cell::EMPTY;
        for y_1 in 0..N as u8 {
            for y_2 in 0..N as u8 {
                if y_1 != pos.y_1 || y_2 != pos.y_2 {
                    possibles |= self.grid[Pos { y_1, y_2, ..pos }];
                }
            }
            if possibles == Cell::FULL {
                return Cell::EMPTY;
            }
        }
        !possibles
    }
    // For a given cell, returns all possibilities of the cell
    // which are not present in the other one of its square.
    // If there is more than one, its incoherent, it means there
    // is more than one enforced value.
    #[must_use]
    fn unic_on_square(&self, pos: Pos) -> Cell<N> {
        let mut possibles = Cell::EMPTY;
        for y_2 in 0..N as u8 {
            for x_2 in 0..N as u8 {
                if y_2 != pos.y_2 || x_2 != pos.x_2 {
                    possibles |= self.grid[Pos { y_2, x_2, ..pos }];
                }
            }
            if possibles == Cell::FULL {
                return Cell::EMPTY;
            }
        }
        !possibles
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
    fn best_pos_for_birfucation(&self) -> Option<Pos> {
        // TODO: try collect more than one candidate for later use
        let mut min = None;
        for pos in Pos::iter::<N>() {
            // how many possibilities
            let len = self.grid[pos].len();
            match (len, min) {
                (0, _) => unreachable!(),
                (1, _) => {}
                // TODO: compare with or without shortcircuit
                (2, _) => {
                    return Some(pos);
                }
                // no previous minimum
                (_, None) => {
                    min = Some(pos);
                }
                // at least 3, compare to previous minimum
                (_, Some(p)) if len < self.grid[p].len() => {
                    min = Some(pos);
                }
                _ => {}
            }
        }
        // if we found a cell
        min
    }

    fn pop_defer(defer: &mut Cell<N>, rng: &mut impl Rng) -> Option<u32> {
        let value = defer.choose(rng)?;
        *defer = *defer - value;
        Some(value)
    }

    // TODO: reuse buffer for remove2 to avoid allocation
    fn brute_force(&mut self, rng: &mut impl Rng) -> Option<Self> {
        if self.is_accepting() {
            return Some(self.clone());
        }

        let mut pos = self.best_pos_for_birfucation()?;
        let mut cell = self.grid[pos];

        let mut pushed: Vec<usize> = Vec::from([]);
        let mut defer: Vec<(Cell<N>, Pos)> = Vec::from([]);

        let mut persist = Defer::<N>::new();

        loop {
            if let Some(value) = Self::pop_defer(&mut cell, rng) {
                if let Some(moved) = self.place_number(value, pos, &mut persist) {
                    if self.is_accepting() {
                        return Some(self.clone());
                    }

                    defer.push((cell, pos));
                    pushed.push(moved);

                    pos = self.best_pos_for_birfucation()?;
                    cell = self.grid[pos];
                }
            } else {
                let Some(unpush) = pushed.pop() else {
                    // all possibilities explored
                    return None;
                };
                self.pop_n_moves(unpush);
                (cell, pos) = defer.pop().unwrap();
            }
        }
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
    fn print(&self, mut writer: impl Write) -> Result<(), std::io::Error> {
        fn print_line_sep(
            mut writer: impl Write,
            n: usize,
            left: char,
            right: char,
            line: char,
            cross_thin: char,
            cross_bold: char,
        ) -> Result<(), std::io::Error> {
            let nn = n * n;
            write!(writer, "{left}{line}{line}{line}")?;
            for x in 1..nn {
                if x % n == 0 {
                    write!(writer, "{cross_bold}{line}{line}{line}")?;
                } else {
                    write!(writer, "{cross_thin}{line}{line}{line}")?;
                }
            }
            writeln!(writer, "{right}")?;
            Ok(())
        }
        print_line_sep(&mut writer, N, '┏', '┓', '━', '┯', '┳')?;
        for y_1 in 0..N {
            for y_2 in 0..N {
                if y_1 > 0 || y_2 > 0 {
                    if y_2 == 0 {
                        print_line_sep(&mut writer, N, '┣', '┫', '━', '┿', '╋')?;
                    } else {
                        print_line_sep(&mut writer, N, '┠', '┨', '─', '┼', '╂')?;
                    }
                }
                for x_1 in 0..N {
                    for x_2 in 0..N {
                        if x_2 == 0 {
                            write!(writer, "┃")?;
                        } else {
                            write!(writer, "│")?;
                        }
                        let c = match self.grid[y_1][y_2][x_1][x_2].to_char() {
                            '_' => ' ',
                            c => c,
                        };
                        write!(writer, " {} ", c)?;
                    }
                }
                writeln!(writer, "┃")?;
            }
        }
        print_line_sep(&mut writer, N, '┗', '┛', '━', '┷', '┻')?;
        Ok(())
    }
    // Because of the way moves are pushed, it enforces that the grid
    // remains coherent. We only have to check how many moves were pushed.
    fn is_accepting(&self) -> bool {
        self.moves.len() == N * N * N * N * (N * N - 1)
    }
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
    pub fn brute_force(&mut self, rng: &mut impl Rng) -> Option<Self> {
        match self {
            SudokuAny::Sudoku1(sudoku) => (sudoku.brute_force(rng)).map(SudokuAny::Sudoku1),
            SudokuAny::Sudoku2(sudoku) => (sudoku.brute_force(rng)).map(SudokuAny::Sudoku2),
            SudokuAny::Sudoku3(sudoku) => (sudoku.brute_force(rng)).map(SudokuAny::Sudoku3),
            SudokuAny::Sudoku4(sudoku) => (sudoku.brute_force(rng)).map(SudokuAny::Sudoku4),
            SudokuAny::Sudoku5(sudoku) => (sudoku.brute_force(rng)).map(SudokuAny::Sudoku5),
            SudokuAny::Sudoku6(sudoku) => (sudoku.brute_force(rng)).map(SudokuAny::Sudoku6),
        }
    }
    pub fn print(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
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
    type Err = LoadingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n = s.chars().filter(|c| !c.is_whitespace()).count();
        Ok(match n {
            1 => Self::Sudoku1(s.parse()?),
            16 => Self::Sudoku2(s.parse()?),
            81 => Self::Sudoku3(s.parse()?),
            256 => Self::Sudoku4(s.parse()?),
            625 => Self::Sudoku5(s.parse()?),
            1296 => Self::Sudoku6(s.parse()?),
            _ => {
                return Err(LoadingError::InvalidSize { received: n });
            }
        })
    }
}

#[derive(Debug)]
pub enum LoadingError {
    InvalidSize {
        received: usize,
    },
    Conflicting {
        pos_x: usize,
        pos_y: usize,
        value: u32,
    },
}

impl<const N: usize> FromStr for Sudoku<N> {
    type Err = LoadingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cells = s
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(Cell::<N>::from_char);
        let mut game = Self::default();
        let mut persist = Defer::<N>::new();
        for (i, pos) in Pos::iter::<N>().enumerate() {
            let Some(cell) = cells.next() else {
                return Err(LoadingError::InvalidSize { received: i });
            };
            if let Some(value) = cell.get_value() {
                let Some(_) = game.place_number(value, pos, &mut persist) else {
                    return Err(LoadingError::Conflicting {
                        pos_x: pos.x_1 as usize * N + pos.x_2 as usize,
                        pos_y: pos.y_1 as usize * N + pos.y_2 as usize,
                        value,
                    });
                };
            }
        }
        Ok(game)
    }
}

// #[cfg(test)]
// mod test {
//     use super::SudokuAny;

//     fn test_explore_space() {
//         let sudoku: SudokuAny = include_str!("../grid-3-c.txt").parse().unwrap();
//         for seed in 0..=256u8 {
//             let mut rng = SmallRng::from_seed([seed; 32]);
//             // sudoku.brute_force(tickets, rng)
//         }
//     }
// }
