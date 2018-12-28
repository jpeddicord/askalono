//  Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
//  SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

type PreprocFn = dyn Fn(&str) -> String;

/// A list of preprocessors that normalize text without removing anything
/// substantial. These operate on one line at a time.
pub const PREPROC_NORMALIZE: [&PreprocFn; 6] = [
    &normalize_unicode,
    &remove_junk,
    &blackbox_urls,
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

fn blackbox_urls(input: &str) -> String {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"https?://\S+").unwrap();
    }
    RX.replace_all(input, "http://blackboxed/url").into()
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

fn lcs_substr(fstr: &str, sstr: &str) -> Option<String> {
    let mut f_chars = fstr.chars();
    let mut s_chars = sstr.chars();
    let mut substr = String::new();

    let skip_whitespace = |c| match c {
        ' ' | '\t' => true,
        _ => false,
    };

    loop {
        let f = match f_chars.next() {
            Some(f_str) => {
                if skip_whitespace(f_str) {
                    s_chars.next();
                    continue;
                } else {
                    f_str
                }
            }
            None => {
                if !substr.is_empty() {
                    return Some(substr);
                } else {
                    return None;
                }
            }
        };

        match s_chars.next() {
            Some(s) => {
                if !skip_whitespace(s) {
                    if f == s {
                        substr.push(s);
                    } else if !substr.is_empty() {
                        return Some(substr);
                    } else {
                        return None;
                    }
                }
            }
            None => {
                if !substr.is_empty() {
                    return Some(substr);
                } else {
                    return None;
                }
            }
        }
    }
}

fn remove_common_tokens(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut largest_substr = String::new();
    let mut l_iter = lines.iter();

    // TODO: consider whether this can all be done in one pass

    // pass 1: iterate through the text to find the largest substring
    // from the start of the line
    #[allow(clippy::while_let_loop)]
    loop {
        let f_line = match l_iter.next() {
            Some(line) => line,
            None => break,
        };

        let new_largest_substr = match l_iter.next() {
            Some(s_line) => match lcs_substr(f_line, s_line) {
                Some(substr) => substr,
                None => break,
            },
            None => break,
        };

        if largest_substr.contains(new_largest_substr.as_str()) || largest_substr.is_empty() {
            largest_substr = new_largest_substr.to_string();
        }
    }

    // pass 2: remove that substring
    if largest_substr.len() > 3 {
        lines
            .iter()
            .filter(|l| match l.find(&largest_substr) {
                Some(index) => index == 0,
                None => false,
            })
            .map(|l| l.replacen(&largest_substr, "", 1))
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        text.to_string()
    }
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
        )
        .unwrap();
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
        // the funky string syntax \n\ is to add a newline but skip the
        // leading whitespace in the source code
        let text = "%%Copyright: Copyright\n\
                    %%Copyright: All rights reserved.\n\
                    %%Copyright: Redistribution and use in source and binary forms, with or\n\
                    %%Copyright: without modification, are permitted provided that the\n\
                    %%Copyright: following conditions are met:\n\
                    \n\
                    abcd";

        let new_text = remove_common_tokens(text);
        println!("{}", new_text);

        assert_eq!(
            new_text.contains("%%Copyright"),
            false,
            "new text shouldn't contain the common substring"
        );

        let text = "this string should still have\n\
                    this word -> this <- in it even though\n\
                    this is still the most common word";
        let new_text = remove_common_tokens(text);
        println!("-- {}", new_text);
        // the "this" at the start of the line can be discarded...
        assert!(!new_text.contains("\nthis"));
        // ...but the "this" in the middle of sentences shouldn't be
        assert!(new_text.contains("this"));

        let text = "aaaa bbbb cccc dddd\n\
                    eeee ffff aaaa gggg\n\
                    hhhh iiii jjjj";
        let new_text = remove_common_tokens(text);
        println!("-- {}", new_text);
        assert!(new_text.contains("aaaa")); // similar to above test
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
