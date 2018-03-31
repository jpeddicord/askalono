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

use std::fmt;

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

#[derive(Serialize, Deserialize)]
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

    pub fn view(&mut self, start: usize, end: usize) {
        let view = match &self.lines_normalized {
            &Some(ref lines) => Self::vec_view(&lines, start, end),
            &None => return, // XXX: probably not the right thing to do
        };
        let view_joined = view.join("\n");
        let processed = apply_aggressive(&view_joined);

        self.match_data = NgramSet::from_str(&processed, 2);
        self.lines_view = (start, view.len());
        self.text_processed = Some(processed);
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

    pub fn find_highest_match(&mut self) -> (usize, usize) {
        (0, 0)
    }

    fn vec_view(lines: &[String], start: usize, end: usize) -> &[String] {
        if end == 0 {
            lines
        } else {
            &lines[start .. end]
        }
    }

    fn binsearch_end(&mut self, other: &TextData) {
        let mut curr_score = self.match_score(other);
        let view = self.lines_view;
        let (mut left, mut right) = view;

        loop {
            if left > right {
                return;
            }

            let mid = (left + right)/2;
            println!("left {} right {} mid {}", left, right, mid);
            self.view(view.0, mid);
            let next_score = self.match_score(other);
            println!("curr: {}, next: {}, end at {}", curr_score, next_score, mid);
            if next_score > curr_score {
                right = mid - 1;
            } else {
                left = mid + 1;
            }
            curr_score = next_score;
        }
    }

    fn binsearch_start(&mut self, other: &TextData) {
        let mut curr_score = self.match_score(other);
        let view = self.lines_view;
        let (mut left, mut right) = view;

        loop {
            if left > right {
                return;
            }

            let mid = (left + right)/2;
            println!("left {} right {} mid {}", left, right, mid);
            self.view(mid, view.1);
            let next_score = self.match_score(other);
            println!("curr: {}, next: {}, start at {}", curr_score, next_score, mid);
            if next_score < curr_score {
                right = mid - 1;
            } else {
                left = mid + 1;
            }
            curr_score = next_score;
        }
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
    fn test_binsearch() {
        let license_text = "this is a license text\nor it pretends to be one\nit's just a test";
        let sample_text = "this is a license text\nor it pretends to be one\nit's just a test\n\n\nhere is some\ncode\nhello();\n\n//a comment too";
        let license = TextData::from(license_text).without_text();

        // try it out...
        let mut sample = TextData::from(sample_text);
        sample.binsearch_end(&license);
        println!("{:?}\n", sample.lines().unwrap());
        assert_eq!(3, sample.lines().unwrap().len(), "license is the first 3 lines");

        // do it again with an extra line to avoid math errors w/ int truncation
        let sample_text = format!("{}\none more line", sample_text);
        let mut sample = TextData::from(sample_text);
        sample.binsearch_end(&license);
        println!("{:?}\n", sample.lines().unwrap());
        assert_eq!(3, sample.lines().unwrap().len(), "license is still the first 3 lines");
        
        // this won't actually move, since the license is at the start
        sample.binsearch_start(&license);
        println!("{:?}", sample.lines().unwrap());
        assert_eq!(0, sample.lines_view.0, "view is at the start");
        assert_eq!(3, sample.lines().unwrap().len(), "license is yet again the first 3 lines");
    }
}