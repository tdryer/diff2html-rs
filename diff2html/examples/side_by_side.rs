//! Side-by-side view with configuration options.
//!
//! Run with: cargo run --example side_by_side

use diff2html::{ColorScheme, Diff2HtmlConfig, DiffStyle, LineMatchingType, OutputFormat, html};

fn main() {
    // Sample diff with multiple changes
    let diff = r#"diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1,8 +1,10 @@
 # My Project

-A simple project.
+A powerful Rust project.

 ## Features

-- Basic functionality
+- Advanced functionality
+- High performance
+- Memory safety
"#;

    // Configure for side-by-side view with dark theme
    let config = Diff2HtmlConfig {
        output_format: OutputFormat::SideBySide,
        diff_style: DiffStyle::Word,
        color_scheme: ColorScheme::Dark,
        draw_file_list: true,
        matching: LineMatchingType::Lines,
        match_words_threshold: 0.25,
        ..Default::default()
    };

    let html_output = html(diff, &config);

    println!("Side-by-side HTML generated ({} bytes)", html_output.len());
    println!();
    println!("Configuration:");
    println!("  - Output format: Side-by-side");
    println!("  - Diff style: Word-level");
    println!("  - Color scheme: Dark");
    println!("  - File list: Enabled");
    println!("  - Line matching: Lines");
}
