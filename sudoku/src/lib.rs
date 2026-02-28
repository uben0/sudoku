#![feature(gen_blocks)]

mod cell;
mod charset;
mod defer;
mod grid;

pub use cell::Cell;
pub use charset::{char_to_value, value_to_char, value_to_char_width};
pub use defer::Defer;
pub use grid::Sudoku;
use rand::prelude::*;
use rand::{SeedableRng, rngs::SmallRng};
use std::{
    io::Write,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
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

impl<const N: usize> Sudoku<N> {
    pub const TTL: usize = 1 << (N + 5);
    pub fn encode_grid(&self, dst: &mut [u8], mask: [[[[bool; N]; N]; N]; N]) {
        assert!(dst.len() >= N * N * N * N);
        let mut i = 0;
        for pos in Pos::iter::<N>() {
            dst[i] = mask[pos]
                .then_some(self[pos])
                .and_then(|c| c.get_value())
                .map(|v| v as u8)
                .unwrap_or(255);
            i += 1;
        }
    }
    pub fn decode_grid(src: &[u8]) -> Option<Self> {
        assert!(src.len() >= N * N * N * N);
        let mut defer = Defer::new();
        let mut grid = Self::default();
        let mut i = 0;
        for pos in Pos::iter::<N>() {
            let cell = match src[i] {
                255 => Cell::FULL,
                value => Cell::from_value(value as u32),
            };
            grid.remove_all(!cell, pos, &mut defer)?;
            i += 1;
        }
        Some(grid)
    }
    fn remove(&mut self, value: u32, pos: Pos, defer: &mut Defer<N>) -> Option<usize> {
        debug_assert!(self[pos].contains(value));
        if self[pos] == Cell::from_value(value) {
            return None;
        }
        defer.clear();

        let mut pushed = 0;
        self.remove_one(value, pos, &mut pushed, defer);

        while let Some(pos) = defer.pop() {
            // if the current cell has a unique possiblity
            // all correlated cells can't have it
            if let Some(value) = self[pos].get_value() {
                for pos in correlated::<N>(pos) {
                    if self[pos].contains(value) {
                        if self[pos] == Cell::from_value(value) {
                            self.pop_n_moves(pushed);
                            return None;
                        }
                        debug_assert!(self[pos].contains(value));
                        self.remove_one(value, pos, &mut pushed, defer);
                    }
                }
            }

            // Now that we removed the `value` possibility of the cell `[y, x]`
            // Maybe a correlated cell now is the only one with it in its correlated neigbourhood
            // If it is the case, it become its only possibility, and we cascade the effect
            for pos in correlated::<N>(pos) {
                // A determine cell will always result in enforcing its value
                // It is already unique, so we don't have to do anything
                if self[pos].len() == 1 {
                    continue;
                }
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

                if !self[pos].contains(value) {
                    self.pop_n_moves(pushed);
                    return None;
                }
                for iv in self[pos] - value {
                    if self[pos] == Cell::from_value(value) {
                        self.pop_n_moves(pushed);
                        return None;
                    }
                    debug_assert!(self[pos].contains(iv));
                    self.remove_one(iv, pos, &mut pushed, defer);
                }
            }
        }
        Some(pushed)
    }
    pub fn brute_force(
        &mut self,
        mut chooser: impl Choose<N>,
        ttl: impl IntoIterator<Item = usize>,
    ) -> impl Iterator<Item = Self> {
        gen move {
            let min = self.best();
            if min == 1 {
                yield self.clone();
                return;
            }
            // let pos_iter = chooser.pos_iter();
            let mut pos = self.min_bifurc(min);
            let mut cell = self[pos];

            let mut stack: Vec<(usize, Cell<N>, Pos)> = Vec::new();
            let mut persist = Defer::<N>::new();

            for i in ttl {
                if let Some(value) = chooser.choose_pop_value_in_cell(&mut cell) {
                    if let Some(moved) =
                        self.remove_all(!Cell::from_value(value), pos, &mut persist)
                    {
                        match self.best() {
                            1 => {
                                println!("{i}");
                                yield self.clone();
                                self.pop_n_moves(moved);
                            }
                            min => {
                                stack.push((moved, cell, pos));
                                pos = self.min_bifurc(min);
                                cell = self[pos];
                            }
                        }
                    }
                } else {
                    let Some((unpush, prev_cell, prev_pos)) = stack.pop() else {
                        return;
                    };
                    self.pop_n_moves(unpush);
                    cell = prev_cell;
                    pos = prev_pos;
                }
            }
            for (unpush, _, _) in stack {
                self.pop_n_moves(unpush);
            }
        }
    }

    // Because of the way moves are pushed, it enforces that the grid
    // remains coherent. We only have to check how many moves were pushed.
    // fn is_accepting(&self) -> bool {
    //     self.moves.len() == N * N * N * N * (N * N - 1)
    // }

    fn min_bifurc(&self, min: usize) -> Pos {
        for pos in Pos::iter::<N>() {
            if self[pos].len() == min {
                return pos;
            }
        }
        unreachable!()
    }
    /// Remove all possibilities of the given cell.
    ///
    /// It has a cascading effect on correlated cells.
    /// It returns the number of pushed moves.
    /// It may fail if the grid turns out to be inconsistent, in that case
    /// no move is pushed and it returns `None`
    #[inline]
    #[must_use]
    pub fn remove_all(&mut self, values: Cell<N>, pos: Pos, defer: &mut Defer<N>) -> Option<usize> {
        let mut count = 0;
        for iv in self[pos] & values {
            // always check again, because the value may have been removed meanwhile
            if self[pos].contains(iv) {
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

    // For a given cell, returns all possibilities of the cell
    // which are not present in the other one of its line (row).
    // If there is more than one, its incoherent, it means there
    // is more than one enforced value.
    #[must_use]
    fn unic_on_row(&self, pos: Pos) -> Cell<N> {
        let mut possibles = Cell::EMPTY;
        for x_1 in 0..N as u8 {
            for x_2 in 0..N as u8 {
                if x_1 != pos.x_1 || x_2 != pos.x_2 {
                    possibles |= self[Pos { x_1, x_2, ..pos }];
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
                    possibles |= self[Pos { y_1, y_2, ..pos }];
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
                    possibles |= self[Pos { y_2, x_2, ..pos }];
                }
            }
            if possibles == Cell::FULL {
                return Cell::EMPTY;
            }
        }
        !possibles
    }

    pub fn long_best(&self) -> usize {
        let mut min = N * N + 1;
        for pos in Pos::iter::<N>() {
            let len = self[pos].len();
            if len > 1 && len < min {
                min = len;
            }
        }
        if min == N * N + 1 {
            return 1;
        }
        min
    }

    pub fn print(
        &self,
        mut writer: impl Write,
        mask: [[[[bool; N]; N]; N]; N],
    ) -> Result<(), std::io::Error> {
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
        for y_1 in 0..N as u8 {
            for y_2 in 0..N as u8 {
                if y_1 > 0 || y_2 > 0 {
                    if y_2 == 0 {
                        print_line_sep(&mut writer, N, '┣', '┫', '━', '┿', '╋')?;
                    } else {
                        print_line_sep(&mut writer, N, '┠', '┨', '─', '┼', '╂')?;
                    }
                }
                for x_1 in 0..N as u8 {
                    for x_2 in 0..N as u8 {
                        if x_2 == 0 {
                            write!(writer, "┃")?;
                        } else {
                            write!(writer, "│")?;
                        }
                        let pos = Pos { y_1, y_2, x_1, x_2 };

                        match Some(pos)
                            .filter(|p| mask[*p])
                            .and_then(|p| self[p].get_value())
                        {
                            None => {
                                write!(writer, "   ")?;
                            }
                            Some(value) => {
                                let c = value_to_char(value).unwrap();
                                match value_to_char_width(value).unwrap() {
                                    1 => {
                                        write!(writer, " {c} ")?;
                                    }
                                    2 => {
                                        write!(writer, " {c}")?;
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        };
                    }
                }
                writeln!(writer, "┃")?;
            }
        }
        print_line_sep(&mut writer, N, '┗', '┛', '━', '┷', '┻')?;
        Ok(())
    }

    pub fn obfuscate(&self, mut rng: impl Rng) -> [[[[bool; N]; N]; N]; N] {
        let mut positions: Vec<Pos> = Pos::iter::<N>().collect();
        let mut mask = [[[[false; N]; N]; N]; N];
        positions.shuffle(&mut rng);
        let mut obfuscated = Self::default();
        let mut defer = Defer::new();
        while let Some(pos) = positions.pop() {
            mask[pos] = true;
            obfuscated.remove_all(!self[pos], pos, &mut defer).unwrap();
            if obfuscated.is_accepting() {
                return mask;
            }
        }
        panic!();
    }
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

impl<const N: usize> Index<Pos> for [[[[Cell<N>; N]; N]; N]; N] {
    type Output = Cell<N>;

    #[inline]
    fn index(&self, index: Pos) -> &Self::Output {
        unsafe {
            self.get_unchecked(index.y_1 as usize)
                .get_unchecked(index.y_2 as usize)
                .get_unchecked(index.x_1 as usize)
                .get_unchecked(index.x_2 as usize)
        }
    }
}
impl<const N: usize> IndexMut<Pos> for [[[[Cell<N>; N]; N]; N]; N] {
    #[inline]
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        unsafe {
            self.get_unchecked_mut(index.y_1 as usize)
                .get_unchecked_mut(index.y_2 as usize)
                .get_unchecked_mut(index.x_1 as usize)
                .get_unchecked_mut(index.x_2 as usize)
        }
        // &mut self[index.y_1 as usize][index.y_2 as usize][index.x_1 as usize][index.x_2 as usize]
    }
}
impl<const N: usize> Index<Pos> for [[[[bool; N]; N]; N]; N] {
    type Output = bool;

    #[inline]
    fn index(&self, index: Pos) -> &Self::Output {
        unsafe {
            self.get_unchecked(index.y_1 as usize)
                .get_unchecked(index.y_2 as usize)
                .get_unchecked(index.x_1 as usize)
                .get_unchecked(index.x_2 as usize)
        }
    }
}
impl<const N: usize> IndexMut<Pos> for [[[[bool; N]; N]; N]; N] {
    #[inline]
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        unsafe {
            self.get_unchecked_mut(index.y_1 as usize)
                .get_unchecked_mut(index.y_2 as usize)
                .get_unchecked_mut(index.x_1 as usize)
                .get_unchecked_mut(index.x_2 as usize)
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

#[derive(Debug)]
pub enum LoadingError {
    InvalidCharacter {
        char: char,
    },
    InvalidSize {
        received: usize,
    },
    Conflicting {
        pos_x: usize,
        pos_y: usize,
        value: u32,
    },
}

pub trait Choose<const N: usize> {
    fn choose_value_in_cell(&mut self, cell: Cell<N>) -> Option<u32>;
    fn choose_pop_value_in_cell(&mut self, cell: &mut Cell<N>) -> Option<u32> {
        let value = self.choose_value_in_cell(*cell)?;
        *cell = *cell - value;
        Some(value)
    }
}

impl<const N: usize> Choose<N> for SmallRng {
    fn choose_value_in_cell(&mut self, cell: Cell<N>) -> Option<u32> {
        cell.choose(self)
    }
}
impl<const N: usize> Choose<N> for () {
    fn choose_value_in_cell(&mut self, cell: Cell<N>) -> Option<u32> {
        cell.first()
    }
}

pub trait RngChild: Rng + SeedableRng {
    fn rng_child(&mut self) -> Self {
        Self::seed_from_u64(self.random())
    }
}
impl RngChild for SmallRng {}

pub const fn mask_full<const N: usize>() -> [[[[bool; N]; N]; N]; N] {
    [[[[true; N]; N]; N]; N]
}
pub const fn mask_empty<const N: usize>() -> [[[[bool; N]; N]; N]; N] {
    [[[[false; N]; N]; N]; N]
}
