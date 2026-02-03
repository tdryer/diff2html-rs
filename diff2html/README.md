# diff2html

A Rust library for parsing diffs and generating HTML output.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
diff2html = { path = "/path/to/diff2html" }
```

## Quick Start

```rust
use diff2html::{Diff2HtmlConfig, html};

fn main() {
    let diff = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1 +1 @@
-old line
+new line
"#;

    let html_output = html(diff, &Diff2HtmlConfig::default());
    println!("{}", html_output);
}
```

## API Overview

| Function | Description |
|----------|-------------|
| `html(diff, config)` | Parse diff and render as HTML |
| `html_from_diff_files(files, config)` | Render pre-parsed files as HTML |
| `json(diff, config)` | Parse diff and output as JSON |
| `json_from_diff_files(files)` | Serialize pre-parsed files to JSON |
| `parse(diff, config)` | Parse diff into `Vec<DiffFile>` |
