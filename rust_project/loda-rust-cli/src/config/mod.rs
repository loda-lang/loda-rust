//! Load the `~/.loda-rust/config.toml` file.
mod config;
mod number_of_workers;
mod validate_config;

pub use config::{config_from_toml_content, Config, MinerCPUStrategy, MinerFilterMode};
pub use number_of_workers::NumberOfWorkers;
pub use validate_config::{ValidateConfigTask, ValidateConfig};
