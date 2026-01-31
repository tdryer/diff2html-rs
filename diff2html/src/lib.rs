//! diff2html - Parse unified diffs and generate HTML.
//!
//! This library provides functionality to parse unified diff format and render
//! it as HTML, similar to the JavaScript diff2html library.
//!
//! # Example
//!
//! ```
//! use diff2html::{parse, DiffParserConfig};
//! use diff2html::render::LineByLineRenderer;
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
//! assert_eq!(files.len(), 1);
//! assert_eq!(files[0].new_name, "test.txt");
//!
//! // Render to HTML
//! let renderer = LineByLineRenderer::default();
//! let html = renderer.render(&files);
//! assert!(html.contains("d2h-wrapper"));
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
pub use render::{
    FileListConfig, FileListRenderer, LineByLineRenderer, RendererConfig, SideBySideRenderer,
};
pub use templates::{CSS, TemplateName, render as render_template, render_by_name};
pub use types::{
    Checksum, ColorScheme, DiffBlock, DiffFile, DiffLine, DiffLineParts, DiffStyle, FileMode,
    LineMatchingType, LineType, OutputFormat,
};
