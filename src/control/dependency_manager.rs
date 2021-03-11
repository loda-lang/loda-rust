use std::fs;
use std::path::{Path,PathBuf};
use std::collections::HashSet;
use crate::parser::parse::*;
use crate::execute::{Program,ProgramRunner,ProgramRunnerManager};

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
            panic!("detected cyclic dependency");
        }

        if self.program_run_manager.contains(program_id) {
            println!("program is already loaded. {}", program_id);
            return;
        }
        self.programids_currently_loading.insert(program_id);
        let path = self.path_to_program(program_id);

        let contents: String = fs::read_to_string(&path)
            .expect("Something went wrong reading the file");
    
        let parsed = match parse(&contents) {
            Ok(value) => value,
            Err(err) => {
                println!("ERROR: {}, file: {:?}", err, path);
                panic!();
            }
        };
    
        let mut program: Program = parsed.created_program.program;
    
        // Obtain a list of dependencies.
        let mut dependent_program_id_vec: Vec<u64> = vec!();
        program.accumulate_call_dependencies(&mut dependent_program_id_vec);
        if !dependent_program_id_vec.is_empty() {
            // TODO: only print in verbose mode
            // println!("program {} dependencies: {:?}", program_id, dependent_program_id_vec);
        }
        for dependent_program_id in dependent_program_id_vec {
            self.load(dependent_program_id);
        }
        program.update_call(&mut self.program_run_manager);

        let runner = ProgramRunner::new(program);
        self.program_run_manager.register(program_id, runner);
        self.programids_currently_loading.remove(&program_id);
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
