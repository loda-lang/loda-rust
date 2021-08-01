use super::{BoxCheckValue, CheckValueUnlimited, CheckValueLimitBits};

#[derive(Clone)]
pub enum NodeRegisterLimit {
    Unlimited,
    LimitBits(u32)
}

impl NodeRegisterLimit {
    pub fn create_boxed_check_value(&self) -> BoxCheckValue {
        match self {
            NodeRegisterLimit::Unlimited => 
                Box::new(CheckValueUnlimited::new()),
            NodeRegisterLimit::LimitBits(max_bits) => 
                Box::new(CheckValueLimitBits::new(*max_bits)),
        }
    }
}
