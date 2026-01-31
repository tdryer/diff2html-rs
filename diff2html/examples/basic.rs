//! Basic example of converting a diff to HTML.
//!
//! Run with: cargo run --example basic

use diff2html::{Diff2HtmlConfig, html};

fn main() {
    // Sample unified diff
    let diff = r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,5 +1,6 @@
 fn main() {
-    println!("Hello, world!");
+    let name = "Rust";
+    println!("Hello, {}!", name);
 }
"#;

    // Generate HTML with default configuration
    let html_output = html(diff, &Diff2HtmlConfig::default());

    // Print a preview of the output
    println!("Generated HTML ({} bytes):", html_output.len());
    println!();

    // Show first 500 characters
    if html_output.len() > 500 {
        println!("{}...", &html_output[..500]);
    } else {
        println!("{}", html_output);
    }
}
