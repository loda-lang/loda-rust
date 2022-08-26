use super::{ProgramCache, ProgramRunnerManager, ProgramSerializer, ProgramSerializerContext, ProgramState, RegisterIndex, EvalError, ValidateCallError};
use std::collections::HashSet;

pub trait Node {
    fn formatted_instruction(&self) -> String;

    fn formatted_instruction_advanced(&self, _context: &dyn ProgramSerializerContext) -> Option<String> {
        None
    }

    /// Generate a human readable version of the program
    /// Append the instruction to the program.
    /// For most nodes, this is irrelevant, so this does nothing by default.
    /// However for loop instructions, there is indentation to deal with.
    fn serialize(&self, serializer: &mut ProgramSerializer) {
        if let Some(text) = self.formatted_instruction_advanced(serializer.context()) {
            serializer.append_raw(text);
        } else {
            serializer.append_raw(self.formatted_instruction());
        }
    }

    /// Execute the primary operation of this node.
    /// If it's an "add" node, then it computes 1 + 3 = 4, and Ok is the result.
    /// The are several ways eval can go wrong, in which case an Error is the result. 
    /// If it's a "div" node and it attempts to do division by zero, then it triggers an Error result.
    fn eval(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError>;

    /// Determine the number of registers required by this program.
    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {}

    /// Determine what registers convey info based on the input data
    fn live_register_indexes(&self, _register_set: &mut HashSet<RegisterIndex>) {}
    
    /// Gather a list of dependencies on other programs.
    /// Every CallNode depends on another program_id. These program_id's gets appended to the result.
    /// For most nodes, this is irrelevant, so this does nothing by default.
    fn accumulate_call_dependencies(&self, _program_id_vec: &mut Vec<u64>) {}
    
    /// Establish links to other programs.
    /// A CallNode looks up its program_id in the `ProgramRunnerManager`.
    /// If found, then establishes a link from the CallNode to the program.
    /// For most nodes, this is irrelevant, so this does nothing by default.
    fn update_call(&mut self, _program_manager: &mut ProgramRunnerManager) {}
    
    /// Check CallNode's have been linked with the program they depend on.
    /// If there is a node with a missing link, then an error is returned.
    /// If there are no missing links, then Ok is returned.
    fn validate_call_nodes(&self) -> Result<(), ValidateCallError> {
        Ok(())
    }
}

pub type BoxNode = Box<dyn Node>;
