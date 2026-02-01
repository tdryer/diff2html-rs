//! CLI argument parsing using Clap.
//!
//! This module defines all command-line arguments for the diff2html CLI tool.

use clap::{Parser, ValueEnum};

/// CLI arguments for diff2html.
#[derive(Parser, Debug)]
#[command(
    name = "diff2html",
    about = "Generate HTML from unified diffs",
    version
)]
pub struct Args {
    /// Output style
    #[arg(short = 's', long, value_enum, default_value = "line")]
    pub style: StyleType,

    /// Diff highlighting style
    #[arg(short = 'd', long = "diffStyle", value_enum, default_value = "word")]
    pub diff_style: DiffStyleType,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "html")]
    pub format: FormatType,

    /// Diff input source
    #[arg(short = 'i', long, value_enum, default_value = "command")]
    pub input: InputType,

    /// Output destination
    #[arg(short = 'o', long, value_enum, default_value = "preview")]
    pub output: OutputType,

    /// Send output to file (overrides output option)
    #[arg(short = 'F', long)]
    pub file: Option<String>,

    /// Page title for HTML output
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Color scheme of HTML output
    #[arg(long = "colorScheme", value_enum, default_value = "auto")]
    pub color_scheme: ColorSchemeType,

    /// Show files summary
    #[arg(long, value_enum, default_value = "closed")]
    pub summary: SummaryType,

    /// Diff line matching type
    #[arg(long, value_enum, default_value = "none")]
    pub matching: LineMatchingType,

    /// Diff line matching word threshold
    #[arg(long = "matchWordsThreshold", default_value = "0.25")]
    pub match_words_threshold: f64,

    /// Maximum line comparisons of a block of changes
    #[arg(long = "matchingMaxComparisons", default_value = "1000")]
    pub matching_max_comparisons: usize,

    /// Number of changed lines after which a file diff is deemed as too big
    #[arg(long = "diffMaxChanges")]
    pub diff_max_changes: Option<u32>,

    /// Number of characters in a diff line after which a file diff is deemed as too big
    #[arg(long = "diffMaxLineLength")]
    pub diff_max_line_length: Option<usize>,

    /// Render nothing if the diff shows no change
    #[arg(long = "renderNothingWhenEmpty")]
    pub render_nothing_when_empty: bool,

    /// Maximum number of characters of the bigger line in a block to apply comparison
    #[arg(long = "maxLineSizeInBlockForComparison", default_value = "200")]
    pub max_line_size_in_block_for_comparison: usize,

    /// Maximum number of characters in a line to apply highlight
    #[arg(long = "maxLineLengthHighlight", default_value = "10000")]
    pub max_line_length_highlight: usize,

    /// Show viewed checkbox to toggle file content
    #[arg(long = "fileContentToggle", default_value = "true")]
    pub file_content_toggle: bool,

    /// Synchronised horizontal scroll for side-by-side view
    #[arg(long = "synchronisedScroll", default_value = "true")]
    pub synchronised_scroll: bool,

    /// Enable syntax highlighting
    #[arg(long = "highlightCode", default_value = "true")]
    pub highlight_code: bool,

    /// Use a custom template when generating markup
    #[arg(long = "htmlWrapperTemplate")]
    pub html_wrapper_template: Option<String>,

    /// Files to exclude from diff
    #[arg(long = "ignore", short = 'g', action = clap::ArgAction::Append)]
    pub ignore: Vec<String>,

    /// Extra arguments passed to git diff (after --)
    #[arg(last = true)]
    pub extra_args: Vec<String>,
}

/// Output style type
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StyleType {
    /// Line-by-line view
    Line,
    /// Side-by-side view
    Side,
}

/// Diff highlighting style type
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DiffStyleType {
    /// Word-level highlighting
    Word,
    /// Character-level highlighting
    Char,
}

/// Output format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FormatType {
    /// HTML output
    Html,
    /// JSON output
    Json,
}

/// Input source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InputType {
    /// Read from file
    File,
    /// Execute git diff command
    Command,
    /// Read from stdin
    Stdin,
}

/// Output destination type
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputType {
    /// Preview in browser
    Preview,
    /// Print to stdout
    Stdout,
}

/// Color scheme type
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorSchemeType {
    /// Auto-detect based on system preference
    Auto,
    /// Light theme
    Light,
    /// Dark theme
    Dark,
}

/// File summary visibility type
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SummaryType {
    /// Summary closed by default
    Closed,
    /// Summary open by default
    Open,
    /// Summary hidden
    Hidden,
}

/// Line matching type
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LineMatchingType {
    /// No line matching
    None,
    /// Match by lines
    Lines,
    /// Match by words
    Words,
}
