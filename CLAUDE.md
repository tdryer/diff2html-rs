This is a Rust port of the JavaScript diff2html-cli tool library. See `PLAN.md` for the phased implementation plan.

Update `CLAUDE.md` with lessons learned.

Run `cargo clippy` and `cargo fmt` before committing.

## Lessons Learned

* If a struct contains `Box<dyn Fn(...) -> ...>`, it cannot derive `Clone` or `Debug`. Implement them manually instead.
* The types use `#[serde(rename_all = "camelCase")]` to match the JavaScript library's JSON output. This is important for compatibility.
* For enums that can be single or multiple values, use `#[serde(untagged)]`.
* Combined diffs (from merge commits) use `@@@` instead of `@@` for hunk headers. When writing tests for combined diffs, ensure the test input includes proper `--- ` and `+++ ` headers, otherwise the file won't be saved (requires `new_name` to be set).
* Use `once_cell::sync::Lazy` for regex patterns to avoid recompiling on every call.
* Deleted files have `+++ /dev/null`, new files have `--- /dev/null`.
* Unified diffs may have timestamps after filenames that need removal.
* Git uses `a/`, `b/` prefixes; also handle `i/`, `w/`, `c/`, `o/`.
* A file with no `new_name` won't be saved to the output list.
