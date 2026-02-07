#![feature(yield_expr, gen_blocks)]

use std::{path::PathBuf, time::Instant};

mod cell;
mod sudoku;
use cell::Cell;
use clap::Parser;
use rand::{SeedableRng, rngs::SmallRng};
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
    let seed = seed.unwrap_or_else(|| rand::random());
    println!("seed: {seed}");
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
                    println!(
                        "Invalid grid size: Got {} but expected either 1, 16, 81, 256, 625 or 1296.",
                        received
                    );
                    return;
                }
                Err(LoadingError::Conflicting {
                    pos_x,
                    pos_y,
                    value,
                }) => {
                    println!(
                        "Conflicting value provided: Value {} is not valid at column {} and row {}.",
                        Cell::<6>::from_value(value).to_char(),
                        pos_x + 1,
                        pos_y + 1
                    );
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
}
