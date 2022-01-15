use super::HistogramInstructionNgram;
use super::histogram_instruction_ngram::RecordTrigram;
use loda_rust_core::parser::InstructionId;
use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;
use std::error::Error;

type HistogramKey = (String,String);
type InstructionAndWeight = (InstructionId,u32);
type HistogramValue = Vec<InstructionAndWeight>;

pub struct SuggestInstruction {
    histogram: HashMap<HistogramKey, HistogramValue>
}

impl SuggestInstruction {
    pub fn new() -> Self {
        Self {
            histogram: HashMap::new()
        }
    }

    pub fn populate(&mut self, ngram: &HistogramInstructionNgram) -> Result<(), Box<dyn Error>> {
        let records: Vec<RecordTrigram> = ngram.loda_trigram()?;
        for record in records {
            let key: HistogramKey = (record.word0, record.word2);
            let instruction_id: InstructionId = match InstructionId::parse(&record.word1, 0) {
                Ok(instruction_id) => instruction_id,
                Err(_) => {
                    continue;
                }
            };
            if instruction_id == InstructionId::LoopBegin ||
                instruction_id == InstructionId::LoopEnd ||
                instruction_id == InstructionId::Clear ||
                instruction_id == InstructionId::EvalSequence {
                // Don't suggest these instructions
                continue;
            }
            let instruction_and_weight: InstructionAndWeight = (instruction_id, record.count);
            let item = self.histogram.entry(key).or_insert(vec!());
            (*item).push(instruction_and_weight);
        }
        Ok(())
    }

    // If it's the beginning of the program then set prev_instruction to None.
    // If it's the end of the program then set next_instruction to None.
    fn candidate_instructions(&self, prev_instruction: Option<InstructionId>, next_instruction: Option<InstructionId>) -> Option<&HistogramValue> {
        let word0: String = match prev_instruction {
            Some(instruction_id) => instruction_id.shortname().to_string(),
            None => "START".to_string()
        };
        let word2: String = match next_instruction {
            Some(instruction_id) => instruction_id.shortname().to_string(),
            None => "STOP".to_string()
        };
        let key: HistogramKey = (word0, word2);
        self.histogram.get(&key)
    }

    pub fn best_instruction(&self, prev_instruction: Option<InstructionId>, next_instruction: Option<InstructionId>) -> Option<InstructionId> {
        let histogram_value: &HistogramValue = match self.candidate_instructions(prev_instruction, next_instruction) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let instruction_id: InstructionId = match histogram_value.first() {
            Some(value) => value.0,
            None => {
                return None;
            }
        };
        Some(instruction_id)
    }

    pub fn choose_weighted<R: Rng + ?Sized>(&self, rng: &mut R, prev_instruction: Option<InstructionId>, next_instruction: Option<InstructionId>) -> Option<InstructionId> {
        let histogram_value: &HistogramValue = match self.candidate_instructions(prev_instruction, next_instruction) {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let instruction_id: InstructionId = histogram_value.choose_weighted(rng, |item| item.1).unwrap().0;
        Some(instruction_id)
    }
}
