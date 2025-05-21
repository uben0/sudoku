use std::ops::{BitAnd, BitOr, BitOrAssign, Not};

/// Represents the content of one cell of the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell<const N: usize> {
    /// The bitset for all possible values
    ///
    /// `1` means could contain
    /// `0` means can't contain
    bitset: u64,
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
        bitset: !(!0 << Self::R),
    };

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
    pub fn get_value(self) -> Option<u32> {
        self.bitset
            .is_power_of_two()
            .then(|| self.bitset.trailing_zeros())
    }

    /// Is `value` one of the possiblities
    #[inline]
    #[must_use]
    pub fn contains(self, value: u32) -> bool {
        debug_assert!((0..Self::R).contains(&value));
        self.bitset & (1 << value) != 0
    }

    /// Remove if present, the `value` possiblity
    #[inline]
    #[must_use]
    pub fn remove(&mut self, value: u32) -> bool {
        debug_assert!((0..Self::R).contains(&value));
        if self.contains(value) && self.len() > 1 {
            self.bitset &= !(1 << value);
            true
        } else {
            false
        }
    }

    /// How many possibilities
    #[inline]
    #[must_use]
    pub const fn len(self) -> usize {
        self.bitset.count_ones() as usize
    }

    // pub fn debug_print(self) {
    //     for v in 0..R {
    //         if self.contains(v) {
    //             print!("{:x}", v);
    //         } else {
    //             print!("_");
    //         }
    //     }
    // }

    pub fn to_char(self) -> char {
        let Some(value) = self.get_value() else {
            return '_';
        };
        debug_assert!(value < Self::R);
        // TODO: move 0 to first place, and remove limit on cell values
        [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
            'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y',
            'Z', '0',
        ][value as usize]
    }
    pub fn from_char(c: char) -> Self {
        match c {
            '1'..='9' => Self::from_value((c as u32 - '1' as u32) + 0),
            'A'..='Z' => Self::from_value((c as u32 - 'A' as u32) + 9),
            '0' => Self::from_value(35),
            '_' => Self::FULL,
            _ => panic!("invalid cell symbol {:?}, expecting alpha-numeric", c),
        }
    }
}

// Implement the bitwise OR operation (|)
impl<const R: usize> BitOr for Cell<R> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bitset: self.bitset | rhs.bitset,
        }
    }
}

// Implement the bitwise OR operation for assignation (|=)
impl<const R: usize> BitOrAssign for Cell<R> {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

// Implement the bitwise AND operation (&)
impl<const R: usize> BitAnd for Cell<R> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bitset: self.bitset & rhs.bitset,
        }
    }
}

// Implement the bitwise NOT operation (!)
impl<const R: usize> Not for Cell<R> {
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
