#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisterType {
    Direct,
    Indirect
}

impl RegisterType {
    pub fn register_prefix(&self) -> &str {
        match self {
            RegisterType::Direct => "$",
            RegisterType::Indirect => "$$"
        }
    }
}
