//! Core types for diff parsing and rendering.

use serde::{Deserialize, Serialize};

/// Parts of a diff line split by prefix and content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffLineParts {
    pub prefix: String,
    pub content: String,
}

/// The type of a diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineType {
    Insert,
    Delete,
    Context,
}

/// A single line in a diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    #[serde(rename = "type")]
    pub line_type: LineType,
    pub content: String,
    pub old_number: Option<u32>,
    pub new_number: Option<u32>,
}

/// A block (hunk) in a diff file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffBlock {
    pub old_start_line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_start_line2: Option<u32>,
    pub new_start_line: u32,
    pub header: String,
    pub lines: Vec<DiffLine>,
}

/// File mode representation that can be a single mode or multiple (for combined diffs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileMode {
    Single(String),
    Multiple(Vec<String>),
}

/// Checksum representation that can be single or multiple (for combined diffs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Checksum {
    Single(String),
    Multiple(Vec<String>),
}

/// A complete diff file with all metadata and blocks.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffFile {
    pub old_name: String,
    pub new_name: String,
    pub added_lines: u32,
    pub deleted_lines: u32,
    pub is_combined: bool,
    pub is_git_diff: bool,
    pub language: String,
    pub blocks: Vec<DiffBlock>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_mode: Option<FileMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_file_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_file_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_deleted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_new: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_copy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_rename: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_binary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_too_big: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unchanged_percentage: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_percentage: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_before: Option<Checksum>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// Output format for HTML rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    #[default]
    LineByLine,
    SideBySide,
}

/// Line matching algorithm type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineMatchingType {
    Lines,
    Words,
    #[default]
    None,
}

/// Diff style for highlighting changes within lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffStyle {
    #[default]
    Word,
    Char,
}

/// Color scheme for rendered output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    #[default]
    Auto,
    Dark,
    Light,
}
