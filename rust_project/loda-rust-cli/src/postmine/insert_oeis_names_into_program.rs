use crate::common::{oeis_ids_from_paths, oeis_ids_from_programs};
use crate::oeis::{OeisId, OeisIdHashSet};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::path::PathBuf;
use anyhow::Context;

fn oeis_ids_from_programs_and_paths(paths: Vec<PathBuf>) -> anyhow::Result<OeisIdHashSet> {
    let oeis_ids0: OeisIdHashSet = oeis_ids_from_programs(paths.clone())
        .with_context(|| format!("Unable to extract oeis ids from {} programs.", paths.len()))?;
    let oeis_ids1: Vec<OeisId> = oeis_ids_from_paths(paths.clone());
    let mut result_hashset: OeisIdHashSet = HashSet::from_iter(oeis_ids1.iter().cloned());
    result_hashset.extend(oeis_ids0);
    Ok(result_hashset)
}

pub fn insert_oeis_names() {
    // let paths = absolute_paths_for_unstaged_programs_that_exist()?;
    // let oeis_ids = oeis_ids_from_programs_and_paths(paths)?;
    // let oeisid_to_name = batch_lookup_names(oeis_ids)?;
    // update_names_in_program_files(paths, oeisid_to_name)?;
}
