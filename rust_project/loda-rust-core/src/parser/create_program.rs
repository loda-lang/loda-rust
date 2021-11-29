use std::error::Error;
use std::fmt;
use super::{Instruction, InstructionId, InstructionParameter, ParameterType};
use super::validate_loops::*;
use crate::execute::{BoxNode, RegisterIndex, RegisterValue, Program};
use crate::execute::node_add::*;
use crate::execute::node_binomial::*;
use crate::execute::node_call::*;
use crate::execute::node_clear::*;
use crate::execute::node_compare::*;
use crate::execute::node_divide::*;
use crate::execute::node_divideif::*;
use crate::execute::node_gcd::*;
use crate::execute::node_loop_constant::*;
use crate::execute::node_loop_register::*;
use crate::execute::node_loop_simple::*;
use crate::execute::node_max::*;
use crate::execute::node_min::*;
use crate::execute::node_modulo::*;
use crate::execute::node_move::*;
use crate::execute::node_multiply::*;
use crate::execute::node_power::*;
use crate::execute::node_subtract::*;
use crate::execute::node_truncate::*;

trait NodeCreator {
    fn create_with_register_and_constant(&self, target: RegisterIndex, source: RegisterValue) -> Result<BoxNode, NodeCreatorError>;
}
type BoxNodeCreator = Box<dyn NodeCreator>;

#[derive(Debug)]
pub enum NodeCreatorError {
    MissingClassForInstruction,
    CannotConstructInstance,
}

impl fmt::Display for NodeCreatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingClassForInstruction => 
                write!(f, "The instruction doesn't have a corresponding class"),
            Self::CannotConstructInstance =>
                write!(f, "The instance cannot be constructed"),
        }
    }
}

impl Error for NodeCreatorError {}

struct ClearNodeCreator {
}

impl NodeCreator for ClearNodeCreator {
    fn create_with_register_and_constant(&self, target: RegisterIndex, source: RegisterValue) -> Result<BoxNode, NodeCreatorError> {
        let node = match NodeClearConstant::create(target, source) {
            Ok(value) => value,
            Err(create_error) => {
                error!("Cannot construct instance: {:?}", create_error);
                return Err(NodeCreatorError::CannotConstructInstance);
            }
        };
        Ok(Box::new(node))
    }
}

struct MoveNodeCreator {
}

impl NodeCreator for MoveNodeCreator {
    fn create_with_register_and_constant(&self, target: RegisterIndex, source: RegisterValue) -> Result<BoxNode, NodeCreatorError> {
        let node = NodeMoveConstant::new(target, source);
        Ok(Box::new(node))
    }
}

fn node_creator_from_instruction(instruction: &Instruction) -> Result<BoxNodeCreator, NodeCreatorError> {
    let node_creator: BoxNodeCreator = match instruction.instruction_id {
        InstructionId::Clear => Box::new(ClearNodeCreator {}),
        InstructionId::Move => Box::new(MoveNodeCreator {}),
        _ => {
            return Err(NodeCreatorError::MissingClassForInstruction);
        }
    };
    // node_creator.create_node_with_register_and_constant()
    Ok(node_creator)
}



#[derive(Debug, PartialEq)]
pub enum CreateInstructionErrorType {
    ExpectZeroParameters,
    ExpectOneOrTwoParameters,
    ExpectTwoParameters,
    ParameterMustBeRegister,
    ParameterMustBeConstant,
    ConstantMustBeNonNegative,
    LoopWithConstantRangeIsTooHigh,
    RegisterIndexMustBeNonNegative,
    RegisterIndexTooHigh,
    NodeCreateError,
}

#[derive(Debug, PartialEq)]
pub struct CreateInstructionError {
    line_number: usize,
    error_type: CreateInstructionErrorType,
}

impl fmt::Display for CreateInstructionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} in line {}", self.error_type, self.line_number)
    }
}

fn register_index_from_parameter(instruction: &Instruction, parameter: &InstructionParameter) 
    -> Result<RegisterIndex, CreateInstructionError> 
{
    if parameter.parameter_type != ParameterType::Register {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::ParameterMustBeRegister,
        };
        return Err(err);
    }
    let parameter_value: i64 = parameter.parameter_value;
    if parameter_value < 0 {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::RegisterIndexMustBeNonNegative,
        };
        return Err(err);
    }
    if parameter_value > 255 {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::RegisterIndexTooHigh,
        };
        return Err(err);
    }
    let register_index_value: u8 = parameter_value as u8;
    Ok(RegisterIndex(register_index_value))
}

impl Instruction {
    // Loop end (lpe) takes zero parameters.
    fn expect_zero_parameters(&self) -> Result<(), CreateInstructionError> {
        if self.parameter_vec.len() != 0 {
            let err = CreateInstructionError {
                line_number: self.line_number,
                error_type: CreateInstructionErrorType::ExpectZeroParameters,
            };
            return Err(err);
        }
        Ok(())
    }

    // Loop begin (lpb) takes a required parameter and a 2nd optional parameter.
    fn expect_one_or_two_parameters(&self) -> Result<(), CreateInstructionError> {
        let len = self.parameter_vec.len();
        if len < 1 || len > 2 {
            let err = CreateInstructionError {
                line_number: self.line_number,
                error_type: CreateInstructionErrorType::ExpectOneOrTwoParameters,
            };
            return Err(err);
        }
        Ok(())
    }

    // The instruction `add $1,1` takes 2 parameters.
    fn expect_two_parameters(&self) -> Result<(), CreateInstructionError> {
        if self.parameter_vec.len() != 2 {
            let err = CreateInstructionError {
                line_number: self.line_number,
                error_type: CreateInstructionErrorType::ExpectTwoParameters,
            };
            return Err(err);
        }
        Ok(())
    }

    fn create_node_with_register_and_constant(&self, target: RegisterIndex, source: RegisterValue) -> Result<BoxNode, CreateInstructionError> {
        let boxed_node: BoxNode = match &self.instruction_id {
            InstructionId::Move =>
                Box::new(NodeMoveConstant::new(target, source)),
            InstructionId::Add =>
                Box::new(NodeAddConstant::new(target, source)),
            InstructionId::Subtract =>
                Box::new(NodeSubtractConstant::new(target, source)),
            InstructionId::Power =>
                Box::new(NodePowerConstant::new(target, source)),
            InstructionId::Multiply =>
                Box::new(NodeMultiplyConstant::new(target, source)),
            InstructionId::Divide =>
                Box::new(NodeDivideConstant::new(target, source)),
            InstructionId::DivideIf =>
                Box::new(NodeDivideIfConstant::new(target, source)),
            InstructionId::Modulo =>
                Box::new(NodeModuloConstant::new(target, source)),
            InstructionId::GCD =>
                Box::new(NodeGCDConstant::new(target, source)),
            InstructionId::Truncate =>
                Box::new(NodeTruncateConstant::new(target, source)),
            InstructionId::Binomial =>
                Box::new(NodeBinomialConstant::new(target, source)),
            InstructionId::Compare =>
                Box::new(NodeCompareConstant::new(target, source)),
            InstructionId::Max =>
                Box::new(NodeMaxConstant::new(target, source)),
            InstructionId::Min =>
                Box::new(NodeMinConstant::new(target, source)),
            InstructionId::Clear => {
                let node = match NodeClearConstant::create(target, source) {
                    Ok(value) => value,
                    Err(create_error) => {
                        error!("NodeClearConstant::create() instruction: {:?} returned: {:?}", self, create_error);
                        let err = CreateInstructionError {
                            line_number: self.line_number,
                            error_type: CreateInstructionErrorType::NodeCreateError,
                        };
                        return Err(err);
                    }
                };
                Box::new(node)
            },
            _ => {
                panic!("unhandled instruction: {:?}", self.instruction_id);
            }
        };
        Ok(boxed_node)
    }

    fn create_node_with_register_and_register(&self, target: RegisterIndex, source: RegisterIndex) -> BoxNode {
        match &self.instruction_id {
            InstructionId::Move => {
                let node = NodeMoveRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Add => {
                let node = NodeAddRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Subtract => {
                let node = NodeSubtractRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Power => {
                let node = NodePowerRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Multiply => {
                let node = NodeMultiplyRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Divide => {
                let node = NodeDivideRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::DivideIf => {
                let node = NodeDivideIfRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Modulo => {
                let node = NodeModuloRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::GCD => {
                let node = NodeGCDRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Truncate => {
                let node = NodeTruncateRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Binomial => {
                let node = NodeBinomialRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Compare => {
                let node = NodeCompareRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Max => {
                let node = NodeMaxRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Min => {
                let node = NodeMinRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Clear => {
                let node = NodeClearRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            _ => {
                panic!("unhandled instruction: {:?}", self.instruction_id);
            }
        }
    }
}

fn create_two_parameter_node(instruction: &Instruction) -> Result<BoxNode, CreateInstructionError> {
    instruction.expect_two_parameters()?;

    let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();
    let register_index0 = register_index_from_parameter(instruction, parameter0)?;

    let parameter1: &InstructionParameter = instruction.parameter_vec.last().unwrap();
    match parameter1.parameter_type {
        ParameterType::Constant => {
            return instruction.create_node_with_register_and_constant(
                register_index0,
                RegisterValue::from_i64(parameter1.parameter_value)
            );
        },
        ParameterType::Register => {
            let register_index1 = register_index_from_parameter(instruction, parameter1)?;
            let node_wrapped = instruction.create_node_with_register_and_register(
                register_index0,
                register_index1,
            );
            return Ok(node_wrapped);
        }
    }
}

fn create_call_node(instruction: &Instruction) -> Result<BoxNode, CreateInstructionError> {
    instruction.expect_two_parameters()?;

    let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();
    let register_index0 = register_index_from_parameter(instruction, parameter0)?;

    let parameter1: &InstructionParameter = instruction.parameter_vec.last().unwrap();
    if parameter1.parameter_type != ParameterType::Constant {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::ParameterMustBeConstant,
        };
        return Err(err);
    }
    if parameter1.parameter_value < 0 {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::ConstantMustBeNonNegative,
        };
        return Err(err);
    }
    let program_id = parameter1.parameter_value as u64;

    let node = NodeCallConstant::new(
        register_index0,
        program_id,
    );
    let node_wrapped = Box::new(node);
    Ok(node_wrapped)
}

enum LoopType {
    Simple,
    RangeLengthWithConstant(u8),
    RangeLengthFromRegister(RegisterIndex),
}

fn node_loop_range_parameter_constant(instruction: &Instruction, parameter: &InstructionParameter) -> Result<LoopType, CreateInstructionError> {
    if parameter.parameter_value < 0 {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::ConstantMustBeNonNegative,
        };
        return Err(err);
    }
    if parameter.parameter_value > 255 {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::LoopWithConstantRangeIsTooHigh,
        };
        return Err(err);
    }
    let range_length: u8 = parameter.parameter_value as u8;
    if range_length == 0 {
        debug!("Loop begin with constant=0. Same as a NOP, does nothing.");
    }
    if range_length == 1 {
        debug!("Loop begin with constant=1. This is redundant.");
    }
    let loop_type = LoopType::RangeLengthWithConstant(range_length);
    return Ok(loop_type);
}

fn node_loop_range_parameter_register(instruction: &Instruction, parameter: &InstructionParameter) -> Result<LoopType, CreateInstructionError> {
    let register_index: RegisterIndex = register_index_from_parameter(instruction, parameter)?;
    let loop_type = LoopType::RangeLengthFromRegister(register_index);
    return Ok(loop_type);
}

fn node_loop_range_parameter(instruction: &Instruction, parameter: &InstructionParameter) -> Result<LoopType, CreateInstructionError> {
    match parameter.parameter_type {
        ParameterType::Constant => {
            return node_loop_range_parameter_constant(instruction, parameter);
        },
        ParameterType::Register => {
            return node_loop_range_parameter_register(instruction, parameter);
        }
    }
}

struct LoopScope {
    register: RegisterIndex,
    loop_type: LoopType,
}

fn process_loopbegin(instruction: &Instruction) -> Result<LoopScope, CreateInstructionError> {
    instruction.expect_one_or_two_parameters()?;

    let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();
    let register_index0 = register_index_from_parameter(instruction, parameter0)?;

    let loop_type: LoopType;
    if instruction.parameter_vec.len() == 2 {
        let parameter: &InstructionParameter = instruction.parameter_vec.last().unwrap();
        if parameter.parameter_value > 6 {
            debug!("loop begin with 2nd parameter: {} which is unusual high", parameter.parameter_value);
        }
        if parameter.parameter_value < 1 {
            debug!("loop begin with 2nd parameter: {} which is less than one!", parameter.parameter_value);
        }
        loop_type = node_loop_range_parameter(instruction, parameter)?;
    } else {
        // No 2nd parameter supplied
        loop_type = LoopType::Simple;
    }

    let ls = LoopScope {
        register: register_index0,
        loop_type: loop_type,
    };
    Ok(ls)
}

pub struct CreatedProgram {
    pub program: Program,
}

#[derive(Debug, PartialEq)]
pub enum CreateProgramError {
    ValidateLoops(ValidateLoopError),
    CreateInstruction(CreateInstructionError),
    CannotPopEmptyStack,
}

impl fmt::Display for CreateProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ValidateLoops(ref err) => 
                write!(f, "ValidateLoops error: {}", err),
            Self::CreateInstruction(ref err) => 
                write!(f, "CreateInstruction error: {}", err),
            Self::CannotPopEmptyStack => 
                write!(f, "Mismatch between loop begins and loop ends. stack is empty."),
        }
    }
}

impl From<ValidateLoopError> for CreateProgramError {
    fn from(err: ValidateLoopError) -> CreateProgramError {
        CreateProgramError::ValidateLoops(err)
    }
}

impl From<CreateInstructionError> for CreateProgramError {
    fn from(err: CreateInstructionError) -> CreateProgramError {
        CreateProgramError::CreateInstruction(err)
    }
}


pub fn create_program(instruction_vec: &Vec<Instruction>) -> Result<CreatedProgram, CreateProgramError> {
    validate_loops(instruction_vec)?;

    let mut stack_vec: Vec<(Program, LoopScope)> = vec!();
    let mut program = Program::new();
    for instruction in instruction_vec {
        let id: InstructionId = instruction.instruction_id.clone();
        match id {
            InstructionId::LoopBegin => {
                let loopscope: LoopScope = process_loopbegin(&instruction)?;
                stack_vec.push((program, loopscope));
                program = Program::new();
            },
            InstructionId::LoopEnd => {
                instruction.expect_zero_parameters()?;
                let stack_item: (Program, LoopScope) = match stack_vec.pop() {
                    Some(value) => value,
                    None => {
                        return Err(CreateProgramError::CannotPopEmptyStack);
                    }
                };
                let program_parent: Program = stack_item.0;
                let loopscope: LoopScope = stack_item.1;

                let loop_register: RegisterIndex = loopscope.register;
                let program_child: Program = program;
                program = program_parent;

                match loopscope.loop_type {
                    LoopType::Simple => {
                        program.push(NodeLoopSimple::new(loop_register, program_child));
                    },
                    LoopType::RangeLengthWithConstant(range_length) => {
                        program.push(NodeLoopConstant::new(loop_register, range_length, program_child));
                    },
                    LoopType::RangeLengthFromRegister(register_with_range_length) => {
                        program.push(NodeLoopRegister::new(loop_register, register_with_range_length, program_child));
                    }
                }
            },
            InstructionId::Move => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Add => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Subtract => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Power => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Multiply => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Divide => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::DivideIf => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Modulo => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::GCD => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Truncate => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Binomial => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Compare => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Clear => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Max => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Min => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::EvalSequence => {
                let node = create_call_node(&instruction)?;
                program.push_boxed(node);
            },
        }
    }

    let created_program = CreatedProgram {
        program: program,
    };
    Ok(created_program)
}
