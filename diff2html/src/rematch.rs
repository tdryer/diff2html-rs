//! Line matching algorithm for pairing similar changed lines.
//!
//! This module implements Levenshtein-based similarity matching to pair up
//! deleted and inserted lines that are similar to each other, improving
//! the visual diff presentation in side-by-side views.
//!
//! The algorithm recursively finds the best matching pair of lines,
//! splits the sequences around that match, and continues matching
//! the remaining segments.

use std::collections::HashMap;

/// Result of finding the best match between two sequences.
#[derive(Debug, Clone, PartialEq)]
pub struct BestMatch {
    pub index_a: usize,
    pub index_b: usize,
    pub score: f64,
}

/// Calculate the Levenshtein distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to change one string
/// into the other.
///
/// # Examples
///
/// ```
/// use diff2html::rematch::levenshtein;
///
/// assert_eq!(levenshtein("kitten", "sitting"), 3);
/// assert_eq!(levenshtein("", "abc"), 3);
/// assert_eq!(levenshtein("abc", "abc"), 0);
/// ```
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a.is_empty() {
        return b.chars().count();
    }
    if b.is_empty() {
        return a.chars().count();
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();

    // Two-row algorithm: only keep track of two rows at a time
    // v0 is the previous row, v1 is the current row being computed
    let mut v0: Vec<usize> = (0..=a_len).collect();
    let mut v1: Vec<usize> = vec![0; a_len + 1];

    for (i, b_char) in b_chars.iter().enumerate() {
        // First element of v1 is the edit distance from b[0..=i] to empty string
        v1[0] = i + 1;

        for (j, a_char) in a_chars.iter().enumerate() {
            let deletion_cost = v0[j + 1] + 1;
            let insertion_cost = v1[j] + 1;
            let substitution_cost = if b_char == a_char { v0[j] } else { v0[j] + 1 };

            v1[j + 1] = deletion_cost.min(insertion_cost).min(substitution_cost);
        }

        // Swap v0 and v1 for the next iteration
        std::mem::swap(&mut v0, &mut v1);
    }

    // After the last swap, the result is in v0
    v0[a_len]
}

/// A function that computes normalized distance between two items.
pub type DistanceFn<T> = fn(&T, &T) -> f64;

/// Create a distance function from a string extractor.
///
/// The returned function computes a normalized Levenshtein distance between
/// two items by extracting strings from them and dividing the raw distance
/// by the sum of the string lengths.
///
/// # Arguments
///
/// * `str_fn` - A function that extracts a string from an item
///
/// # Returns
///
/// A normalized distance in the range [0.0, 1.0], where 0.0 means identical
/// strings and values approaching 1.0 mean very different strings.
pub fn new_distance_fn<T, F>(str_fn: F) -> impl Fn(&T, &T) -> f64
where
    F: Fn(&T) -> String,
{
    move |x: &T, y: &T| {
        let x_value = str_fn(x).trim().to_string();
        let y_value = str_fn(y).trim().to_string();
        let total_len = x_value.chars().count() + y_value.chars().count();

        if total_len == 0 {
            return 0.0;
        }

        let lev = levenshtein(&x_value, &y_value);
        lev as f64 / total_len as f64
    }
}

/// Create a string-based distance function for DiffLine content.
///
/// This is a convenience function that extracts the content from diff lines
/// and computes their normalized Levenshtein distance.
pub fn string_distance(a: &str, b: &str) -> f64 {
    let a_trimmed = a.trim();
    let b_trimmed = b.trim();
    let total_len = a_trimmed.chars().count() + b_trimmed.chars().count();

    if total_len == 0 {
        return 0.0;
    }

    let lev = levenshtein(a_trimmed, b_trimmed);
    lev as f64 / total_len as f64
}

/// A matched group of elements from sequences A and B.
pub type MatchGroup<T> = (Vec<T>, Vec<T>);

/// Find the best matching pair between two sequences.
fn find_best_match<T, F>(
    a: &[T],
    b: &[T],
    distance: &F,
    cache: &mut HashMap<(usize, usize), f64>,
) -> Option<BestMatch>
where
    T: Clone,
    F: Fn(&T, &T) -> f64,
{
    let mut best_match_dist = f64::INFINITY;
    let mut best_match: Option<BestMatch> = None;

    for (i, item_a) in a.iter().enumerate() {
        for (j, item_b) in b.iter().enumerate() {
            let cache_key = (i, j);
            let md = *cache
                .entry(cache_key)
                .or_insert_with(|| distance(item_a, item_b));

            if md < best_match_dist {
                best_match_dist = md;
                best_match = Some(BestMatch {
                    index_a: i,
                    index_b: j,
                    score: best_match_dist,
                });
            }
        }
    }

    best_match
}

/// Group elements from two sequences by matching similar items.
///
/// This function recursively finds the best matching pair of elements,
/// splits the sequences around that match, and continues matching
/// the remaining segments. The result is a list of paired groups
/// where similar items are aligned.
///
/// # Arguments
///
/// * `a` - First sequence of elements
/// * `b` - Second sequence of elements
/// * `distance` - Function to compute distance between elements
///
/// # Returns
///
/// A vector of tuples, where each tuple contains the matched elements
/// from sequence A and sequence B respectively.
///
/// # Examples
///
/// ```
/// use diff2html::rematch::{match_lines, string_distance};
///
/// let old_lines = vec!["hello world", "foo bar"];
/// let new_lines = vec!["hello universe", "baz qux"];
///
/// let groups = match_lines(&old_lines, &new_lines, |a, b| string_distance(a, b));
///
/// // The algorithm pairs similar lines together
/// assert_eq!(groups.len(), 2);
/// ```
pub fn match_lines<T, F>(a: &[T], b: &[T], distance: F) -> Vec<MatchGroup<T>>
where
    T: Clone,
    F: Fn(&T, &T) -> f64,
{
    let mut cache: HashMap<(usize, usize), f64> = HashMap::new();
    group_recursive(a, b, &distance, &mut cache)
}

/// Internal recursive grouping function.
fn group_recursive<T, F>(
    a: &[T],
    b: &[T],
    distance: &F,
    cache: &mut HashMap<(usize, usize), f64>,
) -> Vec<MatchGroup<T>>
where
    T: Clone,
    F: Fn(&T, &T) -> f64,
{
    let bm = find_best_match(a, b, distance, cache);

    // Base case: if no match found or sequences are too small to split
    if bm.is_none() || a.len() + b.len() < 3 {
        return vec![(a.to_vec(), b.to_vec())];
    }

    let bm = bm.unwrap();

    // Split sequences around the best match
    let a1 = &a[..bm.index_a];
    let b1 = &b[..bm.index_b];
    let a_match = vec![a[bm.index_a].clone()];
    let b_match = vec![b[bm.index_b].clone()];
    let tail_a = bm.index_a + 1;
    let tail_b = bm.index_b + 1;
    let a2 = &a[tail_a..];
    let b2 = &b[tail_b..];

    // Create new cache for sub-problems (indices change after splitting)
    let mut group1_cache: HashMap<(usize, usize), f64> = HashMap::new();
    let mut group2_cache: HashMap<(usize, usize), f64> = HashMap::new();

    // Recursively match the sub-sequences
    let group1 = group_recursive(a1, b1, distance, &mut group1_cache);
    let group_match = vec![(a_match, b_match)];
    let group2 = group_recursive(a2, b2, distance, &mut group2_cache);

    // Combine results
    let mut result = group_match;

    if bm.index_a > 0 || bm.index_b > 0 {
        let mut combined = group1;
        combined.extend(result);
        result = combined;
    }

    if a.len() > tail_a || b.len() > tail_b {
        result.extend(group2);
    }

    result
}

/// Configuration for line matching behavior.
#[derive(Debug, Clone)]
pub struct MatchConfig {
    /// Maximum number of comparisons before giving up on matching.
    /// This prevents performance issues with very large diffs.
    pub max_comparisons: usize,

    /// Maximum line size to consider for matching.
    /// Lines longer than this are not matched to avoid expensive comparisons.
    pub max_line_size: usize,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            max_comparisons: 2500,
            max_line_size: 200,
        }
    }
}

/// Match lines with configurable limits.
///
/// This is a wrapper around `match_lines` that respects configuration
/// limits for performance reasons. If the number of potential comparisons
/// exceeds `max_comparisons`, or if any line exceeds `max_line_size`,
/// no matching is performed and lines are returned as unmatched groups.
pub fn match_lines_with_config<T, F>(
    a: &[T],
    b: &[T],
    distance: F,
    config: &MatchConfig,
    get_content: impl Fn(&T) -> &str,
) -> Vec<MatchGroup<T>>
where
    T: Clone,
    F: Fn(&T, &T) -> f64,
{
    // Check if matching would be too expensive
    if a.len() * b.len() > config.max_comparisons {
        return vec![(a.to_vec(), b.to_vec())];
    }

    // Check if any line is too long
    let any_too_long = a
        .iter()
        .any(|line| get_content(line).len() > config.max_line_size)
        || b.iter()
            .any(|line| get_content(line).len() > config.max_line_size);

    if any_too_long {
        return vec![(a.to_vec(), b.to_vec())];
    }

    match_lines(a, b, distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
    }

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("hello world", "hello world"), 0);
    }

    #[test]
    fn test_levenshtein_single_edit() {
        assert_eq!(levenshtein("cat", "hat"), 1); // substitution
        assert_eq!(levenshtein("cat", "cats"), 1); // insertion
        assert_eq!(levenshtein("cats", "cat"), 1); // deletion
    }

    #[test]
    fn test_levenshtein_multiple_edits() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("saturday", "sunday"), 3);
    }

    #[test]
    fn test_levenshtein_unicode() {
        assert_eq!(levenshtein("日本", "日本語"), 1);
        assert_eq!(levenshtein("hello", "héllo"), 1);
    }

    #[test]
    fn test_string_distance_empty() {
        assert_eq!(string_distance("", ""), 0.0);
    }

    #[test]
    fn test_string_distance_identical() {
        assert_eq!(string_distance("hello", "hello"), 0.0);
    }

    #[test]
    fn test_string_distance_completely_different() {
        // "abc" vs "xyz" = 3 edits, total length = 6, distance = 0.5
        assert_eq!(string_distance("abc", "xyz"), 0.5);
    }

    #[test]
    fn test_string_distance_partial_match() {
        // "hello" vs "hallo" = 1 edit, total length = 10, distance = 0.1
        assert_eq!(string_distance("hello", "hallo"), 0.1);
    }

    #[test]
    fn test_string_distance_with_whitespace() {
        // Trimming should be applied
        assert_eq!(string_distance("  hello  ", "hello"), 0.0);
    }

    #[test]
    fn test_new_distance_fn() {
        let distance = new_distance_fn(|s: &String| s.clone());
        let a = "hello".to_string();
        let b = "hello".to_string();
        assert_eq!(distance(&a, &b), 0.0);

        let c = "world".to_string();
        assert!(distance(&a, &c) > 0.0);
    }

    #[test]
    fn test_match_lines_empty() {
        let a: Vec<&str> = vec![];
        let b: Vec<&str> = vec![];
        let groups = match_lines(&a, &b, |x, y| string_distance(x, y));
        assert_eq!(groups.len(), 1);
        assert!(groups[0].0.is_empty());
        assert!(groups[0].1.is_empty());
    }

    #[test]
    fn test_match_lines_single_pair() {
        let a = vec!["hello world"];
        let b = vec!["hello universe"];
        let groups = match_lines(&a, &b, |x, y| string_distance(x, y));

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, vec!["hello world"]);
        assert_eq!(groups[0].1, vec!["hello universe"]);
    }

    #[test]
    fn test_match_lines_multiple_pairs() {
        let a = vec!["apple", "banana"];
        let b = vec!["apples", "bananas"];
        let groups = match_lines(&a, &b, |x, y| string_distance(x, y));

        // Should match similar lines together
        // The exact grouping depends on the algorithm's choices
        assert!(!groups.is_empty());
    }

    #[test]
    fn test_match_lines_unequal_lengths() {
        let a = vec!["line1", "line2", "line3"];
        let b = vec!["line1 modified"];
        let groups = match_lines(&a, &b, |x, y| string_distance(x, y));

        // Should still produce valid groupings
        let total_a: usize = groups.iter().map(|(ga, _)| ga.len()).sum();
        let total_b: usize = groups.iter().map(|(_, gb)| gb.len()).sum();
        assert_eq!(total_a, 3);
        assert_eq!(total_b, 1);
    }

    #[test]
    fn test_match_lines_preserves_order() {
        let a = vec!["first", "second", "third"];
        let b = vec!["1st", "2nd", "3rd"];
        let groups = match_lines(&a, &b, |x, y| string_distance(x, y));

        // Flatten the groups and verify all elements are present
        let flat_a: Vec<&str> = groups
            .iter()
            .flat_map(|(ga, _)| ga.iter().copied())
            .collect();
        let flat_b: Vec<&str> = groups
            .iter()
            .flat_map(|(_, gb)| gb.iter().copied())
            .collect();

        assert_eq!(flat_a, vec!["first", "second", "third"]);
        assert_eq!(flat_b, vec!["1st", "2nd", "3rd"]);
    }

    #[test]
    fn test_match_config_default() {
        let config = MatchConfig::default();
        assert_eq!(config.max_comparisons, 2500);
        assert_eq!(config.max_line_size, 200);
    }

    #[test]
    fn test_match_lines_with_config_exceeds_comparisons() {
        let config = MatchConfig {
            max_comparisons: 1, // Very low limit
            max_line_size: 200,
        };

        let a = vec!["line1", "line2"];
        let b = vec!["line3", "line4"];
        let groups = match_lines_with_config(&a, &b, |x, y| string_distance(x, y), &config, |s| s);

        // Should return single unmatched group due to comparison limit
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0.len(), 2);
        assert_eq!(groups[0].1.len(), 2);
    }

    #[test]
    fn test_match_lines_with_config_line_too_long() {
        let config = MatchConfig {
            max_comparisons: 2500,
            max_line_size: 5, // Very low limit
        };

        let a = vec!["short", "this is a longer line"];
        let b = vec!["short", "another long line here"];
        let groups = match_lines_with_config(&a, &b, |x, y| string_distance(x, y), &config, |s| s);

        // Should return single unmatched group due to line size limit
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_find_best_match_basic() {
        let a = vec!["apple", "banana"];
        let b = vec!["apricot", "berry"];
        let mut cache = HashMap::new();

        let result = find_best_match(
            &a,
            &b,
            &|x: &&str, y: &&str| string_distance(x, y),
            &mut cache,
        );

        assert!(result.is_some());
        let bm = result.unwrap();
        // "apple" should match better with "apricot" (both start with 'ap')
        assert_eq!(bm.index_a, 0);
        assert_eq!(bm.index_b, 0);
    }

    #[test]
    fn test_find_best_match_empty_sequences() {
        let a: Vec<&str> = vec![];
        let b: Vec<&str> = vec![];
        let mut cache = HashMap::new();

        let result = find_best_match(
            &a,
            &b,
            &|x: &&str, y: &&str| string_distance(x, y),
            &mut cache,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_match_lines_real_diff_scenario() {
        // Simulate a real diff scenario where lines are modified
        let old_lines = vec!["function calculate(x) {", "    return x * 2;", "}"];
        let new_lines = vec!["function calculate(x, y) {", "    return x * y;", "}"];

        let groups = match_lines(&old_lines, &new_lines, |x, y| string_distance(x, y));

        // All lines should be matched since they're similar
        let total_old: usize = groups.iter().map(|(a, _)| a.len()).sum();
        let total_new: usize = groups.iter().map(|(_, b)| b.len()).sum();

        assert_eq!(total_old, 3);
        assert_eq!(total_new, 3);
    }
}
