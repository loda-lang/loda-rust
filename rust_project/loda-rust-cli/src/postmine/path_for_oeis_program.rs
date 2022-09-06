use crate::oeis::OeisId;
use super::ParentDirAndChildFile;
use std::path::{Path, PathBuf};

/// Construct a path, like this: `/absolute/path/123/A123456.asm`
pub fn path_for_oeis_program(loda_programs_oeis_dir: &Path, program_id: OeisId) -> ParentDirAndChildFile {
    assert!(loda_programs_oeis_dir.is_dir());
    assert!(loda_programs_oeis_dir.is_absolute());
    let dir_index: u32 = program_id.raw() / 1000;
    let dir_index_string: String = format!("{:0>3}", dir_index);
    let filename_string: String = format!("{}.asm", program_id.a_number());
    let dir_path: PathBuf = loda_programs_oeis_dir.join(dir_index_string);
    let file_path: PathBuf = dir_path.join(filename_string);
    ParentDirAndChildFile::new(dir_path, file_path)
}
