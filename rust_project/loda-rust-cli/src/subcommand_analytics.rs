//! The `loda-rust analytics-MODE` subcommands, prepares data for mining such as histograms/bloomfilters.
use crate::analytics::Analytics;

pub struct SubcommandAnalytics;

impl SubcommandAnalytics {
    pub fn oeis() -> anyhow::Result<()> {
        Analytics::oeis_run_force()
    }

    pub fn arc() -> anyhow::Result<()> {
        Analytics::arc_run_force()
    }
}
