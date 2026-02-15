use super::Pos;

pub struct Defer<const N: usize> {
    grid: [[[[bool; N]; N]; N]; N],
    queue: Vec<Pos>,
}
impl<const N: usize> Defer<N> {
    pub fn new() -> Self {
        Self {
            grid: [[[[false; N]; N]; N]; N],
            queue: Vec::new(),
        }
    }
    pub fn push(&mut self, pos: Pos) {
        if self.grid[pos] {
            return;
        }
        self.grid[pos] = true;
        self.queue.push(pos);
    }
    pub fn pop(&mut self) -> Option<Pos> {
        let pos = self.queue.pop()?;
        self.grid[pos] = false;
        Some(pos)
    }
    pub fn clear(&mut self) {
        self.grid = [[[[false; N]; N]; N]; N];
        self.queue.clear();
    }
}
