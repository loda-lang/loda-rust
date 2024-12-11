//! This crate is a Rust port of the [LODA language].
//! 
//! The original C++ implemenation can be found [here].
//! 
//! [LODA language]: https://loda-lang.org/spec/
//! [here]: https://github.com/loda-lang/loda-cpp

#[macro_use]
extern crate log;

pub mod control;
pub mod execute;
pub mod oeis;
pub mod parser;
pub mod unofficial_function;
pub mod util;
