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
    lines_normalized: Option<Vec<String>>,
    lines_view: (usize, usize),
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
            lines_normalized: Some(normalized),
            lines_view: (0, normalized.len()),
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
            lines_normalized: None,
            lines_view: (0, 0),
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
            Some(ref lines) => Some(&lines),
            None => None,
        }
    }

    pub fn match_score(&self, other: &TextData) -> f32 {
        self.match_data.dice(&other.match_data)
    }

    fn vec_view(lines: &[String], start: usize, end: usize) -> &[String] {
        if end == 0 {
            lines
        } else {
            &lines[start .. end]
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
