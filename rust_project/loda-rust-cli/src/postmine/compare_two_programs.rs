use crate::lodacpp::{LodaCpp, LodaCppEvalStepsExecute, LodaCppEvalSteps};
use std::error::Error;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum CompareTwoProgramsResult {
    Program0,
    Program1,
}

pub struct CompareTwoPrograms {}

impl CompareTwoPrograms {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compare(&self, lodacpp: &LodaCpp, path_program0: &Path, path_program1: &Path, path_benchmark: &Path, time_limit: Duration, term_count: usize) -> CompareTwoProgramsResult {
        let result0 = lodacpp.eval_steps(
            term_count,
            &path_program0, 
            time_limit
        );
        match result0 {
            Ok(value) => {
                debug!("program0 steps:\n{:?}", value.steps());
            },
            Err(error) => {
                debug!("Unable to compute steps for program0: {:?}", error);
            }
        }
        
        let result1 = lodacpp.eval_steps(
            term_count,
            &path_program1, 
            time_limit
        );
        match result1 {
            Ok(value) => {
                debug!("program1 steps:\n{:?}", value.steps());
            },
            Err(error) => {
                debug!("Unable to compute steps for program1: {:?}", error);
            }
        }
        CompareTwoProgramsResult::Program0
    }
}
