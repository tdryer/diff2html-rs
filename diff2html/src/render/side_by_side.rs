//! Side-by-side diff renderer.
//!
//! This module provides a two-column renderer that shows old (left) and new (right)
//! file versions side by side.

use serde_json::json;

use crate::templates::{self, TemplateName};
use crate::types::{DiffBlock, DiffFile, DiffLine, LineType};

use super::utils::{
    CSSLineClass, RendererConfig, color_scheme_to_css, deconstruct_line, diff_highlight,
    escape_for_html, filename_diff, get_file_icon, get_html_id, to_css_class,
};

/// HTML content for left and right columns.
#[derive(Debug, Clone, Default)]
struct FileHtml {
    left: String,
    right: String,
}

/// Side-by-side renderer for generating two-column diff HTML.
pub struct SideBySideRenderer {
    config: RendererConfig,
}

impl Default for SideBySideRenderer {
    fn default() -> Self {
        Self::new(RendererConfig::default())
    }
}

impl SideBySideRenderer {
    /// Create a new SideBySideRenderer with the given configuration.
    pub fn new(config: RendererConfig) -> Self {
        Self { config }
    }

    /// Render a list of diff files to HTML.
    pub fn render(&self, diff_files: &[DiffFile]) -> String {
        let diffs_html: String = diff_files
            .iter()
            .map(|file| {
                let diffs = if !file.blocks.is_empty() {
                    self.generate_file_html(file)
                } else {
                    self.generate_empty_diff()
                };
                self.make_file_diff_html(file, &diffs)
            })
            .collect::<Vec<_>>()
            .join("\n");

        templates::render(
            TemplateName::GenericWrapper,
            &json!({
                "colorScheme": color_scheme_to_css(self.config.render.color_scheme),
                "content": diffs_html,
            }),
        )
        .unwrap_or_default()
    }

    /// Generate the HTML for a single file diff.
    fn make_file_diff_html(&self, file: &DiffFile, diffs: &FileHtml) -> String {
        if self.config.render_nothing_when_empty && file.blocks.is_empty() {
            return String::new();
        }

        let file_icon = get_file_icon(file);
        let file_icon_html = templates::render_by_name(&format!("icon-{}", file_icon), &json!({}))
            .unwrap_or_default();
        let file_tag_html = templates::render_by_name(&format!("tag-{}", file_icon), &json!({}))
            .unwrap_or_default();

        let file_path_html = templates::render(
            TemplateName::GenericFilePath,
            &json!({
                "fileDiffName": filename_diff(file),
                "fileIcon": file_icon_html,
                "fileTag": file_tag_html,
            }),
        )
        .unwrap_or_default();

        templates::render(
            TemplateName::SideBySideFileDiff,
            &json!({
                "file": {
                    "language": file.language,
                },
                "fileHtmlId": get_html_id(file),
                "diffs": {
                    "left": diffs.left,
                    "right": diffs.right,
                },
                "filePath": file_path_html,
            }),
        )
        .unwrap_or_default()
    }

    /// Generate HTML for an empty diff (file with no changes).
    fn generate_empty_diff(&self) -> FileHtml {
        FileHtml {
            left: templates::render(
                TemplateName::GenericEmptyDiff,
                &json!({
                    "contentClass": "d2h-code-side-line",
                    "CSSLineClass": {
                        "INFO": CSSLineClass::Info.as_str(),
                    },
                }),
            )
            .unwrap_or_default(),
            right: String::new(),
        }
    }

    /// Generate HTML for all blocks in a file.
    fn generate_file_html(&self, file: &DiffFile) -> FileHtml {
        file.blocks
            .iter()
            .map(|block| {
                let mut file_html = FileHtml {
                    left: self.make_header_html(&block.header, Some(file)),
                    right: self.make_header_html("", None),
                };

                for (context_lines, old_lines, new_lines) in self.apply_line_grouping(block) {
                    if !old_lines.is_empty() && !new_lines.is_empty() && context_lines.is_empty() {
                        // Changed lines - apply diff highlighting
                        let result =
                            self.process_changed_lines(file.is_combined, &old_lines, &new_lines);
                        file_html.left.push_str(&result.left);
                        file_html.right.push_str(&result.right);
                    } else if !context_lines.is_empty() {
                        // Context lines - show in both columns
                        for line in &context_lines {
                            let parts = deconstruct_line(&line.content, file.is_combined, true);
                            let (left, right) = self.generate_line_html(
                                Some(PreparedLine {
                                    css_class: CSSLineClass::Context,
                                    prefix: parts.prefix.clone(),
                                    content: parts.content.clone(),
                                    number: line.old_number,
                                }),
                                Some(PreparedLine {
                                    css_class: CSSLineClass::Context,
                                    prefix: parts.prefix,
                                    content: parts.content,
                                    number: line.new_number,
                                }),
                            );
                            file_html.left.push_str(&left);
                            file_html.right.push_str(&right);
                        }
                    } else if !old_lines.is_empty() || !new_lines.is_empty() {
                        // Only deletions or only insertions
                        let result =
                            self.process_changed_lines(file.is_combined, &old_lines, &new_lines);
                        file_html.left.push_str(&result.left);
                        file_html.right.push_str(&result.right);
                    }
                }

                file_html
            })
            .fold(FileHtml::default(), |mut acc, html| {
                acc.left.push_str(&html.left);
                acc.right.push_str(&html.right);
                acc
            })
    }

    /// Group lines in a block by type (context, deletions, insertions).
    fn apply_line_grouping(
        &self,
        block: &DiffBlock,
    ) -> Vec<(Vec<DiffLine>, Vec<DiffLine>, Vec<DiffLine>)> {
        let mut groups: Vec<(Vec<DiffLine>, Vec<DiffLine>, Vec<DiffLine>)> = Vec::new();
        let mut old_lines: Vec<DiffLine> = Vec::new();
        let mut new_lines: Vec<DiffLine> = Vec::new();

        for line in &block.lines {
            // Flush accumulated lines when we hit a context line or switch patterns
            if (line.line_type != LineType::Insert && !new_lines.is_empty())
                || (line.line_type == LineType::Context && !old_lines.is_empty())
            {
                groups.push((Vec::new(), old_lines.clone(), new_lines.clone()));
                old_lines.clear();
                new_lines.clear();
            }

            match line.line_type {
                LineType::Context => {
                    groups.push((vec![line.clone()], Vec::new(), Vec::new()));
                }
                LineType::Insert if old_lines.is_empty() => {
                    groups.push((Vec::new(), Vec::new(), vec![line.clone()]));
                }
                LineType::Insert => {
                    new_lines.push(line.clone());
                }
                LineType::Delete => {
                    old_lines.push(line.clone());
                }
            }
        }

        // Flush any remaining lines
        if !old_lines.is_empty() || !new_lines.is_empty() {
            groups.push((Vec::new(), old_lines, new_lines));
        }

        groups
    }

    /// Generate HTML for a block header row.
    fn make_header_html(&self, block_header: &str, file: Option<&DiffFile>) -> String {
        let escaped_header = if file.is_some_and(|f| f.is_too_big == Some(true)) {
            block_header.to_string()
        } else {
            escape_for_html(block_header)
        };

        templates::render(
            TemplateName::GenericBlockHeader,
            &json!({
                "CSSLineClass": {
                    "INFO": CSSLineClass::Info.as_str(),
                },
                "blockHeader": escaped_header,
                "lineClass": "d2h-code-side-linenumber",
                "contentClass": "d2h-code-side-line",
            }),
        )
        .unwrap_or_default()
    }

    /// Process changed lines by pairing deletions with insertions and highlighting differences.
    fn process_changed_lines(
        &self,
        is_combined: bool,
        old_lines: &[DiffLine],
        new_lines: &[DiffLine],
    ) -> FileHtml {
        let mut result = FileHtml::default();
        let max_lines = old_lines.len().max(new_lines.len());

        for i in 0..max_lines {
            let old_line = old_lines.get(i);
            let new_line = new_lines.get(i);

            let diff = match (old_line, new_line) {
                (Some(old), Some(new)) => Some(diff_highlight(
                    &old.content,
                    &new.content,
                    is_combined,
                    &self.config.render,
                )),
                _ => None,
            };

            // Prepare old line
            let prepared_old = old_line.filter(|o| o.old_number.is_some()).map(|old| {
                let (css_class, prefix, content) = if let Some(ref diff) = diff {
                    (
                        CSSLineClass::DeleteChanges,
                        diff.old_line.prefix.clone(),
                        diff.old_line.content.clone(),
                    )
                } else {
                    let parts = deconstruct_line(&old.content, is_combined, true);
                    (to_css_class(old.line_type), parts.prefix, parts.content)
                };

                PreparedLine {
                    css_class,
                    prefix,
                    content,
                    number: old.old_number,
                }
            });

            // Prepare new line
            let prepared_new = new_line.filter(|n| n.new_number.is_some()).map(|new| {
                let (css_class, prefix, content) = if let Some(ref diff) = diff {
                    (
                        CSSLineClass::InsertChanges,
                        diff.new_line.prefix.clone(),
                        diff.new_line.content.clone(),
                    )
                } else {
                    let parts = deconstruct_line(&new.content, is_combined, true);
                    (to_css_class(new.line_type), parts.prefix, parts.content)
                };

                PreparedLine {
                    css_class,
                    prefix,
                    content,
                    number: new.new_number,
                }
            });

            let (left, right) = self.generate_line_html(prepared_old, prepared_new);
            result.left.push_str(&left);
            result.right.push_str(&right);
        }

        result
    }

    /// Generate HTML for a pair of lines (left and right).
    fn generate_line_html(
        &self,
        old_line: Option<PreparedLine>,
        new_line: Option<PreparedLine>,
    ) -> (String, String) {
        (
            self.generate_single_html(old_line),
            self.generate_single_html(new_line),
        )
    }

    /// Generate HTML for a single side-by-side line.
    fn generate_single_html(&self, line: Option<PreparedLine>) -> String {
        let line_class = "d2h-code-side-linenumber";
        let content_class = "d2h-code-side-line";

        let (css_type, actual_line_class, actual_content_class, prefix, content, line_number) =
            if let Some(line) = line {
                (
                    line.css_class.as_str().to_string(),
                    line_class.to_string(),
                    content_class.to_string(),
                    if line.prefix == " " {
                        "&nbsp;".to_string()
                    } else {
                        line.prefix
                    },
                    line.content,
                    line.number.map(|n| n.to_string()).unwrap_or_default(),
                )
            } else {
                (
                    format!("{} d2h-emptyplaceholder", CSSLineClass::Context.as_str()),
                    format!("{} d2h-code-side-emptyplaceholder", line_class),
                    format!("{} d2h-code-side-emptyplaceholder", content_class),
                    String::new(),
                    String::new(),
                    String::new(),
                )
            };

        templates::render(
            TemplateName::GenericLine,
            &json!({
                "type": css_type,
                "lineClass": actual_line_class,
                "contentClass": actual_content_class,
                "prefix": prefix,
                "content": content,
                "lineNumber": line_number,
            }),
        )
        .unwrap_or_default()
    }
}

/// A prepared diff line ready for rendering in the side-by-side view.
///
/// This struct holds all the information needed to render a single line
/// in either the left (old) or right (new) side of a side-by-side diff view.
struct PreparedLine {
    /// The CSS class to apply to this line (e.g., insert, delete, context).
    css_class: CSSLineClass,
    /// The prefix character for this line (typically '+', '-', or ' ').
    prefix: String,
    /// The actual content of the line, potentially with inline diff highlighting.
    content: String,
    /// The line number in the file, or `None` for empty placeholder lines.
    number: Option<u32>,
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
    fn test_render_basic_diff() {
        let files = parse(sample_diff(), &DiffParserConfig::default());
        let renderer = SideBySideRenderer::default();
        let html = renderer.render(&files);

        assert!(html.contains("d2h-wrapper"));
        assert!(html.contains("d2h-file-wrapper"));
        assert!(html.contains("d2h-files-diff"));
        assert!(html.contains("d2h-file-side-diff"));
        assert!(html.contains("test.txt"));
    }

    #[test]
    fn test_render_empty_files() {
        let renderer = SideBySideRenderer::default();
        let html = renderer.render(&[]);

        assert!(html.contains("d2h-wrapper"));
    }

    #[test]
    fn test_render_nothing_when_empty() {
        let config = RendererConfig {
            render_nothing_when_empty: true,
            ..Default::default()
        };
        let renderer = SideBySideRenderer::new(config);

        let file = DiffFile {
            old_name: "test.txt".to_string(),
            new_name: "test.txt".to_string(),
            blocks: vec![],
            ..Default::default()
        };

        let html = renderer.render(&[file]);
        assert!(html.contains("d2h-wrapper"));
    }

    #[test]
    fn test_generate_empty_diff() {
        let renderer = SideBySideRenderer::default();
        let file_html = renderer.generate_empty_diff();

        assert!(file_html.left.contains("File without changes"));
        assert!(file_html.right.is_empty());
    }

    #[test]
    fn test_make_header_html() {
        let renderer = SideBySideRenderer::default();

        // With file (should escape)
        let file = DiffFile::default();
        let header = renderer.make_header_html("@@ -1,3 +1,4 @@", Some(&file));
        assert!(header.contains("d2h-code-side-linenumber"));
        assert!(header.contains("d2h-code-side-line"));

        // Empty header (right column)
        let empty_header = renderer.make_header_html("", None);
        assert!(empty_header.contains("d2h-code-side-linenumber"));
    }

    #[test]
    fn test_line_grouping() {
        let renderer = SideBySideRenderer::default();
        let block = DiffBlock {
            old_start_line: 1,
            old_start_line2: None,
            new_start_line: 1,
            header: "@@ -1,3 +1,3 @@".to_string(),
            lines: vec![
                DiffLine {
                    line_type: LineType::Context,
                    content: " context".to_string(),
                    old_number: Some(1),
                    new_number: Some(1),
                },
                DiffLine {
                    line_type: LineType::Delete,
                    content: "-old".to_string(),
                    old_number: Some(2),
                    new_number: None,
                },
                DiffLine {
                    line_type: LineType::Insert,
                    content: "+new".to_string(),
                    old_number: None,
                    new_number: Some(2),
                },
            ],
        };

        let groups = renderer.apply_line_grouping(&block);
        assert_eq!(groups.len(), 2);

        // First group is context
        assert_eq!(groups[0].0.len(), 1);

        // Second group is old+new pair
        assert_eq!(groups[1].1.len(), 1);
        assert_eq!(groups[1].2.len(), 1);
    }

    #[test]
    fn test_empty_placeholder_html() {
        let renderer = SideBySideRenderer::default();
        let html = renderer.generate_single_html(None);

        assert!(html.contains("d2h-emptyplaceholder"));
        assert!(html.contains("d2h-code-side-emptyplaceholder"));
    }

    #[test]
    fn test_diff_highlighting_in_render() {
        let diff = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1 +1 @@
-hello world
+hello universe
"#;
        let files = parse(diff, &DiffParserConfig::default());
        let renderer = SideBySideRenderer::default();
        let html = renderer.render(&files);

        // Should contain the changed content
        assert!(html.contains("d2h-file-side-diff"));
    }
}
