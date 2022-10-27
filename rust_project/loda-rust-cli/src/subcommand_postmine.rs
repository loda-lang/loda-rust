//! The `loda-rust postmine` subcommand, checks the mined programs for correctness and performance.
use crate::postmine::PostMine;

pub fn subcommand_postmine() -> anyhow::Result<()> {
    PostMine::run()
}
