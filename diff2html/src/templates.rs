//! Template system for diff2html HTML rendering.
//!
//! This module provides template loading and rendering using the Handlebars
//! template engine. Templates are embedded at compile time for zero runtime
//! file I/O.

use handlebars::Handlebars;
use serde::Serialize;
use std::sync::LazyLock;
use thiserror::Error;

/// CSS stylesheet for diff2html rendering.
pub const CSS: &str = include_str!("../css/diff2html.css");

// Embed all templates at compile time
const GENERIC_WRAPPER: &str = include_str!("../templates/generic-wrapper.mustache");
const FILE_SUMMARY_WRAPPER: &str = include_str!("../templates/file-summary-wrapper.mustache");
const FILE_SUMMARY_LINE: &str = include_str!("../templates/file-summary-line.mustache");
const LINE_BY_LINE_FILE_DIFF: &str = include_str!("../templates/line-by-line-file-diff.mustache");
const SIDE_BY_SIDE_FILE_DIFF: &str = include_str!("../templates/side-by-side-file-diff.mustache");
const GENERIC_FILE_PATH: &str = include_str!("../templates/generic-file-path.mustache");
const GENERIC_LINE: &str = include_str!("../templates/generic-line.mustache");
const LINE_BY_LINE_NUMBERS: &str = include_str!("../templates/line-by-line-numbers.mustache");
const GENERIC_BLOCK_HEADER: &str = include_str!("../templates/generic-block-header.mustache");
const GENERIC_EMPTY_DIFF: &str = include_str!("../templates/generic-empty-diff.mustache");
const ICON_FILE: &str = include_str!("../templates/icon-file.mustache");
const ICON_FILE_ADDED: &str = include_str!("../templates/icon-file-added.mustache");
const ICON_FILE_CHANGED: &str = include_str!("../templates/icon-file-changed.mustache");
const ICON_FILE_DELETED: &str = include_str!("../templates/icon-file-deleted.mustache");
const ICON_FILE_RENAMED: &str = include_str!("../templates/icon-file-renamed.mustache");
const TAG_FILE_ADDED: &str = include_str!("../templates/tag-file-added.mustache");
const TAG_FILE_CHANGED: &str = include_str!("../templates/tag-file-changed.mustache");
const TAG_FILE_DELETED: &str = include_str!("../templates/tag-file-deleted.mustache");
const TAG_FILE_RENAMED: &str = include_str!("../templates/tag-file-renamed.mustache");

/// Template names for use with the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateName {
    GenericWrapper,
    FileSummaryWrapper,
    FileSummaryLine,
    LineByLineFileDiff,
    SideBySideFileDiff,
    GenericFilePath,
    GenericLine,
    LineByLineNumbers,
    GenericBlockHeader,
    GenericEmptyDiff,
    IconFile,
    IconFileAdded,
    IconFileChanged,
    IconFileDeleted,
    IconFileRenamed,
    TagFileAdded,
    TagFileChanged,
    TagFileDeleted,
    TagFileRenamed,
}

impl TemplateName {
    /// Returns the string name used in the handlebars registry.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GenericWrapper => "generic-wrapper",
            Self::FileSummaryWrapper => "file-summary-wrapper",
            Self::FileSummaryLine => "file-summary-line",
            Self::LineByLineFileDiff => "line-by-line-file-diff",
            Self::SideBySideFileDiff => "side-by-side-file-diff",
            Self::GenericFilePath => "generic-file-path",
            Self::GenericLine => "generic-line",
            Self::LineByLineNumbers => "line-by-line-numbers",
            Self::GenericBlockHeader => "generic-block-header",
            Self::GenericEmptyDiff => "generic-empty-diff",
            Self::IconFile => "icon-file",
            Self::IconFileAdded => "icon-file-added",
            Self::IconFileChanged => "icon-file-changed",
            Self::IconFileDeleted => "icon-file-deleted",
            Self::IconFileRenamed => "icon-file-renamed",
            Self::TagFileAdded => "tag-file-added",
            Self::TagFileChanged => "tag-file-changed",
            Self::TagFileDeleted => "tag-file-deleted",
            Self::TagFileRenamed => "tag-file-renamed",
        }
    }
}

/// Errors that can occur during template rendering.
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Template rendering failed: {0}")]
    RenderError(#[from] handlebars::RenderError),
}

/// Global template registry initialized on first use.
static TEMPLATES: LazyLock<Handlebars<'static>> = LazyLock::new(|| {
    let mut hbs = Handlebars::new();

    // Disable HTML escaping by default since we handle it ourselves
    hbs.register_escape_fn(handlebars::no_escape);

    // Register all templates
    register_templates(&mut hbs);

    hbs
});

fn register_templates(hbs: &mut Handlebars<'static>) {
    // Main templates
    hbs.register_template_string("generic-wrapper", GENERIC_WRAPPER)
        .expect("Failed to register generic-wrapper template");
    hbs.register_template_string("file-summary-wrapper", FILE_SUMMARY_WRAPPER)
        .expect("Failed to register file-summary-wrapper template");
    hbs.register_template_string("file-summary-line", FILE_SUMMARY_LINE)
        .expect("Failed to register file-summary-line template");
    hbs.register_template_string("line-by-line-file-diff", LINE_BY_LINE_FILE_DIFF)
        .expect("Failed to register line-by-line-file-diff template");
    hbs.register_template_string("side-by-side-file-diff", SIDE_BY_SIDE_FILE_DIFF)
        .expect("Failed to register side-by-side-file-diff template");
    hbs.register_template_string("generic-file-path", GENERIC_FILE_PATH)
        .expect("Failed to register generic-file-path template");
    hbs.register_template_string("generic-line", GENERIC_LINE)
        .expect("Failed to register generic-line template");
    hbs.register_template_string("line-by-line-numbers", LINE_BY_LINE_NUMBERS)
        .expect("Failed to register line-by-line-numbers template");
    hbs.register_template_string("generic-block-header", GENERIC_BLOCK_HEADER)
        .expect("Failed to register generic-block-header template");
    hbs.register_template_string("generic-empty-diff", GENERIC_EMPTY_DIFF)
        .expect("Failed to register generic-empty-diff template");

    // Icon templates (used as partials)
    hbs.register_template_string("icon-file", ICON_FILE)
        .expect("Failed to register icon-file template");
    hbs.register_template_string("icon-file-added", ICON_FILE_ADDED)
        .expect("Failed to register icon-file-added template");
    hbs.register_template_string("icon-file-changed", ICON_FILE_CHANGED)
        .expect("Failed to register icon-file-changed template");
    hbs.register_template_string("icon-file-deleted", ICON_FILE_DELETED)
        .expect("Failed to register icon-file-deleted template");
    hbs.register_template_string("icon-file-renamed", ICON_FILE_RENAMED)
        .expect("Failed to register icon-file-renamed template");

    // Tag templates (used as partials)
    hbs.register_template_string("tag-file-added", TAG_FILE_ADDED)
        .expect("Failed to register tag-file-added template");
    hbs.register_template_string("tag-file-changed", TAG_FILE_CHANGED)
        .expect("Failed to register tag-file-changed template");
    hbs.register_template_string("tag-file-deleted", TAG_FILE_DELETED)
        .expect("Failed to register tag-file-deleted template");
    hbs.register_template_string("tag-file-renamed", TAG_FILE_RENAMED)
        .expect("Failed to register tag-file-renamed template");
}

/// Render a template with the given data.
///
/// # Arguments
///
/// * `template` - The template to render
/// * `data` - Data to pass to the template (must implement Serialize)
///
/// # Panics
///
/// Panics if template rendering fails, which indicates a bug in the code
/// (wrong data structure or type mismatch).
///
/// # Example
///
/// ```
/// use diff2html::templates::{render, TemplateName};
/// use serde_json::json;
///
/// let html = render(TemplateName::GenericWrapper, &json!({
///     "colorScheme": "d2h-light-color-scheme",
///     "content": "<p>Hello</p>"
/// }));
/// ```
pub fn render<T: Serialize>(template: TemplateName, data: &T) -> String {
    TEMPLATES
        .render(template.as_str(), data)
        .unwrap_or_else(|e| panic!("Failed to render template '{}': {}", template.as_str(), e))
}

/// Render a template by name with the given data.
///
/// This is useful when you need to render a template that was dynamically
/// determined at runtime.
///
/// # Arguments
///
/// * `name` - The template name as a string
/// * `data` - Data to pass to the template (must implement Serialize)
///
/// # Panics
///
/// Panics if template rendering fails, which indicates a bug in the code
/// (wrong data structure, type mismatch, or invalid template name).
pub fn render_by_name<T: Serialize>(name: &str, data: &T) -> String {
    TEMPLATES
        .render(name, data)
        .unwrap_or_else(|e| panic!("Failed to render template '{}': {}", name, e))
}

/// Get access to the global Handlebars registry.
///
/// This is useful for advanced use cases where you need direct access
/// to the template engine.
pub fn get_registry() -> &'static Handlebars<'static> {
    &TEMPLATES
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_render_generic_wrapper() {
        let result = render(
            TemplateName::GenericWrapper,
            &json!({
                "colorScheme": "d2h-light-color-scheme",
                "content": "<p>Test content</p>"
            }),
        );

        assert!(result.contains("d2h-wrapper"));
        assert!(result.contains("d2h-light-color-scheme"));
        assert!(result.contains("<p>Test content</p>"));
    }

    #[test]
    fn test_render_file_summary_wrapper() {
        let result = render(
            TemplateName::FileSummaryWrapper,
            &json!({
                "colorScheme": "",
                "filesNumber": 3,
                "files": "<li>file1.txt</li>"
            }),
        );

        assert!(result.contains("d2h-file-list-wrapper"));
        assert!(result.contains("Files changed (3)"));
        assert!(result.contains("<li>file1.txt</li>"));
    }

    #[test]
    fn test_render_line_by_line_numbers() {
        let result = render(
            TemplateName::LineByLineNumbers,
            &json!({
                "oldNumber": "10",
                "newNumber": "15"
            }),
        );

        assert!(result.contains("line-num1"));
        assert!(result.contains("line-num2"));
        assert!(result.contains("10"));
        assert!(result.contains("15"));
    }

    #[test]
    fn test_render_generic_line() {
        let result = render(
            TemplateName::GenericLine,
            &json!({
                "lineClass": "d2h-code-linenumber",
                "type": "d2h-ins",
                "lineNumber": "<div>1</div>",
                "contentClass": "d2h-code-line",
                "prefix": "+",
                "content": "new line"
            }),
        );

        assert!(result.contains("<tr>"));
        assert!(result.contains("d2h-ins"));
        assert!(result.contains("d2h-code-line-prefix"));
        assert!(result.contains("d2h-code-line-ctn"));
        assert!(result.contains("new line"));
    }

    #[test]
    fn test_render_generic_line_empty_content() {
        let result = render(
            TemplateName::GenericLine,
            &json!({
                "lineClass": "d2h-code-linenumber",
                "type": "d2h-cntx",
                "lineNumber": "",
                "contentClass": "d2h-code-line",
                "prefix": "",
                "content": ""
            }),
        );

        // When content is empty, should show <br>
        assert!(result.contains("<br>"));
    }

    #[test]
    fn test_render_tag_file_added() {
        let result = render(TemplateName::TagFileAdded, &json!({}));

        assert!(result.contains("d2h-tag"));
        assert!(result.contains("d2h-added-tag"));
        assert!(result.contains("ADDED"));
    }

    #[test]
    fn test_render_icon_file() {
        let result = render(TemplateName::IconFile, &json!({}));

        assert!(result.contains("<svg"));
        assert!(result.contains("d2h-icon"));
    }

    #[test]
    fn test_render_by_name() {
        let result = render_by_name(
            "generic-wrapper",
            &json!({
                "colorScheme": "",
                "content": "test"
            }),
        );

        assert!(result.contains("d2h-wrapper"));
    }

    #[test]
    fn test_css_embedded() {
        assert!(!CSS.is_empty());
        assert!(CSS.contains(".d2h-wrapper"));
        assert!(CSS.contains(".d2h-file-header"));
        assert!(CSS.contains("--d2h-bg-color"));
    }

    #[test]
    fn test_template_name_as_str() {
        assert_eq!(TemplateName::GenericWrapper.as_str(), "generic-wrapper");
        assert_eq!(
            TemplateName::LineByLineFileDiff.as_str(),
            "line-by-line-file-diff"
        );
        assert_eq!(TemplateName::IconFileAdded.as_str(), "icon-file-added");
    }

    #[test]
    fn test_render_line_by_line_file_diff() {
        let result = render(
            TemplateName::LineByLineFileDiff,
            &json!({
                "fileHtmlId": "d2h-123456",
                "file": {
                    "language": "rust"
                },
                "filePath": "<span>test.rs</span>",
                "diffs": "<tr><td>content</td></tr>"
            }),
        );

        assert!(result.contains("d2h-file-wrapper"));
        assert!(result.contains("d2h-123456"));
        assert!(result.contains("data-lang=\"rust\""));
        assert!(result.contains("<span>test.rs</span>"));
    }

    #[test]
    fn test_render_side_by_side_file_diff() {
        let result = render(
            TemplateName::SideBySideFileDiff,
            &json!({
                "fileHtmlId": "d2h-789",
                "file": {
                    "language": "js"
                },
                "filePath": "<span>app.js</span>",
                "diffs": {
                    "left": "<tr><td>old</td></tr>",
                    "right": "<tr><td>new</td></tr>"
                }
            }),
        );

        assert!(result.contains("d2h-files-diff"));
        assert!(result.contains("d2h-file-side-diff"));
        assert!(result.contains("<tr><td>old</td></tr>"));
        assert!(result.contains("<tr><td>new</td></tr>"));
    }

    #[test]
    fn test_render_generic_empty_diff() {
        let result = render(
            TemplateName::GenericEmptyDiff,
            &json!({
                "CSSLineClass": {
                    "INFO": "d2h-info"
                },
                "contentClass": "d2h-code-line"
            }),
        );

        assert!(result.contains("File without changes"));
    }

    #[test]
    fn test_render_generic_block_header() {
        let result = render(
            TemplateName::GenericBlockHeader,
            &json!({
                "lineClass": "d2h-code-linenumber",
                "CSSLineClass": {
                    "INFO": "d2h-info"
                },
                "contentClass": "d2h-code-line",
                "blockHeader": "@@ -1,3 +1,4 @@"
            }),
        );

        assert!(result.contains("d2h-info"));
        assert!(result.contains("@@ -1,3 +1,4 @@"));
    }
}
