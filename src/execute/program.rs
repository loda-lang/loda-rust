use super::{Node, BoxNode, ProgramState, ProgramRunnerManager, RegisterIndex, RunMode, ValidateCallError};

type BoxNodeVec = Vec<BoxNode>;

pub struct Program {
    node_vec: BoxNodeVec
}

impl Program {
    pub fn new() -> Self {
        Program {
            node_vec: vec!()
        }
    }

    pub fn push<T: Node + 'static>(&mut self, node: T) {
        let node_wrapped = Box::new(node);
        self.node_vec.push(node_wrapped);
    }

    pub fn push_boxed(&mut self, node_wrapped: BoxNode) {
        self.node_vec.push(node_wrapped);
    }

    pub fn run(&self, state: &mut ProgramState) {
        match state.run_mode() {
            RunMode::Verbose => self.run_verbose(state),
            RunMode::Silent => self.run_silent(state),
        }
    }

    pub fn run_silent(&self, state: &mut ProgramState) {
        for node in &self.node_vec {
            node.eval(state);
            state.increment_eval_count();
        }
    }

    pub fn run_verbose(&self, state: &mut ProgramState) {
        for node in &self.node_vec {
            let before = state.register_vec_to_string();
            node.eval(state);
            let after = state.register_vec_to_string();
            let instruction: String = node.formatted_instruction();
            if !instruction.is_empty() {
                println!("{:12} {} => {}", instruction, before, after);
            }
            state.increment_eval_count();
        }
    }

    pub fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        for node in &self.node_vec {
            node.accumulate_register_indexes(register_vec);
        }
    }

    pub fn update_call(&mut self, program_manager: &mut ProgramRunnerManager) {
        for node in &mut self.node_vec {
            node.update_call(program_manager);
        }
    }

    pub fn accumulate_call_dependencies(&self, program_id_vec: &mut Vec<u64>) {
        for node in &self.node_vec {
            node.accumulate_call_dependencies(program_id_vec);
        }
    }

    pub fn validate_call_nodes(&self) -> Result<(), ValidateCallError> {
        for node in &self.node_vec {
            node.validate_call_nodes()?;
        }
        Ok(())
    }

    // This helps determining the number of registers to allocate.
    // If the highest register index used is 33.
    // Then we know we have to allocate 34 regisers.
    pub fn max_register_index(&self) -> u8 {
        // Populate vector with register indexes that a program use
        let mut register_vec: Vec<RegisterIndex> = vec!();
        self.accumulate_register_indexes(&mut register_vec);

        // Max value in the vector
        let indexes: Vec<u8> = register_vec.iter().map(|register_index| {
            register_index.0
        }).collect();
        match indexes.iter().max() {
            Some(value) => {
                return value.clone();
            },
            None => {
                return 0;
            }
        }
    }
}
