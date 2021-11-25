use loda_rust_core;
use loda_rust_core::config::Config;
use crate::mine::{create_cache_files, load_program_ids_csv_file};
use crate::mine::validate_programs;
use crate::mine::load_program_ids_from_deny_file;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Instant;
use std::io;
use std::error::Error;
use crate::mine::find_asm_files_recursively;
use crate::mine::program_id_from_path;

pub fn load_program_ids_from_mismatch_dir(path: &Path) -> Vec<u32> {
    let paths: Vec<PathBuf> = find_asm_files_recursively(path);

    // Extract program_ids from paths
    let mut program_ids: Vec<u32> = vec!();
    for path in paths {
        let program_id: u32 = match program_id_from_path(&path) {
            Some(program_id) => program_id,
            None => {
                warn!("Unable to extract program_id from {:?}", path);
                continue;
            }
        };
        program_ids.push(program_id);
    }
    program_ids.sort();
    program_ids
}
