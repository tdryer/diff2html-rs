# diff2html-rs

A Rust port of the [diff2html](https://diff2html.xyz/) library and CLI tool for generating beautiful HTML from unified diffs.

## Overview

This project provides two crates:

- **`diff2html`** - Core library for parsing unified diffs and generating HTML
- **`diff2html-cli`** - Command-line tool for converting diffs to HTML

## Features

- Parse unified diff format (git diff, GNU diff)
- Generate HTML output in line-by-line or side-by-side views
- Word-level or character-level diff highlighting
- Light, dark, and auto color schemes
- Line matching algorithms for better change visualization
- JSON output for integration with other tools
- Support for binary files, renames, copies, and mode changes
- Combined diff support (merge commits)

## Installation

### CLI Tool

```bash
cargo install diff2html-cli
```

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
diff2html = "0.1"
```

## Quick Start

### CLI Usage

```bash
# Generate HTML from git diff and open in browser
diff2html

# Generate HTML from specific commit
diff2html -- HEAD~1

# Side-by-side view with dark theme
diff2html -s side --colorScheme dark

# Read from stdin
git diff | diff2html -i stdin -o stdout > diff.html

# Read from file
diff2html -i file path/to/diff.patch

# Output to file
diff2html -F output.html -- HEAD~3..HEAD

# JSON output
diff2html -f json -- HEAD~1
```

### Library Usage

```rust
use diff2html::{html, Diff2HtmlConfig, OutputFormat};

let diff = r#"diff --git a/file.txt b/file.txt
--- a/file.txt
+++ b/file.txt
@@ -1 +1 @@
-Hello World
+Hello Rust
"#;

// Simple HTML generation
let html_output = html(diff, &Diff2HtmlConfig::default());

// With configuration
let config = Diff2HtmlConfig {
    output_format: OutputFormat::SideBySide,
    draw_file_list: true,
    ..Default::default()
};
let html_output = html(diff, &config);
```

See the [library documentation](diff2html/README.md) for more details.

## CLI Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--style` | `-s` | Output style: `line` or `side` | `line` |
| `--diffStyle` | `-d` | Diff style: `word` or `char` | `word` |
| `--format` | `-f` | Output format: `html` or `json` | `html` |
| `--input` | `-i` | Input source: `command`, `stdin`, or `file` | `command` |
| `--output` | `-o` | Output destination: `preview` or `stdout` | `preview` |
| `--file` | `-F` | Output file path | - |
| `--title` | `-t` | HTML page title | - |
| `--colorScheme` | | Color scheme: `auto`, `light`, or `dark` | `auto` |
| `--summary` | | Summary visibility: `open`, `closed`, or `hidden` | `closed` |
| `--matching` | | Line matching: `none`, `lines`, or `words` | `none` |
| `--matchWordsThreshold` | | Threshold for word matching (0.0-1.0) | `0.25` |
| `--diffMaxChanges` | | Max lines before "too big" | - |
| `--ignore` | `-g` | Files to exclude | - |

Pass additional arguments to `git diff` after `--`:

```bash
diff2html -- --stat -M HEAD~5
```

## Examples

See the [examples](diff2html/examples/) directory for standalone examples:

- `basic.rs` - Simple diff to HTML conversion
- `side_by_side.rs` - Side-by-side view with configuration
- `json_output.rs` - JSON output for integration
- `custom_parser.rs` - Custom parser configuration
- `line_matching.rs` - Line matching algorithm demonstration

Run examples with:

```bash
cargo run --example basic
cargo run --example line_matching
```

## Project Structure

```
diff2html-rs/
├── diff2html/           # Core library
│   ├── src/
│   │   ├── lib.rs       # Public API
│   │   ├── parser.rs    # Diff parser
│   │   ├── types.rs     # Data types
│   │   ├── rematch.rs   # Line matching
│   │   ├── templates.rs # Template system
│   │   └── render/      # HTML renderers
│   ├── templates/       # Mustache templates
│   └── examples/        # Example programs
├── diff2html-cli/       # CLI tool
│   ├── src/
│   │   ├── main.rs
│   │   ├── args.rs      # CLI arguments
│   │   ├── config.rs    # Configuration
│   │   ├── input.rs     # Input handling
│   │   └── output.rs    # Output handling
│   └── templates/       # HTML wrapper
└── tests/               # Integration tests
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

This is a Rust port of:
- [diff2html](https://github.com/rtfpessoa/diff2html) - Original TypeScript library
- [diff2html-cli](https://github.com/rtfpessoa/diff2html-cli) - Original CLI tool

Created by [rtfpessoa](https://github.com/rtfpessoa).
