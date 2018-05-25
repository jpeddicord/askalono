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
use std::fmt::Display;

use failure::Error;

use super::commands::*;
use askalono::LicenseType;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum FileResult<'a> {
    Ok {
        path: &'a str,
        result: &'a Identification,
    },
    Err {
        path: &'a str,
        error: String,
    },
}

#[derive(Serialize, Debug)]
pub struct Identification {
    pub score: f32,
    pub license: Option<IdentifiedLicense>,
    pub containing: Vec<ContainedResult>,
}

#[derive(Serialize, Debug)]
pub struct IdentifiedLicense {
    pub name: String,
    pub kind: LicenseType,
    pub aliases: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct ContainedResult {
    pub score: f32,
    pub license: IdentifiedLicense,
    pub line_range: (usize, usize),
}

impl<'a> FileResult<'a> {
    pub fn from_identification_result(
        path: &'a str,
        result: &'a Result<Identification, Error>,
    ) -> FileResult<'a> {
        match result {
            Ok(id) => FileResult::Ok { path, result: &id },
            Err(e) => FileResult::Err {
                path,
                error: format!("{}", e),
            },
        }
    }

    pub fn from_error(path: &'a str, error: impl Display) -> FileResult<'a> {
        FileResult::Err {
            path,
            error: format!("{}", error),
        }
    }

    pub fn print_as(&self, output_format: &OutputFormat, show_path: bool) {
        match output_format {
            // with the default text format, follow the unixy conventions of
            // printing successes to stdout and errors to stderr
            OutputFormat::text => match self {
                FileResult::Ok { .. } => println!("{}", self.as_text(show_path)),
                FileResult::Err { .. } => eprintln!("{}", self.as_text(show_path)),
            },
            // for json format, print everything to stdout to ease
            // parsing consistency
            OutputFormat::json => println!("{}", self.as_json()),
        }
    }

    fn as_text(&self, show_path: bool) -> String {
        match self {
            FileResult::Ok { path, result } => if show_path {
                format!("{}\n{}", path, result)
            } else {
                format!("{}", result)
            },
            FileResult::Err { path, error } => if show_path {
                format!("{}\nError: {}", path, error)
            } else {
                format!("Error: {}", error)
            },
        }
    }

    fn as_json(&self) -> String {
        use serde_json;
        serde_json::to_string(self).expect("must produce valid json output")
    }
}

impl fmt::Display for Identification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref license) = self.license {
            write!(
                f,
                "License: {} ({})\nScore: {:.3}",
                license.name, license.kind, self.score
            )?;
            if !license.aliases.is_empty() {
                write!(f, "Aliases: {}\n", license.aliases.join(", "))?;
            }
        } else {
            write!(f, "License: Unknown\nScore: {:.3}", self.score)?;
        }

        if self.containing.is_empty() {
            return Ok(());
        }
        write!(f, "\nContaining:")?;

        for res in &self.containing {
            write!(
                f,
                "\n  License: {} ({})\n  Score: {:.3}\n  Lines: {} - {}",
                res.license.name, res.license.kind, res.score, res.line_range.0, res.line_range.1
            )?;
            if !res.license.aliases.is_empty() {
                write!(f, "\n  Aliases: {}", res.license.aliases.join(", "))?;
            }
        }

        Ok(())
    }
}
