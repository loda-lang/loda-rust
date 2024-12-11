use super::{UnofficialFunction, UnofficialFunctionId};
use num_bigint::BigInt;

pub enum AssertFunctionMode {
    Equal,
    Different,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

pub struct AssertFunction {
    id: u32,
    mode: AssertFunctionMode,
}

impl AssertFunction {
    pub fn new(id: u32, mode: AssertFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for AssertFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 0 }
    }

    fn name(&self) -> String {
        match self.mode {
            AssertFunctionMode::Equal => {
                return "Assert input[0] is equal to input[1].".to_string();
            },
            AssertFunctionMode::Different => {
                return "Assert input[0] is different than input[1].".to_string();
            },
            AssertFunctionMode::LessThan => {
                return "Assert input[0] is less than input[1].".to_string();
            },
            AssertFunctionMode::LessThanOrEqual => {
                return "Assert input[0] is less than or equal to input[1].".to_string();
            },
            AssertFunctionMode::GreaterThan => {
                return "Assert input[0] is greater than input[1].".to_string();
            },
            AssertFunctionMode::GreaterThanOrEqual => {
                return "Assert input[0] is greater than or equal to input[1].".to_string();
            }
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }
        let input0: &BigInt = &input[0];
        let input1: &BigInt = &input[1];

        match self.mode {
            AssertFunctionMode::Equal => {
                if input0 == input1 {
                    return Ok(vec!());
                }
                return Err(anyhow::anyhow!("Expected input0 {} to be equal to input1 {}", input0, input1));
            },
            AssertFunctionMode::Different => {
                if input0 != input1 {
                    return Ok(vec!());
                }
                return Err(anyhow::anyhow!("Expected input0 {} to be different than input1 {}", input0, input1));
            },
            AssertFunctionMode::LessThan => {
                if input0 < input1 {
                    return Ok(vec!());
                }
                return Err(anyhow::anyhow!("Expected input0 {} to be less than input1 {}", input0, input1));
            },
            AssertFunctionMode::LessThanOrEqual => {
                if input0 <= input1 {
                    return Ok(vec!());
                }
                return Err(anyhow::anyhow!("Expected input0 {} to be less than or equal to input1 {}", input0, input1));
            },
            AssertFunctionMode::GreaterThan => {
                if input0 > input1 {
                    return Ok(vec!());
                }
                return Err(anyhow::anyhow!("Expected input0 {} to be greater than input1 {}", input0, input1));
            },
            AssertFunctionMode::GreaterThanOrEqual => {
                if input0 >= input1 {
                    return Ok(vec!());
                }
                return Err(anyhow::anyhow!("Expected input0 {} to be greater than or equal to input1 {}", input0, input1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::perform_run;

    #[test]
    fn test_equal() {
        {
            let f = AssertFunction::new(0, AssertFunctionMode::Equal);
            _ = perform_run(Box::new(f), vec!(0, 0)).expect("should not fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::Equal);
            _ = perform_run(Box::new(f), vec!(-10, 10)).expect_err("should fail");
        }
    }

    #[test]
    fn test_different() {
        {
            let f = AssertFunction::new(0, AssertFunctionMode::Different);
            _ = perform_run(Box::new(f), vec!(-10, 10)).expect("should not fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::Different);
            _ = perform_run(Box::new(f), vec!(0, 0)).expect_err("should fail");
        }
    }

    #[test]
    fn test_less_than() {
        {
            let f = AssertFunction::new(0, AssertFunctionMode::LessThan);
            _ = perform_run(Box::new(f), vec!(-1, 0)).expect("should not fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::LessThan);
            _ = perform_run(Box::new(f), vec!(0, 0)).expect_err("should fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::LessThan);
            _ = perform_run(Box::new(f), vec!(1, 0)).expect_err("should fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::LessThan);
            _ = perform_run(Box::new(f), vec!(-1001, -1000)).expect("should not fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::LessThan);
            _ = perform_run(Box::new(f), vec!(-1000, -1000)).expect_err("should fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::LessThan);
            _ = perform_run(Box::new(f), vec!(-999, -1000)).expect_err("should fail");
        }
    }

    #[test]
    fn test_less_than_or_equal() {
        {
            let f = AssertFunction::new(0, AssertFunctionMode::LessThanOrEqual);
            _ = perform_run(Box::new(f), vec!(-1, 0)).expect("should not fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::LessThanOrEqual);
            _ = perform_run(Box::new(f), vec!(0, 0)).expect("should not fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::LessThanOrEqual);
            _ = perform_run(Box::new(f), vec!(1, 0)).expect_err("should fail");
        }
    }

    #[test]
    fn test_greater_than() {
        {
            let f = AssertFunction::new(0, AssertFunctionMode::GreaterThan);
            _ = perform_run(Box::new(f), vec!(-1, 0)).expect_err("should fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::GreaterThan);
            _ = perform_run(Box::new(f), vec!(0, 0)).expect_err("should fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::GreaterThan);
            _ = perform_run(Box::new(f), vec!(1, 0)).expect("should not fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::GreaterThan);
            _ = perform_run(Box::new(f), vec!(-1001, -1000)).expect_err("should fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::GreaterThan);
            _ = perform_run(Box::new(f), vec!(-1000, -1000)).expect_err("should fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::GreaterThan);
            _ = perform_run(Box::new(f), vec!(-999, -1000)).expect("should not fail");
        }
    }

    #[test]
    fn test_greater_than_or_equal() {
        {
            let f = AssertFunction::new(0, AssertFunctionMode::GreaterThanOrEqual);
            _ = perform_run(Box::new(f), vec!(-1, 0)).expect_err("should fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::GreaterThanOrEqual);
            _ = perform_run(Box::new(f), vec!(0, 0)).expect("should not fail");
        }
        {
            let f = AssertFunction::new(0, AssertFunctionMode::GreaterThanOrEqual);
            _ = perform_run(Box::new(f), vec!(1, 0)).expect("should not fail");
        }
    }
}
