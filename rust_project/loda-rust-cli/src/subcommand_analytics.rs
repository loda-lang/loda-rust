//! The `loda-rust analytics` subcommand, populates histograms/bloomfilters.
use crate::analytics::Analytics;

pub fn subcommand_analytics() -> anyhow::Result<()> {
    Analytics::run()
}
