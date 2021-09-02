use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionId {
    Add,
    Binomial,
    EvalSequence,
    Clear,
    Compare,
    Divide,
    DivideIf,
    GCD,
    LoopBegin,
    LoopEnd,
    Max,
    Min,
    Modulo,
    Move,
    Multiply,
    Power,
    Subtract,
    Truncate,
}

#[derive(Debug, PartialEq)]
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
        "add" => Ok(InstructionId::Add),
        "bin" => Ok(InstructionId::Binomial),
        "cal" => Ok(InstructionId::EvalSequence), // `cal` is deprecated. Use `seq` instead.
        "clr" => Ok(InstructionId::Clear),
        "cmp" => Ok(InstructionId::Compare),
        "dif" => Ok(InstructionId::DivideIf),
        "div" => Ok(InstructionId::Divide),
        "gcd" => Ok(InstructionId::GCD),
        "lpb" => Ok(InstructionId::LoopBegin),
        "lpe" => Ok(InstructionId::LoopEnd),
        "max" => Ok(InstructionId::Max),
        "min" => Ok(InstructionId::Min),
        "mod" => Ok(InstructionId::Modulo),
        "mov" => Ok(InstructionId::Move),
        "mul" => Ok(InstructionId::Multiply),
        "pow" => Ok(InstructionId::Power),
        "seq" => Ok(InstructionId::EvalSequence),
        "sub" => Ok(InstructionId::Subtract),
        "trn" => Ok(InstructionId::Truncate),
        _     => Err(ParseInstructionIdError::UnrecognizedInstructionId(line_number)),
    }
}

impl InstructionId {
    pub fn shortname(&self) -> &str {
        match self {
            Self::Add          => "add",
            Self::Binomial     => "bin",
            Self::EvalSequence => "seq",
            Self::Clear        => "clr",
            Self::Compare      => "cmp",
            Self::Divide       => "div",
            Self::DivideIf     => "dif",
            Self::GCD          => "gcd",
            Self::LoopBegin    => "lpb",
            Self::LoopEnd      => "lpe",
            Self::Max          => "max",
            Self::Min          => "min",
            Self::Modulo       => "mod",
            Self::Move         => "mov",
            Self::Multiply     => "mul",
            Self::Power        => "pow",
            Self::Subtract     => "sub",
            Self::Truncate     => "trn",
        }
    }
}
