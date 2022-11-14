use loda_rust_core;

#[macro_use]
extern crate log;

extern crate env_logger;

use std::str::FromStr;
use regex::Regex;
use loda_rust_core::control::*;

mod analytics;
mod common;
mod config;
mod lodacpp;
mod mine;
mod oeis;
mod pattern;
mod postmine;
mod similar;
mod subcommand_analytics;
mod subcommand_dependencies;
mod subcommand_evaluate;
mod subcommand_export_dataset;
mod subcommand_install;
mod subcommand_mine;
mod subcommand_pattern;
mod subcommand_postmine;
mod subcommand_similar;
mod subcommand_test;

use subcommand_analytics::subcommand_analytics;
use subcommand_dependencies::subcommand_dependencies;
use subcommand_evaluate::{subcommand_evaluate,SubcommandEvaluateMode};
use subcommand_export_dataset::SubcommandExportDataset;
use subcommand_install::subcommand_install;
use subcommand_mine::{SubcommandMine,SubcommandMineMetricsMode};
use subcommand_pattern::SubcommandPattern;
use subcommand_postmine::subcommand_postmine;
use subcommand_similar::subcommand_similar;
use subcommand_test::SubcommandTest;

extern crate clap;
extern crate num_bigint;
extern crate num_traits;

use clap::{Arg, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging from the `RUST_LOG` environment variable.
    env_logger::init();

    let matches = Command::new("loda-rust")
        .version("0.0.1")
        .about("Experimental tool")
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("evaluate")
                .alias("eval")
                .about("Evaluate a program")
                .arg(
                    Arg::new("programid")
                        .required(true)
                )
                .arg(
                    Arg::new("terms")
                        .help("Number of sequence terms (default:20)")
                        .takes_value(true)
                        .short('t')
                        .long("terms")
                )
                .arg(
                    Arg::new("steps")
                        .help("Show the number of steps used for computing a term")
                        .long("steps")
                )
                .arg(
                    Arg::new("debug")
                        .help("Inspect the internal state during execute")
                        .long("debug")
                )
        )
        .subcommand(
            Command::new("dependencies")
                .alias("deps")
                .about("Print all direct/indirect dependencies of a program")
                .arg(
                    Arg::new("programid")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("install")
                .about("Create the $HOME/.loda-rust directory")
        )
        .subcommand(
            Command::new("analytics")
                .about("Prepare data needed for mining, by analyzing the existing programs.")
                .hide(true)
        )
        .subcommand(
            Command::new("mine")
                .about("Run the miner daemon process. Press CTRL-C to stop it.")
                .arg(
                    Arg::new("metrics")
                        .long("metrics")
                        .help("Run a metrics server on localhost:8090 (can be overwritten in the config file)")
                )
        )
        .subcommand(
            Command::new("postmine")
                .about("Validate the accumulated candiate programs for correctness and performance.")
                .hide(true)
        )
        .subcommand(
            Command::new("similar")
                .about("Identify similar programs.")
        )
        .subcommand(
            Command::new("pattern")
                .about("Identify recurring patterns among similar programs.")
                .arg(
                    Arg::new("verbose")
                        .help("Append verbose details to the patterns.")
                        .long("verbose")
                )
        )
        .subcommand(
            Command::new("export-dataset")
                .about("Generates a .csv file with terms and programs, for use as AI training data.")
                .hide(true)
        )
        .subcommand(
            Command::new("test-integration-with-lodacpp")
                .about("Verify that integration with the 'lodacpp' executable is working.")
                .hide(true)
        )
        .get_matches();

    if let Some(sub_m) = matches.subcommand_matches("evaluate") {
        // Fuzzy convert from user input to OEIS id, allows the 'A' to be left out.
        let program_id_raw: &str = sub_m.value_of("programid").unwrap();
        let re = Regex::new("^A?(\\d+)$").unwrap();
        let captures = match re.captures(program_id_raw) {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Unable to extract OEIS id, expected A number such as A000040 or A123456."));
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let program_id_string: String = capture1.to_string();
        let program_id: u64 = program_id_string.parse()
            .map_err(|e| anyhow::anyhow!("Unable to parse OEIS id as u64, expected A number such as A000040 or A123456. error: {:?}", e))?;

        // Number of terms
        let mut number_of_terms: u64 = 20;
        if let Some(number_of_terms_raw) = sub_m.value_of("terms") {
            number_of_terms = u64::from_str(number_of_terms_raw)
                .expect("Unable to parse number of terms.");
        }

        // Eval mode
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
        subcommand_analytics()?;
        return Ok(());
    }

    if let Some(sub_m) = matches.subcommand_matches("mine") {
        let metrics: bool = sub_m.is_present("metrics");
        let metrics_mode: SubcommandMineMetricsMode = match metrics {
            true => SubcommandMineMetricsMode::RunMetricsServer,
            false => SubcommandMineMetricsMode::NoMetricsServer
        };
        SubcommandMine::run(metrics_mode).await?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("postmine") {
        subcommand_postmine()?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("similar") {
        subcommand_similar()?;
        return Ok(());
    }

    if let Some(sub_m) = matches.subcommand_matches("pattern") {
        let append_verbose_details: bool = sub_m.is_present("verbose");
        SubcommandPattern::run(append_verbose_details);
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("export-dataset") {
        SubcommandExportDataset::export_dataset()?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("test-integration-with-lodacpp") {
        SubcommandTest::test_integration_with_lodacpp()?;
        return Ok(());
    }

    panic!("No arguments provided");
}
