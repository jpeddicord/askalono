# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased

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

## [0.1.0] - 2018-01-31

- Initial release
- Non-existent documentation
- Bad tests
- It's fast, though


[0.1.0]: https://github.com/amzn/askalono/releases/tag/0.1.0

