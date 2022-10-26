use loda_rust_core::execute::{EvalError, NodeLoopLimit, ProgramCache, ProgramRunner, RegisterValue, RunMode};
use loda_rust_core::execute::NodeRegisterLimit;
use loda_rust_core::util::BigIntVec;

pub struct TermComputer {
    pub terms: BigIntVec,
    pub steps: Vec<u64>,
    pub step_count: u64,
}

impl TermComputer {
    pub fn new() -> Self {
        Self {
            terms: Vec::with_capacity(40),
            steps: Vec::with_capacity(40),
            step_count: 0,
        }
    }

    pub fn compute(&mut self, cache: &mut ProgramCache, runner: &ProgramRunner, count: usize) -> Result<(), EvalError> {
        let node_register_limit = NodeRegisterLimit::LimitBits(32);
        loop {
            let length: usize = self.terms.len();
            if length >= count {
                break;
            }
            let node_loop_limit: NodeLoopLimit;
            if length <= 10 {
                node_loop_limit = NodeLoopLimit::LimitCount(4000);
            } else {
                if length <= 20 {
                    node_loop_limit = NodeLoopLimit::LimitCount(8000);
                } else {
                    node_loop_limit = NodeLoopLimit::LimitCount(32000);
                }
            }
            let step_count_limit: u64;
            if length <= 10 {
                step_count_limit = 40000;
            } else {
                if length <= 20 {
                    step_count_limit = 80000;
                } else {
                    step_count_limit = 320000;
                }
            }
            let index = length as i64;
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = runner.run(
                &input, 
                RunMode::Silent, 
                &mut self.step_count, 
                step_count_limit, 
                node_register_limit.clone(),
                node_loop_limit.clone(),
                cache
            )?;
            self.terms.push(output.0);
            self.steps.push(self.step_count);
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.terms.clear();
        self.steps.clear();
        self.step_count = 0;
    }
}
