use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterValue};

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
}
