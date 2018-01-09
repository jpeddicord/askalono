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

#![allow(dead_code)]

extern crate askalono;
#[macro_use]
extern crate clap;
extern crate difference;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rayon;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::process::exit;
use std::time::Instant;

use clap::{App, ArgMatches};

use askalono::{LicenseContent, Store};

const MIN_SCORE: f32 = 0.8;

#[cfg(feature = "embedded-cache")]
static CACHE_DATA: &'static [u8] = include_bytes!(env!("ASKALONO_EMBEDDED_CACHE"));

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

    env_logger::init().unwrap();
    rayon::initialize(rayon::Configuration::new()).unwrap();

    let cache_file = matches
        .value_of("cache")
        .unwrap_or("./askalono-cache.bin.gz");

    match matches.subcommand_name() {
        Some("identify") => identify(matches.subcommand_matches("identify").unwrap(), cache_file),
        Some("cache") => cache(matches.subcommand_matches("cache").unwrap(), cache_file),
        _ => unreachable!(),
    }.unwrap();
}

#[allow(unused_variables)]
fn identify(matches: &ArgMatches, cache_file: &str) -> Result<(), Box<Error>> {
    let filename = matches.value_of("FILE").unwrap();
    let want_diff = matches.is_present("diff");

    // load the cache from disk or embedded data
    let cache_inst = Instant::now();
    #[cfg(feature = "embedded-cache")]
    let store = *Store::from_cache(CACHE_DATA)?;
    #[cfg(not(feature = "embedded-cache"))]
    let store = *Store::from_cache_file(cache_file)?;
    info!(
        "Cache loaded in {} ms",
        cache_inst.elapsed().subsec_nanos() as f32 / 1000_000.0
    );

    let mut f = File::open(&filename)?;
    let mut text = String::new();
    f.read_to_string(&mut text)?;
    let content = LicenseContent::from_text(&text, want_diff);

    let inst = Instant::now();
    let matched = store.analyze_content(&content);

    info!(
        "{:?} in {} ms",
        matched,
        inst.elapsed().subsec_nanos() as f32 / 1000_000.0
    );

    if want_diff {
        diff_result(&content, matched.content);
    }

    if matched.score > MIN_SCORE {
        println!("License: {}", matched.name);
        println!("Score: {}", matched.score);
    } else {
        println!("License: Unknown");
        println!("Confidence threshold not high enough for any known license");
        exit(1);
    }

    Ok(())
}

fn cache(matches: &ArgMatches, cache_file: &str) -> Result<(), Box<Error>> {
    // TODO
    cache_load_spdx(matches.subcommand_matches("load-spdx").unwrap(), cache_file)
}

fn cache_load_spdx(matches: &ArgMatches, cache_file: &str) -> Result<(), Box<Error>> {
    info!("Processing licenses...");
    let mut store = Store::new();
    store.load_spdx(
        matches.value_of("DIR").unwrap(),
        matches.is_present("store-texts"),
    )?;
    store.save_cache_file(cache_file)?;
    Ok(())
}

fn diff_result(license: &LicenseContent, other: &LicenseContent) {
    use difference::Changeset;

    let license_texts = &license.texts.as_ref().expect("license texts is Some");
    let other_texts = &other.texts.as_ref().expect("other texts is Some");

    let processed = Changeset::new(&license_texts.processed, &other_texts.processed, " ");
    println!(
        "{}\n\n---\n\n{}\n\n---\n\n{}",
        &license_texts.processed, &other_texts.processed, processed
    );
}
