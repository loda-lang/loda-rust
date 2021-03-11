use super::{ProgramState, ProgramRunnerManager, RegisterIndex};

pub trait Node {
    fn shorthand(&self) -> &str;

    fn formatted_instruction(&self) -> String;

    fn eval(&self, state: &mut ProgramState);

    // Determine the number of registers required by this program.
    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {}
    
    // IDEA: rename from update_call, to mount/connect/link/assign/attach_dependencies
    // Establish links to other programs.
    fn update_call(&mut self, _program_manager: &mut ProgramRunnerManager) {} // does nothing by default
    
    // IDEA: rename to identify_dependencies
    // Gather a list of dependencies on other programs.
    fn accumulate_call_dependencies(&self, _program_id_vec: &mut Vec<u64>) {} // does nothing by default
    
    // IDEA: verify that all CallNode's really have been assigned a program.
}

pub type BoxNode = Box<dyn Node>;
