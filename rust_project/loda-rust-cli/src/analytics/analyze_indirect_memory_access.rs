use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};
use crate::common::create_csv_file;
use crate::common::ToOeisIdVec;
use crate::config::Config;
use loda_rust_core;
use loda_rust_core::parser::ParsedProgram;
use loda_rust_core::oeis::{OeisId, OeisIdHashSet};
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use serde::Serialize;

/// Creates a CSV file with the programs that uses indirect memory access.
///
/// Places where dollar dollar notation is used, that's `Indirect memory access`.
/// 
/// Examples:
/// ```
/// mov $$5,1
/// add $0,$$8
/// ```
/// 
/// Traverses all the programs inside the `loda-programs/oeis` dir.
/// 
/// This outputs a `indirect_memory_access.csv` file, with this format:
/// 
/// ```csv
/// program id
/// 41
/// 123
/// 132
/// 138
/// 141
/// ```
/// 
/// As of 2022-Oct-01 there are 1626 programs that makes use of dollar dollar syntax.
pub struct AnalyzeIndirectMemoryAccess {
    config: Config,
    programs_that_uses_indirect: OeisIdHashSet,
}

impl AnalyzeIndirectMemoryAccess {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
            programs_that_uses_indirect: HashSet::new(),
        }
    }

    fn analyze_inner(&mut self, program_id: u32, parsed_program: &ParsedProgram) {
        if parsed_program.contain_parameter_type_indirect() {
            self.programs_that_uses_indirect.insert(OeisId::from(program_id));
        }
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeIndirectMemoryAccess {
    fn plugin_name(&self) -> &'static str {
        "AnalyzeIndirectMemoryAccess"
    }
    
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> Result<(), Box<dyn Error>> {
        self.analyze_inner(context.program_id, &context.parsed_program);
        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        let oeis_ids: Vec<OeisId> = self.programs_that_uses_indirect.sorted_vec();

        let mut records = Vec::<Record>::new();
        for oeis_id in oeis_ids {
            records.push(Record { program_id: oeis_id.raw() });
        }

        let output_path: PathBuf = self.config.analytics_dir_indirect_memory_access_file();
        create_csv_file(&records, &output_path)
    }

    fn human_readable_summary(&self) -> String {
        let rows: Vec<String> = vec![
            format!("number of programs that uses indirect memory access: {:?}", self.programs_that_uses_indirect.len())
        ];
        rows.join("\n")
    }
}

#[derive(Serialize)]
struct Record {
    #[serde(rename = "program id")]
    program_id: u32,
}
