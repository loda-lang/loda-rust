//! The `loda-rust arc` subcommand, perform ARC Challenge experiments.

pub struct SubcommandARC {}

impl SubcommandARC {
    pub async fn run() -> anyhow::Result<()> {
        println!("hello");
        Ok(())
    }
}
