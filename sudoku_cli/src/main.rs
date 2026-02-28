use clap::Parser;
use rand::{SeedableRng, rngs::SmallRng};
use std::{path::PathBuf, time::Instant};
use sudoku::{Cell, Defer, Pos, RngChild, Sudoku, char_to_value, mask_full};

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long)]
    seed: Option<u64>,
    #[arg(default_value_t = 100)]
    retry: usize,
}

#[derive(clap::Subcommand, Clone)]
enum Command {
    Solve {
        input: PathBuf,
    },
    Generate {
        size: u32,
        #[arg(short, long)]
        sparse: bool,
    },
}

const GRID_SIZE_0: usize = 0000;
const GRID_SIZE_1: usize = 0001;
const GRID_SIZE_2: usize = 0016;
const GRID_SIZE_3: usize = 0081;
const GRID_SIZE_4: usize = 0256;
const GRID_SIZE_5: usize = 0625;
const GRID_SIZE_6: usize = 1296;
const GRID_SIZE_7: usize = 2401;
const GRID_SIZE_8: usize = 4096;

fn main() {
    let Args {
        seed,
        command,
        retry,
    } = Args::parse();
    let seed = seed.unwrap_or_else(|| rand::random());
    match command {
        Command::Solve { input } => {
            let content = match std::fs::read_to_string(&input) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Could not open {:?}: {}.", input, err);
                    return;
                }
            };
            let content: Vec<Option<u32>> = content
                .chars()
                .flat_map(|c| {
                    if c == '_' {
                        Some(None)
                    } else {
                        char_to_value(c).map(Some)
                    }
                })
                .collect();
            match content.len() {
                GRID_SIZE_0 => solve::<0, GRID_SIZE_0>(seed, retry, content.try_into().unwrap()),
                GRID_SIZE_1 => solve::<1, GRID_SIZE_1>(seed, retry, content.try_into().unwrap()),
                GRID_SIZE_2 => solve::<2, GRID_SIZE_2>(seed, retry, content.try_into().unwrap()),
                GRID_SIZE_3 => solve::<3, GRID_SIZE_3>(seed, retry, content.try_into().unwrap()),
                GRID_SIZE_4 => solve::<4, GRID_SIZE_4>(seed, retry, content.try_into().unwrap()),
                GRID_SIZE_5 => solve::<5, GRID_SIZE_5>(seed, retry, content.try_into().unwrap()),
                GRID_SIZE_6 => solve::<6, GRID_SIZE_6>(seed, retry, content.try_into().unwrap()),
                GRID_SIZE_7 => solve::<7, GRID_SIZE_7>(seed, retry, content.try_into().unwrap()),
                GRID_SIZE_8 => solve::<8, GRID_SIZE_8>(seed, retry, content.try_into().unwrap()),
                _ => {
                    eprintln!("invalid grid size");
                    return;
                }
            };
        }
        Command::Generate { size, sparse } => match size {
            0 => generate::<0>(seed, retry, sparse),
            1 => generate::<1>(seed, retry, sparse),
            2 => generate::<2>(seed, retry, sparse),
            3 => generate::<3>(seed, retry, sparse),
            4 => generate::<4>(seed, retry, sparse),
            5 => generate::<5>(seed, retry, sparse),
            6 => generate::<6>(seed, retry, sparse),
            7 => generate::<7>(seed, retry, sparse),
            8 => generate::<8>(seed, retry, sparse),
            _ => {
                eprintln!("invalid grid size {size}, expecting one of 0, 1, 2, 3, 4, 5, 6, 7 or 8.")
            }
        },
    }
}

fn generate<const N: usize>(seed: u64, retry: usize, sparse: bool) {
    for seed in (seed..).take(retry) {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut grid = Sudoku::<N>::default();

        let start = Instant::now();
        if let Some(solution) = grid
            .brute_force(rng.rng_child(), 0..Sudoku::<N>::TTL)
            .next()
        {
            let elapsed = start.elapsed();
            let mask = if sparse {
                solution.obfuscate(&mut rng)
            } else {
                mask_full()
            };

            solution.print(&mut std::io::stdout(), mask).unwrap();
            println!("elapsed: {elapsed:?}");
            return;
        }
        println!("retrying");
    }
    println!("exhausted {retry} attempts without finding a solution");
}

fn solve<const N: usize, const L: usize>(seed: u64, retry: usize, values: [Option<u32>; L]) {
    assert_eq!(N * N * N * N, L);
    let mut grid = Sudoku::<N>::default();
    let mut defer = Defer::new();
    for (pos, value) in Pos::iter::<N>().zip(values) {
        let cell = match value {
            Some(value) => Cell::from_value(value),
            None => Cell::FULL,
        };
        let Some(_) = grid.remove_all(!cell, pos, &mut defer) else {
            eprintln!("conflicting value");
            return;
        };
    }
    for (i, solution) in grid
        .brute_force(SmallRng::seed_from_u64(seed), std::iter::repeat(0))
        .enumerate()
    {
        solution.print(&mut std::io::stdout(), mask_full()).unwrap();
        println!("nth = {}", i + 1);
    }
}
