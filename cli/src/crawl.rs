// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{fs::read_to_string, path::Path};

use failure::Error;
use ignore::Error as IgnoreError;

use askalono::TextData;

use super::{commands::*, formats::*, identify::identify_data, util::*};

pub fn crawl(
    cache_filename: &Path,
    output_format: &OutputFormat,
    directory: &Path,
    optimize: bool,
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
        .build()
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(error) => {
                if let IgnoreError::WithPath { path, err } = error {
                    FileResult::from_error(&path.to_string_lossy(), err)
                        .print_as(&output_format, true);
                } else {
                    FileResult::from_error("", error).print_as(&output_format, false);
                }
                None
            }
        })
        .filter(|entry| !entry.metadata().unwrap().is_dir())
        .for_each(|entry| {
            let path = entry.path();
            let path_lossy = path.to_string_lossy();

            match read_to_string(path) {
                Ok(content) => {
                    let data = TextData::new(&content);
                    let idres = identify_data(&store, &data, optimize, false);
                    let fileres = FileResult::from_identification_result(&path_lossy, &idres);
                    fileres.print_as(&output_format, true);
                }
                Err(err) => {
                    FileResult::from_error(&path_lossy, err).print_as(&output_format, true);
                }
            };
        });

    Ok(())
}
