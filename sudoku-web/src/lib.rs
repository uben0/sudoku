#![allow(static_mut_refs)]

use sudoku::{ChooseAtRandom, Sudoku};

static mut GRID: [u8; 8 * 8 * 8 * 8] = [0u8; 8 * 8 * 8 * 8];

#[unsafe(no_mangle)]
pub extern "C" fn sudoku_ptr() -> *const u8 {
    unsafe { GRID.as_ptr() }
}

#[unsafe(no_mangle)]
pub extern "C" fn sudoku_gen(size: u32, seed: u32) -> u32 {
    match size {
        0 => 1,
        1 => sudoku_gen_n::<1>(seed),
        2 => sudoku_gen_n::<2>(seed),
        3 => sudoku_gen_n::<3>(seed),
        4 => sudoku_gen_n::<4>(seed),
        5 => sudoku_gen_n::<5>(seed),
        6 => sudoku_gen_n::<6>(seed),
        7 => sudoku_gen_n::<7>(seed),
        8 => sudoku_gen_n::<8>(seed),
        _ => 2,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn value_to_char(value: u32) -> u32 {
    sudoku::value_to_char(value).unwrap_or(' ') as u32
}

fn sudoku_gen_n<const N: usize>(seed: u32) -> u32 {
    let chooser = ChooseAtRandom::<N>::new(seed);
    let mut grid = Sudoku::<N>::default();

    match grid.brute_force(chooser, 0..Sudoku::<N>::TTL).next() {
        Some(solution) => {
            solution.encode_grid(unsafe { &mut GRID });
            1
        }
        None => 0,
    }
}
