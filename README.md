# askalono

askalono is a library and command-line tool to help detect license texts. It's designed to be fast, accurate, and to support a wide variety of license texts.

## Notice

This tool does not provide legal advice and it is not a lawyer. It endeavors to match your input to a database of similar license texts, and tell you what it thinks is a close match. But, it can't tell you that the given license is authoritative over a project. Nor can it tell you what to do with a license once it's identified. You are not entitled to rely on the accuracy of the output of this tool, and should seek independent legal advice for any licensing questions that may arise from using this tool.

### Additional pre-release note

This software is in the early stages of its lifecycle. While its goals are to be as accurate as it can be, there may be more bugs than expected of a production product.

## Usage

### On the command line

Basic usage:

    askalono id <filename>

where `<filename>` is a file (not folder) containing license text to analyze. In many projects, this file is called `LICENSE` or `COPYING`. askalono will analyze the text and output what it thinks it is.

### As a library

**Note:** This API is still unstable. Semantic versioning will be respected, but anticipate significant changes throughout development.

## Details

### Implementation

(TODO)

### Name

It means "shallot" in Esperanto. You could try to derive a hidden meaning from it, but the real reason is really just that onions are delicious and Esperanto is an interesting language. In the author's opinion.

### How is this different from other solutions?

There are several other excellent projects in this space, including [licensee](https://github.com/benbalter/licensee) and [LiD](https://source.codeaurora.org/external/qostg/lid/). These projects attempt to get a larger picture of a project's licensing, and can look at other sources of metadata to try to find answers. Both of these inspired the creation of askalono, first as a curiosity, then as a serious project.

askalono instead focuses on the problem of matching text itself -- it's often the piece that is difficult to optimize for speed and accuracy. askalono could be seen as a piece of plumbing in a larger system.

### Where do the licenses come from?

(TODO) https://github.com/spdx/license-list-data

## Contributing

Contributions are very welcome! See [CONTRIBUTING](CONTRIBUTING.md) for more info.

## License

This library is licensed under the [Apache 2.0 License](LICENSE).
