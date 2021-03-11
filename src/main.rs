use std::str::FromStr;
use dotenv::dotenv;

mod parser;
mod execute;
mod control;
use control::{Settings, subcommand_dependencies, subcommand_evaluate};

extern crate clap;
extern crate num_bigint;
extern crate num_traits;

use clap::{App, AppSettings, Arg, SubCommand};

fn main() {
    dotenv().expect("Failed to read .env file");
    let settings = Settings::new();

    let matches = App::new("loda-lab")
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
        subcommand_evaluate(&settings, program_id, number_of_terms, show_instructions);
        return;
    }

    if let Some(sub_m) = matches.subcommand_matches("dependencies") {
        let program_id_raw: &str = sub_m.value_of("programid").unwrap();
        let program_id: u64 = u64::from_str(program_id_raw)
            .expect("Unable to parse program_id.");
        subcommand_dependencies(&settings, program_id);
        return;
    }

    panic!("No arguments provided");
}
