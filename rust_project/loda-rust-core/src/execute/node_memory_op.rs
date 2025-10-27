use super::{EvalError, ProgramCache, Node, ProgramState};
use crate::parser::{InstructionId, InstructionParameter};
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

pub struct NodeMemoryOp {
    instruction_id: InstructionId,
    start: InstructionParameter,
    length: InstructionParameter,
}

impl NodeMemoryOp {
    pub fn new(instruction_id: InstructionId, start: InstructionParameter, length: InstructionParameter) -> Self {
        Self {
            instruction_id,
            start,
            length,
        }
    }

    fn get_memory_range(&self, state: &ProgramState) -> Result<(i64, i64), EvalError> {
        let start_value: BigInt = state.get(&self.start, true)?;
        let start_i64: i64 = match start_value.to_i64() {
            Some(value) => value,
            None => return Err(EvalError::CannotConvertBigIntToAddress),
        };

        let length_value: BigInt = state.get(&self.length, false)?;
        let length_i64: i64 = match length_value.to_i64() {
            Some(value) => value,
            None => return Err(EvalError::CannotConvertBigIntToAddress),
        };

        let (first, second) = if length_i64 > 0 {
            (start_i64, start_i64 + length_i64)
        } else if length_i64 < 0 {
            (start_i64 + length_i64 + 1, start_i64 + 1)
        } else {
            (start_i64, start_i64)
        };

        Ok((first, second))
    }

    fn execute_clear(&self, state: &mut ProgramState) -> Result<(), EvalError> {
        let (first, second) = self.get_memory_range(state)?;
        
        if first < 0 {
            return Err(EvalError::CannotConvertI64ToAddress);
        }

        for i in first..second {
            state.set_i64(i, BigInt::zero())?;
        }
        
        Ok(())
    }

    fn execute_fill(&self, state: &mut ProgramState) -> Result<(), EvalError> {
        let (first, second) = self.get_memory_range(state)?;
        
        if first < 0 {
            return Err(EvalError::CannotConvertI64ToAddress);
        }

        let value: BigInt = state.get(&self.start, false)?;
        
        for i in first..second {
            state.set_i64(i, value.clone())?;
        }
        
        Ok(())
    }

    fn execute_rotate_left(&self, state: &mut ProgramState) -> Result<(), EvalError> {
        let (first, second) = self.get_memory_range(state)?;
        
        if first < 0 {
            return Err(EvalError::CannotConvertI64ToAddress);
        }

        if first >= second {
            return Ok(());
        }

        let leftmost: BigInt = state.get_i64(first)?.clone();
        
        for i in first..(second - 1) {
            let next_value: BigInt = state.get_i64(i + 1)?.clone();
            state.set_i64(i, next_value)?;
        }
        
        state.set_i64(second - 1, leftmost)?;
        
        Ok(())
    }

    fn execute_rotate_right(&self, state: &mut ProgramState) -> Result<(), EvalError> {
        let (first, second) = self.get_memory_range(state)?;
        
        if first < 0 {
            return Err(EvalError::CannotConvertI64ToAddress);
        }

        if first >= second {
            return Ok(());
        }

        let rightmost: BigInt = state.get_i64(second - 1)?.clone();
        
        for i in (first + 1..second).rev() {
            let prev_value: BigInt = state.get_i64(i - 1)?.clone();
            state.set_i64(i, prev_value)?;
        }
        
        state.set_i64(first, rightmost)?;
        
        Ok(())
    }
}

impl Node for NodeMemoryOp {
    fn formatted_instruction(&self) -> String {
        format!("{} {},{}", self.instruction_id, self.start, self.length)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> anyhow::Result<()> {
        match self.instruction_id {
            InstructionId::Clear => self.execute_clear(state)?,
            InstructionId::Fill => self.execute_fill(state)?,
            InstructionId::RotateLeft => self.execute_rotate_left(state)?,
            InstructionId::RotateRight => self.execute_rotate_right(state)?,
            _ => {
                error!("unsupported memory operation: {:?}", self.instruction_id);
                return Err(EvalError::UnsupportedInstruction.into());
            }
        }
        state.increment_step_count()?;
        Ok(())
    }
}
