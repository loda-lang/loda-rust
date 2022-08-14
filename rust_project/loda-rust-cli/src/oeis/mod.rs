//! OEIS code for A-numbers, parsing of the `stripped` file and the `names` file.
mod name_row;
mod oeis_id;
mod process_names_file;
mod process_stripped_file;
mod stripped_row;
mod terms_to_program_id;

pub use name_row::NameRow;
pub use oeis_id::{OeisId, OeisIdHashSet};
pub use process_names_file::ProcessNamesFile;
pub use process_stripped_file::ProcessStrippedFile;
pub use stripped_row::StrippedRow;
pub use terms_to_program_id::{TermsToProgramIdSet, load_terms_to_program_id_set};
