use std::fmt;
use super::{Instruction,InstructionId,InstructionParameter,ParameterType};
use super::validate_loops::*;
use crate::execute::{BoxNode,RegisterIndex,RegisterValue,Program};
use crate::execute::node_add::*;
use crate::execute::node_binomial::*;
use crate::execute::node_call::*;
use crate::execute::node_compare::*;
use crate::execute::node_divide::*;
use crate::execute::node_divideif::*;
use crate::execute::node_gcd::*;
use crate::execute::node_logarithm::*;
use crate::execute::node_loop::*;
use crate::execute::node_move::*;
use crate::execute::node_modulo::*;
use crate::execute::node_multiply::*;
use crate::execute::node_power::*;
use crate::execute::node_subtract::*;
use crate::execute::node_truncate::*;

#[derive(Debug)]
pub enum CreateInstructionErrorType {
    ExpectZeroParameters,
    ExpectOneParameter,
    ExpectTwoParameters,
    ParameterMustBeRegister,
    ParameterMustBeConstant,
    ConstantMustBeNonNegative,
    RegisterIndexMustBeNonNegative,
    RegisterIndexTooHigh,
}

#[derive(Debug)]
pub struct CreateInstructionError {
    line_number: usize,
    error_type: CreateInstructionErrorType,
}

impl fmt::Display for CreateInstructionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} in line {}", self.error_type, self.line_number)
    }
}

impl Instruction {
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

    fn expect_one_parameter(&self) -> Result<(), CreateInstructionError> {
        if self.parameter_vec.len() != 1 {
            let err = CreateInstructionError {
                line_number: self.line_number,
                error_type: CreateInstructionErrorType::ExpectOneParameter,
            };
            return Err(err);
        }
        Ok(())
    }

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

    fn create_node_with_register_and_constant(&self, target: RegisterIndex, source: RegisterValue) -> BoxNode {
        match &self.instruction_id {
            InstructionId::Move => {
                let node = NodeMoveConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Add => {
                let node = NodeAddConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Subtract => {
                let node = NodeSubtractConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Power => {
                let node = NodePowerConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Multiply => {
                let node = NodeMultiplyConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Divide => {
                let node = NodeDivideConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::DivideIf => {
                let node = NodeDivideIfConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Modulo => {
                let node = NodeModuloConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::GCD => {
                let node = NodeGCDConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Truncate => {
                let node = NodeTruncateConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Binomial => {
                let node = NodeBinomialConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Compare => {
                let node = NodeCompareConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            InstructionId::Logarithm => {
                let node = NodeLogarithmConstant::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            _ => {
                panic!("unhandled");
            }
        }
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
            InstructionId::Logarithm => {
                let node = NodeLogarithmRegister::new(target, source);
                let node_wrapped = Box::new(node);
                return node_wrapped;
            },
            _ => {
                panic!("unhandled");
            }
        }
    }
}

fn create_two_parameter_node(instruction: &Instruction) -> Result<BoxNode, CreateInstructionError> {
    instruction.expect_two_parameters()?;

    let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();
    if parameter0.parameter_type != ParameterType::Register {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::ParameterMustBeRegister,
        };
        return Err(err);
    }
    if parameter0.parameter_value < 0 {
        if parameter0.parameter_value < 0 {
            let err = CreateInstructionError {
                line_number: instruction.line_number,
                error_type: CreateInstructionErrorType::RegisterIndexMustBeNonNegative,
            };
            return Err(err);
        }
    }
    if parameter0.parameter_value > 255 {
        if parameter0.parameter_value > 255 {
            let err = CreateInstructionError {
                line_number: instruction.line_number,
                error_type: CreateInstructionErrorType::RegisterIndexTooHigh,
            };
            return Err(err);
        }
    }
    let value0: u8 = parameter0.parameter_value as u8;

    let parameter1: &InstructionParameter = instruction.parameter_vec.last().unwrap();
    match parameter1.parameter_type {
        ParameterType::Constant => {
            let node_wrapped = instruction.create_node_with_register_and_constant(
                RegisterIndex(value0),
                RegisterValue::from_i64(parameter1.parameter_value)
            );
            return Ok(node_wrapped);
        },
        ParameterType::Register => {
            if parameter1.parameter_value < 0 {
                let err = CreateInstructionError {
                    line_number: instruction.line_number,
                    error_type: CreateInstructionErrorType::RegisterIndexMustBeNonNegative,
                };
                return Err(err);
            }
            if parameter1.parameter_value > 255 {
                let err = CreateInstructionError {
                    line_number: instruction.line_number,
                    error_type: CreateInstructionErrorType::RegisterIndexTooHigh,
                };
                return Err(err);
            }
            let value1: u8 = parameter1.parameter_value as u8;
            let node_wrapped = instruction.create_node_with_register_and_register(
                RegisterIndex(value0),
                RegisterIndex(value1),
            );
            return Ok(node_wrapped);
        }
    }
}

fn create_call_node(instruction: &Instruction) -> Result<BoxNode, CreateInstructionError> {
    instruction.expect_two_parameters()?;

    let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();
    if parameter0.parameter_type != ParameterType::Register {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::ParameterMustBeRegister,
        };
        return Err(err);
    }
    if parameter0.parameter_value < 0 {
        if parameter0.parameter_value < 0 {
            let err = CreateInstructionError {
                line_number: instruction.line_number,
                error_type: CreateInstructionErrorType::RegisterIndexMustBeNonNegative,
            };
            return Err(err);
        }
    }
    if parameter0.parameter_value > 255 {
        if parameter0.parameter_value > 255 {
            let err = CreateInstructionError {
                line_number: instruction.line_number,
                error_type: CreateInstructionErrorType::RegisterIndexTooHigh,
            };
            return Err(err);
        }
    }
    let value0: u8 = parameter0.parameter_value as u8;

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
        RegisterIndex(value0),
        program_id,
    );
    let node_wrapped = Box::new(node);
    Ok(node_wrapped)
}

struct LoopScope {
    register: RegisterIndex
}

fn process_loopbegin(instruction: &Instruction) -> Result<LoopScope, CreateInstructionError> {
    instruction.expect_one_parameter()?;
    // IDEA: support two or more parameters. How does this work in LODA?

    let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();
    if parameter0.parameter_type != ParameterType::Register {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::ParameterMustBeRegister,
        };
        return Err(err);
    }
    if parameter0.parameter_value < 0 {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::RegisterIndexMustBeNonNegative,
        };
        return Err(err);
    }
    if parameter0.parameter_value > 255 {
        let err = CreateInstructionError {
            line_number: instruction.line_number,
            error_type: CreateInstructionErrorType::RegisterIndexTooHigh,
        };
        return Err(err);
    }
    let value0: u8 = parameter0.parameter_value as u8;

    let ls = LoopScope {
        register: RegisterIndex(value0),
    };
    Ok(ls)
}

pub struct CreatedProgram {
    pub program: Program,
}

#[derive(Debug)]
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
                program.push(NodeLoopRegister::new(loop_register, program_child));
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
            InstructionId::Logarithm => {
                let node = create_two_parameter_node(&instruction)?;
                program.push_boxed(node);
            },
            InstructionId::Call => {
                let node = create_call_node(&instruction)?;
                program.push_boxed(node);
            },
        }
    }

    // TODO: determine how many registers to allocate
    let created_program = CreatedProgram {
        program: program,
    };
    Ok(created_program)
}
