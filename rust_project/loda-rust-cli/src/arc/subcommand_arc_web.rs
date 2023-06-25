
pub struct SubcommandARCWeb;

impl SubcommandARCWeb {
    /// The `arc-web` subcommand when invoked from the command line.
    /// 
    /// This starts a web server, where a human can explore the ARC data.
    pub async fn run_web_server() -> anyhow::Result<()> {
        println!("Starting the web server...");
        Ok(())
    }
}
