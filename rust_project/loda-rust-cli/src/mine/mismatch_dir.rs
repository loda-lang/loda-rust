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
use crate::mine::program_ids_from_paths;

pub fn load_program_ids_from_mismatch_dir(path: &Path) -> Vec<u32> {
    let paths: Vec<PathBuf> = find_asm_files_recursively(path);
    let program_ids: Vec<u32> = program_ids_from_paths(paths);
    program_ids
}
