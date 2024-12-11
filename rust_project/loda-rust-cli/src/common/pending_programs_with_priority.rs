use super::{find_asm_files_recursively, MineEventDirectoryScan};
use crate::config::Config;
use std::path::PathBuf;
use std::fs;
use regex::Regex;
use anyhow::Context;

#[derive(Clone, Debug)]
pub struct PendingProgramsWithPriority {
    paths_high_prio: Vec<PathBuf>,
    paths_low_prio: Vec<PathBuf>,
}

impl PendingProgramsWithPriority {
    /// Processes all the pending programs inside the `mine-event` dir.
    /// It looks for all the LODA assembly programs there are.
    /// File names of already processed programs contain `keep` or `reject` and are ignored.
    /// 
    /// If there is a `priority: high` in the file, then it saved in `paths_high_prio`.
    /// Otherwise it's considered low priority and saved in `paths_low_prio`.
    pub fn create(config: &Config) -> anyhow::Result<Self> {
        let mine_event_dir: PathBuf = config.mine_event_dir();
        let paths_all: Vec<PathBuf> = find_asm_files_recursively(&mine_event_dir);
        let scan = MineEventDirectoryScan::scan(&paths_all);
        scan.print_summary();
        let paths_pending_programs: Vec<PathBuf> = scan.pending_paths();

        // If this is a new program, then place it in the high priority queue, so it gets analyzed ASAP.
        // Otherwise the program ends up in the low priority queue.
        let mut paths_high_prio = Vec::<PathBuf>::with_capacity(paths_pending_programs.len());
        let mut paths_low_prio = Vec::<PathBuf>::with_capacity(paths_pending_programs.len());
        let regex: Regex = Regex::new("priority: high").unwrap();
        for path in &paths_pending_programs {
            let contents: String = fs::read_to_string(&path)
                .with_context(|| format!("Unable to read program file: {:?}", path))?;
            match regex.captures(&contents) {
                Some(_) => {
                    paths_high_prio.push(path.clone());
                },
                None => {
                    paths_low_prio.push(path.clone());
                }
            }
        }

        let instance = Self {
            paths_high_prio: paths_high_prio,
            paths_low_prio: paths_low_prio,
        };
        Ok(instance)
    }

    pub fn paths_high_prio(&self) -> &Vec<PathBuf> {
        &self.paths_high_prio
    }

    pub fn paths_low_prio(&self) -> &Vec<PathBuf> {
        &self.paths_low_prio
    }
}
