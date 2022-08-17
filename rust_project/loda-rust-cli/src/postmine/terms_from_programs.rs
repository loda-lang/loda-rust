use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Context;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Extract the terms row from a loda .asm program
    // The `(?m)` enables multiline matching.
    static ref EXTRACT_TERMS_FROM_LODA_PROGRAM: Regex = Regex::new(
        "(?m)^; (-?\\d+(?:,-?\\d+)+)$"
    ).unwrap();
}

pub fn terms_from_program(program_path: &Path) -> anyhow::Result<()> {
    let program_contents: String = fs::read_to_string(program_path)
        .with_context(|| format!("Read program from {:?}", program_path))?;

    let re = &EXTRACT_TERMS_FROM_LODA_PROGRAM;
    let captures = match re.captures(&program_contents) {
        Some(value) => value,
        None => {
            println!("Unable to extract sequence number");
            return Ok(());
        }
    };
    let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
    let terms_string: String = capture1.to_string();

    println!("Extracted terms: {:?}", terms_string);
    
    Ok(())
}

pub fn terms_from_programs(paths: &Vec<PathBuf>) -> anyhow::Result<()> {
    for path in paths {
        terms_from_program(&path)?;
    }
    Ok(())
}
