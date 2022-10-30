use crate::config::Config;
use super::{CheckFixedLengthSequence, Funnel, NamedCacheFile};
use std::path::{Path, PathBuf};

pub fn create_funnel(config: &Config) -> Funnel {
    let analytics_dir: PathBuf = config.analytics_dir();
    let filename10: &str = NamedCacheFile::Funnel10All.filename();
    let filename20: &str = NamedCacheFile::Funnel20All.filename();
    let filename30: &str = NamedCacheFile::Funnel30All.filename();
    let filename40: &str = NamedCacheFile::Funnel40All.filename();
    let path10 = analytics_dir.join(Path::new(filename10));
    let path20 = analytics_dir.join(Path::new(filename20));
    let path30 = analytics_dir.join(Path::new(filename30));
    let path40 = analytics_dir.join(Path::new(filename40));
    let checker10: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path10);
    let checker20: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path20);
    let checker30: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path30);
    let checker40: CheckFixedLengthSequence = CheckFixedLengthSequence::load(&path40);
    Funnel::new(
        checker10,
        checker20,
        checker30,
        checker40,
    )
}
