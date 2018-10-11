// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

type PreprocFn = Fn(&str) -> String;

/// A list of preprocessors that normalize text without removing anything
/// substantial. These operate on one line at a time.
pub const PREPROC_NORMALIZE: [&PreprocFn; 5] = [
    &normalize_unicode,
    &remove_junk,
    &normalize_horizontal_whitespace,
    &normalize_punctuation,
    &trim_line,
];

/// A list of preprocessors that more aggressively normalize/mangle text
/// to make for friendlier matching. May remove statements and lines, and
/// more heavily normalize punctuation.
pub const PREPROC_AGGRESSIVE: [&PreprocFn; 7] = [
    &remove_common_tokens,
    &normalize_vertical_whitespace,
    &remove_punctuation,
    &lowercaseify,
    &remove_copyright_statements,
    &collapse_whitespace,
    &final_trim,
];

fn lcs_substr(fstr: &str, sstr: &str) -> String {
    let mut f_chars = fstr.chars();
    let mut longest_substr = String::new();

    loop {
        let mut f = match f_chars.next() {
            Some(s) => s,
            None => return longest_substr,
        };

        let mut substr = String::new();
        let mut s_chars = sstr.chars();

        loop {
            match s_chars.next() {
                Some(s) => {
                    if f == s {
                        substr.push(s);

                        f = match f_chars.next() {
                            Some(f_str) => f_str,
                            None => return longest_substr,
                        };
                    } else {
                        if substr.len() > longest_substr.len() {
                            longest_substr = substr.clone();
                        }

                        substr.clear();
                    }
                }
                None => break,
            }
        }
    }
}

pub fn remove_common_tokens(text: &str) -> String {
    let lines: Vec<&str> = text.split("\n").collect();
    let mut largest_substr = String::new();
    let mut l_iter = lines.iter();

    loop {
        let f_line = match l_iter.next() {
            Some(line) => line,
            None => break,
        };

        largest_substr = match l_iter.next() {
            Some(s_line) => lcs_substr(f_line, s_line),
            None => break,
        }
    }

    let new_text = str::replace(text, largest_substr.as_str(), "");

    new_text.to_string()
}

pub fn apply_normalizers(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for line in text.split('\n') {
        let mut out = line.to_owned();
        for preproc in &PREPROC_NORMALIZE {
            out = preproc(&out);
        }
        lines.push(out);
    }
    debug!("Normalized to:\n{:?}\n---", lines);
    lines
}

pub fn apply_aggressive(text: &str) -> String {
    let mut out = text.to_owned();
    for preproc in &PREPROC_AGGRESSIVE {
        out = preproc(&out);
    }
    debug!("Aggressively normalized to:\n{}\n---", out);
    out
}

// Line-by-line normalizers

fn normalize_unicode(input: &str) -> String {
    input.nfc().collect::<String>()
}

fn remove_junk(input: &str) -> String {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"[^\w\s\pP]+").unwrap();
    }
    RX.replace_all(input, "").into()
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

fn trim_line(input: &str) -> String {
    input.trim().into()
}

// Aggressive preprocessors

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
        static ref RX: Regex = Regex::new(
            r"(?imx)
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
        "
        ).unwrap();
    }

    RX.replace_all(input, "\n\n").into()
}

fn collapse_whitespace(input: &str) -> String {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"\s+").unwrap();
    }
    RX.replace_all(input, " ").into()
}

fn final_trim(input: &str) -> String {
    input.trim().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greatest_substring_removal() {
        let text = "%%Copyright: Copyright
            %%Copyright: All rights reserved.
            %%Copyright: Redistribution and use in source and binary forms, with or
            %%Copyright: without modification, are permitted provided that the
            %%Copyright: following conditions are met:";

        let new_text = remove_common_tokens(text);

        assert_eq!(
            new_text.contains("%%Copyright"),
            false,
            "new text shouldn't contain the common substring"
        );
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
