use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum ParameterType {
    Constant,
    Register,
}

impl FromStr for ParameterType {
    type Err = ();

    fn from_str(input: &str) -> Result<ParameterType, Self::Err> {
        match input {
            ""   => Ok(ParameterType::Constant),
            "$"  => Ok(ParameterType::Register),
            _    => Err(()),
        }
    }
}
