//! Profiling binary for use with valgrind/callgrind.
//!
//! Run with:
//! ```
//! cargo build --release --example profile
//! valgrind --tool=callgrind ./target/release/examples/profile [parse|render|levenshtein|all]
//! ```

use diff2html::{
    Diff2HtmlConfig, DiffParserConfig, LineMatchingType, OutputFormat, html_from_diff_files,
    levenshtein, match_lines, parse, string_distance,
};
use std::env;
use std::hint::black_box;

const ITERATIONS: usize = 1000;

fn generate_large_diff(num_files: usize, lines_per_file: usize) -> String {
    let mut diff = String::new();
    for i in 0..num_files {
        diff.push_str(&format!(
            "diff --git a/file{i}.txt b/file{i}.txt\n\
             index 0000001..0ddf2ba\n\
             --- a/file{i}.txt\n\
             +++ b/file{i}.txt\n\
             @@ -1,{lines_per_file} +1,{lines_per_file} @@\n"
        ));
        for j in 0..lines_per_file {
            if j % 3 == 0 {
                diff.push_str(&format!("-old line {j} in file {i}\n"));
                diff.push_str(&format!("+new line {j} in file {i}\n"));
            } else {
                diff.push_str(&format!(" context line {j} in file {i}\n"));
            }
        }
    }
    diff
}

fn profile_parse() {
    let large_diff = generate_large_diff(10, 50);
    let config = DiffParserConfig::default();

    for _ in 0..ITERATIONS {
        let _ = black_box(parse(black_box(&large_diff), &config));
    }
}

fn profile_render() {
    let large_diff = generate_large_diff(10, 50);
    let parsed = parse(&large_diff, &DiffParserConfig::default());

    let line_by_line_config = Diff2HtmlConfig {
        output_format: OutputFormat::LineByLine,
        draw_file_list: false,
        ..Default::default()
    };

    let side_by_side_config = Diff2HtmlConfig {
        output_format: OutputFormat::SideBySide,
        draw_file_list: false,
        ..Default::default()
    };

    for _ in 0..ITERATIONS {
        let _ = black_box(html_from_diff_files(
            black_box(&parsed),
            &line_by_line_config,
        ));
        let _ = black_box(html_from_diff_files(
            black_box(&parsed),
            &side_by_side_config,
        ));
    }
}

fn profile_render_with_matching() {
    let large_diff = generate_large_diff(10, 50);
    let parsed = parse(&large_diff, &DiffParserConfig::default());

    let words_matching_config = Diff2HtmlConfig {
        matching: LineMatchingType::Words,
        output_format: OutputFormat::SideBySide,
        draw_file_list: false,
        ..Default::default()
    };

    for _ in 0..ITERATIONS {
        let _ = black_box(html_from_diff_files(
            black_box(&parsed),
            &words_matching_config,
        ));
    }
}

fn profile_levenshtein() {
    let long_a = "a".repeat(100);
    let long_b = "b".repeat(100);

    for _ in 0..ITERATIONS * 10 {
        let _ = black_box(levenshtein(black_box(&long_a), black_box(&long_b)));
    }
}

fn profile_string_distance() {
    let a = "the quick brown fox jumps over the lazy dog";
    let b = "the quick brown cat jumps over the lazy dog";

    for _ in 0..ITERATIONS * 100 {
        let _ = black_box(string_distance(black_box(a), black_box(b)));
    }
}

fn profile_match_lines() {
    let many_deleted: Vec<String> = (0..20)
        .map(|i| format!("deleted line number {i}"))
        .collect();
    let many_inserted: Vec<String> = (0..20)
        .map(|i| format!("inserted line number {i}"))
        .collect();

    for _ in 0..ITERATIONS {
        let _ = black_box(match_lines(
            black_box(&many_deleted),
            black_box(&many_inserted),
            |a: &String, b: &String| string_distance(a, b),
        ));
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    match mode {
        "parse" => {
            eprintln!("Profiling: parse ({} iterations)", ITERATIONS);
            profile_parse();
        }
        "render" => {
            eprintln!("Profiling: render ({} iterations)", ITERATIONS);
            profile_render();
        }
        "render_matching" => {
            eprintln!(
                "Profiling: render with line matching ({} iterations)",
                ITERATIONS
            );
            profile_render_with_matching();
        }
        "levenshtein" => {
            eprintln!("Profiling: levenshtein ({} iterations)", ITERATIONS * 10);
            profile_levenshtein();
        }
        "string_distance" => {
            eprintln!(
                "Profiling: string_distance ({} iterations)",
                ITERATIONS * 100
            );
            profile_string_distance();
        }
        "match_lines" => {
            eprintln!("Profiling: match_lines ({} iterations)", ITERATIONS);
            profile_match_lines();
        }
        "all" => {
            eprintln!("Profiling: all functions");
            profile_parse();
            profile_render();
            profile_render_with_matching();
            profile_levenshtein();
            profile_string_distance();
            profile_match_lines();
        }
        _ => {
            eprintln!("Unknown mode '{}', running all functions", mode);
            profile_parse();
            profile_render();
            profile_render_with_matching();
            profile_levenshtein();
            profile_string_distance();
            profile_match_lines();
        }
    }
    eprintln!("Done");
}
