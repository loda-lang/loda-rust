use super::{ProgramCache, ProgramRunnerManager, ProgramSerializer, ProgramState, RegisterIndex};
use std::collections::HashSet;

pub struct ValidateCallError {}

#[derive(Debug)]
pub enum EvalError {
    // During mining it makes little sense if the values are too extreme to 
    // possible lead to a result. Here the CheckValue settings controls the limit.
    // When not-mining there are no limit to the register value.
    InputOutOfRange,
    OutputOutOfRange,

    // Programs are usually well behaved for 0 and greater values.
    // However for negative values the behavior is undefined.
    CallWithNegativeParameter,
    CallOutOfRange,

    DivisionByZero,

    // Binomial with N >= 34 and the result value can no longer fit into a 32bit integer.
    // Binomial with N >= 67 and the result value can no longer fit into a 64bit integer.
    // During mining, it can be a time waster computing binomial with huge values.
    BinomialDomainError,

    // When a mathematical function is evaluated outside of its domain of definition.
    GCDDomainError,
    GCDOutOfRange,

    PowerZeroDivision,
    PowerExponentTooHigh,
    // During mining, it can be a time waster computing power with huge values.
    PowerExceededLimit,

    // Range length is beyond the ProgramState max length
    LoopRangeLengthExceededLimit,

    // Stuck in a loop that takes way too long time to compute
    LoopCountExceededLimit,

    // Using way too many cpu cycles
    StepCountExceededLimit,
}

pub trait Node {
    fn formatted_instruction(&self) -> String;

    // Generate a human readable version of the program
    // Append the instruction to the program.
    // For most nodes, this is irrelevant, so this does nothing by default.
    // However for loop instructions, there is indentation to deal with.
    fn serialize(&self, serializer: &mut ProgramSerializer) {
        serializer.append(self.formatted_instruction());
    }

    // Execute the primary operation of this node.
    // If it's an "add" node, then it computes 1 + 3 = 4, and Ok is the result.
    // The are several ways eval can go wrong, in which case an Error is the result. 
    // If it's a "div" node and it attempts to do division by zero, then it triggers an Error result.
    fn eval(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError>;

    // Determine the number of registers required by this program.
    fn accumulate_register_indexes(&self, _register_vec: &mut Vec<RegisterIndex>) {}

    // Determine what registers convey info based on the input data
    fn live_register_indexes(&self, _register_set: &mut HashSet<RegisterIndex>) {}
    
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

#[derive(Clone)]
pub enum NodeRegisterLimit {
    Unlimited,
    LimitBits(u32)
}

