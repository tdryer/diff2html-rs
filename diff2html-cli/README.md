# diff2html-cli

A command-line tool for generating beautiful HTML from unified diffs.

## Installation

```bash
cargo install diff2html-cli
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

## Usage

```
diff2html [OPTIONS] [-- <EXTRA_ARGS>...]
```

### Input Options

| Option | Short | Description |
|--------|-------|-------------|
| `--input command` | `-i command` | Execute `git diff` with extra args (default) |
| `--input stdin` | `-i stdin` | Read diff from standard input |
| `--input file` | `-i file` | Read diff from file (path in extra args) |

### Output Options

| Option | Short | Description |
|--------|-------|-------------|
| `--output preview` | `-o preview` | Open HTML in browser (default) |
| `--output stdout` | `-o stdout` | Print HTML to stdout |
| `--file <path>` | `-F <path>` | Write output to file |
| `--format html` | `-f html` | Generate HTML output (default) |
| `--format json` | `-f json` | Generate JSON output |

### Display Options

| Option | Description | Values |
|--------|-------------|--------|
| `--style` / `-s` | Output style | `line` (default), `side` |
| `--diffStyle` / `-d` | Highlighting granularity | `word` (default), `char` |
| `--colorScheme` | Color theme | `auto` (default), `light`, `dark` |
| `--summary` | File list visibility | `closed` (default), `open`, `hidden` |
| `--title` / `-t` | HTML page title | String |

### Matching Options

| Option | Description | Default |
|--------|-------------|---------|
| `--matching` | Line matching algorithm | `none` |
| `--matchWordsThreshold` | Similarity threshold (0.0-1.0) | `0.25` |
| `--matchingMaxComparisons` | Max comparisons per block | `1000` |

### Size Limits

| Option | Description |
|--------|-------------|
| `--diffMaxChanges` | Max lines before marking "too big" |
| `--diffMaxLineLength` | Max characters per line |
| `--maxLineSizeInBlockForComparison` | Max line size for matching |
| `--maxLineLengthHighlight` | Max line length for highlighting |

### Other Options

| Option | Short | Description |
|--------|-------|-------------|
| `--ignore` | `-g` | Files to exclude (can repeat) |
| `--fileContentToggle` | Show "viewed" checkbox | Default: true |
| `--synchronisedScroll` | Sync side-by-side scroll | Default: true |
| `--highlightCode` | Enable syntax highlighting | Default: true |
| `--htmlWrapperTemplate` | Custom HTML template |
| `--renderNothingWhenEmpty` | Hide empty diffs | Default: false |

## Examples

### Basic Usage

```bash
# View current uncommitted changes
diff2html

# View changes from last commit
diff2html -- HEAD~1

# View changes between branches
diff2html -- main..feature-branch

# View specific file changes
diff2html -- path/to/file.rs
```

### Output Modes

```bash
# Preview in browser (default)
diff2html

# Print to stdout
diff2html -o stdout

# Save to file
diff2html -F diff.html

# JSON output for scripting
diff2html -f json -o stdout | jq '.[] | .newName'
```

### Input Sources

```bash
# From git diff command (default)
diff2html -- HEAD~5

# From stdin
cat changes.patch | diff2html -i stdin -o stdout

# From file
diff2html -i file changes.patch
```

### Styling

```bash
# Side-by-side with dark theme
diff2html -s side --colorScheme dark

# Character-level highlighting
diff2html -d char

# Hide file list summary
diff2html --summary hidden

# Custom page title
diff2html -t "Code Review: Feature X"
```

### Line Matching

Enable line matching to better visualize similar changes:

```bash
# Match similar lines
diff2html --matching lines

# Match at word level with custom threshold
diff2html --matching words --matchWordsThreshold 0.3
```

### Large Diffs

```bash
# Limit large diffs
diff2html --diffMaxChanges 1000 --diffMaxLineLength 500

# Increase comparison limits
diff2html --matchingMaxComparisons 5000
```

### Git Diff Options

Pass any `git diff` arguments after `--`:

```bash
# Show stats
diff2html -- --stat HEAD~5

# Detect renames
diff2html -- -M HEAD~1

# Ignore whitespace
diff2html -- -w HEAD~1

# Specific commit range
diff2html -- abc123..def456

# Compare with remote
diff2html -- origin/main...HEAD
```

### Filtering Files

```bash
# Exclude files
diff2html --ignore "*.lock" --ignore "vendor/*"

# Git pathspec filtering
diff2html -- HEAD~1 -- "*.rs" "*.toml"
```

### Custom Template

```bash
# Use custom HTML wrapper
diff2html --htmlWrapperTemplate custom-template.html
```

Template placeholders:
- `<!--diff2html-css-->` - CSS styles
- `<!--diff2html-js-ui-->` - JavaScript
- `<!--diff2html-diff-->` - Diff content
- `<!--diff2html-title-->` - Page title

## Integration Examples

### CI/CD Pipeline

```bash
# Generate diff report as artifact
git diff origin/main...HEAD | diff2html -i stdin -F diff-report.html
```

### Code Review

```bash
# Review PR changes
gh pr diff 123 | diff2html -i stdin -t "PR #123 Review"
```

### Git Hooks

```bash
# pre-push hook - preview changes before push
#!/bin/bash
diff2html -- @{push}..HEAD
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Error (invalid args, git error, I/O error) |

## Environment

The tool requires `git` to be available in PATH when using `--input command`.

## License

MIT License
