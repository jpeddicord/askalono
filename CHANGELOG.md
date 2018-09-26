# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased

Planned: a requirement on Rust 1.26.

Many not-yet-documented changes. Coming soon.

### Fixed

- `optimize_bounds` now respects existing line views/windows on TextData structs.

## [0.2.0] - 2018-05-13

### Added

- Full documentation for the public API.
- The CLI can now read a list of filenames from stdin (batch mode).
- The CLI is capable of crawling a directory tree, looking for license-like files and identifying them.
- The library and CLI can now optimize a matched license to tell you _where_ in your source the match was identified.

### Changed

- Revised API. API changes are likely to be much more rare from here on out.
- Many performance improvements. `analyze` is now able to run in 3-5 ms on commodity hardware!
- Text storage and normalization improvements.
- Duplicate SPDX entries are now stored as aliases.
- SPDX definitions have been updated.

### Removed

- The "diff" option is now only available if compiled with the "diagnostics" feature (off by default). This was intended for debugging and had no practical use in the binary.

### Fixed

- Resolved a potential panic for short/empty license files (a divide-by-zero was involved).

## [0.1.0] - 2018-01-31

- Initial release
- Non-existent documentation
- Bad tests
- It's fast, though


[0.1.0]: https://github.com/amzn/askalono/releases/tag/0.1.0

