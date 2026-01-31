//! Custom parser configuration example.
//!
//! Run with: cargo run --example custom_parser

use diff2html::{Diff2HtmlConfig, DiffParserConfig, html_from_diff_files, parse};

fn main() {
    // Sample diff with prefixes
    let diff = r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,100 +1,100 @@
-line 1
-line 2
-line 3
+modified line 1
+modified line 2
+modified line 3
"#;

    // Parser configuration with limits
    let parser_config = DiffParserConfig {
        src_prefix: Some("a/".to_string()),
        dst_prefix: Some("b/".to_string()),
        diff_max_changes: Some(50), // Limit to 50 changes
        diff_max_line_length: Some(200),
        diff_too_big_message: None, // Use default message
    };

    // Parse with custom configuration
    let files = parse(diff, &parser_config);

    println!("Parsed {} file(s)", files.len());
    for file in &files {
        println!("  {} ({} blocks)", file.new_name, file.blocks.len());
        if file.is_too_big.unwrap_or(false) {
            println!("    [Too big to display]");
        }
    }

    // Render with default config
    let html_output = html_from_diff_files(&files, &Diff2HtmlConfig::default());
    println!("\nGenerated HTML: {} bytes", html_output.len());
}
