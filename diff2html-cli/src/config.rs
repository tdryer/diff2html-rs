//! Configuration conversion from CLI arguments to library config.

use anyhow::{Result, bail};

use crate::args::{
    Args, ColorSchemeType, DiffStyleType, FormatType, InputType, LineMatchingType, OutputType,
    StyleType, SummaryType,
};
use diff2html::{
    ColorScheme, Diff2HtmlConfig, DiffStyle, LineMatchingType as LibLineMatchingType, OutputFormat,
};

/// CLI-specific configuration for input/output handling.
#[derive(Debug)]
pub struct CliConfig {
    /// Input source type
    pub input_type: InputType,
    /// Output format (html or json)
    pub format_type: FormatType,
    /// Output destination type
    pub output_type: OutputType,
    /// Output file path (overrides output_type if set)
    pub output_file: Option<String>,
    /// HTML page title
    pub page_title: String,
    /// HTML page header
    pub page_header: String,
    /// Custom HTML wrapper template path
    pub html_wrapper_template: Option<String>,
    /// Whether file list summary is open by default
    pub show_files_open: bool,
    /// Enable file content toggle
    pub file_content_toggle: bool,
    /// Enable synchronised scroll for side-by-side view
    pub synchronised_scroll: bool,
    /// Enable syntax highlighting
    pub highlight_code: bool,
    /// Color scheme for HTML output
    pub color_scheme: ColorSchemeType,
    /// Files to ignore
    pub ignore: Vec<String>,
    /// Extra git diff arguments
    pub extra_args: Vec<String>,
}

/// Parse CLI arguments into library config and CLI-specific config.
///
/// # Errors
///
/// Returns an error if `match_words_threshold` is not in the range 0.0-1.0.
pub fn parse_args(args: &Args) -> Result<(Diff2HtmlConfig, CliConfig)> {
    // Validate match_words_threshold is in range 0.0-1.0
    if !(0.0..=1.0).contains(&args.match_words_threshold) {
        bail!(
            "match_words_threshold must be between 0.0 and 1.0, got {}",
            args.match_words_threshold
        );
    }

    let diff2html_config = Diff2HtmlConfig {
        output_format: match args.style {
            StyleType::Line => OutputFormat::LineByLine,
            StyleType::Side => OutputFormat::SideBySide,
        },
        diff_style: match args.diff_style {
            DiffStyleType::Word => DiffStyle::Word,
            DiffStyleType::Char => DiffStyle::Char,
        },
        color_scheme: match args.color_scheme {
            ColorSchemeType::Auto => ColorScheme::Auto,
            ColorSchemeType::Light => ColorScheme::Light,
            ColorSchemeType::Dark => ColorScheme::Dark,
        },
        draw_file_list: args.summary != SummaryType::Hidden,
        matching: match args.matching {
            LineMatchingType::None => LibLineMatchingType::None,
            LineMatchingType::Lines => LibLineMatchingType::Lines,
            LineMatchingType::Words => LibLineMatchingType::Words,
        },
        match_words_threshold: args.match_words_threshold,
        matching_max_comparisons: args.matching_max_comparisons,
        diff_max_changes: args.diff_max_changes,
        diff_max_line_length: args.diff_max_line_length,
        render_nothing_when_empty: args.render_nothing_when_empty,
        max_line_size_in_block_for_comparison: args.max_line_size_in_block_for_comparison,
        max_line_length_highlight: args.max_line_length_highlight,
        ..Default::default()
    };

    let default_title = "Diff to HTML";
    let default_header = r#"Diff to HTML"#;

    let cli_config = CliConfig {
        input_type: args.input,
        format_type: args.format,
        output_type: args.output,
        output_file: args.file.clone(),
        page_title: args
            .title
            .clone()
            .unwrap_or_else(|| default_title.to_string()),
        page_header: args
            .title
            .clone()
            .unwrap_or_else(|| default_header.to_string()),
        html_wrapper_template: args.html_wrapper_template.clone(),
        show_files_open: args.summary == SummaryType::Open,
        file_content_toggle: args.file_content_toggle,
        synchronised_scroll: args.synchronised_scroll,
        highlight_code: args.highlight_code,
        color_scheme: args.color_scheme,
        ignore: args.ignore.clone(),
        extra_args: args.extra_args.clone(),
    };

    Ok((diff2html_config, cli_config))
}
