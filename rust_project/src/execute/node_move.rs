use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};
use std::collections::HashSet;

pub struct NodeMoveRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeMoveRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMoveRegister {
    fn formatted_instruction(&self) -> String {
        format!("mov {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let value: &RegisterValue = state.get_register_value_ref(&self.source);
        let tmp_value: RegisterValue = value.clone();
        state.set_register_value(self.target.clone(), tmp_value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
    }
    
    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        if register_set.contains(&self.source) {
            register_set.insert(self.target.clone());
        } else {
            // Overwrite content of the target register a non-live register.
            register_set.remove(&self.target);
        }
    }
}

pub struct NodeMoveConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeMoveConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMoveConstant {
    fn formatted_instruction(&self) -> String {
        format!("mov {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let value: RegisterValue = self.source.clone();
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }
    
    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        // Overwrite content of the target register a non-live register.
        register_set.remove(&self.target);
    }
}
