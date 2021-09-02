use std::collections::HashMap;
use std::rc::Rc;
use super::ProgramRunner;

pub struct ProgramRunnerManager {
    registered_program_runners: HashMap<u64, Rc::<ProgramRunner>>
}

impl ProgramRunnerManager {
    pub fn new() -> Self {
        Self {
            registered_program_runners: HashMap::new(),
        }
    }

    pub fn register(&mut self, program_id: u64, program_runner: ProgramRunner) {
        // TODO: abort if the program is already registered
        let program_runner_rc = Rc::new(program_runner);
        self.registered_program_runners.insert(program_id, program_runner_rc);
    }

    pub fn get(&mut self, program_id: u64) -> Option<Rc::<ProgramRunner>> {
        match self.registered_program_runners.get(&program_id) {
            Some(value) => {
                return Some(Rc::clone(value));
            },
            None => {
                return None;
            }
        }
    }

    pub fn contains(&self, program_id: u64) -> bool {
        self.registered_program_runners.contains_key(&program_id)
    }
}
