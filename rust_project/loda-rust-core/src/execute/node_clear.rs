use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex};
use crate::parser::InstructionParameter;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Signed, Zero};

pub struct NodeClear {
    target: InstructionParameter,
    source: InstructionParameter,
}

impl NodeClear {
    pub fn new(target: InstructionParameter, source: InstructionParameter) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeClear {
    fn formatted_instruction(&self) -> String {
        format!("clr {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let target: BigInt = state.get(&self.target, true)?;
        let target_u64: u64 = match target.to_u64() {
            Some(value) => value,
            None => {
                return Err(EvalError::CannotConvertBigIntToAddress);
            }
        };

        let source: BigInt = state.get(&self.source, false)?;
        if source.is_negative() || source.is_zero() {
            // clear instruction with source being negative or zero. Does nothing.
            return Ok(());
        }
        let source_u64: u64 = match source.to_u64() {
            Some(value) => value,
            None => {
                return Err(EvalError::ClearRangeLengthExceedsLimit);
            }
        };

        state.set_register_range_to_zero(RegisterIndex(target_u64), source_u64)?;
        Ok(())
    }
}
