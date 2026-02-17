use rand::{Rng, RngExt};
use std::ops::{BitAnd, BitOr, BitOrAssign, Not, Sub};

/// Represents the content of one cell of the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell<const N: usize> {
    /// The bitset for all possible values
    ///
    /// `1` means could contain
    /// `0` means can't contain
    bitset: u64,
}

impl<const N: usize> Default for Cell<N> {
    fn default() -> Self {
        Self::EMPTY
    }
}

// This is an implementation block.
// It contains all associated constants and methods to Cell.
/// `R` is the range of values, i.e. the MAX+1
impl<const N: usize> Cell<N> {
    pub const R: u32 = (N * N) as u32;

    /// No possible number in that cell
    pub const EMPTY: Self = Self { bitset: 0 };

    /// All possible number in that cell
    pub const FULL: Self = Self {
        bitset: !(!0u64).unbounded_shl(Self::R),
        // bitset: !(!0 << Self::R),
    };

    pub const fn bitset(self) -> u64 {
        self.bitset
    }

    // TODO: try storing values as u8
    /// Only one specific value in that cell
    #[inline]
    #[must_use]
    pub const fn from_value(value: u32) -> Self {
        debug_assert!(value < Self::R);
        Self { bitset: 1 << value }
    }

    /// If one and exactly one value, return it
    #[inline]
    #[must_use]
    pub const fn get_value(self) -> Option<u32> {
        if self.bitset.is_power_of_two() {
            Some(self.bitset.trailing_zeros())
        } else {
            None
        }
    }

    pub fn first(self) -> Option<u32> {
        let value = self.bitset.trailing_zeros();
        if value < (N * N) as u32 {
            Some(value)
        } else {
            None
        }
    }

    pub fn pop_first(&mut self) -> Option<u32> {
        let value = self.first()?;
        *self = *self - value;
        Some(value)
    }

    #[inline]
    #[must_use]
    pub fn choose(self, rng: &mut impl Rng) -> Option<u32> {
        match self.bitset.count_ones() {
            0 => None,
            1 => Some(self.bitset.trailing_zeros()),
            n => match rng.random_range(0..n) {
                // choose last one
                0 => Some(self.bitset.trailing_zeros()),
                // choose first one
                1 => Some(63 - self.bitset.leading_zeros()),
                n => {
                    // iterate through n values
                    let mut bitset = self.bitset;
                    for _ in 0..n - 1 {
                        let value = bitset.trailing_zeros();
                        bitset = bitset & !(1 << value);
                    }
                    Some(bitset.trailing_zeros())
                }
            },
        }
    }

    /// Is `value` one of the possiblities
    #[inline]
    #[must_use]
    pub const fn contains(self, value: u32) -> bool {
        debug_assert!(value < Self::R);
        self.bitset & (1 << value) != 0
    }

    /// Remove if present, the `value` possiblity
    #[inline]
    pub const fn remove(&mut self, value: u32) {
        debug_assert!(value < Self::R);
        debug_assert!(self.contains(value));
        self.bitset &= !(1 << value);
        // debug_assert!(self.len() > 0);
    }

    /// How many possibilities
    #[inline]
    #[must_use]
    pub const fn len(self) -> usize {
        self.bitset.count_ones() as usize
    }

    pub const fn from_char(c: char) -> Option<Self> {
        Some(Self::from_value(match c {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            '9' => 8,
            'A' => 9,
            'B' => 10,
            'C' => 11,
            'D' => 12,
            'E' => 13,
            'F' => 14,
            'G' => 15,
            'H' => 16,
            'I' => 17,
            'J' => 18,
            'K' => 19,
            'L' => 20,
            'M' => 21,
            'N' => 22,
            'O' => 23,
            'P' => 24,
            'Q' => 25,
            'R' => 26,
            'S' => 27,
            'T' => 28,
            'U' => 29,
            'V' => 30,
            'W' => 31,
            'X' => 32,
            'Y' => 33,
            'Z' => 34,
            '0' => 35,
            'Ψ' => 36,
            'Ω' => 37,
            'Φ' => 38,
            'Δ' => 39,
            'Ξ' => 40,
            'Γ' => 41,
            'Π' => 42,
            'Σ' => 43,
            'Д' => 44,
            'Б' => 45,
            'Џ' => 46,
            'Ш' => 47,
            'Ч' => 48,
            'ก' => 49,
            'ข' => 50,
            'ค' => 51,
            'ฉ' => 52,
            'ช' => 53,
            'ง' => 54,
            'ด' => 55,
            'ฮ' => 56,
            'ล' => 57,
            'ห' => 58,
            'น' => 59,
            'ฯ' => 60,
            'ร' => 61,
            'ฆ' => 62,
            'พ' => 63,
            '_' => {
                return Some(Self::FULL);
            }
            _ => {
                return None;
            }
        }))
    }
}

// Implement the bitwise OR operation (|)
impl<const N: usize> BitOr for Cell<N> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bitset: self.bitset | rhs.bitset,
        }
    }
}

// Implement the bitwise OR operation for assignation (|=)
impl<const N: usize> BitOrAssign for Cell<N> {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

// Implement the bitwise AND operation (&)
impl<const N: usize> BitAnd for Cell<N> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bitset: self.bitset & rhs.bitset,
        }
    }
}

// Implement the bitwise NOT operation (!)
impl<const N: usize> Not for Cell<N> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            bitset: !self.bitset & Self::FULL.bitset,
        }
    }
}

// We can iterate on the possible values of a cell
impl<const R: usize> Iterator for Cell<R> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitset == 0 {
            return None;
        }
        let value = self.bitset.trailing_zeros();
        self.bitset = self.bitset & !(1 << value);
        Some(value)
    }
}

impl<const N: usize> Sub<u32> for Cell<N> {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        self & !Self::from_value(rhs)
    }
}

#[test]
fn test_pop_random() {
    use rand::{SeedableRng, rngs::SmallRng};

    let mut full = Cell::<5>::FULL;
    let mut empty = Cell::<5>::EMPTY;
    assert_eq!(full.len(), 25);
    assert_eq!(empty.len(), 0);
    let mut rng = SmallRng::from_seed([145; 32]);
    while full.len() > 0 {
        let value = full.choose(&mut rng).unwrap();
        full.remove(value);
        assert!(!empty.contains(value));
        empty |= Cell::from_value(value);
    }
    assert_eq!(full.len(), 0);
    assert_eq!(empty.len(), 25);
}

#[test]
fn full_cell() {
    assert_eq!(
        Cell::<1>::FULL.bitset,
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001
    );
    assert_eq!(
        Cell::<2>::FULL.bitset,
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111
    );
    assert_eq!(
        Cell::<3>::FULL.bitset,
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_1111_1111
    );
    assert_eq!(
        Cell::<4>::FULL.bitset,
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111_1111_1111_1111
    );
    assert_eq!(
        Cell::<5>::FULL.bitset,
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_1111_1111_1111_1111_1111_1111
    );
    assert_eq!(
        Cell::<6>::FULL.bitset,
        0b0000_0000_0000_0000_0000_0000_0000_1111_1111_1111_1111_1111_1111_1111_1111_1111
    );
    assert_eq!(
        Cell::<7>::FULL.bitset,
        0b0000_0000_0000_0001_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111
    );
    assert_eq!(
        Cell::<8>::FULL.bitset,
        0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111
    );
}
