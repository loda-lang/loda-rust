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
    UnofficialFunction { input_count: u8, output_count: u8 },
    UnofficialLoopBeginSubtract,
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
            Self::UnofficialFunction { input_count, output_count } => {
                return write!(f, "f{}{}", input_count, output_count);
            },
            Self::UnofficialLoopBeginSubtract => "lps",
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
        assert_eq!(InstructionId::UnofficialFunction { input_count: 7, output_count: 6 }, InstructionId::UnofficialFunction { input_count: 7, output_count: 6 });
        assert_ne!(InstructionId::UnofficialFunction { input_count: 7, output_count: 6 }, InstructionId::UnofficialFunction { input_count: 6, output_count: 7 });
        assert_eq!(InstructionId::UnofficialLoopBeginSubtract, InstructionId::UnofficialLoopBeginSubtract);
        assert_ne!(InstructionId::UnofficialLoopBeginSubtract, InstructionId::LoopBegin);
    }

    #[test]
    fn test_20000_to_string() {
        assert_eq!(InstructionId::DivideIf.to_string(), "dif");
        assert_eq!(InstructionId::Multiply.to_string(), "mul");
        assert_eq!(InstructionId::Truncate.to_string(), "trn");
        
        let instruction = InstructionId::UnofficialFunction { input_count: 7, output_count: 6 };
        assert_eq!(instruction.to_string(), "f76");

        assert_eq!(InstructionId::UnofficialLoopBeginSubtract.to_string(), "lps");
    }
}
