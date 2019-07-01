// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use askalono::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, Read};

enum Annotation {
    Begin(String),
    End,
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: annotate-text cache.bin.zstd < input.txt > output.html");
        std::process::exit(1);
    }

    let cache = &args[1];
    let store = Store::from_cache(File::open(cache).expect("couldn't read cache file"))
        .expect("error parsing cache");

    let mut buf = String::new();
    stdin()
        .read_to_string(&mut buf)
        .expect("couldn't read stdin");
    let strategy = ScanStrategy::new(&store)
        .mode(ScanMode::TopDown)
        .confidence_threshold(0.80);
    let results = strategy
        .scan(&TextData::new(&buf))
        .expect("scan didn't complete successfully");

    let mut annotations = HashMap::with_capacity(results.containing.len() * 2);
    for result in &results.containing {
        annotations.insert(
            result.line_range.0,
            Annotation::Begin(result.license.name.to_owned()),
        );
        annotations.insert(result.line_range.1, Annotation::End);
    }

    println!("<html><body>");

    println!("<pre>{:#?}</pre>", results);

    println!("<pre>");
    for (i, line) in buf.lines().enumerate() {
        if annotations.contains_key(&i) {
            let a = annotations.get(&i).unwrap();
            match a {
                Annotation::Begin(license) => {
                    print!(
                        r#"<div style="background-color: rgba(50, 50, 255, 0.3)" title="{}">"#,
                        license
                    );
                }
                Annotation::End => {
                    print!("</div>");
                }
            }
        }
        println!("{}", line);
    }

    println!("</pre></body></html>");
}
