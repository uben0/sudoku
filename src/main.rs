use std::{io::Write, path::PathBuf, time::Instant};

mod cell;
mod resulting;
mod sudoku;
use cell::Cell;
use clap::Parser;
use rand::{SeedableRng, rngs::SmallRng};
use resulting::Resulting;
use sudoku::{LoadingError, SudokuAny};

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long)]
    seed: Option<u128>,
    #[arg(short, long)]
    time: bool,
}

#[derive(clap::Subcommand, Clone)]
enum Command {
    Solve { input: PathBuf },
    Generate { size: u32 },
}

fn main() {
    let Args {
        seed,
        command,
        time,
    } = Args::parse();
    std::thread::Builder::new()
        .stack_size(100_000_000)
        .spawn(move || {
            let seed = seed.unwrap_or_else(|| rand::random());
            let mut seed_block = [0; 32];
            seed_block[0..16].copy_from_slice(&seed.to_be_bytes());
            let mut rng = SmallRng::from_seed(seed_block);
            match command {
                Command::Solve { input } => {
                    let content = match std::fs::read_to_string(&input) {
                        Ok(content) => content,
                        Err(err) => {
                            println!("Could not open {:?}: {}.", input, err);
                            return;
                        }
                    };
                    let start_time = Instant::now();
                    let mut sudoku: SudokuAny = match content.parse() {
                        Ok(sudoku) => sudoku,
                        Err(LoadingError::InvalidSize { received }) => {
                            println!("Invalid grid size: Got {} but expected either 1, 16, 81, 256, 625 or 1296.", received);
                            return;
                        }
                        Err(LoadingError::Conflicting { pos_x, pos_y, value }) => {
                            println!("Conflicting value provided: Value {} is not valid at column {} and row {}.", Cell::<36>::from_value(value).to_char(), pos_x + 1, pos_y + 1);
                            return;
                        }
                    };
                    if let Some(solved) = sudoku.brute_force(&mut rng) {
                        let elapsed = start_time.elapsed();
                        solved.print(&mut std::io::stdout()).unwrap();

                        if time {
                            println!("elapsed: {:?}", elapsed);
                        }
                    }
                }
                Command::Generate { size } => {
                    let start_time = Instant::now();
                    let sudoku = SudokuAny::generate(size, &mut rng);
                    let elapsed = start_time.elapsed();
                    sudoku.print(&mut std::io::stdout()).unwrap();
                    if time {
                        println!("elapsed: {:?}", elapsed);
                    }
                }
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
    fn print(&self, writer: impl Write) -> std::io::Result<()>;

    /// Is the game in a winning state.
    fn is_accepting(&self) -> bool;

    /// Pushes a move (plays the move on the instance).
    ///
    /// If pushing the move is a valid action, it returns
    /// Returns whether the move is invalid or valid and if a valid move is a
    /// winning move (accepting move).
    fn push_move(&mut self, m: Self::Move) -> Option<Resulting<Self, Self::RewindData>>;
}
