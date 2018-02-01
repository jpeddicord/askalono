# askalono

askalono is a library and command-line tool to help detect license texts. It's designed to be fast, accurate, and to support a wide variety of license texts.

[![releases](https://img.shields.io/github/release-date-pre/amzn/askalono.svg)](https://github.com/amzn/askalono/releases)
[![askalono crate](https://img.shields.io/crates/v/askalono.svg)](https://crates.io/crates/askalono)
[![Gitter chat](https://badges.gitter.im/amzn/askalono.png)](https://gitter.im/amzn/askalono)

## Notice

This tool does not provide legal advice and it is not a lawyer. It endeavors to match your input to a database of similar license texts, and tell you what it thinks is a close match. But, it can't tell you that the given license is authoritative over a project. Nor can it tell you what to do with a license once it's identified. You are not entitled to rely on the accuracy of the output of this tool, and should seek independent legal advice for any licensing questions that may arise from using this tool.

### Additional pre-release note

This software is in the early stages of its lifecycle. While its goals are to be as accurate as it can be, there may be more bugs than expected of a production product.

## Usage

### On the command line

**NOTE:** This is not currently `cargo install`-able from crates.io, but when it is you'll find it under `askalono-cli`.

Pre-built binaries are available on the [Releases section](https://github.com/amzn/askalono/releases) on GitHub.

Basic usage:

    askalono id <filename>

where `<filename>` is a file (not folder) containing license text to analyze. In many projects, this file is called `LICENSE` or `COPYING`. askalono will analyze the text and output what it thinks it is.

### As a library

**Note:** This API is still unstable (and relatively undocumented). Semantic versioning will be respected, but anticipate significant changes throughout development.

At the moment, `Store` and `LicenseContent` are exposed for usage. These have a relatively sane API, if unergonmic. Expect improvements in usage and documentation here.

The best way to get an idea of how to use askalono as a library in its early state is to look at the [example](./examples/basic.rs).

## Details

### Implementation

**tl;dr**: Sørensen–Dice scoring, multi-threading, compressed cache file

At its core, askalono builds up bigrams (word pairs) of input text, and compares that with other license texts it knows about to see how similar they are. It scores each match with a [Sørensen–Dice](https://en.wikipedia.org/wiki/S%C3%B8rensen%E2%80%93Dice_coefficient) coefficient and looks for the highest result. There is some minimal preprocessing happening before matching, but there are no hand-maintained regular expressions or curations used to determine a match.

In detail, the matching process:

1. Reads in input text
1. Normalizes everything it reasonably can -- Unicode characters, whitespace, quoting styles, etc. are all whittled down to something common.
    * Lines that tend to change a lot in licenses, like "Copyright 20XX Some Person", are additionally removed.
1. Tokenizes normalized text into a set of bigrams.
1. In parallel, the bigram set is compared with all of the other sets askalono knows about.
1. The resulting list is sorted, the top match identified, and result returned.

To optimize startup, askalono builds up a database of license texts (applying the same normalization techniques described above), and persists this data to a MessagePack'd & gzip'd cache file. This cache is loaded at startup, and is optionally embedded in the binary itself.

### Name

It means "shallot" in Esperanto. You could try to derive a hidden meaning from it, but the real reason is really just that onions are delicious and Esperanto is an interesting language. In the author's opinion. (Sed la verkisto ne estas bonega Esperantisto, do bonvolu konversacii en la angla sur ĉi tiu projekto.)

### How is this different from other solutions?

There are several other excellent projects in this space, including [licensee](https://github.com/benbalter/licensee) and [LiD](https://source.codeaurora.org/external/qostg/lid/). These projects attempt to get a larger picture of a project's licensing, and can look at other sources of metadata to try to find answers. Both of these inspired the creation of askalono, first as a curiosity, then as a serious project.

askalono focuses on the problem of matching text itself -- it's often the piece that is difficult to optimize for speed and accuracy. askalono could be seen as a piece of plumbing in a larger system.

### Where do the licenses come from?

License data is sourced directly from SPDX: https://github.com/spdx/license-list-data

askalono can parse the "json" format included in that repository to generate its cache.

At this time, askalono is not taking requests for additional licenses in its default dataset -- its dataset is SPDX's own.

## Contributing

Contributions are very welcome! See [CONTRIBUTING](CONTRIBUTING.md) for more info.

## License

This library is licensed under the [Apache 2.0 License](LICENSE).
