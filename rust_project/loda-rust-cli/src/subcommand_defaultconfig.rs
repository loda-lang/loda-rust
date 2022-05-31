use loda_rust_core::config::Config;

pub fn subcommand_defaultconfig() {
    println!("{}", Config::default_config());
}
