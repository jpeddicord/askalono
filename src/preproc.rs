// Copyright 2018-2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::collections::HashMap;

use lazy_static::lazy_static;
use log::debug;
use regex::{Regex, Replacer};
use unicode_normalization::UnicodeNormalization;

type PreprocFn = dyn Fn(Cow<str>) -> Cow<str>;

trait CowRegex {
    fn replace_all_cow<'a, R: Replacer>(&self, text: Cow<'a, str>, replace: R) -> Cow<'a, str>;
}

impl CowRegex for Regex {
    fn replace_all_cow<'a, R: Replacer>(&self, text: Cow<'a, str>, replace: R) -> Cow<'a, str> {
        match text {
            Cow::Borrowed(find) => self.replace_all(find, replace),
            Cow::Owned(find) => Cow::Owned(self.replace_all(&find, replace).into_owned()),
        }
    }
}

/// A list of preprocessors that normalize text without removing anything
/// substantial. These operate on one line at a time.
pub const PREPROC_NORMALIZE: [&PreprocFn; 6] = [
    &normalize_unicode,
    &remove_junk,
    &blackbox_urls,
    &normalize_horizontal_whitespace,
    &normalize_punctuation,
    &trim,
];

/// A list of preprocessors that more aggressively normalize/mangle text
/// to make for friendlier matching. May remove statements and lines, and
/// more heavily normalize punctuation.
pub const PREPROC_AGGRESSIVE: [&PreprocFn; 8] = [
    &remove_common_tokens,
    &normalize_vertical_whitespace,
    &remove_punctuation,
    &lowercaseify,
    &remove_title_line,
    &remove_copyright_statements,
    &collapse_whitespace,
    &trim,
];

pub fn apply_normalizers(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for line in text.split('\n') {
        let mut out: Cow<str> = line.into();
        for preproc in &PREPROC_NORMALIZE {
            out = preproc(out);
        }
        lines.push(out.into());
    }
    debug!("Normalized to:\n{:?}\n---", lines);
    lines
}

pub fn apply_aggressive(text: &str) -> String {
    let mut out = text.into();
    for preproc in &PREPROC_AGGRESSIVE {
        out = preproc(out);
    }
    debug!("Aggressively normalized to:\n{}\n---", &out);
    out.into()
}

// Line-by-line normalizers

fn normalize_unicode(input: Cow<str>) -> Cow<str> {
    input.nfc().collect::<String>().into()
}

fn remove_junk(input: Cow<str>) -> Cow<str> {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"[^\w\s\pP]+").unwrap();
    }
    RX.replace_all_cow(input, "")
}

fn blackbox_urls(input: Cow<str>) -> Cow<str> {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"https?://\S+").unwrap();
    }
    RX.replace_all_cow(input, "http://blackboxed/url")
}

fn normalize_horizontal_whitespace(input: Cow<str>) -> Cow<str> {
    lazy_static! {
        // including slashes here as well
        static ref RX: Regex = Regex::new(r"(?x)[ \t\p{Zs} \\ / \| \x2044 ]+").unwrap();
    }
    RX.replace_all_cow(input, " ")
}

fn normalize_punctuation(input: Cow<str>) -> Cow<str> {
    lazy_static! {
        static ref RX_QUOTES: Regex = Regex::new(r#"["'\p{Pi}\p{Pf}]+"#).unwrap();
        static ref RX_DASH: Regex = Regex::new(r"\p{Pd}+").unwrap();
        static ref RX_OPEN: Regex = Regex::new(r"\p{Ps}+").unwrap();
        static ref RX_CLOSE: Regex = Regex::new(r"\p{Pe}+").unwrap();
        static ref RX_UNDER: Regex = Regex::new(r"\p{Pc}+").unwrap();
        static ref RX_COPY: Regex = Regex::new(r"[©Ⓒⓒ]").unwrap();
    }
    let mut out = input;
    out = RX_QUOTES.replace_all_cow(out, "'");
    out = RX_DASH.replace_all_cow(out, "-");
    out = RX_OPEN.replace_all_cow(out, "(");
    out = RX_CLOSE.replace_all_cow(out, ")");
    out = RX_UNDER.replace_all_cow(out, "_");
    out = RX_COPY.replace_all_cow(out, "(c)");
    out
}

fn trim(input: Cow<str>) -> Cow<str> {
    match input {
        Cow::Borrowed(text) => text.trim().into(),
        Cow::Owned(text) => Cow::Owned(text.trim().to_owned()),
    }
}

// Aggressive preprocessors

// Cut prefix of string near given byte index.
// If given index doesn't lie at char boundary,
// returns the biggest prefix with length not exceeding idx.
// If index is bigger than length or string, returns the whole string.
fn trim_byte_adjusted(s: &str, idx: usize) -> &str {
    if idx >= s.len() {
        return s;
    }

    if let Some(sub) = s.get(..idx) {
        sub
    } else {
        // Inspect bytes before index
        let trailing_continuation = s.as_bytes()[..idx]
            .iter()
            .rev()
            // Multibyte characters are encoded in UTF-8 in the following manner:
            //    first byte | rest of bytes
            //    1..10xxxxx   10xxxxxx
            //    ^^^^ number of ones is equal to number of bytes in codepoint
            // Number of 10xxxxxx bytes in codepoint is at most 3 in valid UTF-8-encoded string,
            // so this loop actually runs a little iterations
            .take_while(|&byte| byte & 0b1100_0000 == 0b1000_0000)
            .count();
        // Subtract 1 to take the first byte in codepoint into account
        &s[..idx - trailing_continuation - 1]
    }
}

fn lcs_substr<'a>(f_line: &'a str, s_line: &'a str) -> &'a str {
    // find the length of common prefix in byte representations of strings
    let prefix_len = f_line
        .as_bytes()
        .iter()
        .zip(s_line.as_bytes())
        .take_while(|(&f, &s)| f == s)
        .count();

    trim_byte_adjusted(f_line, prefix_len).trim()
}

fn remove_common_tokens(input: Cow<str>) -> Cow<str> {
    let lines: Vec<&str> = input.split('\n').collect();
    let mut l_iter = lines.iter();

    let mut prefix_counts = HashMap::<_, u32>::new();

    // pass 1: iterate through the text to record common prefixes
    if let Some(first) = l_iter.next() {
        let mut pair = ("", first);
        let line_pairs = std::iter::from_fn(|| {
            pair = (pair.1, l_iter.next()?);
            Some(pair)
        });
        for (a, b) in line_pairs {
            let common = lcs_substr(a, b);

            // why start at 1, then immediately add 1?
            // lcs_substr compares two lines!
            // this doesn't need to be exact, just consistent.
            if common.len() > 3 {
                *prefix_counts.entry(common).or_insert(1) += 1;
            }
        }
    }

    // look at the most common observed prefix
    let most_common = match prefix_counts.iter().max_by_key(|&(_k, v)| v) {
        Some((prefix, _count)) => prefix,
        None => return input,
    };

    // reconcile the count with other longer prefixes that may be stored
    let common_count = prefix_counts
        .iter()
        .filter_map(|(s, count)| Some(count).filter(|_| s.starts_with(most_common)))
        .sum::<u32>();

    // the common string must be at least 80% of the text
    let prefix_threshold = (0.8f32 * lines.len() as f32) as _;
    if common_count < prefix_threshold {
        return input;
    }

    // pass 2: remove that substring
    lines
        .iter()
        .map(|line| {
            if let Some(stripped) = line.strip_prefix(most_common) {
                stripped
            } else {
                line
            }
            .trim()
        })
        .collect::<Vec<_>>()
        .join("\n")
        .into()
}

fn normalize_vertical_whitespace(input: Cow<str>) -> Cow<str> {
    lazy_static! {
        static ref RX_MISC: Regex = Regex::new(r"[\r\n\v\f]").unwrap();
        static ref RX_NUM: Regex = Regex::new(r"\n{3,}").unwrap();
    }
    let mut out = input;
    out = RX_MISC.replace_all_cow(out, "\n");
    out = RX_NUM.replace_all_cow(out, "\n\n");
    out
}

fn remove_punctuation(input: Cow<str>) -> Cow<str> {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"[^\w\s]+").unwrap();
    }
    RX.replace_all_cow(input, "")
}

fn lowercaseify(input: Cow<str>) -> Cow<str> {
    input.to_lowercase().into()
}

fn remove_title_line(input: Cow<str>) -> Cow<str> {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"^.*license( version \S+)?( copyright.*)?\n\n").unwrap();
    }

    RX.replace_all_cow(input, "")
}

fn remove_copyright_statements(input: Cow<str>) -> Cow<str> {
    lazy_static! {
        static ref RX: Regex = Regex::new(
            r"(?mx)
            (
                # either a new paragraph, or the beginning of the text + empty lines
                (\n\n|\A\n*)
                # any number of lines starting with 'copyright' followed by a new paragraph
                (^\x20*copyright.*?$)+
                \n\n
            )
            |
            (
                # or the very first line if it has 'copyright' in it
                \A.*copyright.*$
            )
            |
            (
                # or any lines that really look like a copyright statement
                ^copyright (\s+(c|\d+))+ .*?$
            )
        "
        )
        .unwrap();
    }

    RX.replace_all_cow(input, "\n\n")
}

fn collapse_whitespace(input: Cow<str>) -> Cow<str> {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"\s+").unwrap();
    }
    RX.replace_all_cow(input, " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_byte_adjusted_respects_multibyte_characters() {
        let input = "RustКраб橙蟹🦀";
        let expected = [
            "",
            "R",
            "Ru",
            "Rus",
            "Rust",
            "Rust",
            "RustК",
            "RustК",
            "RustКр",
            "RustКр",
            "RustКра",
            "RustКра",
            "RustКраб",
            "RustКраб",
            "RustКраб",
            "RustКраб橙",
            "RustКраб橙",
            "RustКраб橙",
            "RustКраб橙蟹",
            "RustКраб橙蟹",
            "RustКраб橙蟹",
            "RustКраб橙蟹",
            "RustКраб橙蟹🦀",
        ];

        for (i, &outcome) in expected.iter().enumerate() {
            assert_eq!(outcome, trim_byte_adjusted(input, i))
        }
    }

    #[test]
    fn greatest_substring_removal() {
        // the funky string syntax \n\ is to add a newline but skip the
        // leading whitespace in the source code
        let text = "%%Copyright: Copyright\n\
                    %%Copyright: All rights reserved.\n\
                    %%Copyright: Redistribution and use in source and binary forms, with or\n\
                    %%Copyright: without modification, are permitted provided that the\n\
                    %%Copyright: following conditions are met:\n\
                    \n\
                    abcd";

        let new_text = remove_common_tokens(text.into());
        println!("{}", new_text);

        assert!(
            !new_text.contains("%%Copyright"),
            "new text shouldn't contain the common substring"
        );
    }

    #[test]
    fn greatest_substring_removal_keep_inner() {
        let text = "this string should still have\n\
                    this word -> this <- in it even though\n\
                    this is still the most common word";
        let new_text = remove_common_tokens(text.into());
        println!("-- {}", new_text);
        // the "this" at the start of the line can be discarded...
        assert!(!new_text.contains("\nthis"));
        // ...but the "this" in the middle of sentences shouldn't be
        assert!(new_text.contains("this"));

        let text = "aaaa bbbb cccc dddd\n\
                    eeee ffff aaaa gggg\n\
                    hhhh iiii jjjj";
        let new_text = remove_common_tokens(text.into());
        println!("-- {}", new_text);
        assert!(new_text.contains("aaaa")); // similar to above test
    }

    #[test]
    fn greatest_substring_removal_42() {
        // https://github.com/jpeddicord/askalono/issues/42
        let text = "AAAAAA line 1\n\
                    AAAAAA another line here\n\
                    AAAAAA yet another line here\n\
                    AAAAAA how long will this go on\n\
                    AAAAAA another line here\n\
                    AAAAAA more\n\
                    AAAAAA one more\n\
                    AAAAAA two more\n\
                    AAAAAA three more\n\
                    AAAAAA four more\n\
                    AAAAAA five more\n\
                    AAAAAA six more\n\
                    \n\
                    preserve\n\
                    keep";
        let new_text = remove_common_tokens(text.into());
        println!("{}", new_text);

        assert!(new_text.contains("preserve"));
        assert!(new_text.contains("keep"));
        assert!(!new_text.contains("AAAAAA"));
    }

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
        let normalized_lines = normalized.len();

        assert_eq!(
            text_lines, normalized_lines,
            "normalizers shouldnt change line counts"
        );
    }
}
