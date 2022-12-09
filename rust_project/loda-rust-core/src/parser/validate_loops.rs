use std::fmt;
use super::instruction_id::InstructionId;
use super::instruction::Instruction;

const MAX_ALLOWED_NESTING_LEVEL: u16 = 255;

#[derive(Debug, PartialEq)]
pub enum ValidateLoopError {
    TooDeep,
    EndingTooSoon,
    Unbalanced,
}

impl fmt::Display for ValidateLoopError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::TooDeep => write!(f, "Nesting deeper than the max allowed nesting level."),
            Self::EndingTooSoon => write!(f, "Premature ending of loop in root scope."),
            Self::Unbalanced => write!(f, "Unbalanced number of 'lpb' and 'lpe' encounted."),
        }
    }
}

pub fn validate_loops(instruction_vec: &Vec<Instruction>) -> Result<(), ValidateLoopError> {
    let mut level: u16 = 0;
    for instruction in instruction_vec {
        let id: InstructionId = instruction.instruction_id.clone();
        match id {
            InstructionId::LoopBegin => {
                if level >= MAX_ALLOWED_NESTING_LEVEL {
                    return Err(ValidateLoopError::TooDeep);
                }
                level += 1;
            },
            InstructionId::LoopEnd => {
                if level == 0 {
                    return Err(ValidateLoopError::EndingTooSoon);
                }
                level -= 1;
            },
            _ => {}
        }
    }
    if level != 0 {
        return Err(ValidateLoopError::Unbalanced);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Translate from `InstructionId` vec to `Instruction` vec.
    fn convert(instruction_ids: Vec<InstructionId>) -> Vec<Instruction> {
        let mut instructions = Vec::<Instruction>::new();
        for (index, instruction_id) in instruction_ids.iter().enumerate() {
            let instruction = Instruction {
                instruction_id: *instruction_id,
                parameter_vec: vec!(),
                line_number: index + 1,
            };
            instructions.push(instruction);
        }
        instructions
    }

    #[test]
    fn test_10000_valid_empty() {
        let v: Vec<Instruction> = vec!();
        let result = validate_loops(&v);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_10001_valid_simple() {
        let instruction_ids: Vec<InstructionId> = vec![
            InstructionId::LoopBegin, 
                InstructionId::Move, 
            InstructionId::LoopEnd, 
        ];
        let v: Vec<Instruction> = convert(instruction_ids);
        let result = validate_loops(&v);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_10002_valid_nested() {
        let instruction_ids: Vec<InstructionId> = vec![
            InstructionId::LoopBegin, 
                InstructionId::Move, 
                InstructionId::LoopBegin, 
                    InstructionId::Move, 
                InstructionId::LoopEnd, 
                InstructionId::Move, 
            InstructionId::LoopEnd, 
        ];
        let v: Vec<Instruction> = convert(instruction_ids);
        let result = validate_loops(&v);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_20000_invalid_endingtoosoon() {
        let instruction_ids: Vec<InstructionId> = vec![
            InstructionId::LoopEnd,
        ];
        let v: Vec<Instruction> = convert(instruction_ids);
        let result = validate_loops(&v);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), ValidateLoopError::EndingTooSoon);
    }

    #[test]
    fn test_20001_invalid_unbalanced() {
        let instruction_ids: Vec<InstructionId> = vec![
            InstructionId::LoopBegin,
        ];
        let v: Vec<Instruction> = convert(instruction_ids);
        let result = validate_loops(&v);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), ValidateLoopError::Unbalanced);
    }

    #[test]
    fn test_30000_deeply_nested_ok() {
        let mut instruction_ids: Vec<InstructionId> = vec!();
        for _ in 0..=254 {
            instruction_ids.push(InstructionId::LoopBegin);
        }
        instruction_ids.push(InstructionId::Move);
        for _ in 0..=254 {
            instruction_ids.push(InstructionId::LoopEnd);
        }
        let v: Vec<Instruction> = convert(instruction_ids);
        let result = validate_loops(&v);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_30001_deeply_nested_error_toodeep() {
        let mut instruction_ids: Vec<InstructionId> = vec!();
        for _ in 0..=255 {
            instruction_ids.push(InstructionId::LoopBegin);
        }
        instruction_ids.push(InstructionId::Move);
        for _ in 0..=255 {
            instruction_ids.push(InstructionId::LoopEnd);
        }
        let v: Vec<Instruction> = convert(instruction_ids);
        let result = validate_loops(&v);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), ValidateLoopError::TooDeep);
    }
}
