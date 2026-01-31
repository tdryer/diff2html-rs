//! diff2html - Parse unified diffs and generate HTML.
//!
//! This library provides functionality to parse unified diff format and render
//! it as HTML, similar to the JavaScript diff2html library.
//!
//! # Example
//!
//! ```
//! use diff2html::{parse, DiffParserConfig};
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
//! ```

pub mod parser;
pub mod types;

pub use parser::{DiffParserConfig, parse};
pub use types::{
    Checksum, ColorScheme, DiffBlock, DiffFile, DiffLine, DiffLineParts, DiffStyle, FileMode,
    LineMatchingType, LineType, OutputFormat,
};
