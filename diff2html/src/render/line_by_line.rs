//! Line-by-line diff renderer.
//!
//! This module provides a single-column renderer that shows all diff lines
//! sequentially with old and new line numbers.

use serde_json::json;

use crate::templates::{self, TemplateName};
use crate::types::{DiffBlock, DiffFile, DiffLine, LineType};

use super::utils::{
    CSSLineClass, RendererConfig, color_scheme_to_css, deconstruct_line, diff_highlight,
    escape_for_html, filename_diff, get_file_icon, get_html_id, to_css_class,
};

/// Line-by-line renderer for generating single-column diff HTML.
pub struct LineByLineRenderer {
    config: RendererConfig,
}

impl Default for LineByLineRenderer {
    fn default() -> Self {
        Self::new(RendererConfig::default())
    }
}

impl LineByLineRenderer {
    /// Create a new LineByLineRenderer with the given configuration.
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
    fn make_file_diff_html(&self, file: &DiffFile, diffs: &str) -> String {
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
            TemplateName::LineByLineFileDiff,
            &json!({
                "file": {
                    "language": file.language,
                },
                "fileHtmlId": get_html_id(file),
                "diffs": diffs,
                "filePath": file_path_html,
            }),
        )
        .unwrap_or_default()
    }

    /// Generate HTML for an empty diff (file with no changes).
    fn generate_empty_diff(&self) -> String {
        templates::render(
            TemplateName::GenericEmptyDiff,
            &json!({
                "contentClass": "d2h-code-line",
                "CSSLineClass": {
                    "INFO": CSSLineClass::Info.as_str(),
                },
            }),
        )
        .unwrap_or_default()
    }

    /// Generate HTML for all blocks in a file.
    fn generate_file_html(&self, file: &DiffFile) -> String {
        file.blocks
            .iter()
            .map(|block| {
                let mut lines = templates::render(
                    TemplateName::GenericBlockHeader,
                    &json!({
                        "CSSLineClass": {
                            "INFO": CSSLineClass::Info.as_str(),
                        },
                        "blockHeader": if file.is_too_big == Some(true) {
                            block.header.clone()
                        } else {
                            escape_for_html(&block.header)
                        },
                        "lineClass": "d2h-code-linenumber",
                        "contentClass": "d2h-code-line",
                    }),
                )
                .unwrap_or_default();

                for (context_lines, old_lines, new_lines) in self.apply_line_grouping(block) {
                    if !old_lines.is_empty() && !new_lines.is_empty() && context_lines.is_empty() {
                        // Changed lines - apply diff highlighting
                        let (left, right) = self.process_changed_lines(
                            file,
                            file.is_combined,
                            &old_lines,
                            &new_lines,
                        );
                        lines.push_str(&left);
                        lines.push_str(&right);
                    } else if !context_lines.is_empty() {
                        // Context lines
                        for line in &context_lines {
                            let parts = deconstruct_line(&line.content, file.is_combined, true);
                            lines.push_str(&self.generate_single_line_html(
                                CSSLineClass::Context,
                                &parts.prefix,
                                &parts.content,
                                line.old_number,
                                line.new_number,
                            ));
                        }
                    } else if !old_lines.is_empty() || !new_lines.is_empty() {
                        // Only deletions or only insertions
                        let (left, right) = self.process_changed_lines(
                            file,
                            file.is_combined,
                            &old_lines,
                            &new_lines,
                        );
                        lines.push_str(&left);
                        lines.push_str(&right);
                    }
                }

                lines
            })
            .collect::<Vec<_>>()
            .join("\n")
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

    /// Process changed lines by pairing deletions with insertions and highlighting differences.
    fn process_changed_lines(
        &self,
        _file: &DiffFile,
        is_combined: bool,
        old_lines: &[DiffLine],
        new_lines: &[DiffLine],
    ) -> (String, String) {
        let mut left = String::new();
        let mut right = String::new();

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

            // Process old line
            if let Some(old) = old_line.filter(|o| o.old_number.is_some()) {
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

                left.push_str(&self.generate_single_line_html(
                    css_class,
                    &prefix,
                    &content,
                    old.old_number,
                    old.new_number,
                ));
            }

            // Process new line
            if let Some(new) = new_line.filter(|n| n.new_number.is_some()) {
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

                right.push_str(&self.generate_single_line_html(
                    css_class,
                    &prefix,
                    &content,
                    new.old_number,
                    new.new_number,
                ));
            }
        }

        (left, right)
    }

    /// Generate HTML for a single diff line.
    fn generate_single_line_html(
        &self,
        css_class: CSSLineClass,
        prefix: &str,
        content: &str,
        old_number: Option<u32>,
        new_number: Option<u32>,
    ) -> String {
        let line_number_html = templates::render(
            TemplateName::LineByLineNumbers,
            &json!({
                "oldNumber": old_number.map(|n| n.to_string()).unwrap_or_default(),
                "newNumber": new_number.map(|n| n.to_string()).unwrap_or_default(),
            }),
        )
        .unwrap_or_default();

        let display_prefix = if prefix == " " { "&nbsp;" } else { prefix };

        templates::render(
            TemplateName::GenericLine,
            &json!({
                "type": css_class.as_str(),
                "lineClass": "d2h-code-linenumber",
                "contentClass": "d2h-code-line",
                "prefix": display_prefix,
                "content": content,
                "lineNumber": line_number_html,
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
    fn test_render_basic_diff() {
        let files = parse(sample_diff(), &DiffParserConfig::default());
        let renderer = LineByLineRenderer::default();
        let html = renderer.render(&files);

        assert!(html.contains("d2h-wrapper"));
        assert!(html.contains("d2h-file-wrapper"));
        assert!(html.contains("test.txt"));
    }

    #[test]
    fn test_render_empty_files() {
        let renderer = LineByLineRenderer::default();
        let html = renderer.render(&[]);

        assert!(html.contains("d2h-wrapper"));
    }

    #[test]
    fn test_render_nothing_when_empty() {
        let config = RendererConfig {
            render_nothing_when_empty: true,
            ..Default::default()
        };
        let renderer = LineByLineRenderer::new(config);

        let file = DiffFile {
            old_name: "test.txt".to_string(),
            new_name: "test.txt".to_string(),
            blocks: vec![],
            ..Default::default()
        };

        let html = renderer.render(&[file]);
        // The wrapper should be present but no file content
        assert!(html.contains("d2h-wrapper"));
    }

    #[test]
    fn test_generate_empty_diff() {
        let renderer = LineByLineRenderer::default();
        let html = renderer.generate_empty_diff();

        assert!(html.contains("File without changes"));
    }

    #[test]
    fn test_line_grouping() {
        let renderer = LineByLineRenderer::default();
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
                DiffLine {
                    line_type: LineType::Context,
                    content: " another context".to_string(),
                    old_number: Some(3),
                    new_number: Some(3),
                },
            ],
        };

        let groups = renderer.apply_line_grouping(&block);

        // Should have: context, (old+new pair), context
        assert_eq!(groups.len(), 3);

        // First group is context
        assert_eq!(groups[0].0.len(), 1);
        assert!(groups[0].1.is_empty());
        assert!(groups[0].2.is_empty());

        // Second group is old+new
        assert!(groups[1].0.is_empty());
        assert_eq!(groups[1].1.len(), 1);
        assert_eq!(groups[1].2.len(), 1);

        // Third group is context
        assert_eq!(groups[2].0.len(), 1);
        assert!(groups[2].1.is_empty());
        assert!(groups[2].2.is_empty());
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
        let renderer = LineByLineRenderer::default();
        let html = renderer.render(&files);

        // Should contain diff highlighting tags
        assert!(html.contains("d2h-del") || html.contains("d2h-change"));
        assert!(html.contains("d2h-ins") || html.contains("d2h-change"));
    }
}
