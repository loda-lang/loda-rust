use crate::lodacpp::{LodaCpp, LodaCppEvalStepsExecute, LodaCppEvalSteps};
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
use std::fs::File;
use std::io::Write;

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

    pub fn compare(
        &self, 
        lodacpp: &LodaCpp, 
        path_program0: &Path, 
        path_program1: &Path, 
        path_comparison: &Path, 
        time_limit: Duration, 
        term_count: usize
    ) -> Result<CompareTwoProgramsResult, Box<dyn Error>> {
        let mut file = File::create(path_comparison)?;

        writeln!(&mut file, "program0, measuring steps: {:?}", path_program0);
        let start0 = Instant::now();
        let result0 = lodacpp.eval_steps(
            term_count,
            &path_program0, 
            time_limit
        );
        let elapsed0: u128 = start0.elapsed().as_millis();
        let steps0: LodaCppEvalSteps = match result0 {
            Ok(value) => {
                // debug!("program0 steps:\n{:?}", value.steps());
                writeln!(&mut file, "program0, elapsed: {:?}ms", elapsed0);
                writeln!(&mut file, "program0, steps\n{:?}", value.steps());
                value
            },
            Err(error) => {
                // error!("Unable to compute steps for program0: {:?}", error);
                writeln!(&mut file, "Unable to compute steps for program0: {:?}", error);
                return Ok(CompareTwoProgramsResult::Program1);
            }
        };
        
        writeln!(&mut file, "\n\nprogram1, measuring steps: {:?}", path_program1);
        let start1 = Instant::now();
        let result1 = lodacpp.eval_steps(
            term_count,
            &path_program1, 
            time_limit
        );
        let elapsed1: u128 = start1.elapsed().as_millis();
        let steps1: LodaCppEvalSteps = match result1 {
            Ok(value) => {
                // debug!("program1 steps:\n{:?}", value.steps());
                writeln!(&mut file, "program1, elapsed: {:?}ms", elapsed1);
                writeln!(&mut file, "program1, steps\n{:?}", value.steps());
                value
            },
            Err(error) => {
                // debug!("Unable to compute steps for program1: {:?}", error);
                writeln!(&mut file, "Unable to compute steps for program1: {:?}", error);
                return Ok(CompareTwoProgramsResult::Program0);
            }
        };

        write!(&mut file, "\n\nComparing terms");

        Ok(CompareTwoProgramsResult::Program0)
    }
}
