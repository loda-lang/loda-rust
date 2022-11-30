use super::{Instruction, InstructionId, InstructionParameter, ParameterType};
use super::validate_loops::*;
use crate::execute::{BoxNode, RegisterIndex, RegisterIndexAndType, RegisterType, Program, LOOP_RANGE_MAX_BITS, NodeUnofficialFunction};
use crate::execute::node_calc::*;
use crate::execute::node_clear::*;
use crate::execute::node_loop_constant::*;
use crate::execute::node_loop_register::*;
use crate::execute::node_loop_simple::*;
use crate::execute::node_loop_slow::*;
use crate::execute::node_seq::*;
use crate::execute::compiletime_error::*;
use crate::execute::UnofficialFunctionRegistry;

impl Instruction {
    /// Loop end (lpe) takes zero parameters.
    fn expect_zero_parameters(&self) -> Result<(), CreateInstructionError> {
        if self.parameter_vec.len() != 0 {
            let err = CreateInstructionError::new(
                self.line_number,
                CreateInstructionErrorType::ExpectZeroParameters,
            );
            return Err(err);
        }
        Ok(())
    }

    /// Loop begin (lpb) takes a required parameter and a 2nd optional parameter.
    fn expect_one_or_two_parameters(&self) -> Result<(), CreateInstructionError> {
        let len = self.parameter_vec.len();
        if len < 1 || len > 2 {
            let err = CreateInstructionError::new(
                self.line_number,
                CreateInstructionErrorType::ExpectOneOrTwoParameters,
            );
            return Err(err);
        }
        Ok(())
    }

    /// The instruction `add $1,1` takes 2 parameters.
    fn expect_two_parameters(&self) -> Result<(), CreateInstructionError> {
        if self.parameter_vec.len() != 2 {
            let err = CreateInstructionError::new(
                self.line_number,
                CreateInstructionErrorType::ExpectTwoParameters,
            );
            return Err(err);
        }
        Ok(())
    }
}

fn create_node_clear(instruction: &Instruction) -> Result<BoxNode, CreateInstructionError> {
    instruction.expect_two_parameters()?;
    let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();
    let parameter1: &InstructionParameter = instruction.parameter_vec.last().unwrap();
    let node = NodeClear::new(parameter0.clone(), parameter1.clone());
    let node_wrapped = Box::new(node);
    return Ok(node_wrapped);
}

fn create_node_seq(instruction: &Instruction) -> Result<BoxNode, CreateInstructionError> {
    instruction.expect_two_parameters()?;

    let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();

    // Checks that parameter0 is good.
    // Bail out if parameter0 is ParameterType::Constant.
    // Bail out if parameter0 is a negative value.
    let _register0 = RegisterIndexAndType::from_parameter(instruction, parameter0)?;

    let parameter1: &InstructionParameter = instruction.parameter_vec.last().unwrap();
    if parameter1.parameter_type != ParameterType::Constant {
        let err = CreateInstructionError::new(
            instruction.line_number,
            CreateInstructionErrorType::ParameterMustBeConstant,
        );
        return Err(err);
    }
    if parameter1.parameter_value < 0 {
        let err = CreateInstructionError::new(
            instruction.line_number,
            CreateInstructionErrorType::ConstantMustBeNonNegative,
        );
        return Err(err);
    }
    let program_id = parameter1.parameter_value as u64;

    let node = NodeSeq::new(
        parameter0.clone(),
        program_id,
    );
    let node_wrapped = Box::new(node);
    Ok(node_wrapped)
}

fn create_node_unofficial_function(
    instruction: &Instruction, 
    input_count: u8, 
    output_count: u8,
    unofficial_function_registry: &UnofficialFunctionRegistry
) -> Result<BoxNode, CreateInstructionError> {
    // Check the instruction `fxx` has input count in the range [1..9]
    if input_count < 1 || input_count > 9 {
        let err = CreateInstructionError::new(
            instruction.line_number,
            CreateInstructionErrorType::UnofficialFunctionInvalidInputOutputCount,
        );
        return Err(err);
    }

    // Check the instruction `fxx` has output count in the range [1..9]
    if output_count < 1 || output_count > 9 {
        let err = CreateInstructionError::new(
            instruction.line_number,
            CreateInstructionErrorType::UnofficialFunctionInvalidInputOutputCount,
        );
        return Err(err);
    }

    instruction.expect_two_parameters()?;
    let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();

    // Checks that parameter0 is good.
    // Bail out if parameter0 is ParameterType::Constant.
    // Bail out if parameter0 is a negative value.
    let _register0 = RegisterIndexAndType::from_parameter(instruction, parameter0)?;

    let parameter1: &InstructionParameter = instruction.parameter_vec.last().unwrap();
    if parameter1.parameter_type != ParameterType::Constant {
        let err = CreateInstructionError::new(
            instruction.line_number,
            CreateInstructionErrorType::ParameterMustBeConstant,
        );
        return Err(err);
    }
    if parameter1.parameter_value < 0 {
        let err = CreateInstructionError::new(
            instruction.line_number,
            CreateInstructionErrorType::ConstantMustBeNonNegative,
        );
        return Err(err);
    }
    let function_id = parameter1.parameter_value as u64;

    match unofficial_function_registry.execute() {
        Ok(value) => {
            println!("!!!!!!!! create_node_unofficial_function.execute result: {}", value);
        },
        Err(error) => {
            error!("create_node_unofficial_function.execute error: {:?}", error);
        }
    }

    // TODO: use NodeUnofficialFunction
    let node = NodeUnofficialFunction::new(
        input_count,
        output_count,
        parameter0.clone(),
        function_id,
    );
    let node_wrapped = Box::new(node);
    Ok(node_wrapped)
}

enum LoopType {
    /// When dealing with `ParameterType::Indirect` that are non-trivial to optimize.
    /// It's slow since nothing can be assumed about the `target` register and `source` register.
    Slow { instruction: Instruction },

    /// Optimized where `target` is `ParameterType::Direct` and `source` always has a `range_length=1`.
    /// This is the most popular type of loop.
    Simple,

    /// Optimized where `target` is `ParameterType::Direct` and `source` is `ParameterType::Constant`.
    /// Popularity is medium.
    RangeLengthWithConstant(u64),

    /// Optimized where `target` is `ParameterType::Direct` and `source` is `ParameterType::Direct`.
    /// Popularity is low.
    RangeLengthFromRegister(RegisterIndex),
}

fn node_loop_range_parameter_constant(instruction: &Instruction, parameter: &InstructionParameter) -> Result<LoopType, CreateInstructionError> {
    if parameter.parameter_value < 0 {
        let err = CreateInstructionError::new(
            instruction.line_number,
            CreateInstructionErrorType::ConstantMustBeNonNegative,
        );
        return Err(err);
    }
    let range_length: u64 = parameter.parameter_value as u64;
    if range_length >= (2 ^ LOOP_RANGE_MAX_BITS) {
        let err = CreateInstructionError::new(
            instruction.line_number,
            CreateInstructionErrorType::LoopWithConstantRangeIsTooHigh,
        );
        return Err(err);
    }
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
    let register = RegisterIndexAndType::from_parameter(instruction, parameter)?;
    let register_index: RegisterIndex = register.register_index;
    let loop_type = LoopType::RangeLengthFromRegister(register_index);
    return Ok(loop_type);
}

fn node_loop_range_parameter(instruction: &Instruction, parameter: &InstructionParameter) -> Result<LoopType, CreateInstructionError> {
    match parameter.parameter_type {
        ParameterType::Constant => {
            return node_loop_range_parameter_constant(instruction, parameter);
        },
        ParameterType::Direct => {
            return node_loop_range_parameter_register(instruction, parameter);
        },
        ParameterType::Indirect => {
            let loop_type = LoopType::Slow { instruction: instruction.clone() };
            return Ok(loop_type);
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
    let register0 = RegisterIndexAndType::from_parameter(instruction, parameter0)?;
    let register_index0 = register0.register_index;

    if register0.register_type == RegisterType::Indirect {
        let ls = LoopScope {
            register: register_index0,
            loop_type: LoopType::Slow { instruction: instruction.clone() },
        };
        return Ok(ls)
    }

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

pub struct CreateProgram {
    node_calc_semantic_mode: NodeCalcSemanticMode,
}

impl CreateProgram {
    pub fn new(node_calc_semantic_mode: NodeCalcSemanticMode) -> Self {
        Self {
            node_calc_semantic_mode: node_calc_semantic_mode,
        }
    }

    pub fn create_program(&self, instruction_vec: &Vec<Instruction>, unofficial_function_registry: &UnofficialFunctionRegistry) -> Result<Program, CreateProgramError> {
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
                        LoopType::Slow { instruction } => {
                            let node = NodeLoopSlow::new(instruction, program_child)?;
                            program.push(node);
                        },
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
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Add => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Subtract => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Power => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Multiply => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Divide => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::DivideIf => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Modulo => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::GCD => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Truncate => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Binomial => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Compare => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Clear => {
                    let node = create_node_clear(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Max => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::Min => {
                    let node = self.create_node_calc(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::EvalSequence => {
                    let node = create_node_seq(&instruction)?;
                    program.push_boxed(node);
                },
                InstructionId::UnofficialFunction { input_count, output_count } => {
                    let node = create_node_unofficial_function(
                        &instruction, 
                        input_count, 
                        output_count,
                        unofficial_function_registry,
                    )?;
                    program.push_boxed(node);
                },
            }
        }
    
        Ok(program)
    }    

    fn create_node_calc(&self, instruction: &Instruction) -> Result<BoxNode, CreateInstructionError> {
        instruction.expect_two_parameters()?;
        let parameter0: &InstructionParameter = instruction.parameter_vec.first().unwrap();
        let parameter1: &InstructionParameter = instruction.parameter_vec.last().unwrap();
        let node = NodeCalc::new(
            self.node_calc_semantic_mode, 
            instruction.instruction_id.clone(), 
            parameter0.clone(), 
            parameter1.clone()
        );
        let node_wrapped = Box::new(node);
        return Ok(node_wrapped);
    }
}
