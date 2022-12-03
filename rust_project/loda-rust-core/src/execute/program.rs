use super::{BoxNode, EvalError, Node, ProgramCache, ProgramRunnerManager, ProgramSerializer, ProgramState, RunMode, ValidateCallError};

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

    pub fn serialize(&self, serializer: &mut ProgramSerializer) {
        for node in &self.node_vec {
            node.serialize(serializer);
        }
    }

    pub fn run(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError> {
        match state.run_mode() {
            RunMode::Verbose => self.run_verbose(state, cache),
            RunMode::Silent => self.run_silent(state, cache),
        }
    }

    pub fn run_silent(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError> {
        for node in &self.node_vec {
            node.eval(state, cache)?;
        }
        Ok(())
    }

    pub fn run_verbose(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError> {
        for node in &self.node_vec {
            let before = state.memory_full_to_string();
            let result = node.eval(state, cache);
            if let Err(error) = result {
                let instruction: String = node.formatted_instruction();
                println!("{:12} {} => ERROR: {:?}", instruction, before, error);
                return Err(error);
            }
            let after = state.memory_full_to_string();
            let instruction: String = node.formatted_instruction();
            if !instruction.is_empty() {
                println!("{:12} {} => {}", instruction, before, after);
            }
        }
        Ok(())
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
}
