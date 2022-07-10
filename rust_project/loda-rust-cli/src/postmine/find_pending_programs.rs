use crate::postmine::PostMineError;
use std::error::Error;
use std::path::PathBuf;
use regex::Regex;
use lazy_static::lazy_static;
use std::ffi::OsStr;

lazy_static! {
    /// Determine if the filename contains `keep` or `reject`
    static ref ALREADY_PROCESSED: Regex = Regex::new(
        "[.](?:keep|reject)[.]asm$"
    ).unwrap();
}

/// Find `.asm` files that are waiting to be processed.
/// 
/// These have names like this:
/// 
/// ```csv
/// mine-event/20220710-054915-1251916462.asm
/// mine-event/20220710-055020-1265182884.asm
/// ```
/// 
/// Ignores `.asm` files that have already been processed.
/// 
/// These have names like this:
/// 
/// ```csv
/// mine-event/20220710-054111-1237572183.keep.asm
/// mine-event/20220710-054111-1237578248.reject.asm
/// ```
/// 
/// Returns an error if there are no files waiting for processing.
pub fn find_pending_programs(paths_inside_mineevent_dir: &Vec<PathBuf>, verbose: bool) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let re = &ALREADY_PROCESSED;
    let mut paths_for_processing = Vec::<PathBuf>::new();
    let mut count_already_processed: usize = 0;
    for path in paths_inside_mineevent_dir {
        let filename: &OsStr = match path.file_name() {
            Some(value) => value,
            None => {
                error!("Unable to extract filename from path");
                continue;
            }
        };
        if re.is_match(&filename.to_string_lossy()) {
            count_already_processed += 1;
            continue;
        }
        paths_for_processing.push(PathBuf::from(path));
    }
    if count_already_processed > 0 {
        if verbose {
            println!("Ignoring {} programs that have already been analyzed", count_already_processed);
        }
    }
    let number_of_paths = paths_for_processing.len();
    if verbose {
        println!("Number of pending programs: {}", number_of_paths);
    }
    if number_of_paths <= 0 {
        return Err(Box::new(PostMineError::NoPendingProgramsInMineEventDir));
    }
    paths_for_processing.sort();
    Ok(paths_for_processing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_success() {
        static INPUT: &'static [&'static str] = &[
            "mine-event/20220710-054111-1237572183.keep.asm",
            "mine-event/20220710-054111-1237578248.reject.asm",
            "mine-event/20220710-054111-1237590359.reject.asm",
            "mine-event/20220710-054915-1251916462.asm",
            "mine-event/20220710-055020-1265182884.asm",
            "mine-event/20220710-055920-1279752621.asm",
            "mine-event/20220710-062906-1376725035.asm"
        ];
        let input_paths: Vec<PathBuf> = INPUT.iter().map(|path| PathBuf::from(path) ).collect();
        let result = find_pending_programs(&input_paths, false);
        let output_paths: Vec<PathBuf> = result.expect("Must return ok");
        assert_eq!(output_paths.len(), 4);
    }

    #[test]
    fn test_10001_error_no_pending_programs() {
        static INPUT: &'static [&'static str] = &[
            "mine-event/20220710-054111-1237572183.keep.asm",
            "mine-event/20220710-054111-1237578248.reject.asm",
            "mine-event/20220710-054111-1237590359.reject.asm",
        ];
        let input_paths: Vec<PathBuf> = INPUT.iter().map(|path| PathBuf::from(path) ).collect();
        let result = find_pending_programs(&input_paths, false);
        assert!(result.is_err());
    }
}
