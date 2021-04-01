use std::fmt;
use std::fs;
use std::path::{Path,PathBuf};
use std::collections::HashSet;
use crate::parser::{ParsedProgram, ParseError, parse, parse_program, create_program, CreatedProgram, CreateProgramError};
use crate::execute::{Program, ProgramId, ProgramRunner, ProgramRunnerManager};

#[derive(Debug)]
pub enum DependencyManagerError {
    Parse(ParseError),
    CreateProgram(CreateProgramError),
}

impl fmt::Display for DependencyManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Parse(error) => 
                write!(f, "Failed to parse program. error: {}", error),
            Self::CreateProgram(error) => 
                write!(f, "Failed to create program. error: {}", error),
        }
    }
}

pub struct DependencyManager {
    loda_program_dir: PathBuf,
    pub program_run_manager: ProgramRunnerManager,
    programids_currently_loading: HashSet<u64>,
    programid_dependencies: Vec<u64>,
}

impl DependencyManager {
    pub fn new(loda_program_dir: PathBuf) -> Self {
        Self {
            loda_program_dir: loda_program_dir,
            program_run_manager: ProgramRunnerManager::new(),
            programids_currently_loading: HashSet::new(),
            programid_dependencies: vec!(),
        }        
    }

    pub fn load(&mut self, program_id: u64) {
        self.programid_dependencies.push(program_id);

        if self.programids_currently_loading.contains(&program_id) {
            panic!("detected cyclic dependency. program_id: {}", program_id);
        }
        if self.program_run_manager.contains(program_id) {
            debug!("program is already loaded. program_id: {}", program_id);
            return;
        }
        self.programids_currently_loading.insert(program_id);
        let path = self.path_to_program(program_id);

        let contents: String = fs::read_to_string(&path)
            .expect("Something went wrong reading the file");
    
        let parsed = match parse(&contents) {
            Ok(value) => value,
            Err(err) => {
                panic!("error: {}, file: {:?}", err, path);
            }
        };
    
        let mut program: Program = parsed.created_program.program;
        let program_id_inner = ProgramId::ProgramOEIS(program_id);
        self.load_dependencies(&mut program, &program_id_inner);

        let runner = ProgramRunner::new(
            program_id_inner,
            program
        );
        self.program_run_manager.register(program_id, runner);
        self.programids_currently_loading.remove(&program_id);
    }

    pub fn parse(&mut self, program_id: ProgramId, contents: &String) -> 
        Result<ProgramRunner, DependencyManagerError> 
    {
        let parsed_program: ParsedProgram = match parse_program(contents) {
            Ok(value) => value,
            Err(error0) => {
                let error1 = ParseError::ParseProgram(error0);
                let error2 = DependencyManagerError::Parse(error1);
                return Err(error2);
            }
        };
        self.parse_stage2(program_id, &parsed_program)
    }

    pub fn parse_stage2(&mut self, program_id: ProgramId, parsed_program: &ParsedProgram) -> 
        Result<ProgramRunner, DependencyManagerError> 
    {
        let created_program: CreatedProgram = match create_program(&parsed_program.instruction_vec) {
            Ok(value) => value,
            Err(error) => {
                return Err(DependencyManagerError::CreateProgram(error));
            }
        };
        let mut program: Program = created_program.program;
    
        self.load_dependencies(&mut program, &program_id);

        let runner = ProgramRunner::new(
            program_id,
            program
        );
        Ok(runner)
    }

    fn load_dependencies(&mut self, program: &mut Program, program_id: &ProgramId) {
        let mut dependent_program_id_vec: Vec<u64> = vec!();
        program.accumulate_call_dependencies(&mut dependent_program_id_vec);
        if !dependent_program_id_vec.is_empty() {
            debug!("program_id: {:?}  depends on other programs: {:?}", program_id, dependent_program_id_vec);
        }
        for dependent_program_id in dependent_program_id_vec {
            self.load(dependent_program_id);
        }
        program.update_call(&mut self.program_run_manager);
        if program.validate_call_nodes().is_err() {
            panic!("program_id: {:?}  failed to assign all dependencies", program_id);
        }
    }

    // Construct a path: "/absolute/path/123/a123456.asm"
    fn path_to_program(&self, program_id: u64) -> PathBuf {
        let dir_index: u64 = program_id / 1000;
        let dir_index_string: String = format!("{:0>3}", dir_index);
        let filename_string: String = format!("a{:0>6}.asm", program_id);
        let dirname = Path::new(&dir_index_string);
        let filename = Path::new(&filename_string);
        let pathbuf: PathBuf = self.loda_program_dir.join(dirname).join(filename);
        pathbuf
    }

    pub fn print_dependencies(&self) {
        let strings: Vec<String> = self.programid_dependencies.iter().map(|program_id| {
            program_id.to_string()
        }).collect();
        let program_id_pretty: String = strings.join(",");
        println!("{}", program_id_pretty);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT_A000079: &str = r#"
    ; A000079: Powers of 2: a(n) = 2^n.
    ; 1,2,4,8,16,32,64,128,256,512
    
    mov $1,2
    pow $1,$0
    "#;

    #[test]
    fn test_10000_powers_of_2() {
        let mut dm = DependencyManager::new(
            PathBuf::from("non-existing-dir"),
        );
        let source_code: String = INPUT_A000079.to_string();
        let runner: ProgramRunner = dm.parse(ProgramId::ProgramOEIS(79), &source_code).unwrap();
        assert_eq!(runner.inspect(10), "1,2,4,8,16,32,64,128,256,512");
    }
}
