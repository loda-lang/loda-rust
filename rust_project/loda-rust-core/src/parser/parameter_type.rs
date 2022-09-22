use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum ParameterType {
    Constant,
    Direct,
    Indirect,
}

impl ParameterType {
    pub fn prefix(&self) -> &str {
        match self {
            ParameterType::Constant => "",
            ParameterType::Direct   => "$",
            ParameterType::Indirect => "$$",
        }
    }
}

impl FromStr for ParameterType {
    type Err = ();

    fn from_str(input: &str) -> Result<ParameterType, Self::Err> {
        match input {
            ""   => Ok(ParameterType::Constant),
            "$"  => Ok(ParameterType::Direct),
            "$$" => Ok(ParameterType::Indirect),
            _    => Err(()),
        }
    }
}
