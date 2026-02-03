# diff2html-cli

A command-line tool for generating HTML from diffs.

## Installation

```bash
cargo build --release
cp target/release/diff2html /usr/local/bin/diff2html
```

## Quick Start

```bash
# Generate HTML from current git diff and open in browser
diff2html

# Generate from specific commit
diff2html -- HEAD~1

# Side-by-side view with dark theme
diff2html -s side --colorScheme dark

# Read from stdin
git diff | diff2html -i stdin -o stdout > diff.html

# Output to file
diff2html -F output.html -- HEAD~3..HEAD
```

## Options

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

## Environment

The tool requires `git` to be available in PATH when using `--input command`.
