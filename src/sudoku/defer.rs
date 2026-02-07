use super::{Cell, Pos};

pub struct Defer<const N: usize> {
    grid: [[[[Cell<N>; N]; N]; N]; N],
    queue: Vec<(u32, Pos)>,
}

impl<const N: usize> Defer<N> {
    pub fn new() -> Self {
        Self {
            grid: [[[[Cell::EMPTY; N]; N]; N]; N],
            queue: Vec::new(),
        }
    }
    pub fn push(&mut self, value: u32, pos: Pos) {
        if self.grid[pos].contains(value) {
            return;
        }
        self.grid[pos].remove(value);
        self.queue.push((value, pos));
    }
    pub fn pop(&mut self) -> Option<(u32, Pos)> {
        self.queue.pop()
    }
    pub fn clear(&mut self) {
        self.grid = [[[[Cell::EMPTY; N]; N]; N]; N];
        self.queue.clear();
    }
}

impl<const N: usize, I: IntoIterator<Item = (u32, Pos)>> From<I> for Defer<N> {
    fn from(iter: I) -> Self {
        let mut defer = Self::new();
        for (value, pos) in iter {
            defer.push(value, pos);
        }
        defer
    }
}
