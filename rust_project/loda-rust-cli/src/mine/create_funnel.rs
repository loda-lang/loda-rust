use crate::config::{Config, MinerFilterMode};
use super::{CheckFixedLengthSequence, Funnel, NamedCacheFile};
use std::path::PathBuf;

pub trait CreateFunnel {
    fn create_funnel_with_file_data(config: &Config) -> Funnel;
    fn create_empty_funnel() -> Funnel;
}

impl CreateFunnel for Funnel {
    fn create_funnel_with_file_data(config: &Config) -> Funnel {
        let analytics_dir: PathBuf = config.analytics_dir();
        let names: [NamedCacheFile; 4] = match config.miner_filter_mode() {
            MinerFilterMode::All => NamedCacheFile::group_all(),
            MinerFilterMode::New => NamedCacheFile::group_new(),
        };
        let funnel10_path: PathBuf = names[0].resolve_path(&analytics_dir);
        let funnel20_path: PathBuf = names[1].resolve_path(&analytics_dir);
        let funnel30_path: PathBuf = names[2].resolve_path(&analytics_dir);
        let funnel40_path: PathBuf = names[3].resolve_path(&analytics_dir);
        let checker10: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&funnel10_path);
        let checker20: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&funnel20_path);
        let checker30: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&funnel30_path);
        let checker40: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&funnel40_path);
        Funnel::new(
            checker10,
            checker20,
            checker30,
            checker40,
        )
    }

    fn create_empty_funnel() -> Funnel {
        let checker10: CheckFixedLengthSequence = CheckFixedLengthSequence::new_empty();
        let checker20: CheckFixedLengthSequence = CheckFixedLengthSequence::new_empty();
        let checker30: CheckFixedLengthSequence = CheckFixedLengthSequence::new_empty();
        let checker40: CheckFixedLengthSequence = CheckFixedLengthSequence::new_empty();
        Funnel::new(
            checker10,
            checker20,
            checker30,
            checker40,
        )
    }
}
