use super::{EvalError, ProgramSerializerContext, ProgramCache, Node, RegisterValue, Program, ProgramId, ProgramState, ProgramRunner, ProgramRunnerManager, ValidateCallError, UnofficialFunction};
use super::PerformCheckValue;
use crate::parser::InstructionParameter;
use num_bigint::BigInt;
use num_traits::Signed;
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

    fn eval(&self, state: &mut ProgramState, cache: &mut ProgramCache) -> Result<(), EvalError> {
        if !self.link_established {
            panic!("No link have been establish. This node cannot do its job.");
        }
        let input: BigInt = state.get(&self.target, false)?;

        match self.unofficial_function.execute() {
            Ok(value) => {
                println!("NodeUnofficialFunction.eval execute result: {}", value);
            },
            Err(error) => {
                error!("NodeUnofficialFunction.eval execute error: {:?}", error);
            }
        }
    
        if input.is_negative() {
            // Prevent calling other programs with a negative parameter.
            // It's fragile allowing negative values.
            // Example: If program A depends on program B. 
            // Some day program B gets changed, and it breaks program A,
            // because negative input values was being used.
            return Err(EvalError::EvalSequenceWithNegativeParameter);
        }

        // Abort if the input value is beyond the limit (optional)
        state.check_value().input(&input)?;

        let step_count_limit: u64 = state.step_count_limit();
        let mut step_count: u64 = state.step_count();

        // Invoke the actual run() function
        let input_value = RegisterValue(input.clone());
        let run_result = self.program_runner_rc.run(
            &input_value, 
            state.run_mode(), 
            &mut step_count, 
            step_count_limit,
            state.node_register_limit().clone(),
            state.node_loop_limit().clone(),
            cache,
        );

        // Update statistics, no matter if run succeeded or failed
        state.set_step_count(step_count);

        let output: RegisterValue = match run_result {
            Ok(value) => value,
            Err(error) => {
                // In case run failed, then return the error
                return Err(error);
            }
        };

        // Abort if the output value is beyond the limit (optional)
        state.check_value().output(&output.0)?;

        // In case run succeeded, then pass on the outputted value.
        state.set(&self.target, output.0)?;
        Ok(())
    }

    fn update_call(&mut self, program_manager: &mut ProgramRunnerManager) {
        if self.link_established {
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
        program_id_vec.push(self.function_id as u64);
    }

    fn validate_call_nodes(&self) -> Result<(), ValidateCallError> {
        if !self.link_established {
            // There is no connection with the program that we depend on.
            // Without the working dependency, this node cannot do its job correctly.
            return Err(ValidateCallError {});
        }
        Ok(())
    }
}
