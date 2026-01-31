//! HTML rendering for diff2html.
//!
//! This module provides renderers for converting parsed diff files into HTML.
//! Three renderers are available:
//!
//! - [`LineByLineRenderer`]: Single-column view showing all changes sequentially
//! - [`SideBySideRenderer`]: Two-column view showing old and new files side by side
//! - [`FileListRenderer`]: Summary list of changed files with statistics
//!
//! # Example
//!
//! ```
//! use diff2html::{parse, DiffParserConfig};
//! use diff2html::render::{LineByLineRenderer, SideBySideRenderer, FileListRenderer};
//!
//! let diff = r#"diff --git a/test.txt b/test.txt
//! --- a/test.txt
//! +++ b/test.txt
//! @@ -1 +1 @@
//! -old
//! +new
//! "#;
//!
//! let files = parse(diff, &DiffParserConfig::default());
//!
//! // Render as line-by-line
//! let renderer = LineByLineRenderer::default();
//! let html = renderer.render(&files);
//!
//! // Render as side-by-side
//! let renderer = SideBySideRenderer::default();
//! let html = renderer.render(&files);
//!
//! // Render file list summary
//! let renderer = FileListRenderer::default();
//! let html = renderer.render(&files);
//! ```

pub mod file_list;
pub mod line_by_line;
pub mod side_by_side;
pub mod utils;

pub use file_list::{FileListConfig, FileListRenderer};
pub use line_by_line::LineByLineRenderer;
pub use side_by_side::SideBySideRenderer;
pub use utils::{
    CSSLineClass, HighlightedLines, RenderConfig, RendererConfig, color_scheme_to_css,
    deconstruct_line, diff_highlight, escape_for_html, filename_diff, get_file_icon, get_html_id,
    to_css_class,
};
