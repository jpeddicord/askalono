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
extern crate difference;
extern crate env_logger;
extern crate failure;
extern crate ignore;
#[macro_use]
extern crate log;
extern crate rayon;
#[macro_use]
extern crate structopt;

use failure::{err_msg, Error};
use std::fs::File;
use std::io::stdin;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Instant;

use structopt::StructOpt;

use askalono::{Store, TextData};

const MIN_SCORE: f32 = 0.8;

#[cfg(feature = "embedded-cache")]
static CACHE_DATA: &'static [u8] = include_bytes!(env!("ASKALONO_EMBEDDED_CACHE"));

#[derive(StructOpt)]
#[structopt(name = "askalono")]
struct Opt {
    #[structopt(long = "cache", short = "c", parse(from_os_str))]
    cache: Option<PathBuf>,
    #[structopt(subcommand)]
    subcommand: Subcommand,
}

#[derive(StructOpt)]
enum Subcommand {
    #[structopt(name = "identify", alias = "id")]
    Identify {
        #[structopt(name = "FILE", help = "file to identify", required_unless = "batch",
                    parse(from_os_str))]
        filename: Option<PathBuf>,
        #[structopt(long = "optimize", short = "o",
                    help = "try to find the location of a license within the file")]
        optimize: bool,
        #[structopt(long = "diff", help = "print a colored diff of match (debugging feature)")]
        diff: bool,
        // #[structopt(long = "output", short = "o", help = "output type")]
        // output: Option<OutputType>, // "json"
        #[structopt(long = "batch", short = "b", help = "read in filenames on stdin")]
        batch: bool,
    },
    #[structopt(name = "crawl")]
    Crawl {
        #[structopt(name = "DIR", help = "directory to crawl", parse(from_os_str))]
        directory: PathBuf,
        #[structopt(long = "follow", help = "follow symlinks")]
        follow_links: bool,
        #[structopt(long = "glob",
                    help = "glob of files to check (defaults to license-like files)")]
        glob: Option<String>,
    },
    #[structopt(name = "cache")]
    Cache {
        #[structopt(subcommand)]
        subcommand: CacheSubcommand,
    },
}

#[derive(StructOpt)]
enum CacheSubcommand {
    #[structopt(name = "load-spdx")]
    LoadSpdx {
        #[structopt(name = "DIR", help = "JSON details directory", parse(from_os_str))]
        dir: PathBuf,
        #[structopt(long = "store", help = "store texts in cache along with match data")]
        store_texts: bool,
    },
}

fn main() {
    let options = Opt::from_args();

    env_logger::init();
    rayon::ThreadPoolBuilder::new().build_global().unwrap();

    let cache_file = options
        .cache
        .unwrap_or_else(|| "./askalono-cache.bin.gz".into());

    if let Err(e) = match options.subcommand {
        Subcommand::Identify {
            filename,
            optimize,
            diff,
            batch,
        } => identify(&cache_file, filename, optimize, diff, batch),
        Subcommand::Crawl {
            directory,
            follow_links,
            glob,
        } => crawl(
            &cache_file,
            &directory,
            follow_links,
            glob.as_ref().map(String::as_str),
        ),
        Subcommand::Cache { subcommand } => cache(&cache_file, subcommand),
    } {
        eprintln!("{}", e);
        exit(1);
    }
}

#[allow(unused_variables)]
fn load_store(cache_filename: &Path) -> Result<Store, Error> {
    #[cfg(feature = "embedded-cache")]
    let store = Store::from_cache(CACHE_DATA)?;

    #[cfg(not(feature = "embedded-cache"))]
    let store = Store::from_cache_file(cache_filename)?;

    Ok(store)
}

fn identify(
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

fn identify_file<R>(
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

fn crawl(
    cache_filename: &Path,
    directory: &Path,
    follow_links: bool,
    glob: Option<&str>,
) -> Result<(), Error> {
    use ignore::types::TypesBuilder;
    use ignore::WalkBuilder;

    let store = load_store(cache_filename)?;

    let mut types_builder = TypesBuilder::new();
    if let Some(globstr) = glob {
        types_builder.add("custom", globstr)?;
        types_builder.select("custom");
    } else {
        types_builder.add_defaults();
        types_builder.select("license");
    }
    let matcher = types_builder.build().unwrap();

    WalkBuilder::new(directory)
        .types(matcher)
        .follow_links(follow_links)
        .build() // TODO: build_parallel? see if it's faster overall, or if it just chokes the ID threads
        .filter_map(|e| match e.is_ok() {
            true => Some(e),
            false => {
                eprintln!("{}", e.unwrap_err());
                None
            }
        })
        .filter(|e| match e {
            &Ok(ref entry) => !entry.metadata().unwrap().is_dir(),
            &Err(_) => false,
        })
        .for_each(|e| {
            let entry = e.unwrap();
            let path = entry.path();
            println!("{}", path.display());

            if let Ok(mut reader) = File::open(path) {
                identify_file(&store, &mut reader, false, false).unwrap_or_else(|err| {
                    eprintln!("Error: {}", err);
                });
            }
        });

    Ok(())
}

fn cache(cache_filename: &Path, subcommand: CacheSubcommand) -> Result<(), Error> {
    // TODO
    match subcommand {
        CacheSubcommand::LoadSpdx { dir, store_texts } => {
            cache_load_spdx(cache_filename, &dir, store_texts)
        }
    }
}

fn cache_load_spdx(
    cache_filename: &Path,
    directory: &Path,
    store_texts: bool,
) -> Result<(), Error> {
    info!("Processing licenses...");
    let mut store = Store::new();
    store.load_spdx(directory, store_texts)?;
    let cache_file = File::create(cache_filename)?;
    store.to_cache(&cache_file)?;
    Ok(())
}

fn diff_result(license: &TextData, other: &TextData) {
    use difference::Changeset;

    let license_texts = &license.lines().expect("license texts is Some").join("\n");
    let other_texts = &other.lines().expect("other texts is Some").join("\n");

    let processed = Changeset::new(license_texts, other_texts, " ");
    println!(
        "{}\n\n---\n\n{}\n\n---\n\n{}",
        &license_texts, &other_texts, processed
    );
}
