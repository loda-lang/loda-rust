#[derive(Clone)]
pub enum NodeLoopLimit {
    Unlimited,
    LimitCount(u32)
}

/// Prevent extreme values, a loop with more than 256 (2^8) registers is extreme
///
/// The usual size is 1, which looks like this:
/// `lpb $0`
/// 
/// Sometimes the range can be 2 or more. Looking like this:
/// `lpb $0,2`
/// `lpb $0,5`
pub const LOOP_RANGE_MAX_BITS: u64 = 8;
