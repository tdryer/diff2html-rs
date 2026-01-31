# Rust Port of diff2html-cli

## Overview

Port the `diff2html` library and `diff2html-cli` tool from TypeScript to Rust. The original projects:
- **diff2html**: Core library that parses unified diffs and generates HTML (482-line parser, 19 Mustache templates, 600+ lines CSS)
- **diff2html-cli**: CLI wrapper with multiple input/output modes

## Crate Structure

```
diff2html-rs/
├── Cargo.toml                    # Workspace root
├── diff2html/                    # Core library
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                # Public API: parse(), html()
│   │   ├── types.rs              # DiffFile, DiffBlock, DiffLine, enums
│   │   ├── parser.rs             # Diff parsing state machine
│   │   ├── render/
│   │   │   ├── mod.rs
│   │   │   ├── line_by_line.rs   # Single-column renderer
│   │   │   ├── side_by_side.rs   # Two-column renderer
│   │   │   ├── file_list.rs      # File summary list
│   │   │   └── utils.rs          # HTML escaping, diff highlight
│   │   ├── rematch.rs            # Levenshtein line matching
│   │   └── templates.rs          # Template engine wrapper
│   ├── templates/                # 19 Mustache templates (embedded)
│   └── css/                      # CSS (embedded)
├── diff2html-cli/                # CLI binary
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── args.rs               # Clap argument definitions
│   │   ├── config.rs             # Configuration conversion
│   │   ├── input.rs              # File/stdin/git-command input
│   │   └── output.rs             # Preview/stdout/file output
│   └── templates/
│       └── wrapper.html          # HTML page wrapper
└── tests/                        # Integration tests
```

## Dependencies

### diff2html (core library)
| Crate | Purpose | Replaces |
|-------|---------|----------|
| `serde` + `serde_json` | JSON serialization for output | - |
| `handlebars` | Mustache-compatible templates | `hogan.js` |
| `regex` | Diff header parsing | - |
| `similar` | Word/char-level diff highlighting | `diff` npm package |
| `thiserror` | Error types | - |

Note: The Levenshtein line-matching algorithm (`rematch.ts`) is custom code in the original and will be ported directly - no external dependency needed.

### diff2html-cli
| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `open` | Open files in browser |
| `anyhow` | Error handling |

## Key Type Mappings

```rust
// types.rs
pub enum LineType { Insert, Delete, Context }

pub struct DiffLine {
    pub line_type: LineType,
    pub content: String,
    pub old_number: Option<u32>,
    pub new_number: Option<u32>,
}

pub struct DiffBlock {
    pub old_start_line: u32,
    pub old_start_line2: Option<u32>,  // Combined diffs
    pub new_start_line: u32,
    pub header: String,
    pub lines: Vec<DiffLine>,
}

pub struct DiffFile {
    pub old_name: String,
    pub new_name: String,
    pub added_lines: u32,
    pub deleted_lines: u32,
    pub is_combined: bool,
    pub is_git_diff: bool,
    pub language: Option<String>,
    pub blocks: Vec<DiffBlock>,
    // Metadata: old_mode, new_mode, is_deleted, is_new, is_copy,
    // is_rename, is_binary, is_too_big, checksums, etc.
}

pub enum OutputFormat { LineByLine, SideBySide }
pub enum DiffStyle { Word, Char }
pub enum ColorScheme { Light, Dark, Auto }
pub enum LineMatching { None, Lines, Words }
```

## Implementation Phases

### Phase 1: Core Types and Parser
1. Create workspace with both crates
2. Implement `types.rs` with all structs/enums
3. Port `diff-parser.ts` (482 lines) to `parser.rs`:
   - State machine: currentFile, currentBlock, line counters
   - Regex patterns for metadata extraction
   - Handle: git diff, combined diff, binary files, renames, copies
   - "Too big" detection with configurable limits
4. Add parser unit tests

### Phase 2: Template System
1. Copy 19 Mustache templates to `templates/` directory
2. Implement template loading with `handlebars` crate
3. Embed templates at compile time with `include_str!`
4. Embed CSS files

### Phase 3: HTML Renderers
1. Implement `LineByLineRenderer`:
   - Group lines: context, deletions, insertions
   - Generate table rows with line numbers
2. Implement `SideBySideRenderer`:
   - Left/right parallel rendering
   - Paired change matching
3. Implement `FileListRenderer` for summary view
4. Add `render_utils.rs`:
   - `escape_html()` - HTML entity escaping
   - `diff_highlight()` - Word/char diff using `similar` crate
   - `filename_diff()` - Pretty rename display: `{old → new}`

### Phase 4: Line Matching
1. Implement `rematch.rs`:
   - `levenshtein()` distance function
   - Recursive best-match algorithm for pairing changed lines
   - Configurable threshold and max comparisons

### Phase 5: Public API
1. Implement `lib.rs` with:
   - `parse(diff_input: &str, config: &ParserConfig) -> Vec<DiffFile>`
   - `html(input: &str | &[DiffFile], config: &Diff2HtmlConfig) -> String`
2. Add JSON output support via `serde_json`

### Phase 6: CLI Implementation
1. Define args with Clap (match all 25+ original options)
2. Implement input sources:
   - `file`: Read from filesystem
   - `stdin`: Read from pipe
   - `command`: Execute `git diff --no-color [args]`
3. Implement output destinations:
   - `preview`: Write temp file, `open` in browser
   - `stdout`: Print to console
   - `file`: Write to specified path
4. HTML wrapper: Inject CSS, JS URLs, diff content into template

### Phase 7: Testing and Polish
1. Port test cases from TypeScript tests
2. Integration tests with sample diff files
3. Documentation and examples

## Critical Files to Reference

The original TypeScript projects are in `original/` subdirectory.

| Original | Purpose |
|----------|---------|
| `original/diff2html/src/diff-parser.ts` | Parser state machine (482 lines) |
| `original/diff2html/src/types.ts` | All type definitions |
| `original/diff2html/src/render-utils.ts` | HTML escaping, diff highlighting |
| `original/diff2html/src/rematch.ts` | Levenshtein matching |
| `original/diff2html/src/line-by-line-renderer.ts` | Single-column renderer |
| `original/diff2html/src/side-by-side-renderer.ts` | Two-column renderer |
| `original/diff2html/src/templates/*.mustache` | 19 HTML templates |
| `original/diff2html/src/ui/css/diff2html.css` | All styling |
| `original/diff2html-cli/src/yargs.ts` | CLI argument definitions |
| `original/diff2html-cli/src/cli.ts` | Input/output logic |
| `original/diff2html-cli/template.html` | HTML page wrapper |

## CLI Options to Support

```
-s, --style         line | side (default: line)
-d, --diffStyle     word | char (default: word)
-f, --format        html | json (default: html)
-i, --input         file | command | stdin (default: command)
-o, --output        preview | stdout (default: preview)
-F, --file          Output file path
-t, --title         HTML page title
--colorScheme       auto | dark | light
--highlightCode     Enable syntax highlighting
--synchronisedScroll  Sync side-by-side scroll
--fileContentToggle   Show viewed checkbox
--matching          none | lines | words
--matchWordsThreshold  Float (default: 0.25)
--diffMaxChanges    Max lines before "too big"
--ignore            Files to exclude
-- [git args]       Passthrough to git diff
```

## Verification

1. **Parser tests**: Compare JSON output against TypeScript version
2. **HTML tests**: Render sample diffs, verify structure
3. **CLI tests**:
   - `echo "diff content" | diff2html -i stdin -o stdout`
   - `diff2html -F output.html -- -M HEAD~1`
   - `diff2html -s side --colorScheme dark -- HEAD`
4. **Visual verification**: Open generated HTML in browser
