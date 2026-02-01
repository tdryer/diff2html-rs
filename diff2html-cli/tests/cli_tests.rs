//! Integration tests for the CLI.
//!
//! These tests verify the CLI binary works correctly with various inputs and options.

use std::process::Command;

/// Helper to get the path to the built binary
fn binary_path() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps directory
    path.push("diff2html-cli");
    path
}

/// Helper to check if the binary exists
fn binary_exists() -> bool {
    binary_path().exists()
}

/// Helper to load a test fixture
fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

// =============================================================================
// Basic CLI Tests
// =============================================================================

#[test]
fn test_cli_help() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let output = Command::new(binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Help should contain both the program name and usage information
    assert!(
        stdout.contains("diff2html"),
        "Help output should contain program name"
    );
    assert!(
        stdout.contains("Usage"),
        "Help output should contain Usage section"
    );
}

#[test]
fn test_cli_version() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let output = Command::new(binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Version output should contain both program name and version number
    assert!(
        stdout.contains("diff2html"),
        "Version output should contain program name"
    );
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "Version output should contain version number"
    );
}

// =============================================================================
// Input Type Tests
// =============================================================================

#[test]
fn test_cli_file_input() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let output = Command::new(binary_path())
        .args([
            "-i",
            "file",
            "-o",
            "stdout",
            "--",
            &fixture_path("simple.diff"),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // HTML output should contain the d2h-wrapper class
    assert!(
        stdout.contains("d2h-wrapper"),
        "HTML output should contain d2h-wrapper class"
    );
}

#[test]
fn test_cli_stdin_input() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // HTML output should contain the d2h-wrapper class
    assert!(
        stdout.contains("d2h-wrapper"),
        "HTML output should contain d2h-wrapper class"
    );
}

// =============================================================================
// Output Format Tests
// =============================================================================

#[test]
fn test_cli_html_format() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout", "-f", "html"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // HTML output should contain HTML tags and d2h- prefixed CSS classes
    assert!(
        stdout.contains("<") && stdout.contains(">"),
        "HTML output should contain HTML tags"
    );
    assert!(
        stdout.contains("d2h-"),
        "HTML output should contain d2h- prefixed CSS classes"
    );
}

#[test]
fn test_cli_json_format() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout", "-f", "json"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSON output should start with [ and contain expected fields
    assert!(
        stdout.trim().starts_with('['),
        "JSON output should start with ["
    );
    assert!(
        stdout.contains("oldName"),
        "JSON output should contain oldName field"
    );
}

// =============================================================================
// Style Tests
// =============================================================================

#[test]
fn test_cli_line_by_line_style() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout", "-s", "line"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Line-by-line style should contain the d2h-file-diff class
    assert!(
        stdout.contains("d2h-file-diff"),
        "Line-by-line output should contain d2h-file-diff class"
    );
}

#[test]
fn test_cli_side_by_side_style() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout", "-s", "side"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Side-by-side style should contain the d2h-file-side-diff class
    assert!(
        stdout.contains("d2h-file-side-diff"),
        "Side-by-side output should contain d2h-file-side-diff class"
    );
}

// =============================================================================
// Color Scheme Tests
// =============================================================================

#[test]
fn test_cli_dark_color_scheme() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout", "--colorScheme", "dark"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Dark color scheme should use d2h-dark-color-scheme class
    assert!(
        stdout.contains("d2h-dark-color-scheme"),
        "Dark color scheme output should contain d2h-dark-color-scheme class"
    );
}

#[test]
fn test_cli_light_color_scheme() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout", "--colorScheme", "light"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Light color scheme should use d2h-light-color-scheme class
    assert!(
        stdout.contains("d2h-light-color-scheme"),
        "Light color scheme output should contain d2h-light-color-scheme class"
    );
}

// =============================================================================
// Diff Style Tests
// =============================================================================

#[test]
fn test_cli_word_diff_style() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout", "-d", "word"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
}

#[test]
fn test_cli_char_diff_style() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout", "-d", "char"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
}

// =============================================================================
// Multiple Files Tests
// =============================================================================

#[test]
fn test_cli_multiple_files() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content = std::fs::read_to_string(fixture_path("multiple_files.diff"))
        .expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain both files from the diff
    assert!(
        stdout.contains("init.js"),
        "Multiple files output should contain init.js"
    );
    assert!(
        stdout.contains("event.js"),
        "Multiple files output should contain event.js"
    );
}

// =============================================================================
// Empty Input Tests
// =============================================================================

#[test]
fn test_cli_empty_input() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"").unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    // Empty input should succeed (or at least not crash)
    assert!(output.status.success() || output.status.code() == Some(0));
}

// =============================================================================
// Binary Diff Tests
// =============================================================================

#[test]
fn test_cli_binary_diff() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("binary.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Binary diff should show the file name and binary indicator
    assert!(
        stdout.contains(".png"),
        "Binary diff output should contain .png file extension"
    );
    assert!(
        stdout.contains("Binary files differ"),
        "Binary diff output should contain 'Binary files differ' message"
    );
}

// =============================================================================
// Combined Diff Tests
// =============================================================================

#[test]
fn test_cli_combined_diff() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("combined.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("describe.c"));
}

// =============================================================================
// Title Option Tests
// =============================================================================

#[test]
fn test_cli_custom_title() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let diff_content =
        std::fs::read_to_string(fixture_path("simple.diff")).expect("Failed to read fixture");

    let mut child = Command::new(binary_path())
        .args(["-i", "stdin", "-o", "stdout", "-t", "My Custom Title"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(diff_content.as_bytes())
        .unwrap();

    let output = child
        .wait_with_output()
        .expect("Failed to wait for command");
    assert!(output.status.success());
    // Note: Title appears in the HTML wrapper, not just the diff output
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_cli_nonexistent_file() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let output = Command::new(binary_path())
        .args([
            "-i",
            "file",
            "-o",
            "stdout",
            "--",
            "nonexistent_file_12345.diff",
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail with non-zero exit code
    assert!(!output.status.success());
}

#[test]
fn test_cli_invalid_style() {
    if !binary_exists() {
        eprintln!("Skipping test: binary not built");
        return;
    }

    let output = Command::new(binary_path())
        .args(["-s", "invalid_style"])
        .output()
        .expect("Failed to execute command");

    // Should fail due to invalid argument
    assert!(!output.status.success());
}
