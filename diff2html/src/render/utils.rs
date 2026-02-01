//! Rendering utilities for diff2html.
//!
//! This module provides shared utilities used by the renderers including
//! HTML escaping, line deconstruction, diff highlighting, and CSS class mappings.

use regex::Regex;
use similar::{ChangeTag, TextDiff};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::LazyLock;

use crate::types::{ColorScheme, DiffFile, DiffLineParts, DiffStyle, LineMatchingType, LineType};

/// CSS class names for diff line types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSSLineClass {
    Inserts,
    Deletes,
    Context,
    Info,
    InsertChanges,
    DeleteChanges,
}

impl CSSLineClass {
    /// Returns the CSS class string for this line class.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inserts => "d2h-ins",
            Self::Deletes => "d2h-del",
            Self::Context => "d2h-cntx",
            Self::Info => "d2h-info",
            Self::InsertChanges => "d2h-ins d2h-change",
            Self::DeleteChanges => "d2h-del d2h-change",
        }
    }
}

impl std::fmt::Display for CSSLineClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Highlighted line content after diff highlighting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedLines {
    pub old_line: DiffLineParts,
    pub new_line: DiffLineParts,
}

/// Configuration for rendering.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub matching: LineMatchingType,
    pub match_words_threshold: f64,
    pub max_line_length_highlight: usize,
    pub diff_style: DiffStyle,
    pub color_scheme: ColorScheme,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            matching: LineMatchingType::None,
            match_words_threshold: 0.25,
            max_line_length_highlight: 10000,
            diff_style: DiffStyle::Word,
            color_scheme: ColorScheme::Light,
        }
    }
}

/// Configuration for line-by-line and side-by-side renderers.
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub render: RenderConfig,
    pub render_nothing_when_empty: bool,
    pub matching_max_comparisons: usize,
    pub max_line_size_in_block_for_comparison: usize,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            render: RenderConfig::default(),
            render_nothing_when_empty: false,
            matching_max_comparisons: 2500,
            max_line_size_in_block_for_comparison: 200,
        }
    }
}

const SEPARATOR: char = '/';

/// Check if a filename represents /dev/null.
fn is_dev_null_name(name: &str) -> bool {
    name.contains("dev/null")
}

/// Unify path separators to forward slashes.
fn unify_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Get the prefix length for a diff line based on whether it's a combined diff.
fn prefix_length(is_combined: bool) -> usize {
    if is_combined { 2 } else { 1 }
}

/// Escape special characters for safe HTML rendering.
pub fn escape_for_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            '/' => result.push_str("&#x2F;"),
            _ => result.push(c),
        }
    }
    result
}

/// Deconstruct a diff line by separating the prefix from the content.
///
/// # Arguments
///
/// * `line` - The full diff line including prefix character(s)
/// * `is_combined` - Whether this is a combined diff (2-char prefix) or regular diff (1-char prefix)
/// * `escape` - Whether to HTML-escape the content
pub fn deconstruct_line(line: &str, is_combined: bool, escape: bool) -> DiffLineParts {
    let index_to_split = prefix_length(is_combined);

    // Safe slicing using get() - returns None if index is out of bounds or not on a char boundary
    let (prefix, content) = match (line.get(..index_to_split), line.get(index_to_split..)) {
        (Some(p), Some(c)) => (p, c),
        (Some(p), None) => (p, ""),
        (None, _) => (line, ""),
    };

    DiffLineParts {
        prefix: prefix.to_string(),
        content: if escape {
            escape_for_html(content)
        } else {
            content.to_string()
        },
    }
}

/// Generate a pretty filename diff showing renames.
///
/// Examples:
/// - `{oldName: "my/path/to/file.js", newName: "my/path/to/new-file.js"}`
///   returns `"my/path/to/{file.js -> new-file.js}"`
/// - `{oldName: "my/path/to/file.js", newName: "my/path/for/file.js"}`
///   returns `"my/path/{to -> for}/file.js"`
pub fn filename_diff(file: &DiffFile) -> String {
    let old_filename = unify_path(&file.old_name);
    let new_filename = unify_path(&file.new_name);

    if old_filename != new_filename
        && !is_dev_null_name(&old_filename)
        && !is_dev_null_name(&new_filename)
    {
        let old_parts: Vec<&str> = old_filename.split(SEPARATOR).collect();
        let new_parts: Vec<&str> = new_filename.split(SEPARATOR).collect();

        let old_len = old_parts.len();
        let new_len = new_parts.len();

        // Find common prefix
        let mut i = 0;
        let mut j = old_len.saturating_sub(1);
        let mut k = new_len.saturating_sub(1);

        let mut prefix_paths: Vec<&str> = Vec::new();
        let mut suffix_paths: Vec<&str> = Vec::new();

        while i < j && i < k {
            if old_parts[i] == new_parts[i] {
                prefix_paths.push(new_parts[i]);
                i += 1;
            } else {
                break;
            }
        }

        // Find common suffix
        while j > i && k > i {
            if old_parts[j] == new_parts[k] {
                suffix_paths.insert(0, new_parts[k]);
                j = j.saturating_sub(1);
                k = k.saturating_sub(1);
            } else {
                break;
            }
        }

        let final_prefix = prefix_paths.join(&SEPARATOR.to_string());
        let final_suffix = suffix_paths.join(&SEPARATOR.to_string());

        let old_remaining: Vec<&str> = old_parts[i..=j].to_vec();
        let new_remaining: Vec<&str> = new_parts[i..=k].to_vec();

        let old_remaining_path = old_remaining.join(&SEPARATOR.to_string());
        let new_remaining_path = new_remaining.join(&SEPARATOR.to_string());

        if !final_prefix.is_empty() && !final_suffix.is_empty() {
            format!(
                "{}{}{{{} \u{2192} {}}}{}{}",
                final_prefix,
                SEPARATOR,
                old_remaining_path,
                new_remaining_path,
                SEPARATOR,
                final_suffix
            )
        } else if !final_prefix.is_empty() {
            format!(
                "{}{}{{{} \u{2192} {}}}",
                final_prefix, SEPARATOR, old_remaining_path, new_remaining_path
            )
        } else if !final_suffix.is_empty() {
            format!(
                "{{{} \u{2192} {}}}{}{}",
                old_remaining_path, new_remaining_path, SEPARATOR, final_suffix
            )
        } else {
            format!("{} \u{2192} {}", old_filename, new_filename)
        }
    } else if !is_dev_null_name(&new_filename) {
        new_filename
    } else {
        old_filename
    }
}

/// Compute a hash code for a string.
fn hash_code(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Generate a unique HTML ID for a file diff.
pub fn get_html_id(file: &DiffFile) -> String {
    let hash = hash_code(&filename_diff(file));
    format!("d2h-{:06}", hash % 1_000_000)
}

/// Get the icon template name for a file based on its status.
pub fn get_file_icon(file: &DiffFile) -> &'static str {
    if file.is_rename == Some(true) || file.is_copy == Some(true) {
        "file-renamed"
    } else if file.is_new == Some(true) {
        "file-added"
    } else if file.is_deleted == Some(true) {
        "file-deleted"
    } else if file.new_name != file.old_name {
        "file-renamed"
    } else {
        "file-changed"
    }
}

/// Convert a color scheme to CSS class.
pub fn color_scheme_to_css(color_scheme: ColorScheme) -> &'static str {
    match color_scheme {
        ColorScheme::Dark => "d2h-dark-color-scheme",
        ColorScheme::Auto => "d2h-auto-color-scheme",
        ColorScheme::Light => "d2h-light-color-scheme",
    }
}

/// Convert a LineType to the corresponding CSS class.
pub fn to_css_class(line_type: LineType) -> CSSLineClass {
    match line_type {
        LineType::Context => CSSLineClass::Context,
        LineType::Insert => CSSLineClass::Inserts,
        LineType::Delete => CSSLineClass::Deletes,
    }
}

/// Regex pattern to match <ins> elements in HTML.
static INS_ELEMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<ins[^>]*>(.|\n)*?</ins>").unwrap());

/// Regex pattern to match <del> elements in HTML.
static DEL_ELEMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<del[^>]*>(.|\n)*?</del>").unwrap());

/// Remove <ins> elements from HTML string.
fn remove_ins_elements(line: &str) -> String {
    INS_ELEMENT_REGEX.replace_all(line, "").to_string()
}

/// Remove <del> elements from HTML string.
fn remove_del_elements(line: &str) -> String {
    DEL_ELEMENT_REGEX.replace_all(line, "").to_string()
}

/// Highlight differences between two diff lines.
///
/// Uses the `similar` crate to find word or character-level differences
/// and wraps them in `<ins>` and `<del>` tags.
pub fn diff_highlight(
    diff_line1: &str,
    diff_line2: &str,
    is_combined: bool,
    config: &RenderConfig,
) -> HighlightedLines {
    let line1 = deconstruct_line(diff_line1, is_combined, false);
    let line2 = deconstruct_line(diff_line2, is_combined, false);

    // If lines are too long, skip highlighting
    if line1.content.len() > config.max_line_length_highlight
        || line2.content.len() > config.max_line_length_highlight
    {
        return HighlightedLines {
            old_line: DiffLineParts {
                prefix: line1.prefix,
                content: escape_for_html(&line1.content),
            },
            new_line: DiffLineParts {
                prefix: line2.prefix,
                content: escape_for_html(&line2.content),
            },
        };
    }

    let diff = match config.diff_style {
        DiffStyle::Char => TextDiff::from_chars(&line1.content, &line2.content),
        DiffStyle::Word => TextDiff::from_words(&line1.content, &line2.content),
    };

    let mut highlighted_line = String::new();

    for change in diff.iter_all_changes() {
        let escaped_value = escape_for_html(change.value());
        match change.tag() {
            ChangeTag::Insert => {
                highlighted_line.push_str("<ins>");
                highlighted_line.push_str(&escaped_value);
                highlighted_line.push_str("</ins>");
            }
            ChangeTag::Delete => {
                highlighted_line.push_str("<del>");
                highlighted_line.push_str(&escaped_value);
                highlighted_line.push_str("</del>");
            }
            ChangeTag::Equal => {
                highlighted_line.push_str(&escaped_value);
            }
        }
    }

    HighlightedLines {
        old_line: DiffLineParts {
            prefix: line1.prefix,
            content: remove_ins_elements(&highlighted_line),
        },
        new_line: DiffLineParts {
            prefix: line2.prefix,
            content: remove_del_elements(&highlighted_line),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_for_html() {
        assert_eq!(escape_for_html("hello"), "hello");
        assert_eq!(escape_for_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_for_html("a & b"), "a &amp; b");
        assert_eq!(escape_for_html("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_for_html("it's"), "it&#x27;s");
        assert_eq!(escape_for_html("a/b"), "a&#x2F;b");
    }

    #[test]
    fn test_deconstruct_line_regular() {
        let parts = deconstruct_line("+hello", false, true);
        assert_eq!(parts.prefix, "+");
        assert_eq!(parts.content, "hello");

        let parts = deconstruct_line("-goodbye", false, true);
        assert_eq!(parts.prefix, "-");
        assert_eq!(parts.content, "goodbye");

        let parts = deconstruct_line(" unchanged", false, true);
        assert_eq!(parts.prefix, " ");
        assert_eq!(parts.content, "unchanged");
    }

    #[test]
    fn test_deconstruct_line_combined() {
        let parts = deconstruct_line("++hello", true, true);
        assert_eq!(parts.prefix, "++");
        assert_eq!(parts.content, "hello");

        let parts = deconstruct_line("- goodbye", true, true);
        assert_eq!(parts.prefix, "- ");
        assert_eq!(parts.content, "goodbye");
    }

    #[test]
    fn test_deconstruct_line_escaping() {
        let parts = deconstruct_line("+<html>", false, true);
        assert_eq!(parts.content, "&lt;html&gt;");

        let parts = deconstruct_line("+<html>", false, false);
        assert_eq!(parts.content, "<html>");
    }

    #[test]
    fn test_filename_diff_same_name() {
        let file = DiffFile {
            old_name: "test.txt".to_string(),
            new_name: "test.txt".to_string(),
            ..Default::default()
        };
        assert_eq!(filename_diff(&file), "test.txt");
    }

    #[test]
    fn test_filename_diff_different_file() {
        let file = DiffFile {
            old_name: "my/path/to/file.js".to_string(),
            new_name: "my/path/to/new-file.js".to_string(),
            ..Default::default()
        };
        assert_eq!(
            filename_diff(&file),
            "my/path/to/{file.js \u{2192} new-file.js}"
        );
    }

    #[test]
    fn test_filename_diff_different_dir() {
        let file = DiffFile {
            old_name: "my/path/to/file.js".to_string(),
            new_name: "my/path/for/file.js".to_string(),
            ..Default::default()
        };
        assert_eq!(filename_diff(&file), "my/path/{to \u{2192} for}/file.js");
    }

    #[test]
    fn test_filename_diff_completely_different() {
        let file = DiffFile {
            old_name: "old/file.js".to_string(),
            new_name: "new/other.js".to_string(),
            ..Default::default()
        };
        assert_eq!(filename_diff(&file), "old/file.js \u{2192} new/other.js");
    }

    #[test]
    fn test_filename_diff_dev_null() {
        let file = DiffFile {
            old_name: "/dev/null".to_string(),
            new_name: "new-file.txt".to_string(),
            ..Default::default()
        };
        assert_eq!(filename_diff(&file), "new-file.txt");

        let file = DiffFile {
            old_name: "deleted-file.txt".to_string(),
            new_name: "/dev/null".to_string(),
            ..Default::default()
        };
        assert_eq!(filename_diff(&file), "deleted-file.txt");
    }

    #[test]
    fn test_get_html_id() {
        let file = DiffFile {
            old_name: "test.txt".to_string(),
            new_name: "test.txt".to_string(),
            ..Default::default()
        };
        let id = get_html_id(&file);
        assert!(id.starts_with("d2h-"));
        assert_eq!(id.len(), 10); // "d2h-" + 6 digits
    }

    #[test]
    fn test_get_file_icon() {
        let mut file = DiffFile::default();
        assert_eq!(get_file_icon(&file), "file-changed");

        file.is_new = Some(true);
        assert_eq!(get_file_icon(&file), "file-added");

        file.is_new = None;
        file.is_deleted = Some(true);
        assert_eq!(get_file_icon(&file), "file-deleted");

        file.is_deleted = None;
        file.is_rename = Some(true);
        assert_eq!(get_file_icon(&file), "file-renamed");

        file.is_rename = None;
        file.is_copy = Some(true);
        assert_eq!(get_file_icon(&file), "file-renamed");

        file.is_copy = None;
        file.old_name = "old.txt".to_string();
        file.new_name = "new.txt".to_string();
        assert_eq!(get_file_icon(&file), "file-renamed");
    }

    #[test]
    fn test_color_scheme_to_css() {
        assert_eq!(
            color_scheme_to_css(ColorScheme::Light),
            "d2h-light-color-scheme"
        );
        assert_eq!(
            color_scheme_to_css(ColorScheme::Dark),
            "d2h-dark-color-scheme"
        );
        assert_eq!(
            color_scheme_to_css(ColorScheme::Auto),
            "d2h-auto-color-scheme"
        );
    }

    #[test]
    fn test_to_css_class() {
        assert_eq!(to_css_class(LineType::Context).as_str(), "d2h-cntx");
        assert_eq!(to_css_class(LineType::Insert).as_str(), "d2h-ins");
        assert_eq!(to_css_class(LineType::Delete).as_str(), "d2h-del");
    }

    #[test]
    fn test_css_line_class_display() {
        assert_eq!(format!("{}", CSSLineClass::Inserts), "d2h-ins");
        assert_eq!(
            format!("{}", CSSLineClass::InsertChanges),
            "d2h-ins d2h-change"
        );
    }

    #[test]
    fn test_diff_highlight_basic() {
        let config = RenderConfig::default();
        let result = diff_highlight("-old text", "+new text", false, &config);

        assert_eq!(result.old_line.prefix, "-");
        assert_eq!(result.new_line.prefix, "+");
        // The content should have del tags for old and ins tags for new
        assert!(result.old_line.content.contains("<del>"));
        assert!(result.new_line.content.contains("<ins>"));
    }

    #[test]
    fn test_diff_highlight_long_lines() {
        let config = RenderConfig {
            max_line_length_highlight: 5,
            ..Default::default()
        };
        let result = diff_highlight("-a longer line", "+another longer line", false, &config);

        // No highlighting should be applied due to length limit
        assert!(!result.old_line.content.contains("<del>"));
        assert!(!result.new_line.content.contains("<ins>"));
    }

    #[test]
    fn test_remove_ins_elements() {
        let input = "hello <ins>world</ins> test";
        let result = remove_ins_elements(input);
        assert_eq!(result, "hello  test");
    }

    #[test]
    fn test_remove_del_elements() {
        let input = "hello <del>world</del> test";
        let result = remove_del_elements(input);
        assert_eq!(result, "hello  test");
    }

    #[test]
    fn test_deconstruct_line_multibyte_chars() {
        // Test with multi-byte UTF-8 characters (emoji, CJK characters)
        let parts = deconstruct_line("+🎉hello", false, true);
        assert_eq!(parts.prefix, "+");
        assert_eq!(parts.content, "🎉hello");

        let parts = deconstruct_line("-中文", false, true);
        assert_eq!(parts.prefix, "-");
        assert_eq!(parts.content, "中文");

        // Test with combined diff
        let parts = deconstruct_line("++🚀test", true, true);
        assert_eq!(parts.prefix, "++");
        assert_eq!(parts.content, "🚀test");

        // Test edge case: line shorter than expected prefix
        let parts = deconstruct_line("a", false, true);
        assert_eq!(parts.prefix, "a");
        assert_eq!(parts.content, "");
    }
}
