use crate::common::OeisIdStringMap;
use super::ProgramSerializerContextWithSequenceName;
use loda_rust_core::oeis::OeisId;
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
                return Err(anyhow::anyhow!("Parse program from {:?} error: {:?} content: {:?}", &self.program_path, error, self.program_content));
            }
        };
    
        // Don't load dependencies from the file system,
        // by pretending that all the dependencies are empty programs
        // that are already loaded.
        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::Virtual,
            PathBuf::from("non-existing-dir"),
        );
        for program_id in parsed_program.direct_dependencies() {
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
                return Err(anyhow::anyhow!(
                    "parse_stage2 with program {:?} error: {:?} content: {:?} virtual fs: {:?}", 
                    &self.program_path, 
                    error, 
                    self.program_content, 
                    dm.virtual_filesystem_inspect_filenames()
                ));
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
                    return Err(anyhow::anyhow!("missing sequence name for oeis_id {} for program {:?} content: {:?}", oeis_id, &self.program_path, self.program_content));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_10000_format_program_minimal() -> Result<(), Box<dyn Error>> {
        let program = "mul $0,-1".to_string();
        let fp = FormatProgram::new(program);
        let formatted_program: String = fp.build()?;
        assert_eq!(formatted_program, "\nmul $0,-1\n");
        Ok(())
    }

    #[test]
    fn test_20000_format_program_submitted_by() -> Result<(), Box<dyn Error>> {
        let program = "mul $0,-1".to_string();
        let mut fp = FormatProgram::new(program);
        fp.loda_submitted_by("Euler".to_string());
        let formatted_program: String = fp.build()?;
        assert_eq!(formatted_program, "; Submitted by Euler\n\nmul $0,-1\n");
        Ok(())
    }

    #[test]
    fn test_30000_format_program_terms() -> Result<(), Box<dyn Error>> {
        let program = "mul $0,-1".to_string();
        let mut fp = FormatProgram::new(program);
        fp.terms("1,2,3,4,5,6".to_string());
        let formatted_program: String = fp.build()?;
        assert_eq!(formatted_program, "; 1,2,3,4,5,6\n\nmul $0,-1\n");
        Ok(())
    }

    #[test]
    fn test_40000_format_program_sequence_name() -> Result<(), Box<dyn Error>> {
        let mut oeis_id_name_map = OeisIdStringMap::new();
        oeis_id_name_map.insert(OeisId::from(40), "The primes".to_string());
        let program = "mul $0,-1".to_string();
        let mut fp = FormatProgram::new(program);
        fp.program_oeis_id(OeisId::from(40));
        fp.oeis_id_name_map(oeis_id_name_map);
        let formatted_program: String = fp.build()?;
        assert_eq!(formatted_program, "; A000040: The primes\n\nmul $0,-1\n");
        Ok(())
    }

    #[test]
    fn test_40001_format_program_seq_instructions() -> Result<(), Box<dyn Error>> {
        let mut oeis_id_name_map = OeisIdStringMap::new();
        oeis_id_name_map.insert(OeisId::from(45), "Fibonacci".to_string());
        let program = "seq $0,45".to_string();
        let mut fp = FormatProgram::new(program);
        fp.oeis_id_name_map(oeis_id_name_map);
        let formatted_program: String = fp.build()?;
        assert_eq!(formatted_program, "\nseq $0,45 ; Fibonacci\n");
        Ok(())
    }

    #[test]
    fn test_50000_format_program_trim_comments_and_blanks() -> Result<(), Box<dyn Error>> {
        let program = "; ignore\n   mul $0,-1 ; ignore\n\n; ignore".to_string();
        let fp = FormatProgram::new(program);
        let formatted_program: String = fp.build()?;
        assert_eq!(formatted_program, "\nmul $0,-1\n");
        Ok(())
    }

    #[test]
    fn test_50000_format_program_parameter_type_indirect() -> Result<(), Box<dyn Error>> {
        let program = "add $$0,1\n\n\nmul $1,$$0".to_string();
        let fp = FormatProgram::new(program);
        let formatted_program: String = fp.build()?;
        assert_eq!(formatted_program, "\nadd $$0,1\nmul $1,$$0\n");
        Ok(())
    }

    #[test]
    fn test_90000_format_program_everything() -> Result<(), Box<dyn Error>> {
        // Arrange
        let mut oeis_id_name_map = OeisIdStringMap::new();
        oeis_id_name_map.insert(OeisId::from(40), "The primes".to_string());
        oeis_id_name_map.insert(OeisId::from(72677), "a(n) = prime(prime(n)+1)".to_string());
        let program = "seq $0,40\nseq $0,40".to_string();
        let mut fp = FormatProgram::new(program);
        fp.program_oeis_id(OeisId::from(72677));
        fp.oeis_id_name_map(oeis_id_name_map);
        fp.loda_submitted_by("Euler".to_string());
        fp.terms("5,7,13,19,37,43,61,71,89".to_string());

        // Act
        let actual: String = fp.build()?;

        // Assert
        let expected = 
r#"; A072677: a(n) = prime(prime(n)+1)
; Submitted by Euler
; 5,7,13,19,37,43,61,71,89

seq $0,40 ; The primes
seq $0,40 ; The primes
"#;
        assert_eq!(actual, expected);
        Ok(())
    }
}
