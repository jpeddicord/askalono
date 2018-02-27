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
    text_normalized: Option<String>,
    text_processed: Option<String>,
}

impl TextData {
    pub fn new(text: &str) -> TextData {
        let normalized = apply_normalizers(text);
        let processed = apply_aggressive(&normalized);
        let bigrams = NgramSet::from_str(&processed, 2);

        TextData {
            match_data: bigrams,
            text_normalized: Some(normalized),
            text_processed: Some(processed),
        }
    }

    pub fn without_text(self) -> Self {
        TextData {
            match_data: self.match_data,
            text_normalized: None,
            text_processed: None,
        }
    }

    // TODO: Cow<str>?
    pub fn text(&self) -> Option<&str> {
        match self.text_normalized {
            Some(ref t) => Some(t.as_str()),
            None => None,
        }
    }

    pub fn match_score(&self, other: &TextData) -> f32 {
        self.match_data.dice(&other.match_data)
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
