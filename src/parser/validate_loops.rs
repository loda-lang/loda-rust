use std::fmt;
use super::instruction_id::InstructionId;
use super::instruction::Instruction;

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
    let mut level: u8 = 0;
    for instruction in instruction_vec {
        let id: InstructionId = instruction.instruction_id.clone();
        match id {
            InstructionId::LoopBegin => {
                if level == 255 {
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

    fn push_begin(instruction_vec: &mut Vec<Instruction>) {
        let instruction = Instruction {
            instruction_id: InstructionId::LoopBegin,
            parameter_vec: vec!(),
            line_number: 1,
        };
        instruction_vec.push(instruction);
    }

    fn push_end(instruction_vec: &mut Vec<Instruction>) {
        let instruction = Instruction {
            instruction_id: InstructionId::LoopEnd,
            parameter_vec: vec!(),
            line_number: 1,
        };
        instruction_vec.push(instruction);
    }

    fn push_move(instruction_vec: &mut Vec<Instruction>) {
        let instruction = Instruction {
            instruction_id: InstructionId::Move,
            parameter_vec: vec!(),
            line_number: 1,
        };
        instruction_vec.push(instruction);
    }

    #[test]
    fn test_10000_valid_empty() {
        let v: Vec<Instruction> = vec!();
        let result = validate_loops(&v);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_10001_valid_simple() {
        let mut v: Vec<Instruction> = vec!();
        push_begin(&mut v);
            push_move(&mut v);
        push_end(&mut v);
        let result = validate_loops(&v);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_10002_valid_nested() {
        let mut v: Vec<Instruction> = vec!();
        push_begin(&mut v);
            push_move(&mut v);
            push_begin(&mut v);
                push_move(&mut v);
            push_end(&mut v);
            push_move(&mut v);
        push_end(&mut v);
        let result = validate_loops(&v);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_20000_invalid_endingtoosoon() {
        let mut v: Vec<Instruction> = vec!();
        push_end(&mut v);
        let result = validate_loops(&v);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), ValidateLoopError::EndingTooSoon);
    }

    #[test]
    fn test_20001_invalid_unbalanced() {
        let mut v: Vec<Instruction> = vec!();
        push_begin(&mut v);
        let result = validate_loops(&v);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), ValidateLoopError::Unbalanced);
    }

    #[test]
    fn test_20002_invalid_toodeep() {
        let mut v: Vec<Instruction> = vec!();
        for _ in 0..260 {
            push_begin(&mut v);
        }
        push_move(&mut v);
        for _ in 0..260 {
            push_end(&mut v);
        }
        let result = validate_loops(&v);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), ValidateLoopError::TooDeep);
    }
}
