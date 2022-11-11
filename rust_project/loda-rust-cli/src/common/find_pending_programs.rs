use std::error::Error;
use std::path::PathBuf;
use regex::Regex;
use lazy_static::lazy_static;
use std::ffi::OsStr;

lazy_static! {
    /// Determine if the file extension is `.keep.asm` or `.reject.asm`
    static ref ALREADY_PROCESSED: Regex = Regex::new(
        "[.](?:keep|reject)[.]asm$"
    ).unwrap();
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Pending,
    AlreadyProcessed,
    ErrorGetFileName,
}

struct MineEventDirectoryScan {
    path_and_state_vec: Vec<(PathBuf, State)>,
}

impl MineEventDirectoryScan {
    pub fn scan(paths_inside_mineevent_dir: &Vec<PathBuf>) -> Self {
        let mut path_and_state_vec: Vec<(PathBuf, State)> = vec!();
        let re = &ALREADY_PROCESSED;
        for path in paths_inside_mineevent_dir {
            let filename: &OsStr = match path.file_name() {
                Some(value) => value,
                None => {
                    path_and_state_vec.push((path.clone(), State::ErrorGetFileName));
                    continue;
                }
            };
            if re.is_match(&filename.to_string_lossy()) {
                path_and_state_vec.push((path.clone(), State::AlreadyProcessed));
                continue;
            }
            path_and_state_vec.push((path.clone(), State::Pending));
        }
        Self { path_and_state_vec }
    }

    pub fn paths_for_state(&self, filter_state: State) -> Vec<PathBuf> {
        let path_and_state_vec_filtered: Vec<&(PathBuf, State)> = self.path_and_state_vec.iter()
            .filter(|(_,state)| *state == filter_state).collect();
        let mut paths: Vec<PathBuf> = path_and_state_vec_filtered.iter()
            .map(|(path,_)| { path.clone() }).collect();
        paths.sort();
        paths
    }

    pub fn pending_paths(&self) -> Vec<PathBuf> {
        self.paths_for_state(State::Pending)
    }

    pub fn already_processed_paths(&self) -> Vec<PathBuf> {
        self.paths_for_state(State::AlreadyProcessed)
    }

    pub fn error_get_filename_paths(&self) -> Vec<PathBuf> {
        self.paths_for_state(State::ErrorGetFileName)
    }

    pub fn print_summary(&self) {
        let error_get_filename_paths: Vec<PathBuf> = self.error_get_filename_paths();
        if error_get_filename_paths.len() > 0 {
            error!("Could not extract file_name from {} paths: {:?}", error_get_filename_paths.len(), error_get_filename_paths);
        }
        let count_already_processed: usize = self.already_processed_paths().len();
        if count_already_processed > 0 {
            println!("Ignoring {} programs that have already been analyzed", count_already_processed);
        }
        let count_pending: usize = self.pending_paths().len();
        if count_pending > 0 {
            println!("Number of pending programs: {}", count_pending);
        }
    }
}

/// Find `.asm` files that are waiting to be processed.
/// 
/// These have names like this:
/// 
/// ```csv
/// mine-event/20220710-054915-1251916462.asm
/// mine-event/20220710-055020-1265182884.asm
/// mine-event/manual-coded-program.asm
/// ```
/// 
/// Ignores `.asm` files that have already been processed.
/// 
/// These have names like this:
/// 
/// ```csv
/// mine-event/20220710-054111-1237572183.keep.asm
/// mine-event/20220710-054111-1237578248.reject.asm
/// mine-event/manual-coded-program.keep.asm
/// ```
/// 
/// Returns an error if there are no files waiting for processing.
pub fn find_pending_programs(paths_inside_mineevent_dir: &Vec<PathBuf>, verbose: bool) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let instance = MineEventDirectoryScan::scan(paths_inside_mineevent_dir);
    if verbose {
        instance.print_summary();
    }
    Ok(instance.pending_paths())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_multiple_pending_programs() {
        static INPUT: &'static [&'static str] = &[
            "mine-event/20220710-054111-1237572183.keep.asm",
            "mine-event/20220710-054111-1237578248.reject.asm",
            "mine-event/20220710-054111-1237590359.reject.asm",
            "mine-event/20220710-054915-1251916462.asm",
            "mine-event/20220710-055020-1265182884.asm",
            "mine-event/20220710-055920-1279752621.asm",
            "mine-event/20220710-062906-1376725035.asm",
            "mine-event/manual-coded-program.asm",
        ];
        let input_paths: Vec<PathBuf> = INPUT.iter().map(|path| PathBuf::from(path) ).collect();
        let result = find_pending_programs(&input_paths, false);
        let output_paths: Vec<PathBuf> = result.expect("Must return ok");
        assert_eq!(output_paths.len(), 5);
    }

    #[test]
    fn test_10001_no_pending_programs() {
        static INPUT: &'static [&'static str] = &[
            "mine-event/20220710-054111-1237572183.keep.asm",
            "mine-event/20220710-054111-1237578248.reject.asm",
            "mine-event/20220710-054111-1237590359.reject.asm",
            "mine-event/manual-coded-program.keep.asm",
        ];
        let input_paths: Vec<PathBuf> = INPUT.iter().map(|path| PathBuf::from(path) ).collect();
        let result = find_pending_programs(&input_paths, false);
        let output_paths: Vec<PathBuf> = result.expect("Must return ok");
        assert_eq!(output_paths.len(), 0);
    }
}
