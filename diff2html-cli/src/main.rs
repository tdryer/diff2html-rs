#![forbid(unsafe_code)]

//! diff2html CLI - Generate HTML from unified diffs.
//!
//! A command-line tool for converting unified diff output to HTML.
//! Supports multiple input sources, output formats, and viewing options.

mod args;
mod config;
mod input;
mod output;

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use args::{Args, OutputType};
use config::parse_args;
use input::get_input;
use output::{get_output, preview, write_file};

/// Exit codes matching the original TypeScript implementation.
mod exit_codes {
    pub const SUCCESS: u8 = 0;
    pub const ERROR: u8 = 1;
    pub const EMPTY_INPUT: u8 = 3;
}

fn run() -> Result<u8> {
    let args = Args::parse();
    let (diff2html_config, cli_config) = parse_args(&args)?;

    // Get input from specified source
    let input = get_input(
        cli_config.input_type,
        &cli_config.extra_args,
        &cli_config.ignore,
    )?;

    // Check for empty input
    if input.trim().is_empty() {
        eprintln!(
            "The input is empty. Try piping diff output to diff2html or specify input arguments."
        );
        return Ok(exit_codes::EMPTY_INPUT);
    }

    // Generate output
    let content = get_output(&diff2html_config, &cli_config, &input)?;

    // Write output to appropriate destination
    if let Some(ref file_path) = cli_config.output_file {
        write_file(file_path, &content)?;
        eprintln!("Output written to: {}", file_path);
    } else {
        match cli_config.output_type {
            OutputType::Preview => {
                preview(&content, cli_config.format_type)?;
            }
            OutputType::Stdout => {
                println!("{}", content);
            }
        }
    }

    Ok(exit_codes::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("Error: {:#}", e);
            ExitCode::from(exit_codes::ERROR)
        }
    }
}
