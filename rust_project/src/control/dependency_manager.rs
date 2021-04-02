use std::fmt;
use std::fs;
use std::path::{Path,PathBuf};
use std::collections::HashSet;
use std::rc::Rc;
use crate::parser::{ParsedProgram, ParseProgramError, parse_program, create_program, CreatedProgram, CreateProgramError};
use crate::execute::{Program, ProgramId, ProgramRunner, ProgramRunnerManager};

#[derive(Debug)]
pub enum DependencyManagerError {
    CannotLoadFile,
    CyclicDependency,
    ParseProgram(ParseProgramError),
    CreateProgram(CreateProgramError),
    LookupProgramId,
}

impl fmt::Display for DependencyManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CannotLoadFile =>
                write!(f, "Failed to load the assembler file"),
            Self::CyclicDependency =>
                write!(f, "Detected a cyclic dependency"),
            Self::ParseProgram(error) => 
                write!(f, "Failed to parse program. error: {}", error),
            Self::CreateProgram(error) => 
                write!(f, "Failed to create program. error: {}", error),
            Self::LookupProgramId =>
                write!(f, "Failed to lookup the program id"),
        }
    }
}

pub struct DependencyManager {
    loda_program_dir: PathBuf,
    program_run_manager: ProgramRunnerManager,
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

    pub fn reset(&mut self) {
        self.programid_dependencies.clear();
        self.programids_currently_loading.clear();
    }

    pub fn load(&mut self, program_id: u64) ->
        Result<Rc::<ProgramRunner>, DependencyManagerError> 
    {
        self.load_inner(program_id)?;
        let runner: Rc::<ProgramRunner> = match self.program_run_manager.get(program_id) {
            Some(value) => value,
            None => {
                return Err(DependencyManagerError::LookupProgramId);
            }
        };
        Ok(runner)
    }

    fn load_inner(&mut self, program_id: u64) -> Result<(), DependencyManagerError> {
        self.programid_dependencies.push(program_id);

        if self.program_run_manager.contains(program_id) {
            // program is already loaded. No need to load it again.
            return Ok(());
        }
        if self.programids_currently_loading.contains(&program_id) {
            error!("detected cyclic dependency. program_id: {}", program_id);
            return Err(DependencyManagerError::CyclicDependency);
        }
        self.programids_currently_loading.insert(program_id);
        let path = self.path_to_program(program_id);

        let contents: String = match fs::read_to_string(&path) {
            Ok(value) => value,
            Err(error) => {
                error!("Something went wrong reading the file: {:?}", error);
                return Err(DependencyManagerError::CannotLoadFile);
            }
        };

        let program_id_inner = ProgramId::ProgramOEIS(program_id);
        let runner: ProgramRunner = self.parse(program_id_inner, &contents)?;    
        self.program_run_manager.register(program_id, runner);
        self.programids_currently_loading.remove(&program_id);
        Ok(())
    }

    pub fn parse(&mut self, program_id: ProgramId, contents: &String) -> 
        Result<ProgramRunner, DependencyManagerError> 
    {
        let parsed_program: ParsedProgram = match parse_program(contents) {
            Ok(value) => value,
            Err(error) => {
                return Err(DependencyManagerError::ParseProgram(error));
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
    
        self.load_dependencies(&mut program, &program_id)?;

        let runner = ProgramRunner::new(
            program_id,
            program
        );
        Ok(runner)
    }

    fn load_dependencies(&mut self, program: &mut Program, program_id: &ProgramId) -> Result<(), DependencyManagerError> {
        let mut dependent_program_id_vec: Vec<u64> = vec!();
        program.accumulate_call_dependencies(&mut dependent_program_id_vec);
        if !dependent_program_id_vec.is_empty() {
            debug!("program_id: {:?}  depends on other programs: {:?}", program_id, dependent_program_id_vec);
        }
        for dependent_program_id in dependent_program_id_vec {
            self.load_inner(dependent_program_id)?;
        }
        program.update_call(&mut self.program_run_manager);
        if program.validate_call_nodes().is_err() {
            panic!("program_id: {:?}  failed to assign all dependencies", program_id);
        }
        Ok(())
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
    use std::rc::Rc;

    const INPUT_A000079: &str = r#"
    ; A000079: Powers of 2: a(n) = 2^n.
    ; 1,2,4,8,16,32,64,128,256,512
    
    mov $1,2
    pow $1,$0
    "#;

    #[test]
    fn test_10000_parse_string() {
        let mut dm = DependencyManager::new(
            PathBuf::from("non-existing-dir"),
        );
        let source_code: String = INPUT_A000079.to_string();
        let runner: ProgramRunner = dm.parse(ProgramId::ProgramOEIS(79), &source_code).unwrap();
        assert_eq!(runner.inspect(10), "1,2,4,8,16,32,64,128,256,512");
    }

    fn dependency_manager_mock(relative_path_to_testdir: &str) -> DependencyManager {
        let e = env!("CARGO_MANIFEST_DIR");
        let path = PathBuf::from(e).join(relative_path_to_testdir);
        DependencyManager::new(path)
    }

    #[test]
    fn test_10101_load_simple1() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/dependency_manager_load_simple1");
        let runner: Rc::<ProgramRunner> = dm.load(79).unwrap();
        assert_eq!(runner.inspect(10), "1,2,4,8,16,32,64,128,256,512");
    }

    #[test]
    fn test_10102_load_simple2() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/dependency_manager_load_simple2");
        let runner: Rc::<ProgramRunner> = dm.load(1).unwrap();
        assert_eq!(runner.inspect(10), "1,2,1,2,1,2,1,2,1,2");
    }

    #[test]
    fn test_10201_load_detect_cycle1() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/dependency_manager_load_detect_cycle1");
        assert!(dm.load(666).is_err());
    }

    #[test]
    fn test_10202_load_detect_cycle2() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/dependency_manager_load_detect_cycle2");
        assert!(dm.load(666).is_err());
    }

    #[test]
    fn test_10203_load_detect_cycle3() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/dependency_manager_load_detect_cycle3");
        assert!(dm.load(666).is_err());
    }

    #[test]
    fn test_10301_load_detect_missing1() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/dependency_manager_load_detect_missing1");
        assert!(dm.load(666).is_err());
    }
}
