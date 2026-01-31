//! File list renderer.
//!
//! This module provides a renderer for generating a summary list of changed files
//! with their add/delete statistics.

use serde_json::json;

use crate::templates::{self, TemplateName};
use crate::types::{ColorScheme, DiffFile};

use super::utils::{color_scheme_to_css, filename_diff, get_file_icon, get_html_id};

/// Configuration for the file list renderer.
#[derive(Debug, Clone)]
pub struct FileListConfig {
    pub color_scheme: ColorScheme,
}

impl Default for FileListConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Light,
        }
    }
}

/// File list renderer for generating a summary of changed files.
pub struct FileListRenderer {
    config: FileListConfig,
}

impl Default for FileListRenderer {
    fn default() -> Self {
        Self::new(FileListConfig::default())
    }
}

impl FileListRenderer {
    /// Create a new FileListRenderer with the given configuration.
    pub fn new(config: FileListConfig) -> Self {
        Self { config }
    }

    /// Render a list of diff files to a summary HTML.
    pub fn render(&self, diff_files: &[DiffFile]) -> String {
        let files_html: String = diff_files
            .iter()
            .map(|file| {
                let file_icon = get_file_icon(file);
                let file_icon_html =
                    templates::render_by_name(&format!("icon-{}", file_icon), &json!({}))
                        .unwrap_or_default();

                templates::render(
                    TemplateName::FileSummaryLine,
                    &json!({
                        "fileHtmlId": get_html_id(file),
                        "oldName": file.old_name,
                        "newName": file.new_name,
                        "fileName": filename_diff(file),
                        "deletedLines": format!("-{}", file.deleted_lines),
                        "addedLines": format!("+{}", file.added_lines),
                        "fileIcon": file_icon_html,
                    }),
                )
                .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join("\n");

        templates::render(
            TemplateName::FileSummaryWrapper,
            &json!({
                "colorScheme": color_scheme_to_css(self.config.color_scheme),
                "filesNumber": diff_files.len(),
                "files": files_html,
            }),
        )
        .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{DiffParserConfig, parse};

    fn sample_diff() -> &'static str {
        r#"diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,4 @@
 context line
-old line
+new line
+added line
"#
    }

    #[test]
    fn test_render_basic_file_list() {
        let files = parse(sample_diff(), &DiffParserConfig::default());
        let renderer = FileListRenderer::default();
        let html = renderer.render(&files);

        assert!(html.contains("d2h-file-list-wrapper"));
        assert!(html.contains("Files changed (1)"));
        assert!(html.contains("test.txt"));
    }

    #[test]
    fn test_render_empty_file_list() {
        let renderer = FileListRenderer::default();
        let html = renderer.render(&[]);

        assert!(html.contains("d2h-file-list-wrapper"));
        assert!(html.contains("Files changed (0)"));
    }

    #[test]
    fn test_render_multiple_files() {
        let diff = r#"diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1 +1 @@
-old
+new
diff --git a/file2.txt b/file2.txt
--- a/file2.txt
+++ b/file2.txt
@@ -1 +1,2 @@
 unchanged
+added
"#;
        let files = parse(diff, &DiffParserConfig::default());
        let renderer = FileListRenderer::default();
        let html = renderer.render(&files);

        assert!(html.contains("Files changed (2)"));
        assert!(html.contains("file1.txt"));
        assert!(html.contains("file2.txt"));
    }

    #[test]
    fn test_render_with_line_counts() {
        let diff = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,5 @@
 context
-deleted1
-deleted2
+added1
+added2
+added3
"#;
        let files = parse(diff, &DiffParserConfig::default());
        let renderer = FileListRenderer::default();
        let html = renderer.render(&files);

        // Should show +3 and -2
        assert!(html.contains("+3"));
        assert!(html.contains("-2"));
    }

    #[test]
    fn test_render_new_file() {
        let diff = r#"diff --git a/new-file.txt b/new-file.txt
new file mode 100644
--- /dev/null
+++ b/new-file.txt
@@ -0,0 +1,2 @@
+line 1
+line 2
"#;
        let files = parse(diff, &DiffParserConfig::default());
        let renderer = FileListRenderer::default();
        let html = renderer.render(&files);

        assert!(html.contains("new-file.txt"));
        // The icon should be file-added
        assert!(html.contains("d2h-added") || html.contains("ADDED"));
    }

    #[test]
    fn test_render_deleted_file() {
        let diff = r#"diff --git a/deleted-file.txt b/deleted-file.txt
deleted file mode 100644
--- a/deleted-file.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-line 1
-line 2
"#;
        let files = parse(diff, &DiffParserConfig::default());
        let renderer = FileListRenderer::default();
        let html = renderer.render(&files);

        assert!(html.contains("deleted-file.txt"));
        // The icon should be file-deleted
        assert!(html.contains("d2h-deleted") || html.contains("DELETED"));
    }

    #[test]
    fn test_render_renamed_file() {
        let diff = r#"diff --git a/old-name.txt b/new-name.txt
similarity index 90%
rename from old-name.txt
rename to new-name.txt
--- a/old-name.txt
+++ b/new-name.txt
@@ -1 +1 @@
-old content
+new content
"#;
        let files = parse(diff, &DiffParserConfig::default());
        let renderer = FileListRenderer::default();
        let html = renderer.render(&files);

        // Should show rename format
        assert!(html.contains("old-name.txt") || html.contains("new-name.txt"));
    }

    #[test]
    fn test_color_scheme_configuration() {
        let config = FileListConfig {
            color_scheme: ColorScheme::Dark,
        };
        let renderer = FileListRenderer::new(config);
        let html = renderer.render(&[]);

        assert!(html.contains("d2h-dark-color-scheme"));
    }
}
