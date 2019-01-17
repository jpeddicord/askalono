// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::read_to_string,
    io::{prelude::*, stdin},
    path::{Path, PathBuf},
    time::Instant,
};

use failure::{err_msg, Error};
use log::info;

use super::{commands::*, formats::*, util::*};
use askalono::{Store, TextData};

const MIN_SCORE: f32 = 0.8;

// TODO: replace a bunch of this logic with ScanStrategy https://github.com/amzn/askalono/issues/35

pub fn identify(
    cache_filename: &Path,
    output_format: &OutputFormat,
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

    // normal identification
    if !batch {
        let filename = filename.expect("no filename provided");
        let stdin_indicator: PathBuf = "-".into();
        let content = if filename == stdin_indicator {
            let mut buf = String::new();
            stdin().read_to_string(&mut buf)?;
            buf
        } else {
            read_to_string(&filename)?
        };

        let idres = identify_data(&store, &content.into(), optimize, want_diff);
        let file_lossy = filename.to_string_lossy();
        // NLL plz
        {
            let fileres = FileResult::from_identification_result(&file_lossy, &idres);
            fileres.print_as(&output_format, false);
        }

        return idres.map(|_| ());
    }

    // batch mode: read stdin line by line until eof.
    // don't bubble up errors; just print to stderr
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
                let fileres = FileResult::Err {
                    path: &buf,
                    error: format!("Input error: {}", e),
                };
                fileres.print_as(&output_format, false);
                continue;
            }
        };

        let idres = identify_data(&store, &content.into(), optimize, want_diff);
        let fileres = FileResult::from_identification_result(&buf, &idres);
        fileres.print_as(&output_format, false);
    }

    Ok(())
}

pub fn identify_data(
    store: &Store,
    text_data: &TextData,
    optimize: bool,
    want_diff: bool,
) -> Result<Identification, Error> {
    let inst = Instant::now();
    let matched = store.analyze(&text_data);

    info!(
        "{:?} in {} ms",
        matched,
        inst.elapsed().subsec_nanos() as f32 / 1_000_000.0
    );

    if want_diff {
        diff_result(&text_data, matched.data);
    }

    let mut output = Identification {
        score: matched.score,
        license: None,
        containing: Vec::new(),
    };

    if matched.score > MIN_SCORE {
        output.license = Some(IdentifiedLicense {
            aliases: store.aliases(&matched.name).unwrap().clone(),
            name: matched.name,
            kind: matched.license_type,
        });

        return Ok(output);
    }

    // try again, optimizing for the current best match
    if optimize {
        let inst = Instant::now();
        let (opt, score) = text_data.optimize_bounds(matched.data);
        let (lower, upper) = opt.lines_view();

        info!(
            "Optimized to {} lines ({}, {}) in {} ms",
            score,
            lower,
            upper,
            inst.elapsed().subsec_nanos() as f32 / 1_000_000.0
        );

        if want_diff {
            diff_result(&opt, matched.data);
        }

        if score > MIN_SCORE {
            output.containing.push(ContainedResult {
                score,
                license: IdentifiedLicense {
                    aliases: store.aliases(&matched.name).unwrap().clone(),
                    name: matched.name,
                    kind: matched.license_type,
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
