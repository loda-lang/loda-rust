use super::{EvalError, ProgramSerializerContext, ProgramCache, Node, Program, ProgramId, ProgramState, ProgramRunner, ProgramRunnerManager, ValidateCallError, UnofficialFunction};
use super::PerformCheckValue;
use crate::parser::InstructionParameter;
use num_bigint::BigInt;
use std::rc::Rc;
use std::sync::Arc;

pub struct NodeUnofficialFunction {
    input_count: u8, 
    output_count: u8,
    target: InstructionParameter,
    function_id: u32,
    program_runner_rc: Rc::<ProgramRunner>,
    link_established: bool,
    unofficial_function: Arc<Box<dyn UnofficialFunction>>,
}

impl NodeUnofficialFunction {
    pub fn new(input_count: u8, output_count: u8, target: InstructionParameter, function_id: u32, unofficial_function: Arc<Box<dyn UnofficialFunction>>) -> Self {
        let dummy_program = Program::new();
        let program_runner = ProgramRunner::new(
            ProgramId::ProgramWithoutId,
            dummy_program
        );
        let program_runner_rc = Rc::new(program_runner);

        Self {
            input_count,
            output_count,
            target: target,
            function_id,
            program_runner_rc: program_runner_rc,
            link_established: false,
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
        if !self.link_established { // TODO: get rid of this NodeSeq remain
            panic!("No link have been establish. This node cannot do its job.");
        }
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

    fn update_call(&mut self, program_manager: &mut ProgramRunnerManager) {
        if self.link_established { // TODO: get rid of this NodeSeq remain
            panic!("The link have already been establish. Double assigning a link should not happen.");
        }
        let program_id: u64 = self.function_id as u64;

        let program_runner: Rc::<ProgramRunner> = match program_manager.get(program_id) {
            Some(value) => value,
            None => {
                panic!("NodeCall. Unable to get program_id: {}", program_id);
            }
        };

        self.program_runner_rc = program_runner;
        self.link_established = true;
        //trace!("NodeCall: update_call. program_id: {}", program_id);
    }

    fn accumulate_call_dependencies(&self, program_id_vec: &mut Vec<u64>) {
        program_id_vec.push(self.function_id as u64); // TODO: get rid of this NodeSeq remain
    }

    fn validate_call_nodes(&self) -> Result<(), ValidateCallError> {
        if !self.link_established { // TODO: get rid of this NodeSeq remain
            // There is no connection with the program that we depend on.
            // Without the working dependency, this node cannot do its job correctly.
            return Err(ValidateCallError {});
        }
        Ok(())
    }
}
