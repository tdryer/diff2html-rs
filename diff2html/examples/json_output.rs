//! JSON output for integration with other tools.
//!
//! Run with: cargo run --example json_output

use diff2html::{Diff2HtmlConfig, DiffParserConfig, json, json_from_diff_files_pretty, parse};

fn main() {
    // Sample diff
    let diff = r#"diff --git a/config.toml b/config.toml
--- a/config.toml
+++ b/config.toml
@@ -1,3 +1,4 @@
 [package]
 name = "myapp"
-version = "0.1.0"
+version = "0.2.0"
+edition = "2021"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1 +1,5 @@
-pub fn hello() {}
+pub fn hello() {
+    println!("Hello!");
+}
+
+pub fn goodbye() {}
"#;

    // Method 1: Direct JSON output (compact)
    let json_compact = json(diff, &Diff2HtmlConfig::default()).unwrap();
    println!("Compact JSON ({} bytes)", json_compact.len());

    // Method 2: Parse first, then pretty-print JSON
    let files = parse(diff, &DiffParserConfig::default());
    let json_pretty = json_from_diff_files_pretty(&files).unwrap();

    println!("\nPretty JSON output:\n");
    println!("{}", json_pretty);

    // Access parsed data programmatically
    println!("\n--- Parsed Data Summary ---");
    for file in &files {
        println!(
            "File: {} -> {} (+{} -{} lines)",
            file.old_name, file.new_name, file.added_lines, file.deleted_lines
        );
        for block in &file.blocks {
            println!("  Block: {}", block.header);
            println!("    Lines: {}", block.lines.len());
        }
    }
}
