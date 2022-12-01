use super::{EvalError, ProgramSerializerContext, ProgramCache, Node, ProgramState, ProgramRunnerManager, ValidateCallError};
use super::PerformCheckValue;
use crate::parser::InstructionParameter;
use crate::unofficial_function::UnofficialFunction;
use num_bigint::BigInt;
use std::sync::Arc;

pub struct NodeUnofficialFunction {
    input_count: u8, 
    output_count: u8,
    target: InstructionParameter,
    function_id: u32,
    unofficial_function: Arc<Box<dyn UnofficialFunction>>,
}

impl NodeUnofficialFunction {
    pub fn new(input_count: u8, output_count: u8, target: InstructionParameter, function_id: u32, unofficial_function: Arc<Box<dyn UnofficialFunction>>) -> Self {
        Self {
            input_count,
            output_count,
            target: target,
            function_id,
            unofficial_function: unofficial_function,
        }
    }
}

impl Node for NodeUnofficialFunction {
    fn formatted_instruction(&self) -> String {
        format!("f{}{} {},{}", self.input_count, self.output_count, self.target, self.function_id)
    }

    fn formatted_instruction_advanced(&self, _context: &dyn ProgramSerializerContext) -> Option<String> {
        let name: String = self.unofficial_function.name().to_string();
        let s = format!("f{}{} {},{} ; {}", self.input_count, self.output_count, self.target, self.function_id, name);
        Some(s)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let input: BigInt = state.get(&self.target, false)?;

        // TODO: deal with indirect memory
        let start_address: i64 = self.target.parameter_value;
        if start_address < 0 {
            return Err(EvalError::CannotConvertI64ToAddress);
        }

        // Input to the function
        let mut input_vec: Vec<BigInt> = vec!();
        for i in 0..self.input_count {
            let address = (start_address + (i as i64)) as u64;
            let value: &BigInt = state.get_u64(address);

            // Abort if the input value is beyond the limit (optional)
            state.check_value().input(&input)?;

            input_vec.push(value.clone());
        }

        // Run the function
        let run_result = self.unofficial_function.run(input_vec);
        let output_vec: Vec<BigInt> = match run_result {
            Ok(value) => value,
            Err(error) => {
                error!("NodeUnofficialFunction.eval run error: {:?}", error);
                // TODO: fail with a better error
                return Err(EvalError::DivisionByZero);
            }
        };

        // Output from the function
        if output_vec.len() != (self.output_count as usize) {
            error!("NodeUnofficialFunction.eval Expected {} output values, but got output {}", self.output_count, output_vec.len());
            // TODO: fail with a better error
            return Err(EvalError::DivisionByZero);
        }

        for (index, value) in output_vec.iter().enumerate() {
            let address = (start_address + (index as i64)) as u64;

            // Abort if the output value is beyond the limit (optional)
            state.check_value().output(value)?;

            match state.set_u64(address, value.clone()) {
                Ok(()) => {},
                Err(error) => {
                    error!("NodeUnofficialFunction.eval Cannot set output value into state. address: {}, error: {}", address, error);
                    // TODO: fail with a better error
                    return Err(EvalError::DivisionByZero);
                }
            }
        }

        // TODO: update step counter
    
        Ok(())
    }

    fn update_call(&mut self, _program_manager: &mut ProgramRunnerManager) {
    }

    fn accumulate_call_dependencies(&self, _program_id_vec: &mut Vec<u64>) {
    }

    fn validate_call_nodes(&self) -> Result<(), ValidateCallError> {
        Ok(())
    }
}
