use super::{ProgramState, ProgramRunnerManager, RegisterIndex};
// use std::result;

pub struct ValidateCallError {}

pub trait Node {
    fn shorthand(&self) -> &str;

    fn formatted_instruction(&self) -> String;

    fn eval(&self, state: &mut ProgramState);

    // Determine the number of registers required by this program.
    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {}
    
    // Establish links to other programs.
    fn update_call(&mut self, _program_manager: &mut ProgramRunnerManager) {} // does nothing by default
    
    // Gather a list of dependencies on other programs.
    fn accumulate_call_dependencies(&self, _program_id_vec: &mut Vec<u64>) {} // does nothing by default
    
    // Check CallNode's have been linked with the program they depend on.
    // If there is a node with a missing link, then an error is returned.
    // If there are no missing links, then Ok is returned.
    fn validate_call_nodes(&self) -> Result<(), ValidateCallError> {
        Ok(())
    }
}

pub type BoxNode = Box<dyn Node>;
