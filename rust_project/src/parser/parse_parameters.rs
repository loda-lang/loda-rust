use std::fmt;
use std::str::FromStr;
use super::extract_parameter_re::EXTRACT_PARAMETER_RE;
use super::{InstructionParameter, ParameterType};

#[derive(Debug, PartialEq)]
pub enum ParseParametersError {
    EmptyParameter(usize),
    UnrecognizedParameter(usize),
    UnrecognizedParameterType(usize),
    UnrecognizedParameterValue(usize),
}

impl fmt::Display for ParseParametersError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::EmptyParameter(line_number) => 
                write!(f, "Encountered an empty parameter in line {}", line_number),
            Self::UnrecognizedParameter(line_number) => 
                write!(f, "Unrecognized parameter in line {}", line_number),
            Self::UnrecognizedParameterType(line_number) => 
                write!(f, "Unrecognized parameter type in line {}", line_number),
            Self::UnrecognizedParameterValue(line_number) => 
                write!(f, "Unrecognized parameter value in line {}", line_number),
        }
    }
}

pub fn parse_parameters(parameter_string_raw: &str, line_number: usize) 
    -> Result<Vec<InstructionParameter>,ParseParametersError> 
{
    let parameter_string_trimmed: &str = parameter_string_raw.trim_end();
    if parameter_string_trimmed.is_empty() {
        // There are instructions that takes 0 parameters, so this case is fine.
        return Ok(vec!());
    }

    let re = &EXTRACT_PARAMETER_RE;
    let mut parameter_vec: Vec<InstructionParameter> = vec!();
    for split_item in parameter_string_trimmed.split(",") {
        let trimmed_split_item = split_item.trim();
        if trimmed_split_item.is_empty() {
            return Err(ParseParametersError::EmptyParameter(line_number));
        }

        let captures = match re.captures(trimmed_split_item) {
            Some(value) => value,
            None => {
                return Err(ParseParametersError::UnrecognizedParameter(line_number));
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let capture2: &str = captures.get(2).map_or("", |m| m.as_str());

        let parameter_type: ParameterType = match ParameterType::from_str(capture1) {
            Ok(value) => value,
            _ => {
                return Err(ParseParametersError::UnrecognizedParameterType(line_number));
            }
        };
        let parameter_value: i64 = match i64::from_str(capture2) {
            Ok(value) => value,
            _ => {
                return Err(ParseParametersError::UnrecognizedParameterValue(line_number));
            }
        };
        let parameter = InstructionParameter {
            parameter_type: parameter_type,
            parameter_value: parameter_value,
        };
        parameter_vec.push(parameter);
    }

    Ok(parameter_vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(input: &str) -> String {
        let result = parse_parameters(input, 1);
        let parameter_vec = match result {
            Ok(items) => items,
            Err(error) => {
                return format!("{:?}", error);
            }
        };
        if parameter_vec.is_empty() {
            return "EMPTY".to_string();
        }
        let parameters: Vec<String> = parameter_vec.iter().map(|parameter| {
            parameter.to_string()
        }).collect();
        let parameters_joined: String = parameters.join(",");
        return parameters_joined;
    }

    #[test]
    fn test_10000_ignore_empty() {
        assert_eq!(process(""), "EMPTY");
        assert_eq!(process(" "), "EMPTY");
        assert_eq!(process(" \t \t"), "EMPTY");
    }

    #[test]
    fn test_10001_wellformed() {
        assert_eq!(process("0"), "0");
        assert_eq!(process("-11,-22"), "-11,-22");
        assert_eq!(process("1,2,3"), "1,2,3");
        assert_eq!(process("$0,$0,$0"), "$0,$0,$0");
    }

    #[test]
    fn test_10002_removal_of_spaces() {
        assert_eq!(process(" 0 "), "0");
        assert_eq!(process("  -11  , -22  "), "-11,-22");
        assert_eq!(process("\t\t1 ,\t2,3 "), "1,2,3");
        assert_eq!(process(" $0, $0, $0 "), "$0,$0,$0");
    }

    #[test]
    fn test_10003_reject_empty_parameters() {
        assert_eq!(process(","), "EmptyParameter(1)");
        assert_eq!(process(",0"), "EmptyParameter(1)");
        assert_eq!(process("0,"), "EmptyParameter(1)");
        assert_eq!(process("0,,0"), "EmptyParameter(1)");
    }

    #[test]
    fn test_10004_reject_junk() {
        assert_eq!(process("junk"), "UnrecognizedParameter(1)");
        assert_eq!(process("*"), "UnrecognizedParameter(1)");
        assert_eq!(process("4141junk"), "UnrecognizedParameter(1)");
        assert_eq!(process("$4141junk"), "UnrecognizedParameter(1)");
    }

    #[test]
    fn test_10005_parameter_type() {
        assert_eq!(process("$$$$0"), "UnrecognizedParameterType(1)");
        assert_eq!(process("$$$0"), "UnrecognizedParameterType(1)");
        assert_eq!(process("$$0"), "UnrecognizedParameterType(1)");
        assert_eq!(process("$0"), "$0");
        assert_eq!(process("0"), "0");
    }
}
