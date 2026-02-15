use std::ops::Index;

use crate::{Cell, Defer, Pos};

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
    buckets: [[usize; N]; N],
}

impl<const N: usize> Sudoku<N> {
    pub fn best(&self) -> usize {
        for v_2 in 1..N {
            if self.buckets[0][v_2] != 0 {
                return v_2 + 1;
            }
        }
        for v_1 in 1..N {
            for v_2 in 0..N {
                if self.buckets[v_1][v_2] != 0 {
                    return v_1 * N + v_2 + 1;
                }
            }
        }
        1
    }
    fn bucket(&mut self, index: usize) -> &mut usize {
        &mut self.buckets[index / N][index % N]
    }
    pub fn remove_one(&mut self, value: u32, pos: Pos, pushed: &mut usize, defer: &mut Defer<N>) {
        self.grid[pos].remove(value);
        let len = self[pos].len();
        *self.bucket(len - 0) -= 1;
        *self.bucket(len - 1) += 1;
        self.moves.push((value, pos));
        defer.push(pos);
        *pushed += 1;
    }
    pub fn pop_n_moves(&mut self, n: usize) {
        for _ in 0..n {
            let (value, pos) = self.moves.pop().unwrap();
            let len = self[pos].len();
            *self.bucket(len - 0) += 1;
            *self.bucket(len - 1) -= 1;
            debug_assert!(!self[pos].contains(value));
            self.grid[pos] |= Cell::from_value(value);
        }
    }
}

impl<const N: usize> Index<Pos> for Sudoku<N> {
    type Output = Cell<N>;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.grid[index]
    }
}

impl<const N: usize> Default for Sudoku<N> {
    fn default() -> Self {
        let mut best = [[0; N]; N];
        best[N - 1][N - 1] = N * N * N * N;
        Self {
            grid: [[[[Cell::FULL; N]; N]; N]; N],
            moves: Vec::new(),
            buckets: best,
        }
    }
}
