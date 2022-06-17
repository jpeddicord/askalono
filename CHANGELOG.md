# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.4.6] - 2022-06-16

## Changed

- Updated SPDX dataset

## Fixed

- Fix silent failures in file I/O when using the CLI

## [0.4.5] - 2022-04-09

## Changed

- Updated SPDX dataset
- Bumped `zstd` up to 0.11; thanks [@mellowagain]

[0.4.5]: https://github.com/jpeddicord/askalono/releases/tag/0.4.4
[@mellowagain]: https://github.com/mellowagain

## [0.4.4] - 2021-11-21

### Changed

- Updated SPDX dataset
- Tons of code cleanup; thanks [@hdhoang]
- `zstd` update to 0.8; thanks [@decathorpe]
- Replace `failure` with `anyhow`; thanks [@hdhoang]

[0.4.4]: https://github.com/jpeddicord/askalono/releases/tag/0.4.4
[@hdhoang]: https://github.com/hdhoang
[@decathorpe]: https://github.com/decathorpe

## [0.4.3] - 2020-09-23

### Added

- Re-introduced gzip cache compression as an optional feature. The default is still zstd.
- `Store` gained `get_original` to fetch the canonical license text, instead of needing to access the text via a match.

### Changed

- Updated SPDX dataset

[0.4.3]: https://github.com/jpeddicord/askalono/releases/tag/0.4.3

## [0.4.2] - 2020-01-14

### Changed

- Performance improvements in text pre-processing ([#48], thanks [@AnthonyMikh])
- Updated SPDX dataset

[0.4.2]: https://github.com/jpeddicord/askalono/releases/tag/0.4.2
[#48]: https://github.com/jpeddicord/askalono/pull/48
[@AnthonyMikh]: https://github.com/AnthonyMikh

## [0.4.1] - 2019-12-06

### Fixed

- Removed some extraneous files from the core `askalono` packaged crate.
- Include LICENSE, NOTICE, and README.md in `askalono-cli` crate.

[0.4.1]: https://github.com/jpeddicord/askalono/releases/tag/0.4.1

## [0.4.0] - 2019-11-06

### Added

- askalono will attempt to ignore license "title lines" (e.g. "The MIT License") that occasionally appear on licenses.

### Changed

- (Breaking) Switch to Rust 2018 edition.
- (Breaking) Drop the previously-deprecated `aliases` field. Aliases can instead be queried with the `aliases` function on a `Store`.
- (Breaking) `analyze` (on `Store`) no longer returns a Result. There was no reasonable case where it could fail.
- (Breaking) `with_view`, `white_out`, `optimize_bounds` on `TextData` also no longer return Result, as they never had an expected failure path outside of programming errors. A panic may occur if an out-of-bounds view is used -- this is intentional. See commit 8d11161c.
- Stores are now compressed with zstd instead of gzip, which provides better compression and performance particularly for the dataset used by askalono.
- Fresh SPDX dataset.
- URLs in licenses will be "black-boxed" in case modified/re-hosted.

### Fixed

- The `lcs_removal` preprocessor has been fixed to be less aggressive on certain repeated statements ([#42]).
- Fixed CLI help text strings ([#34])

[0.4.0]: https://github.com/jpeddicord/askalono/releases/tag/0.4.0
[#42]: https://github.com/jpeddicord/askalono/issues/42
[#34]: https://github.com/jpeddicord/askalono/issues/34

## [0.3.0] - 2018-09-27

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

[0.3.0]: https://github.com/jpeddicord/askalono/releases/tag/0.3.0

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

[0.2.0]: https://github.com/jpeddicord/askalono/releases/tag/0.2.0

## [0.1.0] - 2018-01-31

- Initial release
- Non-existent documentation
- Bad tests
- It's fast, though

[0.1.0]: https://github.com/jpeddicord/askalono/releases/tag/0.1.0
