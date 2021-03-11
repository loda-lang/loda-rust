use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionId {
    Move,
    Add,
    Divide,
    DivideIf,
    Subtract,
    LoopBegin,
    LoopEnd,
    Call,
    Power,
    Multiply,
    Modulo,
    GCD,
    Truncate,
    Binomial,
    Compare,
    Logarithm,
}

#[derive(Debug)]
pub enum ParseInstructionIdError {
    UnrecognizedInstructionId(usize),
}

impl fmt::Display for ParseInstructionIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::UnrecognizedInstructionId(line_number) => 
                write!(f, "Unrecognized instruction in line {}", line_number),
        }
    }
}

pub fn parse_instruction_id(input: &str, line_number: usize) 
    -> Result<InstructionId,ParseInstructionIdError> 
{
    match input {
        "mov" => Ok(InstructionId::Move),
        "add" => Ok(InstructionId::Add),
        "sub" => Ok(InstructionId::Subtract),
        "lpb" => Ok(InstructionId::LoopBegin),
        "lpe" => Ok(InstructionId::LoopEnd),
        "cal" => Ok(InstructionId::Call),
        "pow" => Ok(InstructionId::Power),
        "mul" => Ok(InstructionId::Multiply),
        "div" => Ok(InstructionId::Divide),
        "dif" => Ok(InstructionId::DivideIf),
        "mod" => Ok(InstructionId::Modulo),
        "gcd" => Ok(InstructionId::GCD),
        "trn" => Ok(InstructionId::Truncate),
        "bin" => Ok(InstructionId::Binomial),
        "cmp" => Ok(InstructionId::Compare),
        "log" => Ok(InstructionId::Logarithm),
        _     => Err(ParseInstructionIdError::UnrecognizedInstructionId(line_number)),
    }
}

impl InstructionId {
    pub fn shortname(&self) -> &str {
        match self {
            Self::Move      => "mov",
            Self::Add       => "add",
            Self::Subtract  => "sub",
            Self::LoopBegin => "lpb",
            Self::LoopEnd   => "lpe",
            Self::Call      => "cal",
            Self::Power     => "pow",
            Self::Multiply  => "mul",
            Self::Divide    => "div",
            Self::DivideIf  => "dif",
            Self::Modulo    => "mod",
            Self::GCD       => "gcd",
            Self::Truncate  => "trn",
            Self::Binomial  => "bin",
            Self::Compare   => "cmp",
            Self::Logarithm => "log",
        }
    }
}
