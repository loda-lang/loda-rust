use std::fmt;
use std::fs;
use std::path::{Path,PathBuf};
use std::collections::HashSet;
use std::collections::HashMap;
use std::rc::Rc;
use crate::parser::{ParsedProgram, ParseProgramError, create_program, CreatedProgram, CreateProgramError};
use crate::execute::{Program, ProgramId, ProgramRunner, ProgramRunnerManager};

#[derive(Debug, PartialEq)]
pub struct CyclicDependencyError {
    program_id: u64,
}

impl CyclicDependencyError {
    pub fn new(program_id: u64) -> Self {
        Self {
            program_id: program_id,
        }
    }
}

#[derive(Debug)]
pub struct CannotReadProgramFileError {
    program_id: u64,
    #[allow(dead_code)]
    io_error: std::io::Error,
}

impl CannotReadProgramFileError {
    pub fn new(program_id: u64, io_error: std::io::Error) -> Self {
        Self {
            program_id: program_id,
            io_error: io_error,
        }
    }

    #[allow(dead_code)]
    pub fn program_id(&self) -> u64 {
        self.program_id
    }
}

impl PartialEq for CannotReadProgramFileError {
    fn eq(&self, other: &Self) -> bool {
        self.program_id == other.program_id
    }
}

#[derive(Debug, PartialEq)]
pub enum DependencyManagerError {
    CannotReadProgramFile(CannotReadProgramFileError),
    CannotReadProgramFileFromVirtualFileSystem,
    CyclicDependency(CyclicDependencyError),
    ParseProgram(ParseProgramError),
    CreateProgram(CreateProgramError),
    LookupProgramId,
}

impl fmt::Display for DependencyManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CannotReadProgramFile(error) =>
                write!(f, "Failed to load the assembler file. program_id: {}", error.program_id),
            Self::CannotReadProgramFileFromVirtualFileSystem =>
                write!(f, "Failed to load the assembler file from virtual file system"),
            Self::CyclicDependency(error) =>
                write!(f, "Detected a cyclic dependency. program_id: {}", error.program_id),
            Self::ParseProgram(error) => 
                write!(f, "Failed to parse program. error: {}", error),
            Self::CreateProgram(error) => 
                write!(f, "Failed to create program. error: {}", error),
            Self::LookupProgramId =>
                write!(f, "Failed to lookup the program id"),
        }
    }
}

pub enum DependencyManagerFileSystemMode {
    System,
    Virtual
}

pub struct DependencyManager {
    file_system_mode: DependencyManagerFileSystemMode,
    loda_programs_oeis_dir: PathBuf,
    program_run_manager: ProgramRunnerManager,
    programids_currently_loading: HashSet<u64>,
    programid_dependencies: Vec<u64>,
    virtual_filesystem: HashMap<u64, String>,
    metric_read_success: u64,
    metric_read_error: u64,
}

impl DependencyManager {
    pub fn new(file_system_mode: DependencyManagerFileSystemMode, loda_programs_oeis_dir: PathBuf) -> Self {
        Self {
            file_system_mode: file_system_mode,
            loda_programs_oeis_dir: loda_programs_oeis_dir,
            program_run_manager: ProgramRunnerManager::new(),
            programids_currently_loading: HashSet::new(),
            programid_dependencies: vec!(),
            virtual_filesystem: HashMap::new(),
            metric_read_success: 0,
            metric_read_error: 0,
        }        
    }

    pub fn reset(&mut self) {
        self.programid_dependencies.clear();
        self.programids_currently_loading.clear();
    }

    pub fn virtual_filesystem_insert_file(&mut self, program_id: u64, file_content: String) {
        self.virtual_filesystem.insert(program_id, file_content);
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
            // Program is already loaded. No need to load it again.
            return Ok(());
        }
        if self.programids_currently_loading.contains(&program_id) {
            // Detected a cyclic dependency, a chain of programs that calls each other. 
            let error = CyclicDependencyError::new(program_id);
            return Err(DependencyManagerError::CyclicDependency(error));
        }
        self.programids_currently_loading.insert(program_id);

        // Read the file, or return an error if no such file exist.
        let contents: String = match self.file_system_mode {
            DependencyManagerFileSystemMode::System => self.system_read(program_id)?,
            DependencyManagerFileSystemMode::Virtual => self.virtual_read(program_id)?
        };

        let program_id_inner = ProgramId::ProgramOEIS(program_id);
        let runner: ProgramRunner = self.parse(program_id_inner, &contents)?;    
        self.program_run_manager.register(program_id, runner);
        self.programids_currently_loading.remove(&program_id);
        Ok(())
    }

    /// Read a file from the actual file system.
    fn system_read(&mut self, program_id: u64) -> Result<String, DependencyManagerError> {
        let path = self.path_to_program(program_id);
        let contents: String = match fs::read_to_string(&path) {
            Ok(value) => value,
            Err(io_error) => {
                // Something went wrong reading the file.
                self.metric_read_error += 1;
                let error = CannotReadProgramFileError::new(program_id, io_error);
                return Err(DependencyManagerError::CannotReadProgramFile(error));
            }
        };
        self.metric_read_success += 1;
        Ok(contents)
    }

    /// Read a file from a dictionary.
    fn virtual_read(&mut self, program_id: u64) -> Result<String, DependencyManagerError> {
        let contents: String = match self.virtual_filesystem.get(&program_id) {
            Some(value) => value.clone(),
            None => {
                self.metric_read_error += 1;
                return Err(DependencyManagerError::CannotReadProgramFileFromVirtualFileSystem);
            }
        };
        self.metric_read_success += 1;
        Ok(contents)
    }

    pub fn parse(&mut self, program_id: ProgramId, contents: &String) -> 
        Result<ProgramRunner, DependencyManagerError> 
    {
        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(contents) {
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
            //trace!("program_id: {:?}  depends on other programs: {:?}", program_id, dependent_program_id_vec);
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

    /// Construct a path, like this: `/absolute/path/123/A123456.asm`
    pub fn path_to_program(&self, program_id: u64) -> PathBuf {
        let dir_index: u64 = program_id / 1000;
        let dir_index_string: String = format!("{:0>3}", dir_index);
        let filename_string: String = format!("A{:0>6}.asm", program_id);
        let dirname = Path::new(&dir_index_string);
        let filename = Path::new(&filename_string);
        let pathbuf: PathBuf = self.loda_programs_oeis_dir.join(dirname).join(filename);
        pathbuf
    }

    pub fn print_dependencies(&self) {
        let strings: Vec<String> = self.programid_dependencies.iter().map(|program_id| {
            program_id.to_string()
        }).collect();
        let program_id_pretty: String = strings.join(",");
        println!("{}", program_id_pretty);
    }

    pub fn contains(&self, program_id: u64) -> bool {
        self.program_run_manager.contains(program_id)
    }

    pub fn reset_metrics(&mut self) {
        self.metric_read_error = 0;
        self.metric_read_success = 0;
    }

    pub fn metric_read_success(&self) -> u64 {
        self.metric_read_success
    }

    pub fn metric_read_error(&self) -> u64 {
        self.metric_read_error
    }
}

impl fmt::Debug for DependencyManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DependencyManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn test_10000_parse_string() {
        const PROGRAM: &str = r#"        
        mov $1,2
        pow $1,$0
        mov $0,$1
        "#;
        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            PathBuf::from("non-existing-dir"),
        );
        let source_code: String = PROGRAM.to_string();
        let runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, &source_code).unwrap();
        assert_eq!(runner.inspect(10), "1,2,4,8,16,32,64,128,256,512");
    }

    #[test]
    fn test_10001_path_to_program() {
        let basedir = PathBuf::from("non-existing-dir");
        let dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            basedir.clone()
        );
        {
            let actual: PathBuf = dm.path_to_program(79);
            let expected: PathBuf = basedir.join("000/A000079.asm");
            assert_eq!(expected, actual);
        }
        {
            let actual: PathBuf = dm.path_to_program(123456);
            let expected: PathBuf = basedir.join("123/A123456.asm");
            assert_eq!(expected, actual);
        }
    }

    fn dependency_manager_mock(relative_path_to_testdir: &str) -> DependencyManager {
        let e = env!("CARGO_MANIFEST_DIR");
        let path = PathBuf::from(e).join(relative_path_to_testdir);
        DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            path
        )
    }

    #[test]
    fn test_10101_load_simple1() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/load_simple1");
        let runner: Rc::<ProgramRunner> = dm.load(79).unwrap();
        assert_eq!(runner.inspect(10), "1,2,4,8,16,32,64,128,256,512");
    }

    #[test]
    fn test_10102_load_simple2() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/load_simple2");
        let runner: Rc::<ProgramRunner> = dm.load(1).unwrap();
        assert_eq!(runner.inspect(10), "1,2,1,2,1,2,1,2,1,2");
    }

    #[test]
    fn test_10103_load_simple3() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/load_simple3");
        let runner: Rc::<ProgramRunner> = dm.load(120).unwrap();
        assert_eq!(runner.inspect(10), "0,1,1,2,1,2,2,3,1,2");
    }

    impl DependencyManagerError {
        fn expect_cyclic_dependency(&self) -> &CyclicDependencyError {
            match self {
                DependencyManagerError::CyclicDependency(value) => &value,
                _ => {
                    panic!("Expected CyclicDependency, but got something else.");
                }
            }
        }
    }

    #[test]
    fn test_10201_load_detect_cycle1() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/load_detect_cycle1");
        let dm_error: DependencyManagerError = dm.load(666).err().unwrap();
        let error: &CyclicDependencyError = dm_error.expect_cyclic_dependency();
        assert_eq!(error.program_id, 666);
    }

    #[test]
    fn test_10202_load_detect_cycle2() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/load_detect_cycle2");
        let dm_error: DependencyManagerError = dm.load(666).err().unwrap();
        let error: &CyclicDependencyError = dm_error.expect_cyclic_dependency();
        assert_eq!(error.program_id, 666);
    }

    #[test]
    fn test_10203_load_detect_cycle3() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/load_detect_cycle3");
        let dm_error: DependencyManagerError = dm.load(666).err().unwrap();
        let error: &CyclicDependencyError = dm_error.expect_cyclic_dependency();
        assert_eq!(error.program_id, 666);
    }

    impl DependencyManagerError {
        fn expect_cannot_read_program_file(&self) -> &CannotReadProgramFileError {
            match self {
                DependencyManagerError::CannotReadProgramFile(value) => &value,
                _ => {
                    panic!("Expected CannotReadProgramFile, but got something else.");
                }
            }
        }
    }
    #[test]
    fn test_10301_load_detect_missing1() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/load_detect_missing1");
        let dm_error: DependencyManagerError = dm.load(666).err().unwrap();
        let error: &CannotReadProgramFileError = dm_error.expect_cannot_read_program_file();
        assert_eq!(error.program_id(), 668);
    }

    #[test]
    fn test_20001_call_with_negative_parameter1() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/call_with_negative_parameter1");
        let runner: Rc::<ProgramRunner> = dm.load(666).unwrap();
        assert_eq!(runner.inspect(10), "BOOM");
    }

    #[test]
    fn test_20002_call_with_negative_parameter2() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/call_with_negative_parameter2");
        let runner: Rc::<ProgramRunner> = dm.load(666).unwrap();
        assert_eq!(runner.inspect(10), "BOOM");
    }

    #[test]
    fn test_30001_live_register1() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/live_register1");
        let runner: Rc::<ProgramRunner> = dm.load(1).unwrap();
        assert_eq!(runner.live_registers().len(), 1);
        assert_eq!(runner.has_live_registers(), true);
    }

    #[test]
    fn test_30002_live_register2() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/live_register2");
        let runner: Rc::<ProgramRunner> = dm.load(666).unwrap();
        assert_eq!(runner.live_registers().len(), 0);
        assert_eq!(runner.has_live_registers(), false);
    }

    #[test]
    fn test_30003_live_register3() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/live_register3");
        let runner: Rc::<ProgramRunner> = dm.load(1).unwrap();
        assert_eq!(runner.live_registers().len(), 3);
        assert_eq!(runner.has_live_registers(), true);
    }

    #[test]
    fn test_30004_live_register4() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/live_register4");
        let runner: Rc::<ProgramRunner> = dm.load(666).unwrap();
        assert_eq!(runner.live_registers().len(), 0);
        assert_eq!(runner.has_live_registers(), false);
    }

    #[test]
    fn test_30005_live_register5() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/live_register5");
        let runner: Rc::<ProgramRunner> = dm.load(1).unwrap();
        assert_eq!(runner.live_registers().len(), 2);
        assert_eq!(runner.has_live_registers(), true);
    }

    #[test]
    fn test_30006_live_register6() {
        let mut dm: DependencyManager = dependency_manager_mock("tests/live_register6");
        let runner: Rc::<ProgramRunner> = dm.load(1).unwrap();
        assert_eq!(runner.live_registers().len(), 1);
        assert_eq!(runner.has_live_registers(), true);
    }

    #[test]
    fn test_40001_mining_trick_attempt_fixing_the_output_register1() {
        const PROGRAM: &str = r#"
        mov $5,$0
        mov $0,0
        "#;
        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            PathBuf::from("non-existing-dir"),
        );
        let source_code: String = PROGRAM.to_string();
        let mut runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, &source_code).unwrap();
        assert_eq!(runner.live_registers().len(), 1);
        assert_eq!(runner.has_live_registers(), false);
        assert_eq!(runner.mining_trick_attempt_fixing_the_output_register(), true);
        assert_eq!(runner.live_registers().len(), 2);
        assert_eq!(runner.has_live_registers(), true);
    }

    #[test]
    fn test_40002_mining_trick_attempt_fixing_the_output_register2() {
        const PROGRAM: &str = r#"
        mov $5,$0
        mov $0,0
        mul $5,0
        "#;
        let mut dm = DependencyManager::new(
            DependencyManagerFileSystemMode::System,
            PathBuf::from("non-existing-dir"),
        );
        let source_code: String = PROGRAM.to_string();
        let mut runner: ProgramRunner = dm.parse(ProgramId::ProgramWithoutId, &source_code).unwrap();
        assert_eq!(runner.live_registers().len(), 0);
        assert_eq!(runner.has_live_registers(), false);
        assert_eq!(runner.mining_trick_attempt_fixing_the_output_register(), false);
        assert_eq!(runner.live_registers().len(), 0);
        assert_eq!(runner.has_live_registers(), false);
    }
}
