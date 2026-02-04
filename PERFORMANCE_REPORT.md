# Performance Analysis Report for diff2html-rs

## Executive Summary

This report presents profiling results for the diff2html crate, identifying performance bottlenecks and suggesting optimizations. Profiling was performed using Valgrind's callgrind tool on 1000 iterations of each major operation.

## Profiling Results Overview

| Operation | Instructions (1000 iter) | Relative Cost |
|-----------|-------------------------|---------------|
| Parse | 1.38 billion | 1x (baseline) |
| Levenshtein (10K iter) | 4.22 billion | 0.3x per iter |
| Match Lines | 33.4 billion | 24x |
| Render (LBL + SBS) | 85.5 billion | 62x |
| Render w/ Matching | 46.4 billion | 34x |

**Key Finding**: HTML rendering is the dominant cost, consuming 62x more CPU cycles than parsing for the same input.

---

## Detailed Analysis

### 1. Parsing (`parser.rs`)

**Top Hotspots:**
| Function | % of Time | Description |
|----------|-----------|-------------|
| `str::replace` | 9.64% | String replacement operations |
| `_int_malloc` | 9.25% | Memory allocation |
| `regex_automata::backtrack` | 8.66% | Regex matching |
| `_int_free` | 5.59% | Memory deallocation |
| `memchr_aligned` | 5.09% | String searching |
| `__memcpy_avx` | 4.31% | Memory copying |

**Problem Areas:**

1. **Excessive String Allocations** (~20% malloc/free)
   - Location: `parser.rs` throughout
   - The parser creates many temporary strings during line processing

2. **Regex Backtracking** (~9%)
   - Location: `parser.rs:44-102` (LazyLock regex patterns)
   - Some patterns use the slower backtracking engine rather than DFA

3. **String Replace Operations** (~10%)
   - Location: Likely in `get_filename()` and line processing
   - Each replace creates a new String allocation

**Optimization Suggestions:**

```rust
// BEFORE: Creates allocations on each call
fn get_filename(line: &str, ...) -> String {
    // ... regex matches and string operations
}

// AFTER: Consider using Cow<str> to avoid allocations when no changes needed
fn get_filename(line: &str, ...) -> Cow<'_, str> {
    // Return borrowed str when possible
}
```

- **Use `regex::bytes` for hot paths** - byte-based matching can be faster
- **Pre-compile and cache regex patterns** - already done with LazyLock, but consider using `RegexSet` for multiple pattern matching
- **Use `memchr` crate directly** for simple pattern searches instead of regex
- **Consider using `bstr` crate** for more efficient byte string operations

---

### 2. Levenshtein Distance (`rematch.rs:36-76`)

**Top Hotspots:**
| Function | % of Time | Description |
|----------|-----------|-------------|
| `slice::index` | 28.67% | Array indexing bounds checks |
| `levenshtein` (core) | 26.13% | Main algorithm |
| `cmp` | 14.30% | Comparisons |
| `ops::range` | 9.53% | Range iteration |
| `vec::mod` | 4.74% | Vector operations |

**Problem Areas:**

1. **2D Matrix Allocation** (~10%)
   - Location: `rematch.rs:50`
   ```rust
   let mut matrix: Vec<Vec<usize>> = vec![vec![0; a_len + 1]; b_len + 1];
   ```
   - Creates `b_len + 1` separate heap allocations

2. **Bounds Checking Overhead** (~29%)
   - Every `matrix[i][j]` access performs bounds checking

3. **Character Collection** (~5%)
   - Location: `rematch.rs:44-45`
   ```rust
   let a_chars: Vec<char> = a.chars().collect();
   let b_chars: Vec<char> = b.chars().collect();
   ```

**Optimization Suggestions:**

```rust
// OPTIMIZATION 1: Use single allocation with flat indexing
pub fn levenshtein_optimized(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // Single allocation instead of Vec<Vec<>>
    let mut matrix = vec![0usize; (a_len + 1) * (b_len + 1)];
    let width = a_len + 1;

    // Use get_unchecked for inner loop (after validating indices)
    // ... implementation with flat indexing
}

// OPTIMIZATION 2: Two-row algorithm (O(min(m,n)) space)
pub fn levenshtein_two_row(a: &str, b: &str) -> usize {
    // Only keep two rows at a time
    let mut prev_row = vec![0; a_len + 1];
    let mut curr_row = vec![0; a_len + 1];
    // ... swap rows each iteration
}

// OPTIMIZATION 3: Use the `strsim` or `triple_accel` crate
// which has SIMD-optimized Levenshtein implementations
```

- **Use flat 1D array** instead of `Vec<Vec<>>` - eliminates nested allocations
- **Two-row algorithm** - reduces space from O(mn) to O(min(m,n))
- **Consider SIMD** - the `triple_accel` crate provides AVX2-optimized edit distance
- **Early termination** - if distance exceeds a threshold, return early

---

### 3. Line Matching (`rematch.rs:199-261`)

**Top Hotspots:**
| Function | % of Time | Description |
|----------|-----------|-------------|
| `levenshtein` (total) | ~50% | String distance calculations |
| `_int_malloc` | 9.84% | Memory allocation |
| `_int_free` | 5.44% | Memory deallocation |
| `HashMap` operations | ~5% | Distance cache |

**Problem Areas:**

1. **Quadratic Distance Calculations**
   - Location: `rematch.rs:147-163` (`find_best_match`)
   - For n deleted and m inserted lines: O(n*m) distance calculations
   - Each call to `levenshtein` is O(len_a * len_b)

2. **Cache Inefficiency**
   - Location: `rematch.rs:239-240`
   - New HashMaps created for each recursive call
   - Cache keys don't survive across recursive splits

3. **Recursive Algorithm Overhead**
   - Location: `rematch.rs:209-261`
   - Deep recursion with vector cloning at each level

**Optimization Suggestions:**

```rust
// OPTIMIZATION 1: Pre-compute all distances once
fn match_lines_optimized<T, F>(a: &[T], b: &[T], distance: F) -> Vec<MatchGroup<T>> {
    // Compute full distance matrix upfront
    let mut distances = vec![0.0f64; a.len() * b.len()];
    for (i, item_a) in a.iter().enumerate() {
        for (j, item_b) in b.iter().enumerate() {
            distances[i * b.len() + j] = distance(item_a, item_b);
        }
    }
    // Use indices instead of slices for recursion
    // ...
}

// OPTIMIZATION 2: Use Hungarian algorithm for optimal matching
// The current greedy approach may not find optimal matching anyway

// OPTIMIZATION 3: Iterative instead of recursive
fn match_lines_iterative<T, F>(...) -> Vec<MatchGroup<T>> {
    // Use explicit stack instead of recursion
    // Avoid cloning vectors
}
```

- **Pre-compute distance matrix** - calculate all distances once instead of on-demand
- **Use indices instead of slices** - avoid cloning in recursion
- **Consider iterative approach** - eliminates recursion overhead
- **Threshold-based filtering** - skip distance calculation if lines are too different

---

### 4. HTML Rendering (`render/`, `templates.rs`)

**Top Hotspots:**
| Function | % of Time | Description |
|----------|-----------|-------------|
| `__memcpy_avx` | 11.19% | Memory copying |
| `_int_free` | 11.01% | Memory deallocation |
| `_int_malloc` | 7.86% | Memory allocation |
| `malloc` | 7.78% | Memory allocation |
| `free` | 5.00% | Memory deallocation |
| `Handlebars::render` | 1.67% | Template rendering |
| `SipHash::write` | 1.64% | Hashing operations |
| `Context::navigate` | 1.50% | Template context lookup |
| `BTreeMap::insert` | 1.18% | Map operations |

**Problem Areas:**

1. **Memory Churn** (~43% malloc/free/memcpy)
   - Dominated by string allocations during HTML generation
   - Each template render creates many intermediate strings

2. **Handlebars Template Overhead** (~8%)
   - Context navigation and parameter expansion
   - BTreeMap operations for data lookup
   - SipHash for HashMap keys

3. **String Concatenation**
   - Location: Throughout render modules
   - HTML output built incrementally

**Optimization Suggestions:**

```rust
// OPTIMIZATION 1: Pre-size output buffer
fn render_html(diff_files: &[DiffFile]) -> String {
    // Estimate output size based on input
    let estimated_size = estimate_html_size(diff_files);
    let mut output = String::with_capacity(estimated_size);
    // ...
}

// OPTIMIZATION 2: Consider alternative template engines
// - `askama` - compile-time templates, zero-cost abstractions
// - `maud` - compile-time HTML generation
// - Direct string building for simple cases

// OPTIMIZATION 3: Reduce data cloning for templates
// Instead of cloning DiffFile into serde_json::Value,
// implement custom serialization that borrows data
```

- **Pre-allocate output buffers** - estimate final HTML size
- **Consider `askama` or `maud`** - compile-time template engines with less runtime overhead
- **Reduce JSON serialization** - Handlebars requires serde_json::Value, which clones data
- **Batch small writes** - accumulate content before writing
- **Use `Cow<str>`** - avoid cloning when borrowing is possible

---

### 5. HTML Escaping (`render/utils.rs`)

**Current Cost:** 0.33% of rendering time

The `escape_for_html` function is called frequently but is relatively efficient. However, it could be optimized:

```rust
// Consider using the `v_htmlescape` crate for SIMD-accelerated HTML escaping
// Or `askama_escape` which is highly optimized
```

---

## Priority Optimization Recommendations

### High Impact (Recommended)

1. **Optimize Levenshtein Algorithm**
   - Use two-row algorithm to reduce memory
   - Use flat array instead of Vec<Vec<>>
   - Consider `strsim` crate with SIMD support
   - **Expected improvement:** 2-3x faster string distance

2. **Reduce Memory Allocations in Rendering**
   - Pre-size output buffers
   - Use `Cow<str>` where possible
   - Consider template engine alternatives
   - **Expected improvement:** 20-30% faster rendering

3. **Pre-compute Line Distance Matrix**
   - Calculate all distances upfront in match_lines
   - Use indices instead of slice cloning
   - **Expected improvement:** 30-50% faster line matching

### Medium Impact

4. **Optimize Parser String Operations**
   - Use `Cow<str>` return types
   - Replace simple regex with string methods
   - **Expected improvement:** 10-20% faster parsing

5. **Template Engine Evaluation**
   - Benchmark `askama` vs `handlebars`
   - **Expected improvement:** Potentially 2-5x faster rendering

### Low Impact (Nice to Have)

6. **SIMD HTML Escaping**
   - Use `v_htmlescape` crate
   - **Expected improvement:** Marginal

---

## Benchmarking Commands

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench -- parse/
cargo bench -- html_render/
cargo bench -- levenshtein/

# Profile with callgrind
cargo build --release --example profile
valgrind --tool=callgrind ./target/release/examples/profile [parse|render|levenshtein|all]
callgrind_annotate callgrind.out.* | head -100
```

---

## Appendix: Raw Profiling Data

### Instruction Counts (1000 iterations unless noted)

- **Parse:** 1,383,739,818 instructions
- **Render (line-by-line + side-by-side):** 85,506,995,618 instructions
- **Render with line matching:** 46,374,454,415 instructions
- **Levenshtein (10,000 iterations):** 4,216,768,743 instructions
- **Match Lines:** 33,435,606,202 instructions

### Test Configuration

- Input: Generated diff with 10 files, 50 lines each
- Lines contain alternating context/changed content
- Profile tool: Valgrind callgrind 3.22.0
- Build: Release mode with debug symbols
