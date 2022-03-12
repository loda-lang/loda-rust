#[derive(Debug)]
pub enum MinerThreadMessageToCoordinator {
    ReadyForMining,
    NumberOfIterations(u64),
}
