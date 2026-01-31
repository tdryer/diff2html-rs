//! Line matching algorithm demonstration.
//!
//! Run with: cargo run --example line_matching

use diff2html::{levenshtein, match_lines, string_distance};

fn main() {
    println!("=== Levenshtein Distance Examples ===\n");

    // Basic Levenshtein distance
    let pairs = [
        ("hello", "hello"),
        ("hello", "hallo"),
        ("hello", "world"),
        ("rust", "rust"),
        ("rust", "rest"),
        ("kitten", "sitting"),
    ];

    for (a, b) in pairs {
        let distance = levenshtein(a, b);
        println!("  '{}' <-> '{}' = {}", a, b, distance);
    }

    println!("\n=== Normalized String Distance ===\n");

    // Normalized string distance (0.0 to 1.0)
    // 0.0 = identical, higher = more different
    for (a, b) in &pairs {
        let dist = string_distance(a, b);
        println!("  '{}' <-> '{}' = {:.3}", a, b, dist);
    }

    println!("\n=== Line Matching Example ===\n");

    // Match similar lines between old and new versions
    let old_lines = vec![
        "fn main() {",
        "    println!(\"Hello\");",
        "    let x = 42;",
        "}",
    ];

    let new_lines = vec![
        "fn main() {",
        "    println!(\"Hello, World!\");",
        "    let x = 100;",
        "    let y = 200;",
        "}",
    ];

    println!("Old lines:");
    for (i, line) in old_lines.iter().enumerate() {
        println!("  [{}] {}", i, line);
    }

    println!("\nNew lines:");
    for (i, line) in new_lines.iter().enumerate() {
        println!("  [{}] {}", i, line);
    }

    // Match lines using the library function
    // Returns groups of (old_lines, new_lines) that are paired together
    let groups = match_lines(&old_lines, &new_lines, |a, b| string_distance(a, b));

    println!("\nMatched Groups:");
    for (i, (old_group, new_group)) in groups.iter().enumerate() {
        println!("  Group {}:", i);
        println!("    Old: {:?}", old_group);
        println!("    New: {:?}", new_group);
    }
}
