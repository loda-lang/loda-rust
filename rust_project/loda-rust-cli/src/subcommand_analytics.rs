//! The `loda-rust analytics` subcommand, populates histograms/bloomfilters.
use crate::analytics::Analytics;

pub fn subcommand_analytics() -> anyhow::Result<()> {
    // TODO: switch between ARC and OEIS based on command line parameter
    Analytics::arc_run_force()
    // Analytics::oeis_run_force()
}
