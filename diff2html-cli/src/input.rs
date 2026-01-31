//! Input handling for diff2html CLI.
//!
//! This module handles reading diff input from various sources:
//! - File: Read from a file path
//! - Stdin: Read from standard input
//! - Command: Execute `git diff` with arguments

use std::io::{self, Read};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::args::InputType;

/// Default git diff arguments when none are provided.
const DEFAULT_GIT_ARGS: &[&str] = &["-M", "-C", "HEAD"];

/// Get diff input based on input type.
pub fn get_input(
    input_type: InputType,
    extra_args: &[String],
    ignore: &[String],
) -> Result<String> {
    match input_type {
        InputType::File => read_file(extra_args),
        InputType::Stdin => read_stdin(),
        InputType::Command => run_git_diff(extra_args, ignore),
    }
}

/// Read diff from a file.
fn read_file(extra_args: &[String]) -> Result<String> {
    let file_path = extra_args
        .first()
        .context("No file path provided. Use: diff2html -i file -- <path>")?;

    std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))
}

/// Read diff from stdin.
fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;
    Ok(buffer)
}

/// Run git diff command and return its output.
fn run_git_diff(extra_args: &[String], ignore: &[String]) -> Result<String> {
    let git_args = generate_git_diff_args(extra_args, ignore);

    let output = Command::new("git")
        .args(&git_args)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff failed: {}", stderr.trim());
    }

    String::from_utf8(output.stdout).context("git diff output is not valid UTF-8")
}

/// Generate git diff arguments from user input.
fn generate_git_diff_args(extra_args: &[String], ignore: &[String]) -> Vec<String> {
    let mut args = vec!["diff".to_string()];

    // Add --no-color if not already present
    if !extra_args.iter().any(|a| a == "--no-color") {
        args.push("--no-color".to_string());
    }

    // Add user-provided arguments or defaults
    if extra_args.is_empty() {
        args.extend(DEFAULT_GIT_ARGS.iter().map(|s| s.to_string()));
    } else {
        args.extend(extra_args.iter().cloned());
    }

    // Add ignore patterns
    if !ignore.is_empty() {
        // Add -- separator if not already present
        if !extra_args.iter().any(|a| a == "--") {
            args.push("--".to_string());
        }
        for path in ignore {
            args.push(format!(":(exclude){}", path));
        }
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_git_diff_args_default() {
        let args = generate_git_diff_args(&[], &[]);
        assert_eq!(args, vec!["diff", "--no-color", "-M", "-C", "HEAD"]);
    }

    #[test]
    fn test_generate_git_diff_args_with_extra_args() {
        let extra = vec!["HEAD~1".to_string()];
        let args = generate_git_diff_args(&extra, &[]);
        assert_eq!(args, vec!["diff", "--no-color", "HEAD~1"]);
    }

    #[test]
    fn test_generate_git_diff_args_no_color_already_present() {
        let extra = vec!["--no-color".to_string(), "HEAD".to_string()];
        let args = generate_git_diff_args(&extra, &[]);
        assert_eq!(args, vec!["diff", "--no-color", "HEAD"]);
    }

    #[test]
    fn test_generate_git_diff_args_with_ignore() {
        let extra = vec!["HEAD".to_string()];
        let ignore = vec!["package-lock.json".to_string(), "yarn.lock".to_string()];
        let args = generate_git_diff_args(&extra, &ignore);
        assert_eq!(
            args,
            vec![
                "diff",
                "--no-color",
                "HEAD",
                "--",
                ":(exclude)package-lock.json",
                ":(exclude)yarn.lock"
            ]
        );
    }

    #[test]
    fn test_generate_git_diff_args_with_separator_present() {
        let extra = vec!["HEAD".to_string(), "--".to_string(), "src/".to_string()];
        let ignore = vec!["node_modules".to_string()];
        let args = generate_git_diff_args(&extra, &ignore);
        // Should not add another --
        assert_eq!(
            args,
            vec![
                "diff",
                "--no-color",
                "HEAD",
                "--",
                "src/",
                ":(exclude)node_modules"
            ]
        );
    }
}
