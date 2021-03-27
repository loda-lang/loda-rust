#[macro_use]
extern crate log;

extern crate env_logger;

use std::str::FromStr;
use dotenv::dotenv;

mod config;
mod control;
mod execute;
mod mine;
mod parser;
mod oeis;
mod util;
use control::{Settings, subcommand_dependencies, subcommand_evaluate, subcommand_install, subcommand_mine, subcommand_update};

extern crate clap;
extern crate num_bigint;
extern crate num_traits;

use clap::{App, AppSettings, Arg, SubCommand};

fn main() {
    // Prepare environment variables from the `.env` file.
    dotenv().expect("Failed to read .env file");

    // Initialize logging from the `RUST_LOG` environment variable.
    env_logger::init();

    // Load settings from various environment variables.
    let settings = Settings::new();

    let matches = App::new("loda_lab")
        .version("0.0.1")
        .about("Experimental tool")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("evaluate")
                .alias("eval")
                .about("Evaluate a program")
                .arg(
                    Arg::with_name("programid")
                        .required(true)
                )
                .arg(
                    Arg::with_name("terms")
                        .help("Number of sequence terms (default:20)")
                        .takes_value(true)
                        .short("t")
                        .long("terms")
                )
                .arg(
                    Arg::with_name("instructions")
                        .help("Show the assembler instructions as they are being executed")
                        .long("instructions")
                )
        )
        .subcommand(
            SubCommand::with_name("dependencies")
                .alias("deps")
                .about("Dependencies of a program")
                .arg(
                    Arg::with_name("programid")
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Create the $HOME/.loda-lab directory")
        )
        // Experiments with mining new programs
        .subcommand(
            SubCommand::with_name("update")
                .about("Prepare caching files used by validation")
        )
        .subcommand(
            SubCommand::with_name("mine")
                .about("Experimental: Come up with new programs")
        )
        .get_matches();

    if let Some(sub_m) = matches.subcommand_matches("evaluate") {
        let program_id_raw: &str = sub_m.value_of("programid").unwrap();
        let program_id: u64 = u64::from_str(program_id_raw)
            .expect("Unable to parse program_id.");

        let mut number_of_terms: u64 = 20;
        if let Some(number_of_terms_raw) = sub_m.value_of("terms") {
            number_of_terms = u64::from_str(number_of_terms_raw)
                .expect("Unable to parse number of terms.");
        }
        let show_instructions: bool = sub_m.is_present("instructions");
        subcommand_evaluate(program_id, number_of_terms, show_instructions);
        return;
    }

    if let Some(sub_m) = matches.subcommand_matches("dependencies") {
        let program_id_raw: &str = sub_m.value_of("programid").unwrap();
        let program_id: u64 = u64::from_str(program_id_raw)
            .expect("Unable to parse program_id.");
        subcommand_dependencies(&settings, program_id);
        return;
    }

    if let Some(_sub_m) = matches.subcommand_matches("install") {
        subcommand_install(&settings);
        return;
    }

    if let Some(_sub_m) = matches.subcommand_matches("update") {
        subcommand_update(&settings);
        return;
    }

    if let Some(_sub_m) = matches.subcommand_matches("mine") {
        subcommand_mine(&settings);
        return;
    }

    panic!("No arguments provided");
}
