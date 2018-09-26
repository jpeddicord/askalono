# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased

TBD. A 1.0 release is likely next, and will include full WebAssembly support.

## [0.3.0] - TBD

### Added

- Scanning strategies have been added in `ScanStrategy`. These include some common high-level logic that can be used for diving deeper into askalono's results.
- `Store` gained methods to add your own licenses to its dataset at runtime. These are `add_license` and `add_variant`.
- `Store` lets you retrieve and set the list of aliases for a given license.
- `Store` can also tell you which licenses are present with `licenses`.
- `TextData`'s `with_view` is now public. It can be used to change the region of lines in a text you're interested in without needing to wholly re-compute data for the underlying string.
- `TextData` also learned `white_out`, which is the inverse of `with_view`. It erases the active view's lines. This can be useful when a match has been found and you'd like to "hide" it to perform further analysis.
- The command-line application learned how to output JSON for better machine parsing.
- EXPERIMENTAL: askalono is WebAssembly-capable. Loading caches is not yet supported due to missing upstream dependencies. This likely to be available in the near future.

### Changed

- `optimize_bounds` now returns a Result. Previously it would have paniced if `self` was missing text.

### Deprecated

- The `aliases` field has been deprecated and will be removed in a future release. Prefer looking up aliases in the store when needed.

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


[0.3.0]: https://github.com/amzn/askalono/releases/tag/0.3.0
[0.2.0]: https://github.com/amzn/askalono/releases/tag/0.2.0
[0.1.0]: https://github.com/amzn/askalono/releases/tag/0.1.0
