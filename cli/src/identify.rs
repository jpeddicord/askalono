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
use std::fs::File;
use std::io::stdin;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

use failure::{err_msg, Error};

use askalono::{LicenseType, Store, TextData};
use super::util::*;

const MIN_SCORE: f32 = 0.8;

#[derive(Debug)]
pub struct IdResult {
    score: f32,
    license: Option<IdLicense>,
    containing: Vec<ContainedResult>,
}

#[derive(Debug)]
pub struct IdLicense {
    name: String,
    kind: LicenseType,
    aliases: Vec<String>,
}

#[derive(Debug)]
pub struct ContainedResult {
    score: f32,
    license: IdLicense,
    line_range: (usize, usize),
}

impl fmt::Display for IdResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref license) = self.license {
            write!(
                f,
                "License: {} ({})\nScore: {:.3}\n",
                license.name, license.kind, self.score
            )?;
            if license.aliases.len() > 0 {
                write!(f, "Aliases: {}\n", license.aliases.join(", "))?;
            }
        } else {
            write!(f, "License: Unknown\nScore: {:.3}\n", self.score)?;
        }

        if self.containing.len() == 0 {
            return Ok(());
        }
        write!(f, "Containing:\n")?;

        for res in &self.containing {
            write!(
                f,
                "  License: {} ({})\n  Score: {:.3}\n  Lines: {} - {}\n",
                res.license.name, res.license.kind, res.score, res.line_range.0, res.line_range.1
            )?;
            if res.license.aliases.len() > 0 {
                write!(f, "  Aliases: {}\n", res.license.aliases.join(", "))?;
            }
        }

        Ok(())
    }
}

pub fn identify(
    cache_filename: &Path,
    filename: Option<PathBuf>,
    optimize: bool,
    want_diff: bool,
    batch: bool,
) -> Result<(), Error> {
    // load the cache from disk or embedded data
    let cache_inst = Instant::now();
    let store = load_store(cache_filename)?;
    info!(
        "Cache loaded in {} ms",
        cache_inst.elapsed().subsec_nanos() as f32 / 1000_000.0
    );

    if !batch {
        let filename = filename.expect("no filename provided");
        let stdin_indicator: PathBuf = "-".into();
        let mut file: Box<Read> = if filename == stdin_indicator {
            Box::new(stdin())
        } else {
            Box::new(File::open(filename)?)
        };
        return match identify_file(&store, &mut file, optimize, want_diff) {
            Ok(res) => {
                print!("{}", res);
                Ok(())
            }
            Err(err) => Err(err),
        };
    }

    // batch mode: read stdin line by line until eof
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        if buf.is_empty() {
            break;
        }

        let filename: PathBuf = buf.trim().into();
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Input error: {}", e);
                continue;
            }
        };
        match identify_file(&store, &mut file, optimize, want_diff) {
            Ok(res) => {
                print!("{}", res);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        };
    }

    Ok(())
}

pub fn identify_file<R>(
    store: &Store,
    file: &mut R,
    optimize: bool,
    want_diff: bool,
) -> Result<IdResult, Error>
where
    R: Read + Sized,
{
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let text_data: TextData = text.into();

    let inst = Instant::now();
    let matched = store.analyze(&text_data)?;

    info!(
        "{:?} in {} ms",
        matched,
        inst.elapsed().subsec_nanos() as f32 / 1000_000.0
    );

    if want_diff {
        diff_result(&text_data, matched.data);
    }

    let mut output = IdResult {
        score: matched.score,
        license: None,
        containing: Vec::new(),
    };

    if matched.score > MIN_SCORE {
        output.license = Some(IdLicense {
            name: matched.name,
            kind: matched.license_type,
            aliases: matched.aliases,
        });

        return Ok(output);
    }

    // try again, optimizing for the current best match
    if optimize {
        let inst = Instant::now();
        let (opt, score) = text_data.optimize_bounds(&matched.data);
        let (lower, upper) = opt.lines_view();

        info!(
            "Optimized: {:?} in {} ms",
            matched,
            inst.elapsed().subsec_nanos() as f32 / 1000_000.0
        );

        if want_diff {
            diff_result(&opt, matched.data);
        }

        if score > MIN_SCORE {
            output.containing.push(ContainedResult {
                score,
                license: IdLicense {
                    name: matched.name,
                    kind: matched.license_type,
                    aliases: matched.aliases,
                },
                line_range: (lower + 1, upper), // inclusive range using 1-indexed numbers
            });
            return Ok(output);
        }
    }

    Err(err_msg(
        "Confidence threshold not high enough for any known license",
    ))
}
