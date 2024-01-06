use loda_rust_core;

#[macro_use]
extern crate log;

extern crate env_logger;

#[macro_use]
extern crate assert_float_eq;

use std::{str::FromStr, path::PathBuf};
use regex::Regex;
use loda_rust_core::control::*;

#[cfg(feature = "loda-rust-arc")]
mod arc;

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
mod subcommand_arc;
mod subcommand_dependencies;
mod subcommand_evaluate;
mod subcommand_export_dataset;
mod subcommand_install;
mod subcommand_mine;
mod subcommand_pattern;
mod subcommand_similar;
mod subcommand_test;

use subcommand_analytics::SubcommandAnalytics;
use subcommand_arc::{SubcommandARC, SubcommandARCMode};
use subcommand_dependencies::subcommand_dependencies;
use subcommand_evaluate::{subcommand_evaluate,SubcommandEvaluateMode};
use subcommand_export_dataset::SubcommandExportDataset;
use subcommand_install::subcommand_install;
use subcommand_mine::{SubcommandMine,SubcommandMineMetricsMode};
use subcommand_pattern::SubcommandPattern;
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
    // tide::log::with_level(tide::log::LevelFilter::Trace);

    let matches = Command::new("loda-rust")
        .version(env!("CARGO_PKG_VERSION"))
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
            Command::new("analytics-oeis")
                .about("Prepare data needed for mining OEIS sequences, by analyzing the existing programs.")
                .hide(true)
        )
        .subcommand(
            Command::new("analytics-arc")
                .about("Prepare data needed for mining ARC puzzles, by analyzing the existing programs.")
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
        .subcommand(
            Command::new("arc-eval-task")
                .about("ARC - Eval a single task with all the existing solutions.")
                .hide(true)
                .arg(
                    Arg::new("pattern")
                        .help("File name of the task, it doesn't have to be the full name / path.")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("arc-check")
                .about("ARC - Check that all the existing solutions still works.")
                .hide(true)
        )
        .subcommand(
            Command::new("arc-generate-solution-csv")
                .about("ARC - Populate the 'solutions.csv' file by trying out all tasks with all solutions.")
                .hide(true)
        )
        .subcommand(
            Command::new("arc-competition")
                .about("ARC - The code being executed inside the docker image submitted for the `ARCathon` contest.")
                .hide(true)
        )
        .subcommand(
            Command::new("arc-label")
                .about("ARC - Traverse all tasks and classify each puzzle.")
                .hide(true)
        )
        .subcommand(
            Command::new("arc-export")
                .about("ARC - Export dataset for use as AI training data.")
                .hide(true)
        )
        .subcommand(
            Command::new("arc-solve")
                .about("ARC - Run a specific solver with all the tasks and check if the prediction are correct.")
                .hide(true)
                .arg(
                    Arg::new("nameofsolver")
                        .help("Name of the solver. lr = SolveLogisticRegression, one = SolveOneColor.")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("arc-size")
                .about("Predict the output sizes of a single ARC task.")
                .hide(true)
                .arg(
                    Arg::new("file")
                        .help("Absolute path to the task json file. Example: /home/arc-dataset/evaluation/0123abcd.json")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("arc-metadata-histograms")
                .about("Generate metadata with histogram comparisons. Traverse all ARC task json files recursively with the provided directory path.")
                .hide(true)
                .arg(
                    Arg::new("count")
                        .help("Number of histogram items to insert into the metadata. default: 1")
                        .long("count")
                        .takes_value(true)
                )
                .arg(
                    Arg::new("seed")
                        .help("Random seed is a 64 bit unsigned integer. default: 0")
                        .long("seed")
                        .takes_value(true)
                )
                .arg(
                    Arg::new("directory")
                        .help("Absolute path to the directory containing ARC task json files. Example: /home/arc-dataset/evaluation")
                        .long("directory")
                        .required(true)
                        .takes_value(true)
                )
        )
        .subcommand(
            Command::new("arc-web")
                .about("Web server with UI for ARC.")
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

    if let Some(_sub_m) = matches.subcommand_matches("analytics-oeis") {
        SubcommandAnalytics::oeis()?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("analytics-arc") {
        SubcommandAnalytics::arc()?;
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

    if let Some(sub_m) = matches.subcommand_matches("arc-eval-task") {
        let pattern_raw: &str = sub_m.value_of("pattern").expect("pattern");
        let re = Regex::new("^[a-fA-F0-9]+$").unwrap();
        let captures = match re.captures(pattern_raw) {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("Unable to parse pattern, expected hexadecimal text ala \"a7f2\" or \"f00d\""));
            }
        };
        let capture0: &str = captures.get(0).map_or("", |m| m.as_str());
        let pattern_string: String = capture0.to_string();
        let mode = SubcommandARCMode::EvalSingleTask { pattern: pattern_string };
        let blocking_task = tokio::task::spawn_blocking(|| {
            SubcommandARC::run(mode).expect("ok");
        });
        blocking_task.await?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("arc-check") {
        let blocking_task = tokio::task::spawn_blocking(|| {
            SubcommandARC::run(SubcommandARCMode::CheckAllExistingSolutions).expect("ok");
        });
        blocking_task.await?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("arc-generate-solution-csv") {
        SubcommandARC::run(SubcommandARCMode::GenerateSolutionCSV)?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("arc-competition") {
        SubcommandARC::run(SubcommandARCMode::Competition)?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("arc-label") {
        // For logging LODA-RUST uses reqwest in blocking mode, performing a POST with the log message.
        // When running in DEBUG mode, then reqwest doesn't work on the main thread, and panics with this message:
        // thread 'main' panicked at 'Cannot drop a runtime in a context where blocking is not allowed. 
        // This happens when a runtime is dropped from within an asynchronous context.
        //
        // When running in RELEASE mode then reqwest works.
        //
        // When running inside unittests then reqwest works.
        let blocking_task = tokio::task::spawn_blocking(|| {
            SubcommandARC::run(SubcommandARCMode::LabelAllPuzzles).expect("ok");
        });
        blocking_task.await?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("arc-export") {
        SubcommandARC::run(SubcommandARCMode::ExportDataset)?;
        return Ok(());
    }

    if let Some(sub_m) = matches.subcommand_matches("arc-solve") {
        let nameofsolver_raw: &str = sub_m.value_of("nameofsolver").expect("nameofsolver");
        let mode = SubcommandARCMode::SolveWithSpecificSolver { name_of_solver: nameofsolver_raw.to_string() };
        let blocking_task = tokio::task::spawn_blocking(|| {
            SubcommandARC::run(mode).expect("ok");
        });
        blocking_task.await?;
        return Ok(());
    }

    if let Some(sub_m) = matches.subcommand_matches("arc-size") {
        let path_raw: &str = sub_m.value_of("file").expect("path to task json file");
        let task_json_file: PathBuf = PathBuf::from(path_raw);
        let mode = SubcommandARCMode::PredictOutputSizesForSingleTask { task_json_file };
        SubcommandARC::run(mode)?;
        return Ok(());
    }

    if let Some(sub_m) = matches.subcommand_matches("arc-metadata-histograms") {
        let path_raw: &str = sub_m.value_of("directory").expect("path to directory containing task json files");
        let task_json_directory: PathBuf = PathBuf::from(path_raw);
        let count: u16 = match sub_m.value_of("count") {
            Some(raw) => {
                let count: u16 = raw.parse()
                    .map_err(|e| anyhow::anyhow!("Unable to parse count as u16, error: {:?}", e))?;
                count
            },
            None => 1
        };
        let seed: u64 = match sub_m.value_of("seed") {
            Some(raw) => {
                let seed: u64 = raw.parse()
                    .map_err(|e| anyhow::anyhow!("Unable to parse seed as u64, error: {:?}", e))?;
                seed
            },
            None => 0
        };
        let mode = SubcommandARCMode::MetadataHistogram { count, seed, task_json_directory };
        let blocking_task = tokio::task::spawn_blocking(|| {
            SubcommandARC::run(mode).expect("ok");
        });
        blocking_task.await?;
        return Ok(());
    }

    if let Some(_sub_m) = matches.subcommand_matches("arc-web") {
        SubcommandARC::run_web_server().await?;
        return Ok(());
    }

    panic!("No arguments provided");
}
