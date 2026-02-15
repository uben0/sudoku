use clap::Parser;
use std::{collections::HashSet, path::PathBuf, time::Instant};
use sudoku::{Cell, ChooseAtRandom, ChooseFirst, Defer, Pos, SYMBOLS, Sudoku};

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long)]
    seed: Option<u32>,
    #[arg(default_value_t = 100)]
    retry: usize,
}

#[derive(clap::Subcommand, Clone)]
enum Command {
    Solve { input: PathBuf },
    Generate { size: u32 },
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
            let charset = HashSet::from(SYMBOLS);
            let content: Vec<char> = content
                .chars()
                .filter(|c| *c == '_' || charset.contains(&c))
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
        Command::Generate { size } => match size {
            0 => generate::<0>(seed, retry),
            1 => generate::<1>(seed, retry),
            2 => generate::<2>(seed, retry),
            3 => generate::<3>(seed, retry),
            4 => generate::<4>(seed, retry),
            5 => generate::<5>(seed, retry),
            6 => generate::<6>(seed, retry),
            7 => generate::<7>(seed, retry),
            8 => generate::<8>(seed, retry),
            _ => {
                eprintln!("invalid grid size {size}, expecting one of 0, 1, 2, 3, 4, 5, 6, 7 or 8.")
            }
        },
    }
}

fn generate<const N: usize>(seed: u32, retry: usize) {
    for seed in (seed..).take(retry) {
        let chooser = ChooseAtRandom::<N>::new(seed);
        let mut grid = Sudoku::<N>::default();

        let start = Instant::now();
        if let Some(solution) = grid.brute_force(chooser, 0..Sudoku::<N>::TTL).next() {
            let elapsed = start.elapsed();
            solution.print(&mut std::io::stdout()).unwrap();
            println!("elapsed: {elapsed:?}");
            return;
        }
        println!("retrying");
    }
    println!("exhausted {retry} attempts without finding a solution");
}

fn solve<const N: usize, const L: usize>(seed: u32, retry: usize, symbols: [char; L]) {
    assert_eq!(N * N * N * N, L);
    let mut grid = Sudoku::<N>::default();
    let mut defer = Defer::new();
    for ((i, pos), symbol) in Pos::iter::<N>().enumerate().zip(symbols) {
        let Some(cell) = Cell::<N>::from_char(symbol) else {
            eprintln!("invalid symbol {symbol:?}");
            return;
        };
        let Some(_) = grid.remove_all(!cell, pos, &mut defer) else {
            eprintln!("conflicting value");
            return;
        };
    }
    for (i, solution) in grid
        .brute_force(ChooseFirst, std::iter::repeat(0))
        .enumerate()
    {
        solution.print(&mut std::io::stdout()).unwrap();
        println!("nth = {}", i + 1);
    }
}
