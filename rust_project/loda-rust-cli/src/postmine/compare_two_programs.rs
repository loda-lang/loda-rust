use crate::lodacpp::{LodaCpp, LodaCppEvalStepsExecute, LodaCppEvalSteps};
use crate::common::SimpleLog;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
use std::fs::File;
use std::io::Write;

pub enum StatusOfExistingProgram {
    CompareNewWithExisting,
    NoExistingProgram,
    IgnoreExistingProgram { ignore_reason: String },
}

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
        simple_log: SimpleLog,
        lodacpp: &LodaCpp, 
        path_program0: &Path, 
        path_program1: &Path,
        status_of_existing_program: &StatusOfExistingProgram,
        path_comparison: &Path, 
        time_limit: Duration, 
        term_count: usize
    ) -> anyhow::Result<CompareTwoProgramsResult> {
        if !path_program0.is_file() {
            return Err(anyhow::anyhow!("Expected a file, but got none. path_program0: {:?}", path_program0));
        }

        match status_of_existing_program {
            StatusOfExistingProgram::NoExistingProgram => {
                simple_log.println("compare_two_programs: Keeping. This is a new program. There is no previous implementation.");
                return Ok(CompareTwoProgramsResult::Program0);
            },
            StatusOfExistingProgram::IgnoreExistingProgram { ignore_reason } => {
                simple_log.println(format!("compare_two_programs: Keeping the mined program. There is a problem with the previous implementation: {}", ignore_reason));
                return Ok(CompareTwoProgramsResult::Program0);
            },
            StatusOfExistingProgram::CompareNewWithExisting => {
                simple_log.println("compare_two_programs: Comparing new program with existing program, and choosing the best.");
                return self.compare_new_with_existing(
                    lodacpp, 
                    path_program0, 
                    path_program1, 
                    path_comparison, 
                    time_limit, 
                    term_count
                );
            }
        }
    }

    pub fn compare_new_with_existing(
        &self, 
        lodacpp: &LodaCpp, 
        path_program0: &Path, 
        path_program1: &Path,
        path_comparison: &Path, 
        time_limit: Duration, 
        term_count: usize
    ) -> anyhow::Result<CompareTwoProgramsResult> {
        if !path_program0.is_file() {
            return Err(anyhow::anyhow!("Expected a file, but got none. path_program0: {:?}", path_program0));
        }
        if !path_program1.is_file() {
            return Err(anyhow::anyhow!("Expected a file, but got none. path_program1: {:?}", path_program1));
        }

        let mut file = File::create(path_comparison)?;
        writeln!(&mut file, "program0, measuring steps: {:?}", path_program0)?;
        let start0 = Instant::now();
        let result0 = lodacpp.eval_steps(
            term_count,
            &path_program0, 
            time_limit
        );
        let result_steps0: LodaCppEvalSteps = match result0 {
            Ok(value) => {
                let elapsed: u128 = start0.elapsed().as_millis();
                // debug!("program0 steps:\n{:?}", value.steps());
                writeln!(&mut file, "program0, elapsed: {:?}ms", elapsed)?;
                // writeln!(&mut file, "program0, steps\n{:?}", value.steps())?;
                value
            },
            Err(error) => {
                // error!("Unable to compute steps for program0: {:?}", error);
                writeln!(&mut file, "Unable to compute steps for program0: {:?}", error)?;
                return Ok(CompareTwoProgramsResult::Program1);
            }
        };
        if result_steps0.steps().len() != term_count {
            writeln!(&mut file, "ERROR: Problem with program0. Expected {} steps, but got {}", term_count, result_steps0.steps().len())?;
            return Ok(CompareTwoProgramsResult::Program1);
        }
        
        writeln!(&mut file, "\n\nprogram1, measuring steps: {:?}", path_program1)?;
        let start1 = Instant::now();
        let result1 = lodacpp.eval_steps(
            term_count,
            &path_program1, 
            time_limit
        );
        let result_steps1: LodaCppEvalSteps = match result1 {
            Ok(value) => {
                let elapsed: u128 = start1.elapsed().as_millis();
                // debug!("program1 steps:\n{:?}", value.steps());
                writeln!(&mut file, "program1, elapsed: {:?}ms", elapsed)?;
                // writeln!(&mut file, "program1, steps\n{:?}", value.steps())?;
                value
            },
            Err(error) => {
                // debug!("Unable to compute steps for program1: {:?}", error);
                writeln!(&mut file, "Unable to compute steps for program1: {:?}", error)?;
                return Ok(CompareTwoProgramsResult::Program0);
            }
        };
        if result_steps1.steps().len() != term_count {
            writeln!(&mut file, "ERROR: Problem with program1. Expected {} steps, but got {}", term_count, result_steps1.steps().len())?;
            return Ok(CompareTwoProgramsResult::Program0);
        }

        write!(&mut file, "\n\nComparing terms:\nprogram0 vs program1\n")?;

        let step_items0: &Vec<u64> = result_steps0.steps();
        let step_items1: &Vec<u64> = result_steps1.steps();
        assert!(step_items0.len() == step_items1.len());

        let sum0: usize = step_items0.iter().map(|&x| x as usize).sum();
        let sum1: usize = step_items1.iter().map(|&x| x as usize).sum();
        writeln!(&mut file, "sum0: {}  sum1: {}", sum0, sum1)?;

        // let mut step0_less_than_step1: usize = 0;
        let mut last_slice_step0_greater_than_step1: usize = 0;
        // let mut step0_same_step1: usize = 0;
        // let mut step0_greater_than_step1: usize = 0;
        let mut identical: bool = true;
        for index in 0..step_items0.len() {
            let step0: u64 = step_items0[index];
            let step1: u64 = step_items1[index];
            let mut comparison_symbol = " ";
            if step0 == step1 {
                // step0_same_step1 += 1;
                comparison_symbol = " = ";
            }
            if step0 > step1 {
                // step0_greater_than_step1 += 1;
                comparison_symbol = "  >";
                identical = false;
                if index > 15 {
                    last_slice_step0_greater_than_step1 += 1;
                }
            }
            if step0 < step1 {
                // step0_less_than_step1 += 1;
                comparison_symbol = "<  ";
                identical = false;
            }
            writeln!(&mut file, "{:>10} {} {}", step0, comparison_symbol, step1)?;
        }

        if identical {
            writeln!(&mut file, "identical number of steps as the existing program. Keep the existing program.")?;
            return Ok(CompareTwoProgramsResult::Program1);
        }
        if sum0 == sum1 {
            writeln!(&mut file, "same sum as the existing program. Keep the existing program.")?;
            return Ok(CompareTwoProgramsResult::Program1);
        }
        if sum0 > sum1 {
            writeln!(&mut file, "total sum of new program is greater than existing program. Keep the existing program.")?;
            return Ok(CompareTwoProgramsResult::Program1);
        }
        if last_slice_step0_greater_than_step1 > 0 {
            writeln!(&mut file, "last slice of the new program is greater than existing program. Keep the existing program.")?;
            return Ok(CompareTwoProgramsResult::Program1);
        }
        if sum0 < sum1 {
            writeln!(&mut file, "the new program is faster than the existing program. Keep the new program.")?;
            return Ok(CompareTwoProgramsResult::Program0);
        }
        error!("uncaught scenario. Using existing program");
        writeln!(&mut file, "uncaught scenario. Using existing program")?;
        Ok(CompareTwoProgramsResult::Program1)
    }
}
