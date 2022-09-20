#[derive(Clone)]
pub enum NodePowerLimit {
    Unlimited,
    LimitBits(u32)
}
