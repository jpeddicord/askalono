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

use std::collections::HashMap;
use std::fmt;

use failure::Error;

use ngram::NgramSet;
use preproc::{apply_aggressive, apply_normalizers};

#[derive(Clone, Debug)]
pub enum LicenseType {
    Original,
    Header,
    Alternate,
}

impl fmt::Display for LicenseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct TextData {
    match_data: NgramSet,
    lines_view: (usize, usize),
    lines_normalized: Option<Vec<String>>,
    text_processed: Option<String>,
}

impl TextData {
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

    // impl specialization might be nice to indicate that this type
    // is lacking stored text; perhaps there's another way to indicate that?
    // maybe an impl on an enum variant if/when that's available:
    // https://github.com/rust-lang/rfcs/pull/1450
    pub fn without_text(self) -> Self {
        TextData {
            match_data: self.match_data,
            lines_view: (0, 0),
            lines_normalized: None,
            text_processed: None,
        }
    }

    pub fn with_view(&self, start: usize, end: usize) -> Result<Self, Error> {
        let view = match &self.lines_normalized {
            &Some(ref lines) => &lines[start..end],
            &None => return Err(format_err!("TextData does not have original text")),
        };
        let view_joined = view.join("\n");
        let processed = apply_aggressive(&view_joined);
        Ok(TextData {
            match_data: NgramSet::from_str(&processed, 2),
            lines_view: (start, end),
            lines_normalized: self.lines_normalized.clone(),
            text_processed: Some(processed),
        })
    }

    pub fn lines(&self) -> Option<&[String]> {
        match self.lines_normalized {
            Some(ref lines) => Some(&lines[self.lines_view.0 .. self.lines_view.1]),
            None => None,
        }
    }

    pub fn match_score(&self, other: &TextData) -> f32 {
        self.match_data.dice(&other.match_data)
    }

    pub fn optimize_bounds(&self, other: &TextData) -> Self {
        println!("{:?}", self.lines_normalized);
        // optimize the ending bounds of the text match
        let end_optimized = self.search_optimize(
            &|end| self.with_view(0, end).unwrap().match_score(other),
            &|end| self.with_view(0, end).unwrap(),
        );
        let new_end = end_optimized.lines_view.1;
        println!("new_end {}", new_end);

        // then optimize the starting bounds
        let optimized = end_optimized.search_optimize(
            &|start| end_optimized.with_view(start, new_end).unwrap().match_score(other),
            &|start| end_optimized.with_view(start, new_end).unwrap(),
        );
        println!("view {:?}", optimized.lines_view);
        optimized
    }

    fn search_optimize(&self, score: &Fn(usize) -> f32, value: &Fn(usize) -> Self) -> Self {
        // cache score checks, since they're kinda expensive
        let mut memo: HashMap<usize, f32> = HashMap::new();
        let mut check_score = |index: usize| -> f32 {
            *memo.entry(index).or_insert_with(|| score(index))
        };

        fn search(score: &mut FnMut(usize) -> f32, left: usize, right: usize) -> usize {
            println!("  *** {} {}", left, right);
            if right - left <= 3 {
                println!("  final few elements; checking all");
                // find the index of the highest score in the remaining items
                let highest = (left .. right + 1) // inclusive
                  .map(|x| (x, score(x)))
                  .fold((0usize, 0f32), |acc, x| if x.1 > acc.1 { x } else { acc });
                return highest.0;
            }

            let low = (left * 2 + right) / 3;
            let high = (left + right * 2) / 3;
            let score_low = score(low);
            let score_high = score(high);
            println!("    low  {} {}\n    high {} {}", low, score_low, high, score_high);

            // XXX check this one
            if score_low > score_high {
                println!("  >>> low");
                search(score, left, high - 1)
            } else {
                println!("  >>> high");
                search(score, low + 1, right)
            }
        }

        println!("  searching with {:?}", self.lines_view);
        let optimal = search(&mut check_score, self.lines_view.0, self.lines_view.1);
        value(optimal)
    }

}

impl<'a> From<&'a str> for TextData {
    fn from(text: &'a str) -> Self {
        TextData::new(text)
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
    fn test_optimize_bounds() {
        let license_text = "this is a license text\nor it pretends to be one\nit's just a test";
        let sample_text = "this is a license text\nor it pretends to be one\nit's just a test\n\n\nhere is some\ncode\nhello();\n\n//a comment too";
        let license = TextData::from(license_text).without_text();
        let sample = TextData::from(sample_text);

        let optimized = sample.optimize_bounds(&license);
        println!("{:?}", optimized.lines_view);
        println!("{:?}", optimized.lines_normalized.clone().unwrap());
        assert_eq!((0, 3), optimized.lines_view);

        // add more to the string, try again (avoid int trunc screwups)
        let sample_text = format!("{}\none more line", sample_text);
        let sample = TextData::from(sample_text.as_str());
        let optimized = sample.optimize_bounds(&license);
        println!("{:?}", optimized.lines_view);
        println!("{:?}", optimized.lines_normalized.clone().unwrap());
        assert_eq!((0, 3), optimized.lines_view);

        // add to the beginning too
        let sample_text = format!("some content\nat\n\nthe beginning\n{}", sample_text);
        let sample = TextData::from(sample_text.as_str());
        let optimized = sample.optimize_bounds(&license);
        println!("{:?}", optimized.lines_view);
        println!("{:?}", optimized.lines_normalized.clone().unwrap());
        assert_eq!((4, 7), optimized.lines_view);
    }
}