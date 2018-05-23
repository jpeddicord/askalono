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

use askalono::LicenseType;

#[derive(Debug)]
pub struct IdResult {
    pub score: f32,
    pub license: Option<IdLicense>,
    pub containing: Vec<ContainedResult>,
}

#[derive(Debug)]
pub struct IdLicense {
    pub name: String,
    pub kind: LicenseType,
    pub aliases: Vec<String>,
}

#[derive(Debug)]
pub struct ContainedResult {
    pub score: f32,
    pub license: IdLicense,
    pub line_range: (usize, usize),
}

impl fmt::Display for IdResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref license) = self.license {
            write!(
                f,
                "License: {} ({})\nScore: {:.3}\n",
                license.name, license.kind, self.score
            )?;
            if !license.aliases.is_empty() {
                write!(f, "Aliases: {}\n", license.aliases.join(", "))?;
            }
        } else {
            write!(f, "License: Unknown\nScore: {:.3}\n", self.score)?;
        }

        if self.containing.is_empty() {
            return Ok(());
        }
        write!(f, "Containing:\n")?;

        for res in &self.containing {
            write!(
                f,
                "  License: {} ({})\n  Score: {:.3}\n  Lines: {} - {}\n",
                res.license.name, res.license.kind, res.score, res.line_range.0, res.line_range.1
            )?;
            if !res.license.aliases.is_empty() {
                write!(f, "  Aliases: {}\n", res.license.aliases.join(", "))?;
            }
        }

        Ok(())
    }
}

