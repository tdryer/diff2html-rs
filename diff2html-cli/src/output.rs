//! Output handling for diff2html CLI.
//!
//! This module handles output generation and destinations:
//! - HTML wrapping with templates
//! - Preview in browser
//! - Writing to stdout or files

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use diff2html::{Diff2HtmlConfig, html, parse, templates::CSS};

use crate::args::{ColorSchemeType, FormatType};
use crate::config::CliConfig;

/// Default HTML wrapper template.
const DEFAULT_TEMPLATE: &str = include_str!("../templates/wrapper.html");

// highlight.js GitHub theme CDN links
const LIGHT_GITHUB_THEME: &str = r#"<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css" />"#;
const DARK_GITHUB_THEME: &str = r#"<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css" />"#;
const AUTO_GITHUB_THEME: &str = r#"<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css" media="screen and (prefers-color-scheme: light)" />
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css" media="screen and (prefers-color-scheme: dark)" />"#;

// Base styles for light/dark modes
const LIGHT_BASE_STYLE: &str = r#"<style>
body {
  background-color: var(--d2h-bg-color);
}
h1 {
  color: var(--d2h-light-color);
}
</style>"#;

const DARK_BASE_STYLE: &str = r#"<style>
body {
  background-color: rgb(13, 17, 23);
}
h1 {
  color: var(--d2h-dark-color);
}
</style>"#;

const AUTO_BASE_STYLE: &str = r#"<style>
@media screen and (prefers-color-scheme: light) {
  body {
    background-color: var(--d2h-bg-color);
  }
  h1 {
    color: var(--d2h-light-color);
  }
}
@media screen and (prefers-color-scheme: dark) {
  body {
    background-color: rgb(13, 17, 23);
  }
  h1 {
    color: var(--d2h-dark-color);
  }
}
</style>"#;

// diff2html-ui JavaScript bundle CDN
const DIFF2HTML_UI_JS: &str = r#"<script src="https://cdn.jsdelivr.net/npm/diff2html@3.4.55/bundles/js/diff2html-ui.min.js"></script>"#;

/// Escape HTML special characters to prevent XSS injection.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Generate output based on configuration and input.
pub fn get_output(
    diff2html_config: &Diff2HtmlConfig,
    cli_config: &CliConfig,
    input: &str,
) -> Result<String> {
    // Validate custom template exists if specified
    if let Some(ref template_path) = cli_config.html_wrapper_template
        && !PathBuf::from(template_path).exists()
    {
        bail!("Template ('{}') not found!", template_path);
    }

    match cli_config.format_type {
        FormatType::Html => {
            let html_content = html(input, diff2html_config);
            prepare_html(&html_content, cli_config)
        }
        FormatType::Json => {
            let diff_files = parse(input, &diff2html_config.to_parser_config());
            serde_json::to_string(&diff_files).context("Failed to serialize JSON")
        }
    }
}

/// Wrap diff HTML content in a full HTML page.
fn prepare_html(diff_content: &str, config: &CliConfig) -> Result<String> {
    // Load template
    let template = if let Some(ref template_path) = config.html_wrapper_template {
        fs::read_to_string(template_path)
            .with_context(|| format!("Failed to read template: {}", template_path))?
    } else {
        DEFAULT_TEMPLATE.to_string()
    };

    // Determine theme-specific content
    let (github_theme, base_style) = match config.color_scheme {
        ColorSchemeType::Light => (LIGHT_GITHUB_THEME, LIGHT_BASE_STYLE),
        ColorSchemeType::Dark => (DARK_GITHUB_THEME, DARK_BASE_STYLE),
        ColorSchemeType::Auto => (AUTO_GITHUB_THEME, AUTO_BASE_STYLE),
    };

    // Build CSS content
    let css_content = format!(
        "{}\n{}\n<style>\n{}\n</style>",
        base_style, github_theme, CSS
    );

    // Build JavaScript calls based on configuration
    let file_list_toggle = format!("diff2htmlUi.fileListToggle({});", config.show_files_open);
    let file_content_toggle = if config.file_content_toggle {
        "diff2htmlUi.fileContentToggle();"
    } else {
        ""
    };
    let synchronised_scroll = if config.synchronised_scroll {
        "diff2htmlUi.synchronisedScroll();"
    } else {
        ""
    };
    let highlight_code = if config.highlight_code {
        "diff2htmlUi.highlightCode();"
    } else {
        ""
    };

    // Escape user-provided values to prevent XSS injection
    let escaped_title = escape_html(&config.page_title);
    let escaped_header = escape_html(&config.page_header);

    // Perform replacements
    let result = template
        .replace("<!--diff2html-title-->", &escaped_title)
        .replace("<!--diff2html-css-->", &css_content)
        .replace("<!--diff2html-js-ui-->", DIFF2HTML_UI_JS)
        .replace("//diff2html-fileListToggle", &file_list_toggle)
        .replace("//diff2html-fileContentToggle", file_content_toggle)
        .replace("//diff2html-synchronisedScroll", synchronised_scroll)
        .replace("//diff2html-highlightCode", highlight_code)
        .replace("<!--diff2html-header-->", &escaped_header)
        .replace("<!--diff2html-diff-->", diff_content);

    Ok(result)
}

/// Preview content in browser by writing to a temp file.
pub fn preview(content: &str, format: FormatType) -> Result<()> {
    let suffix = match format {
        FormatType::Html => ".html",
        FormatType::Json => ".json",
    };

    // Use tempfile crate for secure temp file creation with random name
    let mut temp_file = tempfile::Builder::new()
        .prefix("diff2html-")
        .suffix(suffix)
        .tempfile()
        .context("Failed to create secure temporary file")?;

    temp_file
        .write_all(content.as_bytes())
        .context("Failed to write to temporary file")?;

    // Keep the file around after the handle is dropped so the browser can open it
    let (_, file_path) = temp_file
        .keep()
        .context("Failed to persist temporary file")?;

    open::that(&file_path)
        .with_context(|| format!("Failed to open file in browser: {}", file_path.display()))?;

    Ok(())
}

/// Write content to a file.
pub fn write_file(path: &str, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("Failed to write to file: {}", path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_html_replaces_placeholders() {
        let config = CliConfig {
            input_type: crate::args::InputType::Command,
            format_type: FormatType::Html,
            output_type: crate::args::OutputType::Preview,
            output_file: None,
            page_title: "Test Title".to_string(),
            page_header: "Test Header".to_string(),
            html_wrapper_template: None,
            show_files_open: false,
            file_content_toggle: true,
            synchronised_scroll: true,
            highlight_code: true,
            color_scheme: ColorSchemeType::Light,
            ignore: vec![],
            extra_args: vec![],
        };

        let result = prepare_html("<div>test content</div>", &config).unwrap();

        assert!(result.contains("Test Title"));
        assert!(result.contains("Test Header"));
        assert!(result.contains("<div>test content</div>"));
        assert!(result.contains("diff2htmlUi.fileListToggle(false);"));
        assert!(result.contains("diff2htmlUi.fileContentToggle();"));
        assert!(result.contains("diff2htmlUi.synchronisedScroll();"));
        assert!(result.contains("diff2htmlUi.highlightCode();"));
    }

    #[test]
    fn test_prepare_html_light_theme() {
        let config = CliConfig {
            input_type: crate::args::InputType::Command,
            format_type: FormatType::Html,
            output_type: crate::args::OutputType::Preview,
            output_file: None,
            page_title: "Test".to_string(),
            page_header: "Test".to_string(),
            html_wrapper_template: None,
            show_files_open: false,
            file_content_toggle: false,
            synchronised_scroll: false,
            highlight_code: false,
            color_scheme: ColorSchemeType::Light,
            ignore: vec![],
            extra_args: vec![],
        };

        let result = prepare_html("", &config).unwrap();
        assert!(result.contains("github.min.css"));
        assert!(!result.contains("github-dark.min.css"));
    }

    #[test]
    fn test_prepare_html_dark_theme() {
        let config = CliConfig {
            input_type: crate::args::InputType::Command,
            format_type: FormatType::Html,
            output_type: crate::args::OutputType::Preview,
            output_file: None,
            page_title: "Test".to_string(),
            page_header: "Test".to_string(),
            html_wrapper_template: None,
            show_files_open: false,
            file_content_toggle: false,
            synchronised_scroll: false,
            highlight_code: false,
            color_scheme: ColorSchemeType::Dark,
            ignore: vec![],
            extra_args: vec![],
        };

        let result = prepare_html("", &config).unwrap();
        assert!(result.contains("github-dark.min.css"));
    }

    #[test]
    fn test_prepare_html_auto_theme() {
        let config = CliConfig {
            input_type: crate::args::InputType::Command,
            format_type: FormatType::Html,
            output_type: crate::args::OutputType::Preview,
            output_file: None,
            page_title: "Test".to_string(),
            page_header: "Test".to_string(),
            html_wrapper_template: None,
            show_files_open: false,
            file_content_toggle: false,
            synchronised_scroll: false,
            highlight_code: false,
            color_scheme: ColorSchemeType::Auto,
            ignore: vec![],
            extra_args: vec![],
        };

        let result = prepare_html("", &config).unwrap();
        assert!(result.contains("prefers-color-scheme: light"));
        assert!(result.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn test_prepare_html_disabled_features() {
        let config = CliConfig {
            input_type: crate::args::InputType::Command,
            format_type: FormatType::Html,
            output_type: crate::args::OutputType::Preview,
            output_file: None,
            page_title: "Test".to_string(),
            page_header: "Test".to_string(),
            html_wrapper_template: None,
            show_files_open: false,
            file_content_toggle: false,
            synchronised_scroll: false,
            highlight_code: false,
            color_scheme: ColorSchemeType::Light,
            ignore: vec![],
            extra_args: vec![],
        };

        let result = prepare_html("", &config).unwrap();
        assert!(!result.contains("diff2htmlUi.fileContentToggle();"));
        assert!(!result.contains("diff2htmlUi.synchronisedScroll();"));
        assert!(!result.contains("diff2htmlUi.highlightCode();"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_html("normal text"), "normal text");
    }

    #[test]
    fn test_prepare_html_escapes_xss_in_title_and_header() {
        let config = CliConfig {
            input_type: crate::args::InputType::Command,
            format_type: FormatType::Html,
            output_type: crate::args::OutputType::Preview,
            output_file: None,
            page_title: "<script>alert('xss')</script>".to_string(),
            page_header: "<img src=x onerror=alert('xss')>".to_string(),
            html_wrapper_template: None,
            show_files_open: false,
            file_content_toggle: false,
            synchronised_scroll: false,
            highlight_code: false,
            color_scheme: ColorSchemeType::Light,
            ignore: vec![],
            extra_args: vec![],
        };

        let result = prepare_html("", &config).unwrap();

        // Verify that the raw script tags are NOT in the output
        assert!(!result.contains("<script>alert"));
        assert!(!result.contains("<img src=x onerror"));

        // Verify that the escaped versions ARE in the output
        assert!(result.contains("&lt;script&gt;"));
        assert!(result.contains("&lt;img src=x onerror"));
    }
}
