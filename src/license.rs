// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    ngram::NgramSet,
    preproc::{apply_aggressive, apply_normalizers},
};

/// The type of a license entry (typically in a `Store`).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseType {
    /// The canonical text of the license.
    Original,
    /// A license header. There may be more than one in a `Store`.
    Header,
    /// An alternate form of a license. This is intended to be used for
    /// alternate _formats_ of a license, not for variants where the text has
    /// different meaning. Not currently used in askalono's SPDX dataset.
    Alternate,
}

impl fmt::Display for LicenseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                LicenseType::Original => "original text",
                LicenseType::Header => "license header",
                LicenseType::Alternate => "alternate text",
            }
        )
    }
}

/// A structure representing compiled text/matching data.
///
/// This is the key structure used to compare two texts against one another. It
/// handles pre-processing the text to n-grams, scoring, and optimizing the
/// result to try to identify specific details about a match.
///
/// # Examples
///
/// Basic scoring of two texts:
///
/// ```
/// use askalono::TextData;
///
/// let license = TextData::from("My First License");
/// let sample = TextData::from("copyright 20xx me irl\n\n //  my   first license");
/// assert_eq!(sample.match_score(&license), 1.0);
/// ```
///
/// The above example is a perfect match, as identifiable copyright statements
/// are stripped out during pre-processing.
///
/// Building on that, TextData is able to tell you _where_ in the text a
/// license is located:
///
/// ```
/// # use std::error::Error;
/// # use askalono::TextData;
/// # fn main() -> Result<(), Box<Error>> {
/// # let license = TextData::from("My First License");
/// let sample = TextData::from("copyright 20xx me irl\n// My First License\nfn hello() {\n ...");
/// let (optimized, score) = sample.optimize_bounds(&license);
/// assert_eq!((1, 2), optimized.lines_view());
/// assert!(score > 0.99f32, "license within text matches");
/// # Ok(())
/// # }
/// ```
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextData {
    match_data: NgramSet,
    lines_view: (usize, usize),
    lines_normalized: Option<Vec<String>>,
    text_processed: Option<String>,
}

const TEXTDATA_TEXT_ERROR: &str = "TextData does not have original text";

impl TextData {
    /// Create a new TextData structure from a string.
    ///
    /// The given text will be normalized, then smashed down into n-grams for
    /// matching. By default, the normalized text is stored inside the
    /// structure for future diagnostics. This is necessary for optimizing a
    /// match and for diffing against other texts. If you don't want this extra
    /// data, you can call `without_text` throw it out. Generally, as a user of
    /// this library you want to keep the text data, but askalono will throw it
    /// away in its own `Store` as it's not needed.
    pub fn new(text: &str) -> TextData {
        let normalized = apply_normalizers(text);
        let normalized_joined = normalized.join("\n");
        let processed = apply_aggressive(&normalized_joined);
        let match_data = NgramSet::from_str(&processed, 2);

        TextData {
            match_data,
            lines_view: (0, normalized.len()),
            lines_normalized: Some(normalized),
            text_processed: Some(processed),
        }
    }

    /// Consume this `TextData`, returning one without normalized/processed
    /// text stored.
    ///
    /// Unless you know you don't want the text, you probably don't want to use
    /// this. Other methods on `TextData` require that text is present.
    pub fn without_text(self) -> Self {
        TextData {
            match_data: self.match_data,
            lines_view: (0, 0),
            lines_normalized: None,
            text_processed: None,
        }
    }

    /// Get the bounds of the active line view.
    ///
    /// This represents the "active" region of lines that matches are generated
    /// from. The bounds are a 0-indexed `(start, end)` tuple, with inclusive
    /// start and exclusive end indicies. See `optimize_bounds`.
    ///
    /// This is largely for informational purposes; other methods in
    /// `TextView`, such as `lines` and `match_score`, will already account for
    /// the line range. However, it's useful to call it after running
    /// `optimize_bounds` to discover where the input text was discovered.
    pub fn lines_view(&self) -> (usize, usize) {
        self.lines_view
    }

    /// Clone this `TextView`, creating a copy with the given view.
    ///
    /// This will re-generate match data for the given view. It's used in
    /// `optimize_bounds` to shrink/expand the view of the text to discover
    /// bounds.
    ///
    /// Other methods on `TextView` respect this boundary, so it's not needed
    /// outside this struct.
    pub fn with_view(&self, start: usize, end: usize) -> Self {
        let view = &self.lines_normalized.as_ref().expect(TEXTDATA_TEXT_ERROR)[start..end];
        let view_joined = view.join("\n");
        let processed = apply_aggressive(&view_joined);
        TextData {
            match_data: NgramSet::from_str(&processed, 2),
            lines_view: (start, end),
            lines_normalized: self.lines_normalized.clone(),
            text_processed: Some(processed),
        }
    }

    /// "Erase" the current lines in view and restore the view to its original
    /// bounds.
    ///
    /// For example, consider a file with two licenses in it. One was identified
    /// (and located) with `optimize_bounds`. Now you want to find the other:
    /// white-out the matched lines, and re-run the overall search to find a
    /// new high score.
    pub fn white_out(&self) -> Self {
        // note that we're not using the view here...
        let lines = self.lines_normalized.as_ref().expect(TEXTDATA_TEXT_ERROR);

        // ...because it's used here to exclude lines
        let new_normalized: Vec<String> = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i >= self.lines_view.0 && i < self.lines_view.1 {
                    "".to_string()
                } else {
                    line.clone()
                }
            })
            .collect();

        let processed = apply_aggressive(&new_normalized.join("\n"));
        TextData {
            match_data: NgramSet::from_str(&processed, 2),
            lines_view: (0, new_normalized.len()),
            lines_normalized: Some(new_normalized),
            text_processed: Some(processed),
        }
    }

    /// Get a slice of the normalized lines in this `TextData`.
    pub fn lines(&self) -> &[String] {
        &self.lines_normalized.as_ref().expect(TEXTDATA_TEXT_ERROR)
            [self.lines_view.0..self.lines_view.1]
    }

    #[doc(hidden)]
    pub fn text_processed(&self) -> Option<&str> {
        self.text_processed.as_ref().map(String::as_ref)
    }

    /// Compare this `TextData` with another, returning a similarity score.
    ///
    /// This is what's used during analysis to rank licenses.
    pub fn match_score(&self, other: &TextData) -> f32 {
        self.match_data.dice(&other.match_data)
    }

    #[cfg(feature = "spdx")]
    pub(crate) fn eq_data(&self, other: &Self) -> bool {
        self.match_data.eq(&other.match_data)
    }

    /// Attempt to optimize a known match to locate possible line ranges.
    ///
    /// Returns a new `TextData` struct and a score. The returned struct is a
    /// clone of `self`, with its view set to the best match against `other`.
    ///
    /// This will respect any views set on the TextData (an optimized result
    /// won't go outside the original view).
    ///
    /// Note that this won't be 100% optimal if there are blank lines
    /// surrounding the actual match, since successive blank lines in a range
    /// will likely have the same score.
    ///
    /// You should check the value of `lines_view` on the returned struct to
    /// find the line ranges.
    pub fn optimize_bounds(&self, other: &TextData) -> (Self, f32) {
        assert!(self.lines_normalized.is_some(), "{}", TEXTDATA_TEXT_ERROR);

        let view = self.lines_view;

        // optimize the ending bounds of the text match
        let (end_optimized, _) = self.search_optimize(
            &|end| self.with_view(view.0, end).match_score(other),
            &|end| self.with_view(view.0, end),
        );
        let new_end = end_optimized.lines_view.1;

        // then optimize the starting bounds
        let (optimized, score) = end_optimized.search_optimize(
            &|start| end_optimized.with_view(start, new_end).match_score(other),
            &|start| end_optimized.with_view(start, new_end),
        );
        (optimized, score)
    }

    fn search_optimize(
        &self,
        score: &dyn Fn(usize) -> f32,
        value: &dyn Fn(usize) -> Self,
    ) -> (Self, f32) {
        // cache score checks, since they're kinda expensive
        let mut memo: HashMap<usize, f32> = HashMap::new();
        let mut check_score =
            |index: usize| -> f32 { *memo.entry(index).or_insert_with(|| score(index)) };

        fn search(score: &mut dyn FnMut(usize) -> f32, left: usize, right: usize) -> (usize, f32) {
            if right - left <= 3 {
                // find the index of the highest score in the remaining items
                return (left..=right)
                    .map(|x| (x, score(x)))
                    .fold((0usize, 0f32), |acc, x| if x.1 >= acc.1 { x } else { acc });
            }

            let low = (left * 2 + right) / 3;
            let high = (left + right * 2) / 3;
            let score_low = score(low);
            let score_high = score(high);

            if score_low > score_high {
                search(score, left, high - 1)
            } else {
                search(score, low + 1, right)
            }
        }

        let optimal = search(&mut check_score, self.lines_view.0, self.lines_view.1);
        (value(optimal.0), optimal.1)
    }
}

impl<'a> From<&'a str> for TextData {
    fn from(text: &'a str) -> Self {
        Self::new(text)
    }
}

impl<'a> From<String> for TextData {
    fn from(text: String) -> Self {
        Self::new(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // psst:
    // cargo test -- --nocapture

    #[test]
    fn optimize_bounds() {
        let license_text = "this is a license text\nor it pretends to be one\nit's just a test";
        let sample_text = "this is a license text\nor it pretends to be one\nit's just a test\nwords\n\nhere is some\ncode\nhello();\n\n//a comment too";
        let license = TextData::from(license_text).without_text();
        let sample = TextData::from(sample_text);

        let (optimized, _) = sample.optimize_bounds(&license);
        println!("{:?}", optimized.lines_view);
        println!("{:?}", optimized.lines_normalized);
        assert_eq!((0, 3), optimized.lines_view);

        // add more to the string, try again (avoid int trunc screwups)
        let sample_text = format!("{}\none more line", sample_text);
        let sample = TextData::from(sample_text.as_str());
        let (optimized, _) = sample.optimize_bounds(&license);
        println!("{:?}", optimized.lines_view);
        println!("{:?}", optimized.lines_normalized);
        assert_eq!((0, 3), optimized.lines_view);

        // add to the beginning too
        let sample_text = format!("some content\nat\n\nthe beginning\n{}", sample_text);
        let sample = TextData::from(sample_text.as_str());
        let (optimized, _) = sample.optimize_bounds(&license);
        println!("{:?}", optimized.lines_view);
        println!("{:?}", optimized.lines_normalized);
        // end bounds at 7 and 8 have the same score, since they're empty lines (not
        // counted). askalono is not smart enough to trim this as close as it
        // can.
        assert!(
            (4, 7) == optimized.lines_view || (4, 8) == optimized.lines_view,
            "bounds are (4, 7) or (4, 8)"
        );
    }

    // if a view is set on the text data, optimize_bounds must not find text
    // outside of that range
    #[test]
    fn optimize_doesnt_grow_view() {
        let sample_text = "0\n1\n2\naaa aaa\naaa\naaa\naaa\n7\n8";
        let license_text = "aaa aaa aaa aaa aaa";
        let sample = TextData::from(sample_text);
        let license = TextData::from(license_text).without_text();

        // sanity: the optimized bounds should be at (3, 7)
        let (optimized, _) = sample.optimize_bounds(&license);
        assert_eq!((3, 7), optimized.lines_view);

        // this should still work
        let sample = sample.with_view(3, 7);
        let (optimized, _) = sample.optimize_bounds(&license);
        assert_eq!((3, 7), optimized.lines_view);

        // but if we shrink the view further, it shouldn't be outside that range
        let sample = sample.with_view(4, 6);
        let (optimized, _) = sample.optimize_bounds(&license);
        assert_eq!((4, 6), optimized.lines_view);

        // restoring the view should still be OK too
        let sample = sample.with_view(0, 9);
        let (optimized, _) = sample.optimize_bounds(&license);
        assert_eq!((3, 7), optimized.lines_view);
    }

    // ensure we don't choke on small TextData matches
    #[test]
    fn match_small() {
        let a = TextData::from("a b");
        let b = TextData::from("a\nlong\nlicense\nfile\n\n\n\n\nabcdefg");

        let x = a.match_score(&b);
        let y = b.match_score(&a);

        assert_eq!(x, y);
    }

    // don't choke on empty TextData either
    #[test]
    fn match_empty() {
        let a = TextData::from("");
        let b = TextData::from("a\nlong\nlicense\nfile\n\n\n\n\nabcdefg");

        let x = a.match_score(&b);
        let y = b.match_score(&a);

        assert_eq!(x, y);
    }

    #[test]
    fn view_and_white_out() {
        let a = TextData::from("aaa\nbbb\nccc\nddd");
        assert_eq!(Some("aaa bbb ccc ddd"), a.text_processed());

        let b = a.with_view(1, 3);
        assert_eq!(2, b.lines().len());
        assert_eq!(Some("bbb ccc"), b.text_processed());

        let c = b.white_out();
        assert_eq!(Some("aaa ddd"), c.text_processed());
    }
}
