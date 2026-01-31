//! Unified diff parser.
//!
//! Parses unified diff format as specified in:
//! - Unified: https://www.gnu.org/software/diffutils/manual/html_node/Unified-Format.html
//! - Git Diff: https://git-scm.com/docs/git-diff-tree#_raw_output_format
//! - Git Combined Diff: https://git-scm.com/docs/git-diff-tree#_combined_diff_format

use once_cell::sync::Lazy;
use regex::Regex;

use crate::types::{Checksum, DiffBlock, DiffFile, DiffLine, FileMode, LineType};

/// Configuration for the diff parser.
#[derive(Default)]
pub struct DiffParserConfig {
    /// Prefix to strip from source file paths.
    pub src_prefix: Option<String>,
    /// Prefix to strip from destination file paths.
    pub dst_prefix: Option<String>,
    /// Maximum number of changes before marking file as "too big".
    pub diff_max_changes: Option<u32>,
    /// Maximum line length before marking file as "too big".
    pub diff_max_line_length: Option<usize>,
    /// Custom message for files that are too big.
    pub diff_too_big_message: Option<Box<dyn Fn(usize) -> String + Send + Sync>>,
}

impl std::fmt::Debug for DiffParserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiffParserConfig")
            .field("src_prefix", &self.src_prefix)
            .field("dst_prefix", &self.dst_prefix)
            .field("diff_max_changes", &self.diff_max_changes)
            .field("diff_max_line_length", &self.diff_max_line_length)
            .field("diff_too_big_message", &self.diff_too_big_message.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

// Regex patterns for parsing diff metadata
static OLD_MODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^old mode (\d{6})").unwrap());
static NEW_MODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^new mode (\d{6})").unwrap());
static DELETED_FILE_MODE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^deleted file mode (\d{6})").unwrap());
static NEW_FILE_MODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^new file mode (\d{6})").unwrap());

static COPY_FROM: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^copy from "?(.+)"?"#).unwrap());
static COPY_TO: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^copy to "?(.+)"?"#).unwrap());

static RENAME_FROM: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^rename from "?(.+)"?"#).unwrap());
static RENAME_TO: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^rename to "?(.+)"?"#).unwrap());

static SIMILARITY_INDEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^similarity index (\d+)%").unwrap());
static DISSIMILARITY_INDEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^dissimilarity index (\d+)%").unwrap());
static INDEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^index ([\da-z]+)\.\.([\da-z]+)\s*(\d{6})?").unwrap());

static BINARY_FILES: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^Binary files (.*) and (.*) differ").unwrap());
static BINARY_DIFF: Lazy<Regex> = Lazy::new(|| Regex::new(r"^GIT binary patch").unwrap());

// Combined diff patterns
static COMBINED_INDEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^index ([\da-z]+),([\da-z]+)\.\.([\da-z]+)").unwrap());
static COMBINED_MODE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^mode (\d{6}),(\d{6})\.\.(\d{6})").unwrap());
static COMBINED_NEW_FILE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^new file mode (\d{6})").unwrap());
static COMBINED_DELETED_FILE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^deleted file mode (\d{6}),(\d{6})").unwrap());

// Hunk header patterns
static HUNK_HEADER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@.*").unwrap());
static COMBINED_HUNK_HEADER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^@@@ -(\d+)(?:,\d+)? -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@@.*").unwrap());

// Git diff start pattern
static GIT_DIFF_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^diff --git "?([a-ciow]/.+)"? "?([a-ciow]/.+)"?"#).unwrap());
static UNIX_DIFF_BINARY_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^Binary files "?([a-ciow]/.+)"? and "?([a-ciow]/.+)"? differ"#).unwrap());

/// Base prefixes used in diff file paths.
const BASE_DIFF_FILENAME_PREFIXES: &[&str] = &["a/", "b/", "i/", "w/", "c/", "o/"];

/// Diff header prefixes.
const OLD_FILE_NAME_HEADER: &str = "--- ";
const NEW_FILE_NAME_HEADER: &str = "+++ ";
const HUNK_HEADER_PREFIX: &str = "@@";

/// Escapes special regex characters in a string.
fn escape_for_regexp(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '-' | '[' | ']' | '/' | '{' | '}' | '(' | ')' | '*' | '+' | '?' | '.' | '\\' | '^'
            | '$' | '|' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Gets file extension from filename.
fn get_extension(filename: &str, language: &str) -> String {
    filename
        .rsplit('.')
        .next()
        .filter(|_| filename.contains('.'))
        .unwrap_or(language)
        .to_string()
}

/// Checks if string starts with any of the given prefixes.
fn starts_with_any(s: &str, prefixes: &[&str]) -> bool {
    prefixes.iter().any(|p| s.starts_with(p))
}

/// Extracts filename from a diff line, removing prefixes and timestamps.
fn get_filename(line: &str, line_prefix: Option<&str>, extra_prefix: Option<&str>) -> String {
    let prefixes: Vec<&str> = if let Some(extra) = extra_prefix {
        let mut p: Vec<&str> = BASE_DIFF_FILENAME_PREFIXES.to_vec();
        p.push(extra);
        p
    } else {
        BASE_DIFF_FILENAME_PREFIXES.to_vec()
    };

    let filename = if let Some(prefix) = line_prefix {
        let pattern = format!(r#"^{} "?(.+?)"?$"#, escape_for_regexp(prefix));
        let re = Regex::new(&pattern).unwrap();
        re.captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default()
    } else {
        let re = Regex::new(r#"^"?(.+?)"?$"#).unwrap();
        re.captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default()
    };

    // Remove matching prefix
    let fname_without_prefix = prefixes
        .iter()
        .find(|p| filename.starts_with(*p))
        .map(|p| filename[p.len()..].to_string())
        .unwrap_or(filename);

    // Remove timestamp suffix (e.g., "2016-10-25 11:37:14.000000000 +0200")
    let timestamp_re = Regex::new(r"\s+\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(?:\.\d+)? [+-]\d{4}.*$")
        .unwrap();
    timestamp_re.replace(&fname_without_prefix, "").to_string()
}

/// Gets source filename from a "--- " line.
fn get_src_filename(line: &str, src_prefix: Option<&str>) -> String {
    get_filename(line, Some("---"), src_prefix)
}

/// Gets destination filename from a "+++ " line.
fn get_dst_filename(line: &str, dst_prefix: Option<&str>) -> String {
    get_filename(line, Some("+++"), dst_prefix)
}

/// Parser state for tracking current file and block.
struct ParserState {
    files: Vec<DiffFile>,
    current_file: Option<DiffFile>,
    current_block: Option<DiffBlock>,
    old_line: Option<u32>,
    old_line2: Option<u32>,
    new_line: Option<u32>,
    possible_old_name: Option<String>,
    possible_new_name: Option<String>,
}

impl ParserState {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            current_file: None,
            current_block: None,
            old_line: None,
            old_line2: None,
            new_line: None,
            possible_old_name: None,
            possible_new_name: None,
        }
    }

    /// Saves current block to current file.
    fn save_block(&mut self) {
        if let (Some(block), Some(file)) = (self.current_block.take(), &mut self.current_file) {
            file.blocks.push(block);
        }
    }

    /// Saves current file to files list.
    fn save_file(&mut self) {
        if let Some(mut file) = self.current_file.take() {
            if file.old_name.is_empty()
                && let Some(name) = self.possible_old_name.take()
            {
                file.old_name = name;
            }
            if file.new_name.is_empty()
                && let Some(name) = self.possible_new_name.take()
            {
                file.new_name = name;
            }
            if !file.new_name.is_empty() {
                self.files.push(file);
            }
        }
        self.possible_old_name = None;
        self.possible_new_name = None;
    }

    /// Starts a new file.
    fn start_file(&mut self) {
        self.save_block();
        self.save_file();
        self.current_file = Some(DiffFile::default());
    }

    /// Starts a new block (hunk).
    fn start_block(&mut self, line: &str) {
        self.save_block();

        if let Some(file) = &mut self.current_file {
            if let Some(caps) = HUNK_HEADER.captures(line) {
                file.is_combined = false;
                self.old_line = caps.get(1).and_then(|m| m.as_str().parse().ok());
                self.new_line = caps.get(2).and_then(|m| m.as_str().parse().ok());
                self.old_line2 = None;
            } else if let Some(caps) = COMBINED_HUNK_HEADER.captures(line) {
                file.is_combined = true;
                self.old_line = caps.get(1).and_then(|m| m.as_str().parse().ok());
                self.old_line2 = caps.get(2).and_then(|m| m.as_str().parse().ok());
                self.new_line = caps.get(3).and_then(|m| m.as_str().parse().ok());
            } else {
                if line.starts_with(HUNK_HEADER_PREFIX) {
                    eprintln!("Failed to parse hunk header, starting at 0!");
                }
                self.old_line = Some(0);
                self.new_line = Some(0);
                self.old_line2 = None;
                file.is_combined = false;
            }
        }

        self.current_block = Some(DiffBlock {
            lines: Vec::new(),
            old_start_line: self.old_line.unwrap_or(0),
            old_start_line2: self.old_line2,
            new_start_line: self.new_line.unwrap_or(0),
            header: line.to_string(),
        });
    }

    /// Creates a diff line from a source line.
    fn create_line(&mut self, line: &str) {
        let (Some(file), Some(block), Some(old_line), Some(new_line)) = (
            &mut self.current_file,
            &mut self.current_block,
            &mut self.old_line,
            &mut self.new_line,
        ) else {
            return;
        };

        let added_prefixes: &[&str] = if file.is_combined {
            &["+ ", " +", "++"]
        } else {
            &["+"]
        };
        let deleted_prefixes: &[&str] = if file.is_combined {
            &["- ", " -", "--"]
        } else {
            &["-"]
        };

        let diff_line = if starts_with_any(line, added_prefixes) {
            file.added_lines += 1;
            let ln = DiffLine {
                line_type: LineType::Insert,
                content: line.to_string(),
                old_number: None,
                new_number: Some(*new_line),
            };
            *new_line += 1;
            ln
        } else if starts_with_any(line, deleted_prefixes) {
            file.deleted_lines += 1;
            let ln = DiffLine {
                line_type: LineType::Delete,
                content: line.to_string(),
                old_number: Some(*old_line),
                new_number: None,
            };
            *old_line += 1;
            ln
        } else {
            let ln = DiffLine {
                line_type: LineType::Context,
                content: line.to_string(),
                old_number: Some(*old_line),
                new_number: Some(*new_line),
            };
            *old_line += 1;
            *new_line += 1;
            ln
        };

        block.lines.push(diff_line);
    }
}

/// Checks if there's a hunk header before the next file starts.
fn exist_hunk_header(lines: &[&str], start_idx: usize) -> bool {
    let mut idx = start_idx;
    while idx < lines.len().saturating_sub(3) {
        let line = lines[idx];
        if line.starts_with("diff") {
            return false;
        }
        if lines[idx].starts_with(OLD_FILE_NAME_HEADER)
            && lines[idx + 1].starts_with(NEW_FILE_NAME_HEADER)
            && lines[idx + 2].starts_with(HUNK_HEADER_PREFIX)
        {
            return true;
        }
        idx += 1;
    }
    false
}

/// Parses a unified diff string into a list of DiffFile structures.
pub fn parse(diff_input: &str, config: &DiffParserConfig) -> Vec<DiffFile> {
    let mut state = ParserState::new();

    // Normalize line endings and remove "No newline at end of file" markers
    let normalized = diff_input
        .replace("\\ No newline at end of file", "")
        .replace("\r\n", "\n")
        .replace('\r', "\n");

    let diff_lines: Vec<&str> = normalized.split('\n').collect();

    for (line_index, line) in diff_lines.iter().enumerate() {
        // Skip empty lines and unmerged paths markers
        if line.is_empty() || line.starts_with('*') {
            continue;
        }

        let prev_line = if line_index > 0 {
            Some(diff_lines[line_index - 1])
        } else {
            None
        };
        let next_line = diff_lines.get(line_index + 1).copied();
        let after_next_line = diff_lines.get(line_index + 2).copied();

        // Handle git diff start
        if line.starts_with("diff --git") || line.starts_with("diff --combined") {
            state.start_file();

            if let Some(caps) = GIT_DIFF_START.captures(line) {
                state.possible_old_name = caps
                    .get(1)
                    .map(|m| get_filename(m.as_str(), None, config.dst_prefix.as_deref()));
                state.possible_new_name = caps
                    .get(2)
                    .map(|m| get_filename(m.as_str(), None, config.src_prefix.as_deref()));
            }

            if let Some(file) = &mut state.current_file {
                file.is_git_diff = true;
            }
            continue;
        }

        // Handle binary files in non-git diff
        if line.starts_with("Binary files")
            && state
                .current_file
                .as_ref()
                .map(|f| !f.is_git_diff)
                .unwrap_or(true)
        {
            state.start_file();
            if let Some(caps) = UNIX_DIFF_BINARY_START.captures(line) {
                state.possible_old_name = caps
                    .get(1)
                    .map(|m| get_filename(m.as_str(), None, config.dst_prefix.as_deref()));
                state.possible_new_name = caps
                    .get(2)
                    .map(|m| get_filename(m.as_str(), None, config.src_prefix.as_deref()));
            }
            if let Some(file) = &mut state.current_file {
                file.is_binary = Some(true);
            }
            continue;
        }

        // Start new file if needed for non-git diff
        let should_start_file = state.current_file.is_none()
            || (state
                .current_file
                .as_ref()
                .map(|f| !f.is_git_diff)
                .unwrap_or(false)
                && line.starts_with(OLD_FILE_NAME_HEADER)
                && next_line.is_some_and(|l| l.starts_with(NEW_FILE_NAME_HEADER))
                && after_next_line.is_some_and(|l| l.starts_with(HUNK_HEADER_PREFIX)));

        if should_start_file {
            state.start_file();
        }

        // Skip if file is marked as too big
        if state
            .current_file
            .as_ref()
            .is_some_and(|f| f.is_too_big == Some(true))
        {
            continue;
        }

        // Check for too big threshold
        if let Some(file) = &mut state.current_file {
            let too_many_changes = config
                .diff_max_changes
                .is_some_and(|max| file.added_lines + file.deleted_lines > max);
            let line_too_long = config
                .diff_max_line_length
                .is_some_and(|max| line.len() > max);

            if too_many_changes || line_too_long {
                file.is_too_big = Some(true);
                file.added_lines = 0;
                file.deleted_lines = 0;
                file.blocks.clear();
                state.current_block = None;

                let message = config
                    .diff_too_big_message
                    .as_ref()
                    .map(|f| f(state.files.len()))
                    .unwrap_or_else(|| "Diff too big to be displayed".to_string());
                state.start_block(&message);
                continue;
            }
        }

        // Handle file name headers
        let is_old_header = line.starts_with(OLD_FILE_NAME_HEADER);
        let is_new_header = line.starts_with(NEW_FILE_NAME_HEADER);
        let prev_is_old = prev_line.is_some_and(|l| l.starts_with(OLD_FILE_NAME_HEADER));
        let next_is_new = next_line.is_some_and(|l| l.starts_with(NEW_FILE_NAME_HEADER));

        if ((is_old_header && next_is_new) || (is_new_header && prev_is_old))
            && let Some(file) = &mut state.current_file
        {
            if file.old_name.is_empty() && line.starts_with("--- ") {
                let name = get_src_filename(line, config.src_prefix.as_deref());
                file.old_name = name.clone();
                file.language = get_extension(&name, &file.language);
                continue;
            }

            if file.new_name.is_empty() && line.starts_with("+++ ") {
                let name = get_dst_filename(line, config.dst_prefix.as_deref());
                file.new_name = name.clone();
                file.language = get_extension(&name, &file.language);
                continue;
            }
        }

        // Handle hunk header
        if state.current_file.is_some() {
            let is_hunk_header = line.starts_with(HUNK_HEADER_PREFIX);
            let should_start_block = state.current_file.as_ref().is_some_and(|f| {
                f.is_git_diff && !f.old_name.is_empty() && !f.new_name.is_empty()
            }) && state.current_block.is_none();

            if is_hunk_header || should_start_block {
                state.start_block(line);
                continue;
            }
        }

        // Handle diff lines
        if state.current_block.is_some()
            && (line.starts_with('+') || line.starts_with('-') || line.starts_with(' '))
        {
            state.create_line(line);
            continue;
        }

        // Handle git-specific metadata
        let does_not_exist_hunk_header = !exist_hunk_header(&diff_lines, line_index);

        let Some(file) = &mut state.current_file else {
            continue;
        };

        if let Some(caps) = OLD_MODE.captures(line) {
            file.old_mode = caps.get(1).map(|m| FileMode::Single(m.as_str().to_string()));
        } else if let Some(caps) = NEW_MODE.captures(line) {
            file.new_mode = caps.get(1).map(|m| m.as_str().to_string());
        } else if let Some(caps) = DELETED_FILE_MODE.captures(line) {
            file.deleted_file_mode = caps.get(1).map(|m| m.as_str().to_string());
            file.is_deleted = Some(true);
        } else if let Some(caps) = NEW_FILE_MODE.captures(line) {
            file.new_file_mode = caps.get(1).map(|m| m.as_str().to_string());
            file.is_new = Some(true);
        } else if let Some(caps) = COPY_FROM.captures(line) {
            if does_not_exist_hunk_header {
                file.old_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            }
            file.is_copy = Some(true);
        } else if let Some(caps) = COPY_TO.captures(line) {
            if does_not_exist_hunk_header {
                file.new_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            }
            file.is_copy = Some(true);
        } else if let Some(caps) = RENAME_FROM.captures(line) {
            if does_not_exist_hunk_header {
                file.old_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            }
            file.is_rename = Some(true);
        } else if let Some(caps) = RENAME_TO.captures(line) {
            if does_not_exist_hunk_header {
                file.new_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            }
            file.is_rename = Some(true);
        } else if let Some(caps) = BINARY_FILES.captures(line) {
            file.is_binary = Some(true);
            file.old_name = caps
                .get(1)
                .map(|m| get_filename(m.as_str(), None, config.src_prefix.as_deref()))
                .unwrap_or_default();
            file.new_name = caps
                .get(2)
                .map(|m| get_filename(m.as_str(), None, config.dst_prefix.as_deref()))
                .unwrap_or_default();
            state.start_block("Binary file");
        } else if BINARY_DIFF.is_match(line) {
            file.is_binary = Some(true);
            state.start_block(line);
        } else if let Some(caps) = SIMILARITY_INDEX.captures(line) {
            file.unchanged_percentage = caps.get(1).and_then(|m| m.as_str().parse().ok());
        } else if let Some(caps) = DISSIMILARITY_INDEX.captures(line) {
            file.changed_percentage = caps.get(1).and_then(|m| m.as_str().parse().ok());
        } else if let Some(caps) = INDEX.captures(line) {
            file.checksum_before = caps.get(1).map(|m| Checksum::Single(m.as_str().to_string()));
            file.checksum_after = caps.get(2).map(|m| m.as_str().to_string());
            file.mode = caps.get(3).map(|m| m.as_str().to_string());
        } else if let Some(caps) = COMBINED_INDEX.captures(line) {
            file.checksum_before = Some(Checksum::Multiple(vec![
                caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
                caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default(),
            ]));
            file.checksum_after = caps.get(1).map(|m| m.as_str().to_string());
        } else if let Some(caps) = COMBINED_MODE.captures(line) {
            file.old_mode = Some(FileMode::Multiple(vec![
                caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
                caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default(),
            ]));
            file.new_mode = caps.get(1).map(|m| m.as_str().to_string());
        } else if let Some(caps) = COMBINED_NEW_FILE.captures(line) {
            file.new_file_mode = caps.get(1).map(|m| m.as_str().to_string());
            file.is_new = Some(true);
        } else if let Some(caps) = COMBINED_DELETED_FILE.captures(line) {
            file.deleted_file_mode = caps.get(1).map(|m| m.as_str().to_string());
            file.is_deleted = Some(true);
        }
    }

    state.save_block();
    state.save_file();

    state.files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_diff() {
        let diff = r#"diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,4 @@
 line1
-line2
+line2 modified
+new line
 line3
"#;

        let files = parse(diff, &DiffParserConfig::default());
        assert_eq!(files.len(), 1);

        let file = &files[0];
        assert_eq!(file.old_name, "test.txt");
        assert_eq!(file.new_name, "test.txt");
        assert!(file.is_git_diff);
        assert!(!file.is_combined);
        assert_eq!(file.added_lines, 2);
        assert_eq!(file.deleted_lines, 1);
        assert_eq!(file.blocks.len(), 1);

        let block = &file.blocks[0];
        assert_eq!(block.old_start_line, 1);
        assert_eq!(block.new_start_line, 1);
        assert_eq!(block.lines.len(), 5);
    }

    #[test]
    fn test_parse_new_file() {
        let diff = r#"diff --git a/newfile.txt b/newfile.txt
new file mode 100644
index 0000000..1234567
--- /dev/null
+++ b/newfile.txt
@@ -0,0 +1,2 @@
+line1
+line2
"#;

        let files = parse(diff, &DiffParserConfig::default());
        assert_eq!(files.len(), 1);

        let file = &files[0];
        assert_eq!(file.new_name, "newfile.txt");
        assert_eq!(file.is_new, Some(true));
        assert_eq!(file.new_file_mode, Some("100644".to_string()));
        assert_eq!(file.added_lines, 2);
    }

    #[test]
    fn test_parse_deleted_file() {
        let diff = r#"diff --git a/deleted.txt b/deleted.txt
deleted file mode 100644
index 1234567..0000000
--- a/deleted.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-line1
-line2
"#;

        let files = parse(diff, &DiffParserConfig::default());
        assert_eq!(files.len(), 1);

        let file = &files[0];
        assert_eq!(file.old_name, "deleted.txt");
        assert_eq!(file.is_deleted, Some(true));
        assert_eq!(file.deleted_lines, 2);
    }

    #[test]
    fn test_parse_rename() {
        let diff = r#"diff --git a/old.txt b/new.txt
similarity index 95%
rename from old.txt
rename to new.txt
index 1234567..abcdefg 100644
"#;

        let files = parse(diff, &DiffParserConfig::default());
        assert_eq!(files.len(), 1);

        let file = &files[0];
        assert_eq!(file.old_name, "old.txt");
        assert_eq!(file.new_name, "new.txt");
        assert_eq!(file.is_rename, Some(true));
        assert_eq!(file.unchanged_percentage, Some(95));
    }

    #[test]
    fn test_parse_binary_file() {
        let diff = r#"diff --git a/image.png b/image.png
index 1234567..abcdefg 100644
Binary files a/image.png and b/image.png differ
"#;

        let files = parse(diff, &DiffParserConfig::default());
        assert_eq!(files.len(), 1);

        let file = &files[0];
        assert_eq!(file.is_binary, Some(true));
    }

    #[test]
    fn test_parse_multiple_files() {
        let diff = r#"diff --git a/file1.txt b/file1.txt
index 1234567..abcdefg 100644
--- a/file1.txt
+++ b/file1.txt
@@ -1 +1 @@
-old
+new
diff --git a/file2.txt b/file2.txt
index 1234567..abcdefg 100644
--- a/file2.txt
+++ b/file2.txt
@@ -1 +1 @@
-foo
+bar
"#;

        let files = parse(diff, &DiffParserConfig::default());
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].new_name, "file1.txt");
        assert_eq!(files[1].new_name, "file2.txt");
    }

    #[test]
    fn test_parse_unified_diff_without_git() {
        let diff = r#"--- a/test.txt	2024-01-01 00:00:00.000000000 +0000
+++ b/test.txt	2024-01-02 00:00:00.000000000 +0000
@@ -1,2 +1,2 @@
 unchanged
-removed
+added
"#;

        let files = parse(diff, &DiffParserConfig::default());
        assert_eq!(files.len(), 1);

        let file = &files[0];
        assert!(!file.is_git_diff);
        assert_eq!(file.old_name, "test.txt");
        assert_eq!(file.new_name, "test.txt");
    }

    #[test]
    fn test_line_types() {
        let diff = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 context
-deleted
+inserted
"#;

        let files = parse(diff, &DiffParserConfig::default());
        let lines = &files[0].blocks[0].lines;

        assert_eq!(lines[0].line_type, LineType::Context);
        assert_eq!(lines[0].old_number, Some(1));
        assert_eq!(lines[0].new_number, Some(1));

        assert_eq!(lines[1].line_type, LineType::Delete);
        assert_eq!(lines[1].old_number, Some(2));
        assert_eq!(lines[1].new_number, None);

        assert_eq!(lines[2].line_type, LineType::Insert);
        assert_eq!(lines[2].old_number, None);
        assert_eq!(lines[2].new_number, Some(2));
    }

    #[test]
    fn test_diff_too_big() {
        let diff = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,5 @@
 context
+line1
+line2
+line3
+line4
"#;

        let config = DiffParserConfig {
            diff_max_changes: Some(2),
            ..Default::default()
        };

        let files = parse(diff, &config);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].is_too_big, Some(true));
        assert_eq!(files[0].added_lines, 0);
    }

    #[test]
    fn test_escape_for_regexp() {
        assert_eq!(escape_for_regexp("a.b"), "a\\.b");
        assert_eq!(escape_for_regexp("a+b"), "a\\+b");
        assert_eq!(escape_for_regexp("a/b"), "a\\/b");
        assert_eq!(escape_for_regexp("[test]"), "\\[test\\]");
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("file.rs", "txt"), "rs");
        assert_eq!(get_extension("file.test.rs", "txt"), "rs");
        assert_eq!(get_extension("noextension", "default"), "default");
    }

    #[test]
    fn test_json_serialization() {
        let diff = r#"diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1 +1 @@
-old
+new
"#;

        let files = parse(diff, &DiffParserConfig::default());
        let json = serde_json::to_string(&files).unwrap();

        // Verify it contains expected fields with camelCase
        assert!(json.contains("\"oldName\""));
        assert!(json.contains("\"newName\""));
        assert!(json.contains("\"addedLines\""));
        assert!(json.contains("\"deletedLines\""));
        assert!(json.contains("\"isGitDiff\""));
        assert!(json.contains("\"isCombined\""));
        assert!(json.contains("\"oldStartLine\""));
        assert!(json.contains("\"newStartLine\""));

        // Verify line types are lowercase
        assert!(json.contains("\"type\":\"insert\"") || json.contains("\"type\": \"insert\""));
        assert!(json.contains("\"type\":\"delete\"") || json.contains("\"type\": \"delete\""));
    }

    #[test]
    fn test_combined_diff() {
        // Combined diff format from a merge commit
        let diff = r#"diff --combined file.txt
index abc123,def456..789012
--- a/file.txt
+++ b/file.txt
@@@ -1,2 -1,2 +1,3 @@@
  unchanged
 -deleted from first
 + added in merge
++added in both
"#;

        let files = parse(diff, &DiffParserConfig::default());
        assert_eq!(files.len(), 1);

        let file = &files[0];
        assert!(file.is_combined);
        assert_eq!(file.blocks.len(), 1);
        assert!(file.blocks[0].old_start_line2.is_some());
        assert_eq!(file.blocks[0].old_start_line, 1);
        assert_eq!(file.blocks[0].old_start_line2, Some(1));
        assert_eq!(file.blocks[0].new_start_line, 1);
    }
}
