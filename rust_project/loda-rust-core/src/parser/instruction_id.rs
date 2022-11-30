use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
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

impl fmt::Display for InstructionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: &'static str = match self {
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
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_10000_equal() {
        assert_eq!(InstructionId::Add, InstructionId::Add);
        assert_ne!(InstructionId::Add, InstructionId::Subtract);
    }

    #[test]
    fn test_20000_to_string() {
        assert_eq!(InstructionId::DivideIf.to_string(), "dif");
        assert_eq!(InstructionId::Multiply.to_string(), "mul");
        assert_eq!(InstructionId::Truncate.to_string(), "trn");
    }
}
