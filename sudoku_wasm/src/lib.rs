#![allow(static_mut_refs)]

use rand::{SeedableRng, rngs::SmallRng};
use sudoku::{RngChild, Sudoku, mask_full};

const SUCCESS: u32 = 0;
const NOT_FOUND: u32 = 1;
const INVALID_SIZE: u32 = 2;
const INVALID_GRID: u32 = 3;

static mut GRID: [u8; 8 * 8 * 8 * 8] = [0u8; 8 * 8 * 8 * 8];

#[unsafe(no_mangle)]
pub extern "C" fn sudoku_ptr() -> *const u8 {
    unsafe { GRID.as_ptr() }
}

#[unsafe(no_mangle)]
pub extern "C" fn sudoku_fill(size: u32, seed: u32, sparse: bool) -> u32 {
    match size {
        0 => sudoku_fill_n::<0>(seed, sparse),
        1 => sudoku_fill_n::<1>(seed, sparse),
        2 => sudoku_fill_n::<2>(seed, sparse),
        3 => sudoku_fill_n::<3>(seed, sparse),
        4 => sudoku_fill_n::<4>(seed, sparse),
        5 => sudoku_fill_n::<5>(seed, sparse),
        6 => sudoku_fill_n::<6>(seed, sparse),
        7 => sudoku_fill_n::<7>(seed, sparse),
        8 => sudoku_fill_n::<8>(seed, sparse),
        _ => INVALID_SIZE,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn value_to_char(value: u32) -> u32 {
    sudoku::value_to_char(value).unwrap_or(' ') as u32
}

fn sudoku_fill_n<const N: usize>(seed: u32, sparse: bool) -> u32 {
    let Some(mut grid) = Sudoku::<N>::decode_grid(unsafe { &GRID }) else {
        return INVALID_GRID;
    };
    let mut rng = SmallRng::seed_from_u64(seed as u64);
    let Some(solution) = grid
        .brute_force(rng.rng_child(), 0..Sudoku::<N>::TTL)
        .next()
    else {
        return NOT_FOUND;
    };
    let mask = match sparse {
        true => solution.obfuscate(rng),
        false => mask_full(),
    };
    solution.encode_grid(unsafe { &mut GRID }, mask);
    SUCCESS
}
