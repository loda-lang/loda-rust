use super::KeyMetricU32;

#[derive(Debug)]
pub enum MinerThreadMessageToCoordinator {
    ReadyForMining,
    MetricU32(KeyMetricU32, u32),
}
