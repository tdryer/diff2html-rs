# diff2html

A Rust library for parsing unified diffs and generating HTML output.

## Features

- Parse unified diff format (git diff, GNU diff, combined diffs)
- Generate HTML in line-by-line or side-by-side view
- Word-level or character-level diff highlighting
- Light, dark, and auto color schemes
- Line matching algorithms using Levenshtein distance
- JSON output support
- Configurable limits for large diffs
- Support for binary files, renames, copies, and mode changes

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
diff2html = "0.1"
```

## Quick Start

```rust
use diff2html::{html, Diff2HtmlConfig};

let diff = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1 +1 @@
-old line
+new line
"#;

let html_output = html(diff, &Diff2HtmlConfig::default());
println!("{}", html_output);
```

## API Overview

### Main Functions

| Function | Description |
|----------|-------------|
| `html(diff, config)` | Parse diff and render as HTML |
| `html_from_diff_files(files, config)` | Render pre-parsed files as HTML |
| `json(diff, config)` | Parse diff and output as JSON |
| `json_from_diff_files(files)` | Serialize pre-parsed files to JSON |
| `parse(diff, config)` | Parse diff into `Vec<DiffFile>` |

### Two-Stage Processing

For more control, parse and render separately:

```rust
use diff2html::{parse, html_from_diff_files, Diff2HtmlConfig, DiffParserConfig, OutputFormat};

let diff = "--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n";

// Parse once
let files = parse(diff, &DiffParserConfig::default());

// Render multiple times with different configs
let line_view = html_from_diff_files(&files, &Diff2HtmlConfig::default());

let side_view = html_from_diff_files(&files, &Diff2HtmlConfig {
    output_format: OutputFormat::SideBySide,
    ..Default::default()
});
```

## Configuration

### Diff2HtmlConfig

The main configuration struct for the library:

```rust
use diff2html::{Diff2HtmlConfig, OutputFormat, DiffStyle, ColorScheme, LineMatchingType};

let config = Diff2HtmlConfig {
    // Parser options
    src_prefix: None,           // Prefix to strip from source paths
    dst_prefix: None,           // Prefix to strip from dest paths
    diff_max_changes: Some(1000), // Max changes before "too big"
    diff_max_line_length: Some(500), // Max line length

    // Renderer options
    output_format: OutputFormat::SideBySide,
    draw_file_list: true,       // Show file summary at top
    diff_style: DiffStyle::Char, // Character-level highlighting
    color_scheme: ColorScheme::Dark,
    matching: LineMatchingType::Lines,
    match_words_threshold: 0.25, // Similarity threshold (0.0-1.0)
    max_line_length_highlight: 10000,
    render_nothing_when_empty: false,
    matching_max_comparisons: 2500,
    max_line_size_in_block_for_comparison: 200,
};
```

### Output Formats

```rust
use diff2html::OutputFormat;

// Single-column view (default)
OutputFormat::LineByLine

// Two-column parallel view
OutputFormat::SideBySide
```

### Diff Styles

```rust
use diff2html::DiffStyle;

// Highlight changed words (default)
DiffStyle::Word

// Highlight individual characters
DiffStyle::Char
```

### Color Schemes

```rust
use diff2html::ColorScheme;

// Light background (default)
ColorScheme::Light

// Dark background
ColorScheme::Dark

// Auto-detect from system preference
ColorScheme::Auto
```

### Line Matching

Line matching pairs similar deleted and inserted lines for better visualization:

```rust
use diff2html::LineMatchingType;

// No matching (default)
LineMatchingType::None

// Match at line level
LineMatchingType::Lines

// Match at word level
LineMatchingType::Words
```

## Data Types

### DiffFile

Represents a single file in the diff:

```rust
pub struct DiffFile {
    pub old_name: String,
    pub new_name: String,
    pub added_lines: u32,
    pub deleted_lines: u32,
    pub is_combined: bool,
    pub is_git_diff: bool,
    pub language: Option<String>,
    pub blocks: Vec<DiffBlock>,
    // ... additional metadata
}
```

### DiffBlock

A hunk/block of changes:

```rust
pub struct DiffBlock {
    pub old_start_line: u32,
    pub new_start_line: u32,
    pub header: String,
    pub lines: Vec<DiffLine>,
}
```

### DiffLine

A single line in the diff:

```rust
pub struct DiffLine {
    pub line_type: LineType,
    pub content: String,
    pub old_number: Option<u32>,
    pub new_number: Option<u32>,
}

pub enum LineType {
    Context,
    Insert,
    Delete,
}
```

## JSON Output

Get JSON for integration with other tools:

```rust
use diff2html::{json, Diff2HtmlConfig};

let diff = "--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n";
let json_output = json(diff, &Diff2HtmlConfig::default()).unwrap();
```

Output structure:

```json
[
  {
    "oldName": "file.txt",
    "newName": "file.txt",
    "addedLines": 1,
    "deletedLines": 1,
    "isGitDiff": false,
    "isCombined": false,
    "blocks": [
      {
        "oldStartLine": 1,
        "newStartLine": 1,
        "header": "@@ -1 +1 @@",
        "lines": [
          {"type": "delete", "content": "old", "oldNumber": 1},
          {"type": "insert", "content": "new", "newNumber": 1}
        ]
      }
    ]
  }
]
```

## CSS

The library embeds CSS for styling. Access it via:

```rust
use diff2html::CSS;

// Include in your HTML
let css = CSS;
```

## Advanced Usage

### Custom Parser Configuration

```rust
use diff2html::{parse, DiffParserConfig};

let config = DiffParserConfig {
    src_prefix: Some("a/".to_string()),
    dst_prefix: Some("b/".to_string()),
    diff_max_changes: Some(5000),
    diff_max_line_length: Some(1000),
    diff_too_big_message: Some("Diff too large to display".to_string()),
};

let files = parse(diff_input, &config);
```

### Line Matching with Levenshtein Distance

The library includes a line matching algorithm that pairs similar changed lines:

```rust
use diff2html::{match_lines_with_config, MatchConfig, new_distance_fn};

let old_lines = vec!["hello world".to_string(), "foo bar".to_string()];
let new_lines = vec!["hello rust".to_string(), "foo baz".to_string()];

let distance_fn = new_distance_fn();
let config = MatchConfig {
    matching_max_comparisons: 2500,
    max_line_size_in_block_for_comparison: 200,
};

let matches = match_lines_with_config(&old_lines, &new_lines, &distance_fn, &config);
```

## License

MIT License
