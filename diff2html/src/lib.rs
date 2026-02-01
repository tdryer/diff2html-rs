#![forbid(unsafe_code)]

//! diff2html - Parse unified diffs and generate HTML.
//!
//! This library provides functionality to parse unified diff format and render
//! it as HTML, similar to the JavaScript diff2html library.
//!
//! # Quick Start
//!
//! The simplest way to use this library is through the [`html`] function:
//!
//! ```
//! use diff2html::{html, Diff2HtmlConfig};
//!
//! let diff = r#"diff --git a/test.txt b/test.txt
//! --- a/test.txt
//! +++ b/test.txt
//! @@ -1 +1 @@
//! -old
//! +new
//! "#;
//!
//! // Generate HTML from diff string
//! let html_output = html(diff, &Diff2HtmlConfig::default());
//! assert!(html_output.contains("d2h-wrapper"));
//! ```
//!
//! # Two-Stage Processing
//!
//! For more control, you can parse and render separately:
//!
//! ```
//! use diff2html::{parse, html_from_diff_files, Diff2HtmlConfig, DiffParserConfig};
//!
//! let diff = r#"diff --git a/test.txt b/test.txt
//! --- a/test.txt
//! +++ b/test.txt
//! @@ -1 +1 @@
//! -old
//! +new
//! "#;
//!
//! // Parse first
//! let files = parse(diff, &DiffParserConfig::default());
//! assert_eq!(files.len(), 1);
//!
//! // Then render
//! let html_output = html_from_diff_files(&files, &Diff2HtmlConfig::default());
//! assert!(html_output.contains("d2h-wrapper"));
//! ```
//!
//! # JSON Output
//!
//! You can also get JSON output for integration with other tools:
//!
//! ```
//! use diff2html::{json, Diff2HtmlConfig};
//!
//! let diff = r#"diff --git a/test.txt b/test.txt
//! --- a/test.txt
//! +++ b/test.txt
//! @@ -1 +1 @@
//! -old
//! +new
//! "#;
//!
//! let json_output = json(diff, &Diff2HtmlConfig::default()).unwrap();
//! assert!(json_output.contains("\"newName\":\"test.txt\""));
//! ```
//!
//! # Configuration
//!
//! The [`Diff2HtmlConfig`] struct provides comprehensive configuration options:
//!
//! ```
//! use diff2html::{html, Diff2HtmlConfig, OutputFormat, DiffStyle, ColorScheme, LineMatchingType};
//!
//! let config = Diff2HtmlConfig {
//!     output_format: OutputFormat::SideBySide,
//!     diff_style: DiffStyle::Char,
//!     color_scheme: ColorScheme::Dark,
//!     draw_file_list: true,
//!     matching: LineMatchingType::Lines,
//!     ..Default::default()
//! };
//!
//! let diff = "--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n";
//! let html_output = html(diff, &config);
//! assert!(html_output.contains("d2h-dark-color-scheme"));
//! ```

pub mod parser;
pub mod rematch;
pub mod render;
pub mod templates;
pub mod types;

pub use parser::{DiffParserConfig, parse};
pub use rematch::{
    BestMatch, MatchConfig, MatchGroup, levenshtein, match_lines, match_lines_with_config,
    new_distance_fn, string_distance,
};
pub use render::utils::{CSSLineClass, HighlightedLines, RenderConfig};
pub use render::{
    FileListConfig, FileListRenderer, LineByLineRenderer, RendererConfig, SideBySideRenderer,
};
pub use templates::{CSS, TemplateName, render as render_template, render_by_name};
pub use types::{
    Checksum, ColorScheme, DiffBlock, DiffFile, DiffLine, DiffLineParts, DiffStyle, FileMode,
    LineMatchingType, LineType, OutputFormat,
};

/// Unified configuration for diff2html.
///
/// This struct combines all configuration options for parsing and rendering diffs.
/// It provides a single configuration point matching the JavaScript diff2html API.
///
/// # Example
///
/// ```
/// use diff2html::{Diff2HtmlConfig, OutputFormat, ColorScheme};
///
/// let config = Diff2HtmlConfig {
///     output_format: OutputFormat::SideBySide,
///     color_scheme: ColorScheme::Dark,
///     draw_file_list: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Diff2HtmlConfig {
    // Parser options
    /// Prefix to strip from source file paths.
    pub src_prefix: Option<String>,
    /// Prefix to strip from destination file paths.
    pub dst_prefix: Option<String>,
    /// Maximum number of changes before marking file as "too big".
    pub diff_max_changes: Option<u32>,
    /// Maximum line length before marking file as "too big".
    pub diff_max_line_length: Option<usize>,

    // Renderer options
    /// Output format: line-by-line or side-by-side view.
    pub output_format: OutputFormat,
    /// Whether to draw the file list summary at the top.
    pub draw_file_list: bool,
    /// Diff highlighting style: word or character level.
    pub diff_style: DiffStyle,
    /// Color scheme for the output.
    pub color_scheme: ColorScheme,
    /// Line matching algorithm for pairing similar lines.
    pub matching: LineMatchingType,
    /// Threshold for word matching (0.0 to 1.0, default 0.25).
    pub match_words_threshold: f64,
    /// Maximum line length for diff highlighting.
    pub max_line_length_highlight: usize,
    /// Whether to render nothing when diff is empty.
    pub render_nothing_when_empty: bool,
    /// Maximum comparisons for line matching algorithm.
    pub matching_max_comparisons: usize,
    /// Maximum line size in a block for comparison.
    pub max_line_size_in_block_for_comparison: usize,
}

impl Default for Diff2HtmlConfig {
    fn default() -> Self {
        Self {
            // Parser defaults
            src_prefix: None,
            dst_prefix: None,
            diff_max_changes: None,
            diff_max_line_length: None,

            // Renderer defaults
            output_format: OutputFormat::LineByLine,
            draw_file_list: true,
            diff_style: DiffStyle::Word,
            color_scheme: ColorScheme::Light,
            matching: LineMatchingType::None,
            match_words_threshold: 0.25,
            max_line_length_highlight: 10000,
            render_nothing_when_empty: false,
            matching_max_comparisons: 2500,
            max_line_size_in_block_for_comparison: 200,
        }
    }
}

impl Diff2HtmlConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to parser configuration.
    pub fn to_parser_config(&self) -> DiffParserConfig {
        DiffParserConfig {
            src_prefix: self.src_prefix.clone(),
            dst_prefix: self.dst_prefix.clone(),
            diff_max_changes: self.diff_max_changes,
            diff_max_line_length: self.diff_max_line_length,
            diff_too_big_message: None,
        }
    }

    /// Convert to renderer configuration.
    pub fn to_renderer_config(&self) -> RendererConfig {
        RendererConfig {
            render: RenderConfig {
                matching: self.matching,
                match_words_threshold: self.match_words_threshold,
                max_line_length_highlight: self.max_line_length_highlight,
                diff_style: self.diff_style,
                color_scheme: self.color_scheme,
            },
            render_nothing_when_empty: self.render_nothing_when_empty,
            matching_max_comparisons: self.matching_max_comparisons,
            max_line_size_in_block_for_comparison: self.max_line_size_in_block_for_comparison,
        }
    }

    /// Convert to file list configuration.
    pub fn to_file_list_config(&self) -> FileListConfig {
        FileListConfig {
            color_scheme: self.color_scheme,
        }
    }
}

/// Parse a diff string and render it as HTML.
///
/// This is the main entry point for converting diff text to HTML output.
/// It combines parsing and rendering in a single function call.
///
/// # Arguments
///
/// * `diff_input` - The unified diff text to parse and render
/// * `config` - Configuration options for parsing and rendering
///
/// # Returns
///
/// HTML string containing the rendered diff
///
/// # Example
///
/// ```
/// use diff2html::{html, Diff2HtmlConfig, OutputFormat};
///
/// let diff = "--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n";
/// let config = Diff2HtmlConfig {
///     output_format: OutputFormat::SideBySide,
///     ..Default::default()
/// };
///
/// let html_output = html(diff, &config);
/// assert!(html_output.contains("d2h-file-side-diff"));
/// ```
pub fn html(diff_input: &str, config: &Diff2HtmlConfig) -> String {
    let diff_files = parse(diff_input, &config.to_parser_config());
    html_from_diff_files(&diff_files, config)
}

/// Render already-parsed diff files as HTML.
///
/// Use this function when you have already parsed the diff and want to
/// render it with different configurations without re-parsing.
///
/// # Arguments
///
/// * `diff_files` - Parsed diff files from [`parse`]
/// * `config` - Configuration options for rendering
///
/// # Returns
///
/// HTML string containing the rendered diff
///
/// # Example
///
/// ```
/// use diff2html::{parse, html_from_diff_files, Diff2HtmlConfig, DiffParserConfig, OutputFormat};
///
/// let diff = "--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n";
/// let files = parse(diff, &DiffParserConfig::default());
///
/// // Render line-by-line
/// let config1 = Diff2HtmlConfig::default();
/// let html1 = html_from_diff_files(&files, &config1);
///
/// // Render side-by-side with same parsed data
/// let config2 = Diff2HtmlConfig {
///     output_format: OutputFormat::SideBySide,
///     ..Default::default()
/// };
/// let html2 = html_from_diff_files(&files, &config2);
/// ```
pub fn html_from_diff_files(diff_files: &[DiffFile], config: &Diff2HtmlConfig) -> String {
    let renderer_config = config.to_renderer_config();

    let file_list = if config.draw_file_list {
        let file_list_config = config.to_file_list_config();
        let file_list_renderer = FileListRenderer::new(file_list_config);
        file_list_renderer.render(diff_files)
    } else {
        String::new()
    };

    let diff_output = match config.output_format {
        OutputFormat::SideBySide => {
            let renderer = SideBySideRenderer::new(renderer_config);
            renderer.render(diff_files)
        }
        OutputFormat::LineByLine => {
            let renderer = LineByLineRenderer::new(renderer_config);
            renderer.render(diff_files)
        }
    };

    file_list + &diff_output
}

/// Parse a diff string and return JSON output.
///
/// This function parses the diff and serializes the result to JSON format,
/// which can be useful for integration with other tools or for debugging.
///
/// # Arguments
///
/// * `diff_input` - The unified diff text to parse
/// * `config` - Configuration options for parsing
///
/// # Returns
///
/// A `Result` containing the JSON string or a serialization error
///
/// # Example
///
/// ```
/// use diff2html::{json, Diff2HtmlConfig};
///
/// let diff = "--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n";
/// let json_output = json(diff, &Diff2HtmlConfig::default()).unwrap();
/// assert!(json_output.contains("\"oldName\":\"file.txt\""));
/// ```
pub fn json(diff_input: &str, config: &Diff2HtmlConfig) -> Result<String, serde_json::Error> {
    let diff_files = parse(diff_input, &config.to_parser_config());
    json_from_diff_files(&diff_files)
}

/// Serialize already-parsed diff files to JSON.
///
/// Use this function when you have already parsed the diff and want to
/// get JSON output without re-parsing.
///
/// # Arguments
///
/// * `diff_files` - Parsed diff files from [`parse`]
///
/// # Returns
///
/// A `Result` containing the JSON string or a serialization error
///
/// # Example
///
/// ```
/// use diff2html::{parse, json_from_diff_files, DiffParserConfig};
///
/// let diff = "--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n";
/// let files = parse(diff, &DiffParserConfig::default());
/// let json_output = json_from_diff_files(&files).unwrap();
/// ```
pub fn json_from_diff_files(diff_files: &[DiffFile]) -> Result<String, serde_json::Error> {
    serde_json::to_string(diff_files)
}

/// Serialize already-parsed diff files to pretty-printed JSON.
///
/// Same as [`json_from_diff_files`] but with indentation for readability.
///
/// # Arguments
///
/// * `diff_files` - Parsed diff files from [`parse`]
///
/// # Returns
///
/// A `Result` containing the pretty-printed JSON string or a serialization error
pub fn json_from_diff_files_pretty(diff_files: &[DiffFile]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(diff_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_DIFF: &str = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1 +1 @@
-old line
+new line
"#;

    const MULTI_FILE_DIFF: &str = r#"diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1 +1 @@
-old
+new
diff --git a/file2.txt b/file2.txt
--- a/file2.txt
+++ b/file2.txt
@@ -1 +1 @@
-foo
+bar
"#;

    #[test]
    fn test_html_basic() {
        let output = html(SIMPLE_DIFF, &Diff2HtmlConfig::default());
        assert!(output.contains("d2h-wrapper"));
        assert!(output.contains("d2h-file-header"));
        assert!(output.contains("test.txt"));
    }

    #[test]
    fn test_html_line_by_line() {
        let config = Diff2HtmlConfig {
            output_format: OutputFormat::LineByLine,
            ..Default::default()
        };
        let output = html(SIMPLE_DIFF, &config);
        assert!(output.contains("d2h-file-diff"));
        // Line-by-line should not have side-by-side specific classes
        assert!(!output.contains("d2h-file-side-diff"));
    }

    #[test]
    fn test_html_side_by_side() {
        let config = Diff2HtmlConfig {
            output_format: OutputFormat::SideBySide,
            ..Default::default()
        };
        let output = html(SIMPLE_DIFF, &config);
        assert!(output.contains("d2h-file-side-diff"));
    }

    #[test]
    fn test_html_with_file_list() {
        let config = Diff2HtmlConfig {
            draw_file_list: true,
            ..Default::default()
        };
        let output = html(MULTI_FILE_DIFF, &config);
        assert!(output.contains("d2h-file-list"));
    }

    #[test]
    fn test_html_without_file_list() {
        let config = Diff2HtmlConfig {
            draw_file_list: false,
            ..Default::default()
        };
        let output = html(MULTI_FILE_DIFF, &config);
        assert!(!output.contains("d2h-file-list"));
    }

    #[test]
    fn test_html_color_scheme_light() {
        let config = Diff2HtmlConfig {
            color_scheme: ColorScheme::Light,
            ..Default::default()
        };
        let output = html(SIMPLE_DIFF, &config);
        assert!(output.contains("d2h-light-color-scheme"));
    }

    #[test]
    fn test_html_color_scheme_dark() {
        let config = Diff2HtmlConfig {
            color_scheme: ColorScheme::Dark,
            ..Default::default()
        };
        let output = html(SIMPLE_DIFF, &config);
        assert!(output.contains("d2h-dark-color-scheme"));
    }

    #[test]
    fn test_html_color_scheme_auto() {
        let config = Diff2HtmlConfig {
            color_scheme: ColorScheme::Auto,
            ..Default::default()
        };
        let output = html(SIMPLE_DIFF, &config);
        assert!(output.contains("d2h-auto-color-scheme"));
    }

    #[test]
    fn test_html_from_diff_files() {
        let files = parse(SIMPLE_DIFF, &DiffParserConfig::default());
        let output = html_from_diff_files(&files, &Diff2HtmlConfig::default());
        assert!(output.contains("d2h-wrapper"));
        assert!(output.contains("test.txt"));
    }

    #[test]
    fn test_json_basic() {
        let output = json(SIMPLE_DIFF, &Diff2HtmlConfig::default()).unwrap();
        assert!(output.contains("\"newName\":\"test.txt\""));
        assert!(output.contains("\"oldName\":\"test.txt\""));
        assert!(output.contains("\"blocks\""));
    }

    #[test]
    fn test_json_from_diff_files() {
        let files = parse(SIMPLE_DIFF, &DiffParserConfig::default());
        let output = json_from_diff_files(&files).unwrap();
        assert!(output.contains("\"newName\":\"test.txt\""));
    }

    #[test]
    fn test_json_from_diff_files_pretty() {
        let files = parse(SIMPLE_DIFF, &DiffParserConfig::default());
        let output = json_from_diff_files_pretty(&files).unwrap();
        // Pretty output should have newlines
        assert!(output.contains('\n'));
        assert!(output.contains("\"newName\": \"test.txt\""));
    }

    #[test]
    fn test_config_default() {
        let config = Diff2HtmlConfig::default();
        assert_eq!(config.output_format, OutputFormat::LineByLine);
        assert!(config.draw_file_list);
        assert_eq!(config.diff_style, DiffStyle::Word);
        assert_eq!(config.color_scheme, ColorScheme::Light);
        assert_eq!(config.matching, LineMatchingType::None);
        assert!((config.match_words_threshold - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_config_to_parser_config() {
        let config = Diff2HtmlConfig {
            src_prefix: Some("a/".to_string()),
            dst_prefix: Some("b/".to_string()),
            diff_max_changes: Some(1000),
            diff_max_line_length: Some(500),
            ..Default::default()
        };
        let parser_config = config.to_parser_config();
        assert_eq!(parser_config.src_prefix, Some("a/".to_string()));
        assert_eq!(parser_config.dst_prefix, Some("b/".to_string()));
        assert_eq!(parser_config.diff_max_changes, Some(1000));
        assert_eq!(parser_config.diff_max_line_length, Some(500));
    }

    #[test]
    fn test_config_to_renderer_config() {
        let config = Diff2HtmlConfig {
            matching: LineMatchingType::Lines,
            match_words_threshold: 0.5,
            diff_style: DiffStyle::Char,
            color_scheme: ColorScheme::Dark,
            ..Default::default()
        };
        let renderer_config = config.to_renderer_config();
        assert_eq!(renderer_config.render.matching, LineMatchingType::Lines);
        assert!((renderer_config.render.match_words_threshold - 0.5).abs() < f64::EPSILON);
        assert_eq!(renderer_config.render.diff_style, DiffStyle::Char);
        assert_eq!(renderer_config.render.color_scheme, ColorScheme::Dark);
    }

    #[test]
    fn test_empty_diff() {
        let output = html("", &Diff2HtmlConfig::default());
        // Empty diff should produce minimal output
        assert!(output.is_empty() || output.contains("d2h"));
    }

    #[test]
    fn test_multi_file_diff() {
        let output = html(MULTI_FILE_DIFF, &Diff2HtmlConfig::default());
        assert!(output.contains("file1.txt"));
        assert!(output.contains("file2.txt"));
    }
}
