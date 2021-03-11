use super::{Node,RegisterIndex,RegisterValue,ProgramState};

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
    fn shorthand(&self) -> &str {
        "mov register"
    }

    fn formatted_instruction(&self) -> String {
        format!("mov {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) {
        let value: RegisterValue = state.get_register_value(self.source.clone());
        state.set_register_value(self.target.clone(), value);
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
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
    fn shorthand(&self) -> &str {
        "mov constant"
    }

    fn formatted_instruction(&self) -> String {
        format!("mov {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState) {
        let value: RegisterValue = self.source.clone();
        state.set_register_value(self.target.clone(), value);
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }
}
