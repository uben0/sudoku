use std::{io::Write, path::PathBuf};

mod cell;
mod resulting;
mod sudoku;
use clap::Parser;
use rand::{SeedableRng, rngs::SmallRng};
use resulting::Resulting;
use sudoku::SudokuAny;

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Clone)]
enum Command {
    Solve { input: PathBuf },
    Generate { size: u32, seed: Option<u128> },
}

fn main() {
    let Args { command } = Args::parse();
    std::thread::Builder::new()
        .stack_size(100_000_000)
        .spawn(move || match command {
            Command::Solve { input } => {
                let content = std::fs::read_to_string(input).unwrap();
                let mut sudoku: SudokuAny = content.parse().unwrap();
                if let Some(solved) = sudoku.brute_force(&mut 20_000) {
                    solved.print(&mut std::io::stdout());
                }
            }
            Command::Generate { size, seed } => {
                let seed = seed.unwrap_or_else(|| rand::random());
                let mut seed_block = [0; 32];
                seed_block[0..16].copy_from_slice(&seed.to_be_bytes());
                let mut rng = SmallRng::from_seed(seed_block);
                let sudoku = SudokuAny::generate(size, &mut rng);
                sudoku.print(&mut std::io::stdout());
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The interface abstracting the idea of a game.
pub trait Game {
    /// The type representing a move.
    type Move;

    /// The data that allow a game to know what to undo to go back at certain step.
    type RewindData;

    /// Prints a pretty representation of the instance.
    fn print(&self, writer: impl Write);

    /// Is the game in a winning state.
    fn is_accepting(&self) -> bool;

    /// Pushes a move (plays the move on the instance).
    ///
    /// If pushing the move is a valid action, it returns
    /// Returns whether the move is invalid or valid and if a valid move is a
    /// winning move (accepting move).
    fn push_move(&mut self, m: Self::Move) -> Option<Resulting<Self, Self::RewindData>>;
}
