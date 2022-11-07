//! The `loda-rust similar` subcommand, identifies similar programs.
use crate::similar::Similar;

pub fn subcommand_similar() -> anyhow::Result<()> {
    Similar::run()
}
