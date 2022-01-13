use loda_rust_core;
use loda_rust_core::config::Config;
use std::path::{Path, PathBuf};
use std::error::Error;
use super::find_asm_files_recursively;
use super::program_ids_from_paths;

pub struct HistogramInstructionConstantAnalyzer {
    config: Config,
    program_ids: Vec<u32>
}

impl HistogramInstructionConstantAnalyzer {
    pub fn create() -> Self {
        let mut instance = Self {
            config: Config::load(),
            program_ids: vec!()
        };
        instance
    }

    pub fn save(&self) {
        println!("saving, number of program_ids: {:?}", self.program_ids.len());
    }
}
