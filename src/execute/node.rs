use super::{ProgramState, ProgramRunnerManager, RegisterIndex};

pub struct ValidateCallError {}

#[derive(Debug)]
pub enum EvalError {
    DivisionByZero,

    // When a mathematical function is evaluated outside of its domain of definition.
    LogDomainError,

    // When a mathematical function is evaluated outside of its domain of definition.
    GCDDomainError,

    PowerZeroDivision,
    PowerExponentTooHigh,
}

pub trait Node {
    fn shorthand(&self) -> &str;

    fn formatted_instruction(&self) -> String;

    fn eval(&self, state: &mut ProgramState);

    fn eval_advanced(&self, state: &mut ProgramState) -> Result<(), EvalError> {
        self.eval(state);
        Ok(())
    }

    // Determine the number of registers required by this program.
    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {}
    
    // Gather a list of dependencies on other programs.
    // Every CallNode depends on another program_id. These program_id's gets appended to the result.
    // For most nodes, this is irrelevant, so this does nothing by default.
    fn accumulate_call_dependencies(&self, _program_id_vec: &mut Vec<u64>) {}
    
    // Establish links to other programs.
    // A CallNode looks up its program_id in the `ProgramRunnerManager`.
    // If found, then establishes a link from the CallNode to the program.
    // For most nodes, this is irrelevant, so this does nothing by default.
    fn update_call(&mut self, _program_manager: &mut ProgramRunnerManager) {}
    
    // Check CallNode's have been linked with the program they depend on.
    // If there is a node with a missing link, then an error is returned.
    // If there are no missing links, then Ok is returned.
    fn validate_call_nodes(&self) -> Result<(), ValidateCallError> {
        Ok(())
    }
}

pub type BoxNode = Box<dyn Node>;
