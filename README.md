# diff2html-rs

An experimental Rust port of the [diff2html](https://diff2html.xyz/) library and CLI tool for generating HTML from diffs, built using [Claude Code](https://claude.com/product/claude-code).

**This repository contains code that was primarily written by an AI coding agent.**

## Overview

This project provides two crates:

- [`diff2html`](diff2html/README.md) - Core library for parsing unified diffs and generating HTML
- [`diff2html-cli`](diff2html-cli/README.md) - Command-line tool for converting diffs to HTML

## Features

- Parse unified diff format (git diff, GNU diff)
- Generate HTML output in line-by-line or side-by-side views
- Word-level or character-level diff highlighting
- Light, dark, and auto color schemes
- Line matching algorithms for better change visualization
- JSON output for integration with other tools
- Support for binary files, renames, copies, and mode changes
- Combined diff support (merge commits)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

This is a port [diff2html](https://diff2html.xyz/) by [Rodrigo Fernandes](https://github.com/rtfpessoa):

- [diff2html](https://github.com/rtfpessoa/diff2html) - Original TypeScript library
- [diff2html-cli](https://github.com/rtfpessoa/diff2html-cli) - Original CLI tool
