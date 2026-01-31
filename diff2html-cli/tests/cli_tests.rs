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
    assert!(stdout.contains("diff2html") || stdout.contains("Usage"));
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
    assert!(stdout.contains("diff2html") || stdout.contains(env!("CARGO_PKG_VERSION")));
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
    assert!(stdout.contains("d2h-wrapper") || stdout.contains("sample"));
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
    assert!(stdout.contains("d2h-wrapper") || stdout.contains("sample"));
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
    assert!(stdout.contains("<") && stdout.contains(">"));
    assert!(stdout.contains("d2h-") || stdout.contains("html"));
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
    // JSON output should start with [ and be valid JSON
    assert!(stdout.trim().starts_with('['));
    assert!(stdout.contains("oldName") || stdout.contains("old_name"));
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
    assert!(stdout.contains("d2h-file-diff") || stdout.contains("d2h-diff-table"));
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
    assert!(stdout.contains("d2h-file-side-diff") || stdout.contains("d2h-files-diff"));
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
    assert!(stdout.contains("dark") || stdout.contains("d2h-dark"));
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
    assert!(stdout.contains("light") || stdout.contains("d2h-light"));
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
    // Should contain both files
    assert!(stdout.contains("init.js") || stdout.contains("event.js"));
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
    assert!(stdout.contains("Binary") || stdout.contains("binary") || stdout.contains(".png"));
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
