use super::{DependencyManager, Settings};
use crate::mine::check_fixed_length_sequence::CheckFixedLengthSequence;
use crate::parser::{InstructionId, ParameterType};
use crate::execute::{EvalError, Program, ProgramRunner, RegisterValue, RunMode};
use crate::oeis::stripped_sequence::BigIntVec;
use std::path::Path;
use num_bigint::BigInt;
use rand::{Rng,SeedableRng};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

pub fn subcommand_mine(settings: &Settings) {
    println!("step1");
    let cache_file = Path::new("cache/fixed_length_sequence_5terms.json");
    let checker: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&cache_file);
    println!("step2");

    // TODO: mining
    run_experiment0(settings, &checker);
}

enum MutateValue {
    Increment,
    Decrement,
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

    fn mutate_trigger_division_by_zero(&mut self) {
        self.instruction_id = InstructionId::Divide;
        self.source_type = ParameterType::Constant;
        self.source_value = 0;
    }

    fn mutate_randomize_instruction<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let instructions: Vec<InstructionId> = vec![
            InstructionId::Add,
            InstructionId::Binomial,
            InstructionId::Compare,
            InstructionId::Divide,
            InstructionId::DivideIf,
            InstructionId::GCD,
            InstructionId::Logarithm,
            InstructionId::Max,
            InstructionId::Min,
            InstructionId::Modulo,
            InstructionId::Move,
            InstructionId::Multiply,
            InstructionId::Power,
            InstructionId::Subtract,
            InstructionId::Truncate,
        ];
        let instruction: &InstructionId = instructions.choose(rng).unwrap();
        self.instruction_id = instruction.clone();
    }

    fn mutate_source_value(&mut self, mutation: &MutateValue) -> bool {
        let (status, new_value) = self.mutate_value(mutation, self.source_value);
        self.source_value = new_value;
        status
    }

    fn mutate_target_value(&mut self, mutation: &MutateValue) -> bool {
        let (status, new_value) = self.mutate_value(mutation, self.target_value);
        self.target_value = new_value;
        status
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as underflow, overflow.
    fn mutate_value(&mut self, mutation: &MutateValue, mut value: u16) -> (bool, u16) {
        match mutation {
            MutateValue::Increment => {
                if value >= 255 {
                    return (false, value);
                }
                value += 1;
            },
            MutateValue::Decrement => {
                if value == 0 {
                    return (false, value);
                }
                value -= 1;
            },
        }
        (true, value)
    }

    fn mutate_source_type(&mut self) {
        match self.source_type {
            ParameterType::Constant => {
                self.source_type = ParameterType::Register;
            },
            ParameterType::Register => {
                self.source_type = ParameterType::Constant;
            },
        }
    }

    fn mutate_swap_source_target_value(&mut self) {
        let tmp = self.source_value;
        self.source_value = self.target_value;
        self.target_value = tmp;
    }

    fn mutate_sanitize_program_row(&mut self) -> bool {
        // Things to prevent 
        // division by zero
        // multiply by zero
        // raise to power 0
        // move/max/min/sub/mod/div/dif with same register
        // too huge constants
        // too huge register indexes
        // call to a non-existing program
        true
    }

    fn to_program_row(&self) -> String {
        match &self.instruction_id {
            InstructionId::LoopBegin => {
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
            InstructionId::LoopEnd => {
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

// Ideas for more mutations
// insert random row
// swap 2 rows
enum MutateGenome {
    Instruction,
    SourceConstant,
    SourceType,
    SwapRegisters,
    SourceRegister,
    TargetRegister,
}

struct Genome {
    genome_vec: Vec<GenomeItem>
}

impl Genome {
    fn new() -> Self {
        let mut genome_vec: Vec<GenomeItem> = vec!();
        for _ in 0..5 {
            genome_vec.push(GenomeItem::new());
        }
        // genome_vec[2].mutate_trigger_division_by_zero();
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

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as no instructions that use a constant, underflow, overflow.
    fn mutate_source_value_constant<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use constants
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.source_type == ParameterType::Constant {
                indexes.push(index);
            }
        }
        if indexes.is_empty() {
            return false;
        }

        // Pick a random mutation
        let mutation_vec: Vec<MutateValue> = vec![
            MutateValue::Increment,
            MutateValue::Decrement,
        ];
        let mutation: &MutateValue = mutation_vec.choose(rng).unwrap();

        // Mutate one of the instructions that use a constant
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        if !genome_item.mutate_source_value(mutation) {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as no instructions that use a source_type=register, underflow, overflow.
    fn mutate_source_register<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use source_type=register
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.source_type == ParameterType::Register {
                indexes.push(index);
            }
        }
        if indexes.is_empty() {
            return false;
        }

        // Pick a random mutation
        let mutation_vec: Vec<MutateValue> = vec![
            MutateValue::Increment,
            MutateValue::Decrement,
        ];
        let mutation: &MutateValue = mutation_vec.choose(rng).unwrap();

        // Mutate one of the instructions that use a constant
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        if !genome_item.mutate_source_value(mutation) {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_target_register<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        assert!(length > 0);
        let index: usize = rng.gen_range(0..length);

        // Pick a random mutation
        let mutation_vec: Vec<MutateValue> = vec![
            MutateValue::Increment,
            MutateValue::Decrement,
        ];
        let mutation: &MutateValue = mutation_vec.choose(rng).unwrap();

        // Mutate one of the instructions
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];
        if !genome_item.mutate_target_value(mutation) {
            return false;
        }
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_instruction<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        assert!(length > 0);
        let index: usize = rng.gen_range(0..length);
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];

        genome_item.mutate_randomize_instruction(rng);
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_source_type<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        assert!(length > 0);
        let index: usize = rng.gen_range(0..length);
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index];

        genome_item.mutate_source_type();
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    fn mutate_swap_registers<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        // Identify all the instructions that use two registers
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.source_type == ParameterType::Register {
                indexes.push(index);
            }
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions that use two registers
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        genome_item.mutate_swap_source_target_value();
        genome_item.mutate_sanitize_program_row()
    }

    // Return `true` when the mutation was successful.
    // Return `false` in case of failure.
    fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mutation_vec: Vec<MutateGenome> = vec![
            MutateGenome::Instruction,
            MutateGenome::SourceConstant,
            MutateGenome::SourceType,
            MutateGenome::SwapRegisters,
            MutateGenome::SourceRegister,
            MutateGenome::TargetRegister,
        ];
        let mutation: &MutateGenome = mutation_vec.choose(rng).unwrap();
        match mutation {
            MutateGenome::Instruction => {
                return self.mutate_instruction(rng);
            },
            MutateGenome::SourceConstant => {
                return self.mutate_source_value_constant(rng);
            },
            MutateGenome::SourceType => {
                return self.mutate_source_type(rng);
            },
            MutateGenome::SwapRegisters => {
                return self.mutate_swap_registers(rng);
            }
            MutateGenome::SourceRegister => {
                return self.mutate_source_register(rng);
            }
            MutateGenome::TargetRegister => {
                return self.mutate_target_register(rng);
            }
        }
    }
}

impl ProgramRunner {
    fn compute_terms(&self, count: u64) -> Result<BigIntVec, EvalError> {
        let mut terms: BigIntVec = vec!();
        for index in 0..(count as i64) {
            let input = RegisterValue::from_i64(index);
            let output: RegisterValue = self.run(input, RunMode::Silent)?;
            terms.push(output.0.clone());
            if index == 0 {
                print!("{}", output.0);
                continue;
            }
            print!(",{}", output.0);
        }
        print!("\n");
        Ok(terms)
    }
}

fn run_experiment0(settings: &Settings, checker: &CheckFixedLengthSequence) {
    let seed: u64 = 248;
    debug!("random seed: {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let mut dm = DependencyManager::new(
        settings.loda_program_rootdir.clone(),
    );
    let mut genome = Genome::new();
    genome.print();
    for _ in 0..4 {
        genome.mutate(&mut rng);
    }
    genome.print();

    let program: Program = dm.parse(&genome.to_program_string()).unwrap();
    let runner = ProgramRunner::new(program);
    let number_of_terms: u64 = 5;
    let terms: BigIntVec = runner.compute_terms(number_of_terms).unwrap();

    let check_result: bool = checker.check(&terms);
    println!("check_result: {:?}", check_result);
}
