use super::{DependencyManager, Settings};
use crate::mine::check_fixed_length_sequence::CheckFixedLengthSequence;
use std::path::Path;
use crate::parser::{InstructionId, ParameterType};
use crate::execute::{Program, ProgramRunner, ProgramRunnerManager, RegisterValue, RunMode};

pub fn subcommand_mine(settings: &Settings) {
    println!("step1");
    let cache_file = Path::new("cache/fixed_length_sequence_5terms.json");
    let checker: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&cache_file);
    println!("step2");

    // TODO: mining
    run_experiment0(settings);
}


struct GenomeItem {
    instruction_id: InstructionId,
    target_value: u16,
    source_type: ParameterType,
    source_value: u16,
}

impl GenomeItem {
    fn new() -> Self {
        Self {
            instruction_id: InstructionId::Move,
            target_value: 1,
            source_type: ParameterType::Register,
            source_value: 0,
        }
    }

    fn to_program_row(&self) -> String {
        match &self.instruction_id {
            LoopBegin => {
                if self.source_type == ParameterType::Register {
                    return format!("{} ${},${}",
                        self.instruction_id.shortname(), 
                        self.target_value, 
                        self.source_value
                    );
                }
                if self.target_value >= 2 {
                    return format!("{} ${},{}", 
                        self.instruction_id.shortname(), 
                        self.target_value, 
                        self.source_value
                    );
                } else {
                    return format!("{} ${}", 
                        self.instruction_id.shortname(), 
                        self.target_value 
                    );
                }
            },
            LoopEnd => {
                return self.instruction_id.shortname().to_string();
            },
            _ => {
                return format!("{} ${},{}{}", 
                    self.instruction_id.shortname(), 
                    self.target_value, 
                    self.source_type.prefix(), 
                    self.source_value
                );
            }
        }
    }
}

struct Genome {
    genome_vec: Vec<GenomeItem>
}

impl Genome {
    fn new() -> Self {
        let mut genome_vec: Vec<GenomeItem> = vec!();
        for _ in 0..3 {
            genome_vec.push(GenomeItem::new());
        }
        Self {
            genome_vec: genome_vec,
        }
    }

    fn to_program_string(&self) -> String {
        let program_rows: Vec<String> = self.genome_vec.iter().map(|genome_item| {
            genome_item.to_program_row()
        }).collect();
        program_rows.join("\n")
    }

    fn print(&self) {
        println!("program:\n{}", self.to_program_string());
    }
}

impl ProgramRunner {
    fn compute_terms(&self, count: u64) {
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(input, RunMode::Silent);
            if index == 0 {
                print!("{}", output.0);
                continue;
            }
            print!(",{}", output.0);
        }
        print!("\n");
    }
}

fn run_experiment0(settings: &Settings) {
    let mut dm = DependencyManager::new(
        settings.loda_program_rootdir.clone(),
    );
    let mut genome = Genome::new();
    genome.print();

    let program: Program = dm.parse(&genome.to_program_string());
    let runner = ProgramRunner::new(program);
    let number_of_terms: u64 = 5;
    runner.compute_terms(number_of_terms);
}
