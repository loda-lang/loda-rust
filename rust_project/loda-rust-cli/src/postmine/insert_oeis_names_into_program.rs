use crate::config::Config;
use crate::common::{oeis_id_from_path, oeis_ids_from_programs, OeisIdStringMap};
use crate::oeis::{OeisId, OeisIdHashSet};
use loda_rust_core::execute::{ProgramId, ProgramRunner, ProgramSerializer};
use loda_rust_core::parser::ParsedProgram;
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use super::{batch_lookup_names, batch_lookup_terms, filter_asm_files, git_absolute_paths_for_unstaged_files, PathTermsMap, ProgramSerializerContextWithSequenceName, terms_from_programs};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use anyhow::Context;

fn update_names_in_program_file(
    program_path: &Path,
    path_terms_map: &PathTermsMap,
    oeis_id_terms_map: &OeisIdStringMap,
    oeis_id_name_map: &OeisIdStringMap,
    loda_submitted_by: &String
) -> anyhow::Result<()> {
    let optional_program_oeis_id: Option<OeisId> = oeis_id_from_path(program_path);
    let program_oeis_id: OeisId = match optional_program_oeis_id {
        Some(value) => value,
        None => {
            return Err(anyhow::anyhow!("Expected path to contain an OeisId, but got none from path {:?}", &program_path));
        }
    };

    let program_contents: String = fs::read_to_string(program_path)
        .with_context(|| format!("Read program from {:?}", program_path))?;
    
    let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&program_contents) {
        Ok(value) => value,
        Err(error) => {
            return Err(anyhow::anyhow!("Parse program from {:?} error: {:?}", &program_path, error));
        }
    };

    // Don't load dependencies from the file system
    let mut dm = DependencyManager::new(
        DependencyManagerFileSystemMode::Virtual,
        PathBuf::from("non-existing-dir"),
    );
    for (oeis_id, _name) in &*oeis_id_name_map {
        let program_id: u64 = oeis_id.raw() as u64;
        dm.virtual_filesystem_insert_file(program_id, "".to_string());
    }

    // Create program from instructions
    let result_parse = dm.parse_stage2(
        ProgramId::ProgramWithoutId, 
        &parsed_program
    );
    let runner: ProgramRunner = match result_parse {
        Ok(value) => value,
        Err(error) => {
            return Err(anyhow::anyhow!("parse_stage2 with program {:?} error: {:?}", &program_path, error));
        }
    };

    let mut serializer = ProgramSerializer::new();

    // Pass on the `oeis_id_name_map` all the way to the formatting code
    // of the `seq` instruction, so that the sequence name can be inserted as a comment.
    // Like this: `seq $2,40 ; The prime numbers.`
    let context = ProgramSerializerContextWithSequenceName::new(oeis_id_name_map.clone());
    serializer.set_context(Box::new(context));

    // Insert the sequence name
    let optional_name: Option<&String> = oeis_id_name_map.get(&program_oeis_id);
    let mut resolved_name: String = "Missing sequence name".to_string();
    if let Some(name) = optional_name {
        resolved_name = name.clone();
    }
    serializer.append_comment(format!("{}: {}", program_oeis_id, resolved_name));

    // Submitted by Euler
    serializer.append_comment(format!("Submitted by {}", loda_submitted_by));

    // Prefer using the terms of the original program file, as they are.
    let mut optional_terms: Option<&String> = path_terms_map.get(program_path);
    if optional_terms == None {
        // If no comment with terms could be found,
        // then use take the terms from the OEIS 'stripped' file.
        optional_terms = oeis_id_terms_map.get(&program_oeis_id);
    }
    let resolved_terms: String;
    if let Some(terms) = optional_terms {
        resolved_terms = terms.clone();
    } else {
        error!("Unable to resolve terms for the program: {:?}", program_path);
        resolved_terms = "Missing sequence terms".to_string();
    }
    serializer.append_comment(resolved_terms);

    serializer.append_empty_line();
    runner.serialize(&mut serializer);
    serializer.append_empty_line();
    let formatted_program: String = serializer.to_string();

    // Replace the existing file
    debug!("updated program: {:?}", program_path);
    let mut output_file = File::create(&program_path)?;
    output_file.write_all(formatted_program.as_bytes())?;

    Ok(())
}

fn update_names_in_program_files(
    paths: &Vec<PathBuf>,
    path_terms_map: &PathTermsMap,
    oeis_id_terms_map: &OeisIdStringMap,
    oeis_id_name_map: &OeisIdStringMap,
    loda_submitted_by: &String
) -> Result<(), Box<dyn Error>> {
    for path in paths {
        update_names_in_program_file(
            path,
            path_terms_map,
            oeis_id_terms_map,
            oeis_id_name_map, 
            loda_submitted_by
        )?;
    }
    Ok(())
}

fn insert_oeis_names() -> Result<(), Box<dyn Error>> {
    let config = Config::load();
    let loda_programs_oeis_dir: PathBuf = config.loda_programs_oeis_dir();
    let oeis_names_file: PathBuf = config.oeis_names_file();
    let oeis_stripped_file: PathBuf = config.oeis_stripped_file();
    let loda_submitted_by: String = config.loda_submitted_by();

    let unfiltered_paths: Vec<PathBuf> = git_absolute_paths_for_unstaged_files(&loda_programs_oeis_dir)?;
    let filtered_paths: Vec<PathBuf> = filter_asm_files(&unfiltered_paths);

    // Collect paths and corresponding OeisId.
    let mut path_oeis_id_map = HashMap::<PathBuf, OeisId>::new();
    let mut paths = Vec::<PathBuf>::new();
    for path in &filtered_paths {
        let oeis_id: OeisId = match oeis_id_from_path(path) {
            Some(oeis_id) => oeis_id,
            None => {
                error!("Ignoring file. Unable to extract oeis_id from {:?}", path);
                continue;
            }
        };
        paths.push(PathBuf::from(path));
        path_oeis_id_map.insert(PathBuf::from(path), oeis_id);
    }
    if paths.len() != unfiltered_paths.len() {
        debug!("filtered out some paths. unfiltered_paths.len: {} paths.len: {}", unfiltered_paths.len(), paths.len());
        debug!("unfiltered_paths: {:?}", unfiltered_paths);
    }
    println!("number of programs for processing: {:?}", paths.len());
    debug!("paths: {:?}", paths);
    let oeis_ids_paths: OeisIdHashSet = path_oeis_id_map.into_values().collect();

    let path_terms_map: PathTermsMap = terms_from_programs(&paths)
        .with_context(|| format!("Unable to extract terms from programs."))?;
        debug!("path_terms_map: {:?}", path_terms_map);

    let oeis_ids_programs: OeisIdHashSet = oeis_ids_from_programs(&paths)
        .with_context(|| format!("Unable to extract oeis ids from {} programs.", paths.len()))?;

    let mut oeis_ids_name: OeisIdHashSet = oeis_ids_programs.clone();
    oeis_ids_name.extend(oeis_ids_paths.clone());
    debug!("oeis_ids_name: {:?}", oeis_ids_name);

    // Obtain terms from the OEIS 'stripped' FILE
    let file0 = File::open(oeis_stripped_file).unwrap();
    let filesize0: usize = file0.metadata().unwrap().len() as usize;
    let mut reader0 = BufReader::new(file0);
    let oeis_id_terms_map: OeisIdStringMap = batch_lookup_terms(
        &mut reader0,
        filesize0,
        &oeis_ids_paths
    )?;
    
    // Obtain names for UNION(oeis_ids_paths, oeis_ids_programs)
    let file1 = File::open(oeis_names_file).unwrap();
    let filesize1: usize = file1.metadata().unwrap().len() as usize;
    let mut reader1 = BufReader::new(file1);
    let oeis_id_name_map: OeisIdStringMap = batch_lookup_names(
        &mut reader1,
        filesize1,
        &oeis_ids_name
    )?;
    
    update_names_in_program_files(
        &paths, 
        &path_terms_map,
        &oeis_id_terms_map,
        &oeis_id_name_map,
        &loda_submitted_by
    )?;

    debug!("Successfully updated {} programs", paths.len());
    Ok(())
}

pub struct InsertNames {}

impl InsertNames {
    pub fn run() -> Result<(), Box<dyn Error>> {
        insert_oeis_names()?;
        Ok(())
    }
}
