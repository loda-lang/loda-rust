#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisterType {
    Direct,
    Indirect
}

impl RegisterType {
    pub fn prefix(&self) -> &str {
        match self {
            RegisterType::Direct   => "$",
            RegisterType::Indirect => "$$"
        }
    }
}
