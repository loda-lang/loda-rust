use std::fmt;

/// A `RegisterIndex` can be in the range [0..255], so that it can be represented by an 8bit unsigned integer.
/// I can't imagine any OEIS sequences requiring an algorithm using more than 256 registers.
// TODO: replace u8 with u64
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct RegisterIndex(pub u8);

impl fmt::Display for RegisterIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "${}", self.0)
    }
}
