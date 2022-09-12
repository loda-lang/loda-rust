use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterIndexAndType, RegisterType, RegisterValue};
use super::{BoxCheckValue, PerformCheckValue};
use super::Semantics;
use crate::parser::{Instruction, InstructionId, InstructionParameter, ParameterType};
use std::collections::HashSet;
use num_bigint::BigInt;
use num_traits::Zero;

pub struct NodeCalc {
    instruction_id: InstructionId,
    target: InstructionParameter,
    source: InstructionParameter,
}

impl NodeCalc {
    pub fn new(instruction_id: InstructionId, target: InstructionParameter, source: InstructionParameter) -> Self {
        Self {
            instruction_id: instruction_id,
            target: target,
            source: source,
        }
    }

    fn calc(&self, target: &BigInt, source: &BigInt) -> Result<BigInt, EvalError> {
        match self.instruction_id {
            InstructionId::Move => Semantics::move_value(target, source),
            InstructionId::Add => Semantics::add(target, source),
            InstructionId::Subtract => Semantics::subtract(target, source),
            InstructionId::Truncate => Semantics::truncate(target, source),
            InstructionId::Multiply => Semantics::multiply(target, source),
            InstructionId::Divide => Semantics::divide(target, source),
            InstructionId::DivideIf => Semantics::divide_if(target, source),
            InstructionId::Modulo => Semantics::modulo(target, source),
            _ => {
                error!("unsupported instruction: {:?}", self.instruction_id);
                return Err(EvalError::UnsupportedInstruction);
            }            
        }
    }
}

impl Node for NodeCalc {
    fn formatted_instruction(&self) -> String {
        format!("{} {},{}", self.instruction_id.shortname(), self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let target: BigInt = state.get(&self.target, false)?;
        let source: BigInt = state.get(&self.source, false)?;
        let value: BigInt = self.calc(&target, &source)?;
        state.set(&self.target, value)?;
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        // TODO: deal with indirect
        // register_vec.push(self.target.register_index.clone());
        // register_vec.push(self.source.register_index.clone());
        for i in 0..=254 {
            register_vec.push(RegisterIndex(i));
        }
    }
    
    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        // TODO: deal with indirect
        // if register_set.contains(&self.source.register_index) {
        //     register_set.insert(self.target.register_index.clone());
        // } else {
        //     // Overwrite content of the target register a non-live register.
        //     register_set.remove(&self.target.register_index);
        // }
    }
}
