use super::Settings;
use crate::mine::check_fixed_length_sequence::CheckFixedLengthSequence;
use std::path::Path;
use crate::parser::{InstructionId, ParameterType};

pub fn subcommand_mine(_settings: &Settings) {
    println!("step1");
    let cache_file = Path::new("cache/fixed_length_sequence_5terms.json");
    let checker: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&cache_file);
    println!("step2");

    // TODO: mining
    run_experiment0();
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

    fn print_program(&self) {
        let mut program_rows: Vec<String> = vec!();
        for item in &self.genome_vec {
            program_rows.push(item.to_program_row());
        }
        let program: String = program_rows.join("\n");
        println!("program:\n{}", program);
    }
}

fn run_experiment0() {
    let mut genome = Genome::new();
    genome.print_program();
}
