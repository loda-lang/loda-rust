use super::{EvalError, ProgramCache, Node, ProgramState};
use super::{Semantics, SemanticsWithoutLimits, SemanticsWithSmallLimits};
use crate::parser::{InstructionId, InstructionParameter};
use num_bigint::BigInt;

#[derive(Clone, Copy, Debug)]
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
                return self.calc_with_semantics::<SemanticsWithoutLimits>(target, source)
            },
            NodeCalcSemanticMode::SmallLimits => {
                return self.calc_with_semantics::<SemanticsWithSmallLimits>(target, source)
            },
        }
    }

    fn calc_with_semantics<S: Semantics>(&self, target: &BigInt, source: &BigInt) -> Result<BigInt, EvalError> {
        match self.instruction_id {
            InstructionId::Move            => Ok(source.clone()),
            InstructionId::Add             => S::add(target, source),
            InstructionId::Subtract        => S::subtract(target, source),
            InstructionId::Truncate        => S::truncate(target, source),
            InstructionId::Multiply        => S::multiply(target, source),
            InstructionId::Divide          => S::divide(target, source),
            InstructionId::DivideIf        => S::divide_if(target, source),
            InstructionId::DivideIfRepeat  => S::divide_if_repeat(target, source),
            InstructionId::Modulo          => S::modulo(target, source),
            InstructionId::Power           => S::power(target, source),
            InstructionId::GCD             => S::gcd(target, source),
            InstructionId::Binomial        => S::binomial(target, source),
            InstructionId::Compare         => S::compare(target, source),
            InstructionId::Min             => S::min(target, source),
            InstructionId::Max             => S::max(target, source),
            InstructionId::Logarithm       => S::logarithm(target, source),
            InstructionId::NthRoot         => S::nthroot(target, source),
            InstructionId::DigitSum        => S::digitsum(target, source),
            InstructionId::DigitalRoot     => S::digitalroot(target, source),
            InstructionId::Equal           => S::equal(target, source),
            InstructionId::NotEqual        => S::notequal(target, source),
            InstructionId::LessOrEqual     => S::lessorequal(target, source),
            InstructionId::GreaterOrEqual  => S::greaterorequal(target, source),
            InstructionId::BitwiseAnd      => S::bitwiseand(target, source),
            InstructionId::BitwiseOr       => S::bitwiseor(target, source),
            InstructionId::BitwiseXor      => S::bitwisexor(target, source),
            InstructionId::LargestExponent => S::largestexponent(target, source),
            _ => {
                error!("unsupported instruction: {:?}", self.instruction_id);
                return Err(EvalError::UnsupportedInstruction);
            }            
        }
    }
}

impl Node for NodeCalc {
    fn formatted_instruction(&self) -> String {
        format!("{} {},{}", self.instruction_id, self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> anyhow::Result<()> {
        let target: BigInt = state.get(&self.target, false)?;
        let source: BigInt = state.get(&self.source, false)?;
        let value: BigInt = self.calc(&target, &source)?;
        state.set(&self.target, value)?;
        state.increment_step_count()?;
        Ok(())
    }
}
