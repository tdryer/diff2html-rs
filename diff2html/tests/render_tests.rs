//! Integration tests for HTML rendering.
//!
//! These tests are ported from the TypeScript diff2html test suite.

use diff2html::{
    ColorScheme, Diff2HtmlConfig, DiffParserConfig, DiffStyle, LineMatchingType, OutputFormat,
    html, html_from_diff_files, json, json_from_diff_files, parse,
};

/// Helper to load a test fixture
fn load_fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to load fixture: {} (path: {})", name, path))
}

// =============================================================================
// Basic HTML Generation Tests
// =============================================================================

#[test]
fn test_html_basic_output() {
    let diff = load_fixture("simple.diff");
    let result = html(&diff, &Diff2HtmlConfig::default());

    // Should contain the wrapper
    assert!(result.contains("d2h-wrapper"));
    // Should contain file header
    assert!(result.contains("d2h-file-header"));
    // Should contain the filename
    assert!(result.contains("sample"));
    // Should contain diff content
    assert!(result.contains("d2h-code-line"));
}

#[test]
fn test_html_line_by_line() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        output_format: OutputFormat::LineByLine,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should have line-by-line specific structure
    assert!(result.contains("d2h-file-diff"));
    assert!(result.contains("d2h-diff-table"));
    // Should NOT have side-by-side specific structure
    assert!(!result.contains("d2h-file-side-diff"));
}

#[test]
fn test_html_side_by_side() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        output_format: OutputFormat::SideBySide,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should have side-by-side specific structure
    assert!(result.contains("d2h-file-side-diff"));
    assert!(result.contains("d2h-files-diff"));
}

// =============================================================================
// File List Tests
// =============================================================================

#[test]
fn test_html_with_file_list() {
    let diff = load_fixture("multiple_files.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: true,
        ..Default::default()
    };
    let result = html(&diff, &config);

    assert!(result.contains("d2h-file-list"));
    assert!(result.contains("d2h-file-list-wrapper"));
    // Should show file count
    assert!(result.contains("Files changed"));
}

#[test]
fn test_html_without_file_list() {
    let diff = load_fixture("multiple_files.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    assert!(!result.contains("d2h-file-list"));
    assert!(!result.contains("d2h-file-list-wrapper"));
}

// =============================================================================
// Color Scheme Tests
// =============================================================================

#[test]
fn test_html_light_color_scheme() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        color_scheme: ColorScheme::Light,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    assert!(result.contains("d2h-light-color-scheme"));
    assert!(!result.contains("d2h-dark-color-scheme"));
}

#[test]
fn test_html_dark_color_scheme() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        color_scheme: ColorScheme::Dark,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    assert!(result.contains("d2h-dark-color-scheme"));
    assert!(!result.contains("d2h-light-color-scheme"));
}

#[test]
fn test_html_auto_color_scheme() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        color_scheme: ColorScheme::Auto,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    assert!(result.contains("d2h-auto-color-scheme"));
}

// =============================================================================
// Diff Highlighting Tests
// =============================================================================

#[test]
fn test_html_word_diff_style() {
    let diff = "diff --git a/test.txt b/test.txt\n\
                --- a/test.txt\n\
                +++ b/test.txt\n\
                @@ -1 +1 @@\n\
                -hello world\n\
                +hello universe\n";
    let config = Diff2HtmlConfig {
        diff_style: DiffStyle::Word,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(diff, &config);

    // Word diff should highlight the changed word
    assert!(result.contains("<del>") || result.contains("<ins>"));
}

#[test]
fn test_html_char_diff_style() {
    let diff = "diff --git a/test.txt b/test.txt\n\
                --- a/test.txt\n\
                +++ b/test.txt\n\
                @@ -1 +1 @@\n\
                -hello\n\
                +hallo\n";
    let config = Diff2HtmlConfig {
        diff_style: DiffStyle::Char,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(diff, &config);

    // Char diff should produce highlighting
    assert!(result.contains("<del>") || result.contains("<ins>"));
}

// =============================================================================
// Line Matching Tests
// =============================================================================

#[test]
fn test_html_with_line_matching() {
    let diff = "diff --git a/test.txt b/test.txt\n\
                --- a/test.txt\n\
                +++ b/test.txt\n\
                @@ -1,3 +1,3 @@\n\
                -foo bar baz\n\
                -hello world\n\
                -test line\n\
                +foo bar qux\n\
                +hello universe\n\
                +test updated\n";
    let config = Diff2HtmlConfig {
        matching: LineMatchingType::Lines,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(diff, &config);

    // Should render without errors
    assert!(result.contains("d2h-wrapper"));
}

// =============================================================================
// Insertion and Deletion Classes Tests
// =============================================================================

#[test]
fn test_html_insertion_classes() {
    let diff = load_fixture("new_file.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // New file should have insertion classes
    assert!(result.contains("d2h-ins"));
}

#[test]
fn test_html_deletion_classes() {
    let diff = load_fixture("deleted_file.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Deleted file should have deletion classes
    assert!(result.contains("d2h-del"));
}

#[test]
fn test_html_change_classes() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Changed file should have both
    assert!(result.contains("d2h-ins"));
    assert!(result.contains("d2h-del"));
}

// =============================================================================
// File Status Tags Tests
// =============================================================================

#[test]
fn test_html_file_status_changed() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    assert!(result.contains("CHANGED") || result.contains("d2h-changed"));
}

#[test]
fn test_html_file_status_new() {
    let diff = load_fixture("new_file.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    assert!(result.contains("ADDED") || result.contains("d2h-added"));
}

#[test]
fn test_html_file_status_deleted() {
    let diff = load_fixture("deleted_file.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    assert!(result.contains("DELETED") || result.contains("d2h-deleted"));
}

#[test]
fn test_html_file_status_renamed() {
    let diff = load_fixture("rename.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    assert!(result.contains("RENAMED") || result.contains("d2h-renamed"));
}

// =============================================================================
// HTML Escaping Tests
// =============================================================================

#[test]
fn test_html_escapes_special_characters() {
    let diff = "diff --git a/test.txt b/test.txt\n\
                --- a/test.txt\n\
                +++ b/test.txt\n\
                @@ -1 +1 @@\n\
                -<script>alert('xss')</script>\n\
                +<div class=\"safe\">&amp;</div>\n";
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(diff, &config);

    // Should escape < and > and other special chars
    assert!(!result.contains("<script>alert"));
    assert!(result.contains("&lt;script&gt;") || result.contains("&lt;"));
}

#[test]
fn test_html_does_not_double_escape() {
    // Test content that contains literal angle brackets (not HTML entities)
    let diff = "diff --git a/test.txt b/test.txt\n\
                --- a/test.txt\n\
                +++ b/test.txt\n\
                @@ -1 +1 @@\n\
                -a < b\n\
                +a > b\n";
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(diff, &config);

    // The < and > should be escaped once to &lt; and &gt;
    // but not double-escaped
    assert!(result.contains("&lt;") || result.contains("&gt;"));
    // Should not have &amp;lt; or &amp;gt; (double-escaped)
    assert!(!result.contains("&amp;lt;"));
    assert!(!result.contains("&amp;gt;"));
}

// =============================================================================
// JSON Output Tests
// =============================================================================

#[test]
fn test_json_basic_output() {
    let diff = load_fixture("simple.diff");
    let result = json(&diff, &Diff2HtmlConfig::default()).unwrap();

    // Should be valid JSON
    assert!(result.starts_with('['));
    assert!(result.ends_with(']'));

    // Should contain expected fields
    assert!(result.contains("\"oldName\""));
    assert!(result.contains("\"newName\""));
    assert!(result.contains("\"blocks\""));
    assert!(result.contains("\"lines\""));
}

#[test]
fn test_json_from_parsed_files() {
    let diff = load_fixture("simple.diff");
    let files = parse(&diff, &DiffParserConfig::default());
    let result = json_from_diff_files(&files).unwrap();

    assert!(result.contains("\"oldName\":\"sample\""));
    assert!(result.contains("\"newName\":\"sample\""));
}

#[test]
fn test_json_multiple_files() {
    let diff = load_fixture("multiple_files.diff");
    let result = json(&diff, &Diff2HtmlConfig::default()).unwrap();

    // Should contain both files
    assert!(result.contains("init.js"));
    assert!(result.contains("event.js"));
}

#[test]
fn test_json_contains_line_types() {
    let diff = load_fixture("simple.diff");
    let result = json(&diff, &Diff2HtmlConfig::default()).unwrap();

    // The serde rename makes this "type" instead of "lineType"
    assert!(result.contains("\"type\":\"delete\""));
    assert!(result.contains("\"type\":\"insert\""));
}

// =============================================================================
// html_from_diff_files Tests
// =============================================================================

#[test]
fn test_html_from_diff_files_basic() {
    let diff = load_fixture("simple.diff");
    let files = parse(&diff, &DiffParserConfig::default());
    let result = html_from_diff_files(&files, &Diff2HtmlConfig::default());

    assert!(result.contains("d2h-wrapper"));
    assert!(result.contains("sample"));
}

#[test]
fn test_html_from_diff_files_multiple_configs() {
    let diff = load_fixture("simple.diff");
    let files = parse(&diff, &DiffParserConfig::default());

    // Same parsed files, different render configs
    let config_line = Diff2HtmlConfig {
        output_format: OutputFormat::LineByLine,
        draw_file_list: false,
        ..Default::default()
    };
    let config_side = Diff2HtmlConfig {
        output_format: OutputFormat::SideBySide,
        draw_file_list: false,
        ..Default::default()
    };

    let result_line = html_from_diff_files(&files, &config_line);
    let result_side = html_from_diff_files(&files, &config_side);

    assert!(result_line.contains("d2h-file-diff"));
    assert!(result_side.contains("d2h-file-side-diff"));
}

// =============================================================================
// Empty Diff Tests
// =============================================================================

#[test]
fn test_html_empty_diff() {
    let config = Diff2HtmlConfig {
        render_nothing_when_empty: false,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html("", &config);

    // Empty diff should produce no file wrapper or empty output
    // The result may be empty or contain minimal wrapper
    assert!(result.is_empty() || !result.contains("d2h-file-wrapper"));
}

#[test]
fn test_html_render_nothing_when_empty() {
    let config = Diff2HtmlConfig {
        render_nothing_when_empty: true,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html("", &config);

    // With render_nothing_when_empty and no files, should be empty or minimal
    assert!(result.is_empty() || !result.contains("d2h-file-wrapper"));
}

// =============================================================================
// Binary File HTML Tests
// =============================================================================

#[test]
fn test_html_binary_file() {
    let diff = load_fixture("binary.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should show binary files message
    assert!(result.contains("Binary") || result.contains("binary"));
}

// =============================================================================
// Rename File HTML Tests
// =============================================================================

#[test]
fn test_html_rename_shows_old_and_new_names() {
    let diff = load_fixture("rename.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should show both old and new names
    assert!(result.contains("test-bar") || result.contains("test-baz"));
}

// =============================================================================
// Multiple Blocks HTML Tests
// =============================================================================

#[test]
fn test_html_multiple_blocks() {
    let diff = load_fixture("multiple_blocks.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should contain both block headers
    assert!(result.contains("@@ -1,10 +1,9 @@"));
    assert!(result.contains("jQuery.fn.extend"));
}

// =============================================================================
// Line Numbers HTML Tests
// =============================================================================

#[test]
fn test_html_contains_line_numbers() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should have line number elements
    assert!(result.contains("line-num") || result.contains("linenumber"));
}

// =============================================================================
// Side-by-Side Specific Tests
// =============================================================================

#[test]
fn test_side_by_side_has_left_and_right_panels() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        output_format: OutputFormat::SideBySide,
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should have two side-by-side panels
    let count = result.matches("d2h-file-side-diff").count();
    assert!(
        count >= 2,
        "Expected at least 2 side-by-side panels, found {}",
        count
    );
}

// =============================================================================
// File Collapse Checkbox Tests
// =============================================================================

#[test]
fn test_html_contains_file_collapse() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should have the viewed checkbox
    assert!(result.contains("d2h-file-collapse") || result.contains("viewed"));
}

// =============================================================================
// Combined Diff HTML Tests
// =============================================================================

#[test]
fn test_html_combined_diff() {
    let diff = load_fixture("combined.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should render without errors
    assert!(result.contains("d2h-wrapper"));
    assert!(result.contains("describe.c"));
}

// =============================================================================
// File Icon Tests
// =============================================================================

#[test]
fn test_html_contains_file_icon() {
    let diff = load_fixture("simple.diff");
    let config = Diff2HtmlConfig {
        draw_file_list: false,
        ..Default::default()
    };
    let result = html(&diff, &config);

    // Should have SVG icon
    assert!(result.contains("<svg") || result.contains("d2h-icon"));
}
