use std::fmt;

/// A `RegisterIndex` can be in the range [0..u64::max].
/// 
/// Most LODA programs use less than 10 registers.
/// 
/// Some LODA programs use a variable number of registers, and this can be a LOT!
/// Example the A000041 uses double dollar notation (aka. ParameterType::Indirect).
/// Invoking this with n=10000, then it will use around 10000 registers.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct RegisterIndex(pub u64);

impl fmt::Display for RegisterIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "${}", self.0)
    }
}
