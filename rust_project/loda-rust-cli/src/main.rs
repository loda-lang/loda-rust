use loda_rust_core;

#[macro_use]
extern crate log;

extern crate env_logger;

use std::str::FromStr;

use loda_rust_core::control::*;

mod analytics;
mod common;
mod config;
mod mine;
mod oeis;
mod pattern;
mod similar;
mod subcommand_analytics;
mod subcommand_dependencies;
mod subcommand_evaluate;
mod subcommand_install;
mod subcommand_mine;
mod subcommand_pattern;
mod subcommand_similar;

use subcommand_analytics::subcommand_analytics;
use subcommand_dependencies::subcommand_dependencies;
use subcommand_evaluate::{subcommand_evaluate,SubcommandEvaluateMode};
use subcommand_install::subcommand_install;
use subcommand_mine::{SubcommandMine,SubcommandMineMetricsMode};
use subcommand_pattern::subcommand_pattern;
use subcommand_similar::subcommand_similar;

extern crate clap;
extern crate num_bigint;
extern crate num_traits;

use clap::{App, AppSettings, Arg, SubCommand};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize logging from the `RUST_LOG` environment variable.
    env_logger::init();

    let matches = App::new("loda-rust")
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
                        .short('t')
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
                .about("Print all direct/indirect dependencies of a program")
                .arg(
                    Arg::with_name("programid")
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Create the $HOME/.loda-rust directory")
        )
        .subcommand(
            SubCommand::with_name("analytics")
                .about("Prepare data needed for mining, by analyzing the existing programs.")
        )
        .subcommand(
            SubCommand::with_name("mine")
                .about("Run the miner daemon process. Press CTRL-C to stop it.")
                .arg(
                    Arg::with_name("metrics")
                        .long("metrics")
                        .help("Run a metrics server on localhost:8090 (can be overwritten in the config file)")
                )
        )
        .subcommand(
            SubCommand::with_name("similar")
                .about("Identify similar programs.")
        )
        .subcommand(
            SubCommand::with_name("pattern")
                .about("Identify recurring patterns among similar programs.")
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
        return Ok(());
    }

    if let Some(sub_m) = matches.subcommand_matches("dependencies") {
        let program_id_raw: &str = sub_m.value_of("programid").unwrap();
        let program_id: u64 = u64::from_str(program_id_raw)
            .expect("Unable to parse program_id.");
        subcommand_dependencies(program_id);
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("install") {
        subcommand_install();
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("analytics") {
        return subcommand_analytics();
    }

    if let Some(sub_m) = matches.subcommand_matches("mine") {
        let metrics: bool = sub_m.is_present("metrics");
        let metrics_mode: SubcommandMineMetricsMode = match metrics {
            true => SubcommandMineMetricsMode::RunMetricsServer,
            false => SubcommandMineMetricsMode::NoMetricsServer
        };
        let mut instance = SubcommandMine::new(metrics_mode);
        instance.check_prerequisits()?;
        instance.print_info();
        instance.populate_prevent_flooding_mechanism()?;
        instance.run().await?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("similar") {
        subcommand_similar();
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("pattern") {
        subcommand_pattern();
        return Ok(());
    }

    panic!("No arguments provided");
}
