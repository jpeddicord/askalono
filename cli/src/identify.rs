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

use std::fs::read_to_string;
use std::io::prelude::*;
use std::io::stdin;
use std::path::{Path, PathBuf};
use std::time::Instant;

use failure::{err_msg, Error};

use super::util::*;
use super::formats::*;
use askalono::{Store, TextData};

const MIN_SCORE: f32 = 0.8;

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
        cache_inst.elapsed().subsec_nanos() as f32 / 1_000_000.0
    );

    if !batch {
        let filename = filename.expect("no filename provided");
        let stdin_indicator: PathBuf = "-".into();
        let content = if filename == stdin_indicator {
            let mut buf = String::new();
            stdin().read_to_string(&mut buf)?;
            buf
        } else {
            read_to_string(filename)?
        };
        return identify_data(&store, &content.into(), optimize, want_diff).map(|res| {
            print!("{}", res);
            ()
        });
    }

    // batch mode: read stdin line by line until eof
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        if buf.is_empty() {
            break;
        }

        let filename: PathBuf = buf.trim().into();
        let content = match read_to_string(filename) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Input error: {}", e);
                continue;
            }
        };

        match identify_data(&store, &content.into(), optimize, want_diff) {
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

pub fn identify_data(
    store: &Store,
    text_data: &TextData,
    optimize: bool,
    want_diff: bool,
) -> Result<IdResult, Error> {
    let inst = Instant::now();
    let matched = store.analyze(&text_data)?;

    info!(
        "{:?} in {} ms",
        matched,
        inst.elapsed().subsec_nanos() as f32 / 1_000_000.0
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
        let (opt, score) = text_data.optimize_bounds(matched.data);
        let (lower, upper) = opt.lines_view();

        info!(
            "Optimized: {:?} in {} ms",
            matched,
            inst.elapsed().subsec_nanos() as f32 / 1_000_000.0
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
