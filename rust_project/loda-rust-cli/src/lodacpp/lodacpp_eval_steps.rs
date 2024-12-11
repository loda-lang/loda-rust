use super::LodaCppError;
use std::error::Error;

pub struct LodaCppEvalSteps {
    steps: Vec<u64>,
}

impl LodaCppEvalSteps {
    fn new(steps: Vec<u64>) -> Self {
        Self {
            steps: steps,
        }
    }

    pub fn parse(raw_output_from_lodacpp: &String, term_count: usize) -> Result<LodaCppEvalSteps, Box<dyn Error>> {
        let trimmed_output: String = raw_output_from_lodacpp.trim().to_string();
        if trimmed_output.is_empty() {
            error!("No output to parse.");
            return Err(Box::new(LodaCppError::NoOutput));
        }
        let step_strings = trimmed_output.split(",");
        let mut steps_vec = Vec::<u64>::with_capacity(term_count);
        for step_string in step_strings {
            if step_string.starts_with("+") {
                error!("Positive number should not start with plus symbol. '{}'", step_string);
                return Err(Box::new(LodaCppError::ParseSteps));
            }
            let value: u64 = match step_string.parse::<u64>() {
                Ok(value) => value,
                Err(error) => {
                    error!("Unable to parse a number as u64. '{}', error: {:?}", step_string, error);
                    return Err(Box::new(LodaCppError::ParseSteps));
                }
            };
            steps_vec.push(value);
        };
        Ok(LodaCppEvalSteps::new(steps_vec))
    }

    #[allow(dead_code)]
    pub fn steps(&self) -> &Vec<u64> {
        &self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_compact_comma_string(steps: &Vec<u64>) -> String {
        let strings: Vec<String> = steps.iter().map(|item| {
            item.to_string()
        }).collect();
        strings.join(",")
    }

    fn parse(input: &str) -> String {
        let s = input.to_string();
        let eval_ok: LodaCppEvalSteps = match LodaCppEvalSteps::parse(&s, 0) {
            Ok(value) => value,
            Err(error) => {
                if let Some(lodacpp_error) = error.downcast_ref::<LodaCppError>() {
                    if LodaCppError::NoOutput == *lodacpp_error {
                        return "ERROR NoOutput".to_string();
                    }
                    if LodaCppError::ParseSteps == *lodacpp_error {
                        return "ERROR ParseSteps".to_string();
                    }
                    return format!("ERROR LODACPP: {:?}", lodacpp_error);
                }
                return format!("ERROR OTHER: {:?}", error);
            }
        };
        to_compact_comma_string(eval_ok.steps())
    }

    #[test]
    fn test_10000_parse_ok() {
        assert_eq!(parse("\n  42\t  "), "42");
        assert_eq!(parse("0"), "0");
        assert_eq!(parse("0,0"), "0,0");
        assert_eq!(parse("0,1,2,3"), "0,1,2,3");
        assert_eq!(parse("0000555"), "555");
    }
    
    #[test]
    fn test_20000_parse_error() {
        assert_eq!(parse(""), "ERROR NoOutput");
        assert_eq!(parse(" "), "ERROR NoOutput");
        assert_eq!(parse(" \n "), "ERROR NoOutput");
        assert_eq!(parse("1,2,overflow"), "ERROR ParseSteps");
        assert_eq!(parse("c++ exception"), "ERROR ParseSteps");
        assert_eq!(parse("-1,2,-3,4"), "ERROR ParseSteps");
        assert_eq!(parse("+123"), "ERROR ParseSteps");
    }
}
