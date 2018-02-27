// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License").
// You may not use this file except in compliance with the License.
// A copy of the License is located at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// or in the "license" file accompanying this file. This file is distributed
// on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing
// permissions and limitations under the License.

use std::borrow::Cow;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

type PreprocFn = Fn(&str) -> String;

/// A list of preprocessors that normalize text without removing anything
/// substantial. Shouldn't remove entire lines for the sake of preserving
/// line number information.
pub const PREPROC_NORMALIZE: [&PreprocFn; 5] = [
    &normalize_unicode,
    &normalize_junk,
    &normalize_horizontal_whitespace,
    &normalize_punctuation,
    &trim_lines,
];

/// A list of preprocessors that more aggressively normalize/mangle text
/// to make for friendlier matching. May remove statements and lines, and
/// more heavily normalize punctuation.
pub const PREPROC_AGGRESSIVE: [&PreprocFn; 5] = [
    &normalize_vertical_whitespace,
    &remove_punctuation,
    &lowercaseify,
    &remove_copyright_statements,
    &collapse_whitespace,
];

pub fn apply_normalizers(text: &str) -> String {
    let mut out = text.to_owned();
    for p in &PREPROC_NORMALIZE {
        out = p(&out);
    }
    debug!("Normalized to:\n{}\n---", out);
    out
}

pub fn apply_aggressive(text: &str) -> String {
    let mut out = text.to_owned();
    for p in &PREPROC_AGGRESSIVE {
        out = p(&out);
    }
    debug!("Aggressively normalized to:\n{}\n---", out);
    out
}

fn normalize_unicode(input: &str) -> String {
    input.nfc().collect::<String>()
}

fn normalize_junk(input: &str) -> String {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"[^\w\s\pP]+").unwrap();
    }
    RX.replace_all(input, "").into()
}

fn normalize_vertical_whitespace(input: &str) -> String {
    lazy_static! {
        static ref RX_MISC: Regex = Regex::new(r"[\r\n\v\f]").unwrap();
        static ref RX_NUM: Regex = Regex::new(r"\n{3,}").unwrap();
    }
    let out = Cow::Borrowed(input);
    let out = RX_MISC.replace_all(&out, "\n");
    let out = RX_NUM.replace_all(&out, "\n\n");
    out.into()
}

fn normalize_horizontal_whitespace(input: &str) -> String {
    lazy_static! {
        // including slashes here as well
        static ref RX: Regex = Regex::new(r"(?x)[ \t\p{Zs} \\ / \| \x2044 ]+").unwrap();
    }
    RX.replace_all(input, " ").into()
}

fn normalize_punctuation(input: &str) -> String {
    lazy_static! {
        static ref RX_QUOTES: Regex = Regex::new(r#"["'\p{Pi}\p{Pf}]+"#).unwrap();
        static ref RX_DASH: Regex = Regex::new(r"\p{Pd}+").unwrap();
        static ref RX_OPEN: Regex = Regex::new(r"\p{Ps}+").unwrap();
        static ref RX_CLOSE: Regex = Regex::new(r"\p{Pe}+").unwrap();
        static ref RX_UNDER: Regex = Regex::new(r"\p{Pc}+").unwrap();
        static ref RX_COPY: Regex = Regex::new(r"[©Ⓒⓒ]").unwrap();
    }
    let out = Cow::Borrowed(input);
    let out = RX_QUOTES.replace_all(&out, "'");
    let out = RX_DASH.replace_all(&out, "-");
    let out = RX_OPEN.replace_all(&out, "(");
    let out = RX_CLOSE.replace_all(&out, ")");
    let out = RX_UNDER.replace_all(&out, "_");
    let out = RX_COPY.replace_all(&out, "(c)");
    out.into()
}

fn trim_lines(input: &str) -> String {
    let mut out = Vec::new();
    for line in input.split('\n') {
        out.push(line.trim());
    }
    out.join("\n")
}

fn remove_punctuation(input: &str) -> String {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"[^\w\s]+").unwrap();
    }
    RX.replace_all(input, "").into()
}

fn lowercaseify(input: &str) -> String {
    input.to_lowercase()
}

fn remove_copyright_statements(input: &str) -> String {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"(?imx)
            (
                # either a new paragraph, or the beginning of the text + empty lines
                (\n\n|\A\n*)
                # any number of lines starting with 'copyright' followed by a new paragraph
                (^\x20*copyright.*?$)+
                \n\n
            )
            |
            (
                # or any lines that really look like a copyright statement
                ^copyright (\s+(c|\d+))+ .*?$
            )
        ").unwrap();
    }

    RX.replace_all(input, "\n\n").into()
}

fn collapse_whitespace(input: &str) -> String {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"\s+").unwrap();
    }
    RX.replace_all(input, " ").into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_no_line_mangle() {
        let text = "some license

        copyright 2012 person

        \tlicense\r
        text

        \t



        goes
        here";

        let text_lines = text.lines().count();

        let normalized = apply_normalizers(text);
        let normalized_lines = normalized.lines().count();

        assert_eq!(
            text_lines, normalized_lines,
            "normalizers shouldnt change line counts"
        );
    }

}
