use crate::config::Config;
use crate::common::{oeis_ids_from_paths, oeis_ids_from_programs};
use crate::oeis::{NameRow, OeisId, OeisIdHashSet, ProcessNamesFile};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use anyhow::Context;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::time::{Duration, Instant};
use console::Style;
use indicatif::{HumanDuration, ProgressBar};

// git: obtain modified-files and new-file
// https://stackoverflow.com/a/26891150/78336
fn git_absolute_paths_for_unstaged_files(dir_inside_repo: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let original_path: PathBuf = env::current_dir()
        .context("get current dir")?;
    env::set_current_dir(&dir_inside_repo)
        .with_context(|| format!("set current dir {:?}", dir_inside_repo))?;

    let output_result = Command::new("git")
        .arg("ls-files")
        .arg("--exclude-standard")
        .arg("--modified")
        .arg("--others")
        .output();
    
    env::set_current_dir(&original_path)
        .with_context(|| format!("set current dir to original dir {:?}", original_path))?;
    let actual_path: PathBuf = env::current_dir()
        .context("get current dir3")?;
    if original_path != actual_path {
        return Err(anyhow::anyhow!("Unable to restore current directory. Expected: {:?}, Actual: {:?}", original_path, actual_path));
    }

    let output: Output = output_result    
        .with_context(|| format!("git ls-files inside dir {:?}", dir_inside_repo))?;

    let output_stdout: String = String::from_utf8_lossy(&output.stdout).to_string();    
    // debug!("output: {:?}", output_stdout);

    let path_strings = output_stdout.trim().split("\n");
    // debug!("path_strings: {:?}", path_strings);

    let mut paths = Vec::<PathBuf>::new();
    for path_string in path_strings {
        let absolute_path: PathBuf = dir_inside_repo.join(path_string);
        paths.push(absolute_path);
    }
    Ok(paths)
}

// def absolute_paths_for_unstaged_programs_that_exist
//     paths1 = absolute_paths_for_unstaged_files(LODA_PROGRAMS_OEIS)
//     paths2 = paths1.filter { |path| File.exist?(path) }
//     paths3 = paths2.filter { |path| path =~ /[.]asm$/ }
//     paths4 = paths3.filter { |path| path =~ /\boeis\b/ }
//     paths4
// end

fn oeis_ids_from_programs_and_paths(paths: &Vec<PathBuf>) -> anyhow::Result<OeisIdHashSet> {
    let oeis_ids0: OeisIdHashSet = oeis_ids_from_programs(paths.clone())
        .with_context(|| format!("Unable to extract oeis ids from {} programs.", paths.len()))?;
    let oeis_ids1: Vec<OeisId> = oeis_ids_from_paths(paths.clone());
    let mut result_hashset: OeisIdHashSet = HashSet::from_iter(oeis_ids1.iter().cloned());
    result_hashset.extend(oeis_ids0);
    Ok(result_hashset)
}

type OeisIdNameMap = HashMap<OeisId,String>;

fn batch_lookup_names(
    reader: &mut dyn io::BufRead,
    filesize: usize,
    oeis_ids: &OeisIdHashSet
) -> Result<OeisIdNameMap, Box<dyn Error>> {
    let start = Instant::now();
    println!("Looking up in the OEIS 'names' file");

    let mut oeis_id_name_map = OeisIdNameMap::new();
    let pb = ProgressBar::new(filesize as u64);
    let callback = |row: &NameRow, count_bytes: usize| {
        pb.set_position(count_bytes as u64);
        if oeis_ids.contains(&row.oeis_id()) {
            let message = format!("{}: {}", row.oeis_id().a_number(), row.name());
            pb.println(message);
            oeis_id_name_map.insert(row.oeis_id(), row.name().to_string());
        }
    };
    
    let oeis_ids_to_ignore = OeisIdHashSet::new();
    let mut processor = ProcessNamesFile::new();
    processor.execute(
        reader, 
        &oeis_ids_to_ignore,
        callback
    );
    pb.finish_and_clear();

    let green_bold = Style::new().green().bold();        
    println!(
        "{:>12} Lookups in the OEIS 'names' file, in {}",
        green_bold.apply_to("Finished"),
        HumanDuration(start.elapsed())
    );

    Ok(oeis_id_name_map)
}

pub fn insert_oeis_names() -> Result<(), Box<dyn Error>> {
    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();

    let paths: Vec<PathBuf> = git_absolute_paths_for_unstaged_files(&loda_programs_oeis_dir)?;
    println!("paths: {:?}", paths);

    let oeis_ids: OeisIdHashSet = oeis_ids_from_programs_and_paths(&paths)?;
    println!("oeis_ids: {:?}", oeis_ids);

    let oeis_names_file: PathBuf = config.oeis_names_file();
    let file = File::open(oeis_names_file).unwrap();
    let filesize: usize = file.metadata().unwrap().len() as usize;
    let mut reader = BufReader::new(file);
    let oeis_id_name_map: OeisIdNameMap = batch_lookup_names(
        &mut reader,
        filesize,
        &oeis_ids
    )?;
    
    // update_names_in_program_files(paths, oeis_id_name_map)?;
    Ok(())
}
