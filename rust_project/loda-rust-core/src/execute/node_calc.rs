use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex};
use super::{Semantics, SemanticsWithoutLimits, SemanticsWithSmallLimits};
use crate::parser::{InstructionId, InstructionParameter, ParameterType};
use std::collections::HashSet;
use std::convert::TryFrom;
use num_bigint::BigInt;

pub enum NodeCalcSemanticMode {
    Unlimited,
    SmallLimits,
}

pub struct NodeCalc {
    semantic_mode: NodeCalcSemanticMode,
    instruction_id: InstructionId,
    target: InstructionParameter,
    source: InstructionParameter,
}

impl NodeCalc {
    pub fn new(semantic_mode: NodeCalcSemanticMode, instruction_id: InstructionId, target: InstructionParameter, source: InstructionParameter) -> Self {
        Self {
            semantic_mode: semantic_mode,
            instruction_id: instruction_id,
            target: target,
            source: source,
        }
    }

    fn calc(&self, target: &BigInt, source: &BigInt) -> Result<BigInt, EvalError> {
        match self.semantic_mode {
            NodeCalcSemanticMode::Unlimited => {
                let semantics = SemanticsWithoutLimits {};
                return self.calc_inner(target, source, &semantics)
            },
            NodeCalcSemanticMode::SmallLimits => {
                let semantics = SemanticsWithSmallLimits {};
                return self.calc_inner(target, source, &semantics)
            },
        }
    }

    fn calc_inner<S: Semantics>(&self, target: &BigInt, source: &BigInt, semantics: &S) -> Result<BigInt, EvalError> {
        match self.instruction_id {
            InstructionId::Move     => Ok(source.clone()),
            InstructionId::Add      => semantics.add(target, source),
            InstructionId::Subtract => semantics.subtract(target, source),
            InstructionId::Truncate => semantics.truncate(target, source),
            InstructionId::Multiply => semantics.multiply(target, source),
            InstructionId::Divide   => semantics.divide(target, source),
            InstructionId::DivideIf => semantics.divide_if(target, source),
            InstructionId::Modulo   => semantics.modulo(target, source),
            InstructionId::Power    => semantics.power(target, source),
            InstructionId::GCD      => semantics.gcd(target, source),
            InstructionId::Binomial => semantics.binomial(target, source),
            InstructionId::Compare  => semantics.compare(target, source),
            InstructionId::Min      => semantics.min(target, source),
            InstructionId::Max      => semantics.max(target, source),
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
        match self.target.parameter_type {
            ParameterType::Direct | ParameterType::Indirect => {
                let value: u64 = u64::try_from(self.target.parameter_value).unwrap_or(255);
                register_vec.push(RegisterIndex(value));
            },
            ParameterType::Constant => {}
        }
        match self.source.parameter_type {
            ParameterType::Direct | ParameterType::Indirect => {
                let value: u64 = u64::try_from(self.target.parameter_value).unwrap_or(255);
                register_vec.push(RegisterIndex(value));
            },
            ParameterType::Constant => {}
        }
    }
    
    fn live_register_indexes(&self, _register_set: &mut HashSet<RegisterIndex>) {
        // TODO: deal with indirect
        // if register_set.contains(&self.source.register_index) {
        //     register_set.insert(self.target.register_index.clone());
        // } else {
        //     // Overwrite content of the target register a non-live register.
        //     register_set.remove(&self.target.register_index);
        // }
    }
}
