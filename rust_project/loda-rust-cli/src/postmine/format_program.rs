use crate::common::OeisIdStringMap;
use crate::oeis::OeisId;
use super::ProgramSerializerContextWithSequenceName;
use loda_rust_core::execute::{ProgramId, ProgramRunner, ProgramSerializer};
use loda_rust_core::parser::ParsedProgram;
use loda_rust_core::control::{DependencyManager,DependencyManagerFileSystemMode};
use std::path::{Path, PathBuf};

/// Formatting of a LODA program.
/// 
/// Insert a header, like this:
/// ; A123456: Oeis name
/// ; Submitted by John Doe
/// ; 1,2,3,4,5,6
/// 
/// When encountering a `seq` instruction, then insert the corresponding oeis name.
pub struct FormatProgram {
    program_content: String,
    oeis_id_name_map: OeisIdStringMap,
    program_oeis_id: Option<OeisId>,
    program_path: Option<PathBuf>,
    loda_submitted_by: Option<String>,
    terms: Option<String>,
}

impl FormatProgram {
    pub fn new(program_content: String) -> Self {
        Self {
            program_content: program_content,
            oeis_id_name_map: OeisIdStringMap::new(),
            program_oeis_id: None,
            program_path: None,
            loda_submitted_by: None,
            terms: None,
        }
    }

    pub fn oeis_id_name_map(&mut self, oeis_id_name_map: OeisIdStringMap) -> &mut FormatProgram {
        self.oeis_id_name_map = oeis_id_name_map;
        self
    }    

    pub fn program_path(&mut self, program_path: &Path) -> &mut FormatProgram {
        self.program_path = Some(PathBuf::from(program_path));
        self
    }

    pub fn program_oeis_id(&mut self, program_oeis_id: OeisId) -> &mut FormatProgram {
        self.program_oeis_id = Some(program_oeis_id);
        self
    }

    pub fn loda_submitted_by(&mut self, loda_submitted_by: String) -> &mut FormatProgram {
        self.loda_submitted_by = Some(loda_submitted_by);
        self
    }

    pub fn terms(&mut self, terms: String) -> &mut FormatProgram {
        self.terms = Some(terms);
        self
    }

    pub fn build(&self) -> anyhow::Result<String> {
        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&self.program_content) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("Parse program from {:?} error: {:?}", &self.program_path, error));
            }
        };
    
        // Don't load dependencies from the file system,
        // by pretending that all the dependencies are empty programs
        // that are already loaded.
        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::Virtual,
            PathBuf::from("non-existing-dir"),
        );
        for (oeis_id, _name) in &self.oeis_id_name_map {
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
                return Err(anyhow::anyhow!("parse_stage2 with program {:?} error: {:?}", &self.program_path, error));
            }
        };

        let mut serializer = ProgramSerializer::new();

        // Pass on the `oeis_id_name_map` all the way to the formatting code
        // of the `seq` instruction, so that the sequence name can be inserted as a comment.
        // Like this: `seq $2,40 ; The prime numbers.`
        let context = ProgramSerializerContextWithSequenceName::new(self.oeis_id_name_map.clone());
        serializer.set_context(Box::new(context));
    
        // The sequence name
        if let Some(oeis_id) = self.program_oeis_id {
            let optional_name: Option<&String> = self.oeis_id_name_map.get(&oeis_id);
            match optional_name {
                Some(name) => {
                    serializer.append_comment(format!("{}: {}", oeis_id, name));
                },
                None => {
                    return Err(anyhow::anyhow!("missing sequence name for oeis_id {} for program {:?}", oeis_id, &self.program_path));
                }
            }
        }
    
        // Submitted by Euler
        if let Some(loda_submitted_by) = &self.loda_submitted_by {
            serializer.append_comment(format!("Submitted by {}", loda_submitted_by));
        }
    
        // The initital terms
        if let Some(terms) = &self.terms {
            serializer.append_comment(terms);
        }
    
        serializer.append_empty_line();
        runner.serialize(&mut serializer);
        serializer.append_empty_line();
        let file_content: String = serializer.to_string();
        Ok(file_content)
    }
}
