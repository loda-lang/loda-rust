use lodalab_core;

#[macro_use]
extern crate log;

extern crate env_logger;

use std::str::FromStr;

use lodalab_core::control::*;

mod subcommand_evaluate;
use subcommand_evaluate::{subcommand_evaluate,SubcommandEvaluateMode};

extern crate clap;
extern crate num_bigint;
extern crate num_traits;

use clap::{App, AppSettings, Arg, SubCommand};

fn main() {
    // Initialize logging from the `RUST_LOG` environment variable.
    env_logger::init();

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
                    Arg::with_name("steps")
                        .help("Show the number of steps used for computing a term")
                        .long("steps")
                )
                .arg(
                    Arg::with_name("debug")
                        .help("Inspect the internal state during execute")
                        .long("debug")
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
        let show_steps: bool = sub_m.is_present("steps");
        let show_debug: bool = sub_m.is_present("debug");
        let mode: SubcommandEvaluateMode = match (show_debug, show_steps) {
            (false,false) => SubcommandEvaluateMode::PrintTerms,
            (false,true) => SubcommandEvaluateMode::PrintSteps,
            (true,false) => SubcommandEvaluateMode::PrintDebug,
            (true,true) => {
                panic!("Invalid combo of parameters");
            }
        };
        subcommand_evaluate(program_id, number_of_terms, mode);
        return;
    }

    if let Some(sub_m) = matches.subcommand_matches("dependencies") {
        let program_id_raw: &str = sub_m.value_of("programid").unwrap();
        let program_id: u64 = u64::from_str(program_id_raw)
            .expect("Unable to parse program_id.");
        subcommand_dependencies(program_id);
        return;
    }

    if let Some(_sub_m) = matches.subcommand_matches("install") {
        subcommand_install();
        return;
    }

    if let Some(_sub_m) = matches.subcommand_matches("update") {
        // TODO: subcommand_update();
        return;
    }

    if let Some(_sub_m) = matches.subcommand_matches("mine") {
        // TODO: subcommand_mine();
        return;
    }

    panic!("No arguments provided");
}
