//! Load the `~/.loda-rust/config.toml` file.
mod config;
mod number_of_workers;

pub use config::{Config, MinerCPUStrategy};
pub use number_of_workers::NumberOfWorkers;
