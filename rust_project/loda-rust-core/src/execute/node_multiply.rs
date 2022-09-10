use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterIndexAndType, RegisterType, RegisterValue};
use super::{BoxCheckValue, PerformCheckValue};
use std::collections::HashSet;
use num_bigint::BigInt;
use num_traits::Zero;

fn perform_operation(check: &BoxCheckValue, x: &RegisterValue, y: &RegisterValue) -> Result<RegisterValue, EvalError> {
    let xx: &BigInt = &x.0;
    check.input(xx)?;

    let yy: &BigInt = &y.0;
    check.input(yy)?;

    let zz: BigInt = xx * yy;
    check.output(&zz)?;

    Ok(RegisterValue(zz))
}

pub struct NodeMultiplyAdvanced {
    target: RegisterIndexAndType,
    source: RegisterIndexAndType,
}

impl NodeMultiplyAdvanced {
    pub fn new(target: RegisterIndexAndType, source: RegisterIndexAndType) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMultiplyAdvanced {
    fn formatted_instruction(&self) -> String {
        format!("mov {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let mut tmp_value: RegisterValue;
        match self.source.register_type {
            RegisterType::Direct => {
                let value: &RegisterValue = state.get_register_value_ref(&self.source.register_index);
                tmp_value = value.clone();
            },
            RegisterType::Indirect => {
                // TODO: deal with indirect
                // let value: &RegisterValue = state.get_register_value_ref(&self.source.register_index);
                // // TODO: convert from value to register index
                // let value2: &RegisterValue = state.get_register_value_ref(value);
                // tmp_value = value2.clone();
                let reg_value: &RegisterValue = state.get_register_value_ref(&self.source.register_index);
                let reg_value2: Option<i64> = reg_value.try_to_i64();

                // TODO: deal with indirect
                let register_index: RegisterIndex;
                match reg_value2 {
                    Some(value) => {
                        if value < 0 {
                            panic!("indirect source out of range, too low");
                        }
                        if value > 255 {
                            panic!("indirect source out of range, too high");
                        }
                        register_index = RegisterIndex(value as u8)
                    },
                    None => {
                        panic!("indirect source out of range, not an i64");
                    }
                }
                let value: &RegisterValue = state.get_register_value_ref(&register_index);
                tmp_value = value.clone();
                // panic!("boom indirect source");
            }
        }

        let lhs: &RegisterValue = state.get_register_value_ref(&self.target.register_index);
        let rhs: &RegisterValue = &tmp_value; //state.get_register_value_ref(&self.source.register_index);
        tmp_value = perform_operation(state.check_value(), lhs, rhs)?;

        match self.target.register_type {
            RegisterType::Direct => {
                state.set_register_value(self.target.register_index.clone(), tmp_value);
            },
            RegisterType::Indirect => {
                let reg_value: &RegisterValue = state.get_register_value_ref(&self.target.register_index);
                let reg_value2: Option<i64> = reg_value.try_to_i64();

                // TODO: deal with indirect
                let register_index: RegisterIndex;
                match reg_value2 {
                    Some(value) => {
                        if value < 0 {
                            panic!("indirect target out of range, too low");
                        }
                        if value > 255 {
                            panic!("indirect target out of range, too high");
                        }
                        register_index = RegisterIndex(value as u8)
                    },
                    None => {
                        panic!("indirect target out of range, not an i64");
                    }
                }
                state.set_register_value(register_index, tmp_value);
            }
        }
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        // TODO: deal with indirect
        register_vec.push(self.target.register_index.clone());
        register_vec.push(self.source.register_index.clone());
        for i in 0..=254 {
            register_vec.push(RegisterIndex(i));
        }
    }
    
    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        // TODO: deal with indirect
        if register_set.contains(&self.source.register_index) {
            register_set.insert(self.target.register_index.clone());
        } else {
            // Overwrite content of the target register a non-live register.
            register_set.remove(&self.target.register_index);
        }
    }
}

pub struct NodeMultiplyRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeMultiplyRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMultiplyRegister {
    fn formatted_instruction(&self) -> String {
        format!("mul {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = state.get_register_value_ref(&self.source);
        let value = perform_operation(state.check_value(), lhs, rhs)?;
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
    }
    
    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        if register_set.contains(&self.source) {
            register_set.insert(self.target.clone());
        }
    }
}

pub struct NodeMultiplyConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeMultiplyConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMultiplyConstant {
    fn formatted_instruction(&self) -> String {
        format!("mul {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let lhs: &RegisterValue = state.get_register_value_ref(&self.target);
        let rhs: &RegisterValue = &self.source;
        let value = perform_operation(state.check_value(), lhs, rhs)?;
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }

    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        if self.source.0.is_zero() {
            register_set.remove(&self.target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CheckValueLimitBits;

    fn process(left: i64, right: i64) -> String {
        let check_value: BoxCheckValue = Box::new(CheckValueLimitBits::new(32));
        let result = perform_operation(
            &check_value,
            &RegisterValue::from_i64(left),
            &RegisterValue::from_i64(right)
        );
        match result {
            Ok(value) => return value.to_string(),
            Err(EvalError::InputOutOfRange) => return "BOOM-INPUT".to_string(),
            Err(EvalError::OutputOutOfRange) => return "BOOM-OUTPUT".to_string(),
            Err(_) => return "BOOM-OTHER".to_string()
        }
    }

    #[test]
    fn test_10000() {
        assert_eq!(process(1, 1000), "1000");
        assert_eq!(process(1000, 1), "1000");
        assert_eq!(process(-1, -1), "1");
        assert_eq!(process(0, 0), "0");
        assert_eq!(process(0, 1), "0");
        assert_eq!(process(1, 0), "0");
        assert_eq!(process(1, 1), "1");
        assert_eq!(process(-500, 2), "-1000");
    }

    #[test]
    fn test_10001_out_of_range() {
        {
            assert_eq!(process(0x7fffffff, 1), "2147483647");
            assert_eq!(process(1, 0x7fffffff), "2147483647");
            assert_eq!(process(-1, -0x7fffffff), "2147483647");
            assert_eq!(process(0x7fffffff, -1), "-2147483647");
            assert_eq!(process(1, -0x7fffffff), "-2147483647");
        }
        {
            assert_eq!(process(0x80000000, 1), "BOOM-INPUT");
            assert_eq!(process(1, 0x80000000), "BOOM-INPUT");
            assert_eq!(process(-0x80000000, 1), "BOOM-INPUT");
            assert_eq!(process(1, -0x80000000), "BOOM-INPUT");
        }
        {
            assert_eq!(process(0x7fffffff, 2), "BOOM-OUTPUT");
            assert_eq!(process(2, 0x7fffffff), "BOOM-OUTPUT");
            assert_eq!(process(0x8000000, 0x10), "BOOM-OUTPUT");
            assert_eq!(process(0x10, 0x8000000), "BOOM-OUTPUT");
            assert_eq!(process(-0x10, -0x8000000), "BOOM-OUTPUT");
            assert_eq!(process(0x10, -0x8000000), "BOOM-OUTPUT");
        }
    }
}
