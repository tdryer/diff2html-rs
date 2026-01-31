//! Integration tests for the diff parser.
//!
//! These tests are ported from the TypeScript diff2html test suite.

use diff2html::{Checksum, DiffParserConfig, FileMode, LineType, parse};

/// Helper to load a test fixture
fn load_fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to load fixture: {} (path: {})", name, path))
}

// =============================================================================
// Basic Parsing Tests
// =============================================================================

#[test]
fn test_parse_simple_diff() {
    let diff = load_fixture("simple.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    assert_eq!(file.old_name, "sample");
    assert_eq!(file.new_name, "sample");
    assert_eq!(file.added_lines, 1);
    assert_eq!(file.deleted_lines, 1);
    assert!(file.is_git_diff);
    assert!(!file.is_combined);

    assert_eq!(file.blocks.len(), 1);
    let block = &file.blocks[0];
    assert_eq!(block.header, "@@ -1 +1 @@");
    assert_eq!(block.lines.len(), 2);
    assert_eq!(block.lines[0].line_type, LineType::Delete);
    assert_eq!(block.lines[0].content, "-test");
    assert_eq!(block.lines[1].line_type, LineType::Insert);
    assert_eq!(block.lines[1].content, "+test1r");
}

#[test]
fn test_parse_unix_line_endings() {
    let diff = "diff --git a/sample b/sample\n\
                index 0000001..0ddf2ba\n\
                --- a/sample\n\
                +++ b/sample\n\
                @@ -1 +1 @@\n\
                -test\n\
                +test1r\n";
    let result = parse(diff, &DiffParserConfig::default());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].blocks[0].lines.len(), 2);
}

#[test]
fn test_parse_windows_line_endings() {
    let diff = "diff --git a/sample b/sample\r\n\
                index 0000001..0ddf2ba\r\n\
                --- a/sample\r\n\
                +++ b/sample\r\n\
                @@ -1 +1 @@\r\n\
                -test\r\n\
                +test1r\r\n";
    let result = parse(diff, &DiffParserConfig::default());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].blocks[0].lines.len(), 2);
}

#[test]
fn test_parse_old_mac_line_endings() {
    let diff = "diff --git a/sample b/sample\r\
                index 0000001..0ddf2ba\r\
                --- a/sample\r\
                +++ b/sample\r\
                @@ -1 +1 @@\r\
                -test\r\
                +test1r\r";
    let result = parse(diff, &DiffParserConfig::default());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].blocks[0].lines.len(), 2);
}

#[test]
fn test_parse_mixed_line_endings() {
    let diff = "diff --git a/sample b/sample\n\
                index 0000001..0ddf2ba\r\n\
                --- a/sample\r\
                +++ b/sample\r\n\
                @@ -1 +1 @@\n\
                -test\r\
                +test1r\n";
    let result = parse(diff, &DiffParserConfig::default());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].blocks[0].lines.len(), 2);
}

// =============================================================================
// New File Tests
// =============================================================================

#[test]
fn test_parse_new_file() {
    let diff = load_fixture("new_file.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    assert_eq!(file.old_name, "/dev/null");
    assert_eq!(file.new_name, "test.js");
    assert_eq!(file.is_new, Some(true));
    assert_eq!(file.added_lines, 5);
    assert_eq!(file.deleted_lines, 0);
    assert_eq!(file.new_file_mode.as_deref(), Some("100644"));
}

// =============================================================================
// Deleted File Tests
// =============================================================================

#[test]
fn test_parse_deleted_file() {
    let diff = load_fixture("deleted_file.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    assert_eq!(file.old_name, "src/var/strundefined.js");
    assert_eq!(file.new_name, "/dev/null");
    assert_eq!(file.is_deleted, Some(true));
    assert_eq!(file.added_lines, 0);
    assert_eq!(file.deleted_lines, 3);
    assert_eq!(file.deleted_file_mode.as_deref(), Some("100644"));
}

// =============================================================================
// Multiple Files Tests
// =============================================================================

#[test]
fn test_parse_multiple_files() {
    let diff = load_fixture("multiple_files.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 2);

    // First file
    assert_eq!(result[0].old_name, "src/core/init.js");
    assert_eq!(result[0].new_name, "src/core/init.js");
    assert_eq!(result[0].blocks.len(), 1);
    assert_eq!(result[0].added_lines, 1);
    assert_eq!(result[0].deleted_lines, 1);

    // Second file
    assert_eq!(result[1].old_name, "src/event.js");
    assert_eq!(result[1].new_name, "src/event.js");
    assert_eq!(result[1].blocks.len(), 1);
    assert_eq!(result[1].deleted_lines, 1);
}

// =============================================================================
// Multiple Blocks Tests
// =============================================================================

#[test]
fn test_parse_multiple_blocks() {
    let diff = load_fixture("multiple_blocks.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    assert_eq!(file.blocks.len(), 2);

    // First block
    assert_eq!(file.blocks[0].old_start_line, 1);
    assert_eq!(file.blocks[0].new_start_line, 1);
    assert_eq!(file.blocks[0].header, "@@ -1,10 +1,9 @@");

    // Second block
    assert_eq!(file.blocks[1].old_start_line, 128);
    assert_eq!(file.blocks[1].new_start_line, 127);
    assert!(file.blocks[1].header.contains("jQuery.fn.extend"));
}

// =============================================================================
// Rename Tests
// =============================================================================

#[test]
fn test_parse_rename() {
    let diff = load_fixture("rename.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    assert_eq!(file.old_name, "src/test-bar.js");
    assert_eq!(file.new_name, "src/test-baz.js");
    assert_eq!(file.is_rename, Some(true));
    assert_eq!(file.unchanged_percentage, Some(98));
}

// =============================================================================
// Copy Tests
// =============================================================================

#[test]
fn test_parse_copy() {
    let diff = load_fixture("copy.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    assert_eq!(file.old_name, "index.js");
    assert_eq!(file.new_name, "more-index.js");
    assert_eq!(file.is_copy, Some(true));
    assert_eq!(file.changed_percentage, Some(5));
}

// =============================================================================
// Binary File Tests
// =============================================================================

#[test]
fn test_parse_binary_file() {
    let diff = load_fixture("binary.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    assert_eq!(file.old_name, "last-changes-config.png");
    assert_eq!(file.new_name, "last-changes-config.png");
    assert_eq!(file.language.as_str(), "png");

    assert_eq!(file.blocks.len(), 1);
    assert_eq!(file.blocks[0].header, "Binary files differ");
    assert!(file.blocks[0].lines.is_empty());
}

// =============================================================================
// Combined Diff Tests
// =============================================================================

#[test]
fn test_parse_combined_diff() {
    let diff = load_fixture("combined.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    assert!(file.is_combined);
    assert_eq!(file.old_name, "describe.c");
    assert_eq!(file.new_name, "describe.c");

    // Combined diffs have oldStartLine2
    let block = &file.blocks[0];
    assert!(block.old_start_line2.is_some());
    assert_eq!(block.old_start_line, 98);
    assert_eq!(block.old_start_line2, Some(98));
}

// =============================================================================
// Non-Git Unified Diff Tests
// =============================================================================

#[test]
fn test_parse_unified_non_git() {
    let diff = load_fixture("unified_non_git.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    assert_eq!(file.old_name, "sample.js");
    assert_eq!(file.new_name, "sample.js");
    assert!(!file.is_git_diff);
    assert_eq!(file.language.as_str(), "js");
    assert_eq!(file.added_lines, 2);
    assert_eq!(file.deleted_lines, 1);
}

#[test]
fn test_parse_unified_with_timestamps() {
    let diff = load_fixture("unified_with_timestamps.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    // Timestamps should be stripped
    assert_eq!(file.old_name, "sample.js");
    assert_eq!(file.new_name, "sample.js");
}

// =============================================================================
// Special Characters Tests
// =============================================================================

#[test]
fn test_parse_special_characters() {
    let diff = load_fixture("special_chars.diff");
    let result = parse(&diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    // Filename should contain tab character
    assert!(file.new_name.contains('\t') || file.new_name.contains("tab"));
    assert_eq!(file.language.as_str(), "scala");
    assert_eq!(file.added_lines, 2);
    assert_eq!(file.deleted_lines, 1);
}

// =============================================================================
// Line Number Tests
// =============================================================================

#[test]
fn test_line_numbers_context() {
    let diff = "diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 context1
-old
+new
 context2
";
    let result = parse(diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let lines = &result[0].blocks[0].lines;

    // Context line
    assert_eq!(lines[0].old_number, Some(1));
    assert_eq!(lines[0].new_number, Some(1));

    // Delete line
    assert_eq!(lines[1].old_number, Some(2));
    assert_eq!(lines[1].new_number, None);

    // Insert line
    assert_eq!(lines[2].old_number, None);
    assert_eq!(lines[2].new_number, Some(2));

    // Context line
    assert_eq!(lines[3].old_number, Some(3));
    assert_eq!(lines[3].new_number, Some(3));
}

#[test]
fn test_line_numbers_with_offset() {
    let diff = "diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -10,3 +20,3 @@
 context
-old
+new
";
    let result = parse(diff, &DiffParserConfig::default());

    let lines = &result[0].blocks[0].lines;
    assert_eq!(lines[0].old_number, Some(10));
    assert_eq!(lines[0].new_number, Some(20));
}

// =============================================================================
// Checksum Tests
// =============================================================================

#[test]
fn test_parse_checksums() {
    let diff = "diff --git a/sample b/sample\n\
                index 0000001..0ddf2ba\n\
                --- a/sample\n\
                +++ b/sample\n\
                @@ -1 +1 @@\n\
                -test\n\
                +test1r\n";
    let result = parse(diff, &DiffParserConfig::default());

    let file = &result[0];
    match &file.checksum_before {
        Some(Checksum::Single(s)) => assert_eq!(s, "0000001"),
        _ => panic!("Expected single checksum"),
    }
    assert_eq!(file.checksum_after.as_deref(), Some("0ddf2ba"));
}

// =============================================================================
// Configuration Tests
// =============================================================================

#[test]
fn test_parse_with_custom_prefixes() {
    let diff = "diff --git i/sample w/sample\n\
                index 0000001..0ddf2ba\n\
                --- i/sample\n\
                +++ w/sample\n\
                @@ -1 +1 @@\n\
                -test\n\
                +test1r\n";
    let config = DiffParserConfig {
        src_prefix: Some("i/".to_string()),
        dst_prefix: Some("w/".to_string()),
        ..Default::default()
    };
    let result = parse(diff, &config);

    assert_eq!(result[0].old_name, "sample");
    assert_eq!(result[0].new_name, "sample");
}

#[test]
fn test_parse_diff_max_changes() {
    // Create a large diff
    let mut diff = "diff --git a/large.txt b/large.txt\n\
                    --- a/large.txt\n\
                    +++ b/large.txt\n\
                    @@ -1,100 +1,100 @@\n"
        .to_string();
    for i in 0..50 {
        diff.push_str(&format!("-line{}\n", i));
        diff.push_str(&format!("+newline{}\n", i));
    }

    let config = DiffParserConfig {
        diff_max_changes: Some(10),
        ..Default::default()
    };
    let result = parse(&diff, &config);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].is_too_big, Some(true));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_parse_empty_diff() {
    let result = parse("", &DiffParserConfig::default());
    assert!(result.is_empty());
}

#[test]
fn test_parse_no_newline_at_eof() {
    let diff = "diff --git a/sample.scala b/sample.scala\n\
                index b583263..8b2fc3e 100644\n\
                --- a/sample.scala\n\
                +++ b/sample.scala\n\
                @@ -1,2 +1,2 @@\n\
                -old\n\
                \\ No newline at end of file\n\
                +new\n";
    let result = parse(diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    // The "no newline" marker should be handled
    let block = &result[0].blocks[0];
    assert!(!block.lines.is_empty());
}

#[test]
fn test_parse_diff_with_context_in_content() {
    // Test that --- and +++ in content don't confuse the parser
    let diff = "--- sample.js\n\
                +++ sample.js\n\
                @@ -1,8 +1,8 @@\n\
                 test\n\
                 \n\
                -- 1\n\
                --- 1\n\
                ---- 1\n\
                 \n\
                ++ 2\n\
                +++ 2\n\
                ++++ 2";
    let result = parse(diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    // Lines starting with -- and --- should be parsed as deletions (not as file headers)
    let lines = &result[0].blocks[0].lines;
    assert!(lines.iter().any(|l| l.content.contains("-- 1")));
    assert!(lines.iter().any(|l| l.content.contains("--- 1")));
}

#[test]
fn test_parse_unified_diff_multiple_hunks_and_files() {
    let diff = "--- sample.js\n\
                +++ sample.js\n\
                @@ -1 +1,2 @@\n\
                -test\n\
                @@ -10 +20,2 @@\n\
                +test\n\
                --- sample1.js\n\
                +++ sample1.js\n\
                @@ -1 +1,2 @@\n\
                +test1";
    let result = parse(diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].blocks.len(), 2);
    assert_eq!(result[1].blocks.len(), 1);
}

#[test]
fn test_parse_diff_without_proper_hunk_headers() {
    let diff = "--- sample.js\n\
                +++ sample.js\n\
                @@ @@\n\
                 test";
    let result = parse(diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].blocks.len(), 1);
    assert_eq!(result[0].blocks[0].old_start_line, 0);
    assert_eq!(result[0].blocks[0].new_start_line, 0);
}

// =============================================================================
// File Mode Tests
// =============================================================================

#[test]
fn test_parse_mode_change() {
    let diff = "diff --git a/script.sh b/script.sh\n\
                old mode 100644\n\
                new mode 100755\n\
                index abc1234..def5678\n\
                --- a/script.sh\n\
                +++ b/script.sh\n\
                @@ -1 +1 @@\n\
                -echo old\n\
                +echo new\n";
    let result = parse(diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    let file = &result[0];
    match &file.old_mode {
        Some(FileMode::Single(m)) => assert_eq!(m, "100644"),
        _ => panic!("Expected single old mode"),
    }
    assert_eq!(file.new_mode.as_deref(), Some("100755"));
}

// =============================================================================
// Nested Diff Tests
// =============================================================================

#[test]
fn test_parse_nested_diff_content() {
    // A diff that contains a diff as content (e.g., in a test file)
    let diff = "diff --git a/src/offset.js b/src/offset.js\n\
                index cc6ffb4..fa51f18 100644\n\
                --- a/src/offset.js\n\
                +++ b/src/offset.js\n\
                @@ -1,3 +1,3 @@\n\
                +var text = 'diff --git a/test b/test\\n--- a/test\\n+++ b/test';\n\
                 console.log(text);\n\
                -var old = true;\n";
    let result = parse(diff, &DiffParserConfig::default());

    assert_eq!(result.len(), 1);
    // Should only parse as one file, not be confused by nested diff content
    assert_eq!(result[0].old_name, "src/offset.js");
}
