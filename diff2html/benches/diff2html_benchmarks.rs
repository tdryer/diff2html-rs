use criterion::{Criterion, black_box, criterion_group, criterion_main};
use diff2html::{
    Diff2HtmlConfig, DiffParserConfig, LineMatchingType, OutputFormat, html, html_from_diff_files,
    json, levenshtein, match_lines, parse, string_distance,
};

// Simple single-file diff
const SIMPLE_DIFF: &str = r#"diff --git a/sample b/sample
index 0000001..0ddf2ba
--- a/sample
+++ b/sample
@@ -1 +1 @@
-test
+test1r
"#;

// Multi-file diff
const MULTI_FILE_DIFF: &str = r#"diff --git a/src/core/init.js b/src/core/init.js
index e49196a..50f310c 100644
--- a/src/core/init.js
+++ b/src/core/init.js
@@ -101,7 +101,7 @@ var rootjQuery,
     // HANDLE: $(function)
     // Shortcut for document ready
     } else if ( jQuery.isFunction( selector ) ) {
-      return typeof rootjQuery.ready !== "undefined" ?
+      return rootjQuery.ready !== undefined ?
         rootjQuery.ready( selector ) :
         // Execute immediately if ready is not present
         selector( jQuery );
diff --git a/src/event.js b/src/event.js
index 7336f4d..6183f70 100644
--- a/src/event.js
+++ b/src/event.js
@@ -1,6 +1,5 @@
 define([
   "./core",
-  "./var/strundefined",
   "./var/rnotwhite",
   "./var/hasOwn",
   "./var/slice",
"#;

// Diff with multiple blocks
const MULTI_BLOCK_DIFF: &str = r#"diff --git a/src/attributes/classes.js b/src/attributes/classes.js
index c617824..c8d1393 100644
--- a/src/attributes/classes.js
+++ b/src/attributes/classes.js
@@ -1,10 +1,9 @@
 define([
   "../core",
   "../var/rnotwhite",
-  "../var/strundefined",
   "../data/var/dataPriv",
   "../core/init"
-], function( jQuery, rnotwhite, strundefined, dataPriv ) {
+], function( jQuery, rnotwhite, dataPriv ) {

 var rclass = /[\t\r\n\f]/g;

@@ -128,7 +127,7 @@ jQuery.fn.extend({
         }

       // Toggle whole class name
-      } else if ( type === strundefined || type === "boolean" ) {
+      } else if ( value === undefined || type === "boolean" ) {
         if ( this.className ) {
           // store className if set
           dataPriv.set( this, "__className__", this.className );
"#;

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

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    group.bench_function("simple_diff", |b| {
        b.iter(|| parse(black_box(SIMPLE_DIFF), &DiffParserConfig::default()))
    });

    group.bench_function("multi_file_diff", |b| {
        b.iter(|| parse(black_box(MULTI_FILE_DIFF), &DiffParserConfig::default()))
    });

    group.bench_function("multi_block_diff", |b| {
        b.iter(|| parse(black_box(MULTI_BLOCK_DIFF), &DiffParserConfig::default()))
    });

    let large_diff = generate_large_diff(10, 50);
    group.bench_function("large_diff_10_files", |b| {
        b.iter(|| parse(black_box(&large_diff), &DiffParserConfig::default()))
    });

    group.finish();
}

fn bench_html_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_render");

    // Pre-parse the diffs to isolate rendering performance
    let simple_parsed = parse(SIMPLE_DIFF, &DiffParserConfig::default());
    let multi_file_parsed = parse(MULTI_FILE_DIFF, &DiffParserConfig::default());
    let large_diff = generate_large_diff(10, 50);
    let large_parsed = parse(&large_diff, &DiffParserConfig::default());

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

    group.bench_function("simple_line_by_line", |b| {
        b.iter(|| html_from_diff_files(black_box(&simple_parsed), &line_by_line_config))
    });

    group.bench_function("simple_side_by_side", |b| {
        b.iter(|| html_from_diff_files(black_box(&simple_parsed), &side_by_side_config))
    });

    group.bench_function("multi_file_line_by_line", |b| {
        b.iter(|| html_from_diff_files(black_box(&multi_file_parsed), &line_by_line_config))
    });

    group.bench_function("multi_file_side_by_side", |b| {
        b.iter(|| html_from_diff_files(black_box(&multi_file_parsed), &side_by_side_config))
    });

    group.bench_function("large_line_by_line", |b| {
        b.iter(|| html_from_diff_files(black_box(&large_parsed), &line_by_line_config))
    });

    group.bench_function("large_side_by_side", |b| {
        b.iter(|| html_from_diff_files(black_box(&large_parsed), &side_by_side_config))
    });

    group.finish();
}

fn bench_html_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_end_to_end");

    let config = Diff2HtmlConfig::default();

    group.bench_function("simple_diff", |b| {
        b.iter(|| html(black_box(SIMPLE_DIFF), &config))
    });

    group.bench_function("multi_file_diff", |b| {
        b.iter(|| html(black_box(MULTI_FILE_DIFF), &config))
    });

    let large_diff = generate_large_diff(10, 50);
    group.bench_function("large_diff", |b| {
        b.iter(|| html(black_box(&large_diff), &config))
    });

    group.finish();
}

fn bench_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("json");

    let config = Diff2HtmlConfig::default();

    group.bench_function("simple_diff", |b| {
        b.iter(|| json(black_box(SIMPLE_DIFF), &config))
    });

    group.bench_function("multi_file_diff", |b| {
        b.iter(|| json(black_box(MULTI_FILE_DIFF), &config))
    });

    let large_diff = generate_large_diff(10, 50);
    group.bench_function("large_diff", |b| {
        b.iter(|| json(black_box(&large_diff), &config))
    });

    group.finish();
}

fn bench_levenshtein(c: &mut Criterion) {
    let mut group = c.benchmark_group("levenshtein");

    group.bench_function("short_strings_similar", |b| {
        b.iter(|| levenshtein(black_box("hello"), black_box("hallo")))
    });

    group.bench_function("short_strings_different", |b| {
        b.iter(|| levenshtein(black_box("hello"), black_box("world")))
    });

    group.bench_function("medium_strings", |b| {
        b.iter(|| {
            levenshtein(
                black_box("the quick brown fox jumps over the lazy dog"),
                black_box("the quick brown cat jumps over the lazy dog"),
            )
        })
    });

    let long_a = "a".repeat(100);
    let long_b = "b".repeat(100);
    group.bench_function("long_strings_different", |b| {
        b.iter(|| levenshtein(black_box(&long_a), black_box(&long_b)))
    });

    let long_similar_a = "x".repeat(100);
    let mut long_similar_b = "x".repeat(99);
    long_similar_b.push('y');
    group.bench_function("long_strings_similar", |b| {
        b.iter(|| levenshtein(black_box(&long_similar_a), black_box(&long_similar_b)))
    });

    group.finish();
}

fn bench_string_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_distance");

    group.bench_function("short_strings", |b| {
        b.iter(|| string_distance(black_box("hello"), black_box("hallo")))
    });

    group.bench_function("medium_strings", |b| {
        b.iter(|| {
            string_distance(
                black_box("the quick brown fox jumps over the lazy dog"),
                black_box("the quick brown cat jumps over the lazy dog"),
            )
        })
    });

    group.finish();
}

fn bench_match_lines(c: &mut Criterion) {
    let mut group = c.benchmark_group("match_lines");

    let deleted_lines: Vec<&str> = vec![
        "old line one",
        "old line two",
        "old line three",
        "old line four",
    ];
    let inserted_lines: Vec<&str> = vec![
        "new line one",
        "new line two",
        "new line three",
        "new line four",
    ];

    group.bench_function("small_set", |b| {
        b.iter(|| {
            match_lines(
                black_box(&deleted_lines),
                black_box(&inserted_lines),
                |a: &&str, b: &&str| string_distance(a, b),
            )
        })
    });

    let many_deleted: Vec<String> = (0..20)
        .map(|i| format!("deleted line number {i}"))
        .collect();
    let many_inserted: Vec<String> = (0..20)
        .map(|i| format!("inserted line number {i}"))
        .collect();

    group.bench_function("medium_set", |b| {
        b.iter(|| {
            match_lines(
                black_box(&many_deleted),
                black_box(&many_inserted),
                |a: &String, b: &String| string_distance(a, b),
            )
        })
    });

    group.finish();
}

fn bench_line_matching_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_matching_render");

    let multi_file_parsed = parse(MULTI_FILE_DIFF, &DiffParserConfig::default());

    let no_matching_config = Diff2HtmlConfig {
        matching: LineMatchingType::None,
        ..Default::default()
    };

    let lines_matching_config = Diff2HtmlConfig {
        matching: LineMatchingType::Lines,
        ..Default::default()
    };

    let words_matching_config = Diff2HtmlConfig {
        matching: LineMatchingType::Words,
        ..Default::default()
    };

    group.bench_function("no_matching", |b| {
        b.iter(|| html_from_diff_files(black_box(&multi_file_parsed), &no_matching_config))
    });

    group.bench_function("lines_matching", |b| {
        b.iter(|| html_from_diff_files(black_box(&multi_file_parsed), &lines_matching_config))
    });

    group.bench_function("words_matching", |b| {
        b.iter(|| html_from_diff_files(black_box(&multi_file_parsed), &words_matching_config))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_html_render,
    bench_html_end_to_end,
    bench_json,
    bench_levenshtein,
    bench_string_distance,
    bench_match_lines,
    bench_line_matching_render,
);
criterion_main!(benches);
