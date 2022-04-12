use loda_rust_core;
use loda_rust_core::config::Config;
use loda_rust_core::parser::{Instruction, InstructionId, InstructionParameter, ParsedProgram};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use csv::WriterBuilder;
use serde::Serialize;
use super::{BatchProgramAnalyzerPlugin, BatchProgramAnalyzerContext};

pub struct AnalyzeProgramComplexity {
}

impl AnalyzeProgramComplexity {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl BatchProgramAnalyzerPlugin for AnalyzeProgramComplexity {
    fn analyze(&mut self, context: &BatchProgramAnalyzerContext) -> bool {
        let mut has_loops: bool = false;
        for instruction in &context.parsed_program.instruction_vec {
            if instruction.instruction_id == InstructionId::LoopBegin {
                has_loops = true;
            }
        }
        true
    }

    fn save(&self) {
    }
}
