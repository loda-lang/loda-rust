#[derive(Clone)]
pub enum NodeLoopLimit {
    Unlimited,
    LimitCount(u32)
}
