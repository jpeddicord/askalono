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

use std::fs::File;
use std::io::stdin;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

use failure::{err_msg, Error};

use askalono::{Store, TextData};
use super::util::*;

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
        return identify_file(&store, &mut file, optimize, want_diff);
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
        identify_file(&store, &mut file, optimize, want_diff).unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
        });
    }

    Ok(())
}

pub fn identify_file<R>(
    store: &Store,
    file: &mut R,
    optimize: bool,
    want_diff: bool,
) -> Result<(), Error>
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

    if matched.score > MIN_SCORE {
        println!("License: {} ({})", matched.name, matched.license_type);
        println!("Score: {:.3}", matched.score);

        if matched.aliases.len() > 0 {
            println!("Aliases: {}", matched.aliases.join(", "));
        }

        return Ok(());
    }

    println!("License: Unknown");

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
            println!(
                "But, there's probably {} ({}) at lines {} - {} with a score of {:.3}",
                matched.name,
                matched.license_type,
                lower + 1,
                upper,
                score
            );
            return Ok(());
        }
    }

    Err(err_msg(
        "Confidence threshold not high enough for any known license",
    ))
}
