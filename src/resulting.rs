use std::ops::{Deref, DerefMut};

use super::Game;

/// Store data to restore game as before pushing a move
///
/// Returned by `push_move` to remember how to restore
/// the game as before the call to `push_move`. It will
/// automoatically rewind to previous game state when
/// dropped (goes out of scop).
pub struct Resulting<'a, G, D>
where
    G: Game,
    G: ?Sized,
{
    /// The targeted game
    pub game: &'a mut G,
    /// The data to remember what to undo
    pub data: D,
    /// The function that will perfom the undo
    pub rewind: fn(&mut G, &D),
}

impl<'a, G, D> Resulting<'a, G, D>
where
    G: Game + ?Sized,
{
    pub fn no_rewind(mut self) {
        self.rewind = |_, _| ();
    }
}

/// On drop, it will automatically undo the changes
impl<'a, G, D> Drop for Resulting<'a, G, D>
where
    G: Game + ?Sized,
{
    fn drop(&mut self) {
        (self.rewind)(&mut self.game, &self.data);
    }
}

/// It gives access to the game
impl<'a, G, D> Deref for Resulting<'a, G, D>
where
    G: Game,
{
    type Target = G;

    fn deref(&self) -> &Self::Target {
        self.game
    }
}
/// It gives mutable access to the game
impl<'a, G, D> DerefMut for Resulting<'a, G, D>
where
    G: Game,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.game
    }
}
