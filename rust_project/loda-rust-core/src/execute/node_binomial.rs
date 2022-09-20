#[derive(Clone)]
pub enum NodeBinomialLimit {
    Unlimited,
    LimitN(u8)
}
