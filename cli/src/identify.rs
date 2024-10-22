// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::read_to_string,
    io::{prelude::*, stdin},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{format_err, Error};
use log::info;

use super::{commands::*, formats::*, util::*};
use askalono::{ScanMode, ScanStrategy, Store, TextData};

const MIN_SCORE: f32 = 0.8;

pub fn identify(
    cache_filename: &Path,
    output_format: &OutputFormat,
    filename: Option<PathBuf>,
    optimize: bool,
    want_diff: bool,
    batch: bool,
    topdown: bool,
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

        let idres = identify_data(&store, &content.into(), optimize, want_diff, topdown);
        let file_lossy = filename.to_string_lossy();
        let fileres = FileResult::from_identification_result(&file_lossy, &idres);
        fileres.print_as(output_format, false);

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
                fileres.print_as(output_format, false);
                continue;
            }
        };

        let idres = identify_data(&store, &content.into(), optimize, want_diff, topdown);
        let fileres = FileResult::from_identification_result(&buf, &idres);
        fileres.print_as(output_format, false);
    }

    Ok(())
}

pub fn identify_data(
    store: &Store,
    text_data: &TextData,
    optimize: bool,
    want_diff: bool,
    topdown: bool,
) -> Result<CLIIdentification, Error> {
    let inst = Instant::now();
    let scan_mode = if topdown {
        ScanMode::TopDown
    } else {
        ScanMode::Elimination
    };

    let strategy = ScanStrategy::new(store)
        .mode(scan_mode)
        .confidence_threshold(MIN_SCORE)
        .optimize(optimize)
        .max_passes(1);
    let result = strategy.scan(text_data)?;

    info!(
        "{:?} in {} ms",
        result,
        inst.elapsed().subsec_nanos() as f32 / 1_000_000.0
    );

    // start building an output structure to print
    let mut output = CLIIdentification {
        score: result.score,
        license: None,
        containing: result
            .containing
            .iter()
            .map(|cr| CLIContainedResult {
                score: cr.score,
                license: CLIIdentifiedLicense {
                    aliases: store.aliases(cr.license.name).unwrap().clone(),
                    name: cr.license.name.to_owned(),
                    kind: cr.license.kind,
                },
                line_range: cr.line_range,
            })
            .collect(),
    };

    // include the overall license if present
    if let Some(license) = result.license {
        output.license = Some(CLIIdentifiedLicense {
            aliases: store.aliases(license.name).unwrap().clone(),
            name: license.name.to_owned(),
            kind: license.kind,
        });

        if want_diff {
            diff_result(text_data, license.data);
        }

        return Ok(output);
    }

    // not a good enough match overall, but maybe inside
    if !output.containing.is_empty() {
        if want_diff {
            diff_result(text_data, result.containing[0].license.data);
        }
        return Ok(output);
    }

    Err(format_err!(
        "Confidence threshold not high enough for any known license",
    ))
}
