#[derive(Debug)]
pub enum MinerThreadMessageToCoordinator {
    NumberOfIterations(u64),
}
