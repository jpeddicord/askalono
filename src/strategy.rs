// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// TODO: Investigate top-down scanning strategy for things like attribution
/// docuements: - Take the entire text and shrink the view down to (0,
/// chunk_size) where   chunk_size is a sane value to increment by. Perhaps 10
/// lines? - Attempt to identify.
/// - White out identified texts (this may not work if the license got chopped
///   off -- figure out how to deal with that)
/// - Grow the region.
/// - Repeat from identification step above.
///
/// This may require extra integration work in TextData. This also may not be
/// necessary at all! I think computing the dice coefficient & optimizing (as
/// ScanStrategy does) should still work fine, but I wonder
/// if I'm missing something real-world. Backup plans.
use std::borrow::Cow;

use failure::Error;

use license::LicenseType;
use license::TextData;
use store::{Match, Store};

/// A struct describing a license that was identified, as well as its type.
#[derive(Serialize, Debug)]
pub struct IdentifiedLicense {
    /// The identifier of the license.
    pub name: String,
    /// The type of the license that was matched.
    pub kind: LicenseType,
}

/// Information about scanned content.
///
/// Produced by `ScanStrategy.scan`.
#[derive(Serialize, Debug)]
pub struct ScanResult {
    /// The confidence of the match from 0.0 to 1.0.
    pub score: f32,
    /// The identified license of the overall text, or None if nothing met the
    /// confidence threshold.
    pub license: Option<IdentifiedLicense>,
    /// Any licenses discovered inside the text, if `optimize` was enabled.
    pub containing: Vec<ContainedResult>,
}

/// A struct describing a single license identified within a larger text.
#[derive(Serialize, Debug)]
pub struct ContainedResult {
    /// The confidence of the match within the line range from 0.0 to 1.0.
    pub score: f32,
    /// The license identified in this portion of the text.
    pub license: IdentifiedLicense,
    /// A 0-indexed (inclusive, exclusive) range of line numbers identifying
    /// where in the overall text a license was identified.
    ///
    /// See `TextData.lines_view()` for more information.
    pub line_range: (usize, usize),
}

/// A `ScanStrategy` can be used as a high-level wrapped over a `Store`'s
/// analysis logic.
///
/// A strategy configured here can be run repeatedly to scan a document for
/// multiple licenses, or to automatically optimize to locate texts within a
/// larger text.
///
/// # Examples
///
/// ```rust,should_panic
/// # use std::error::Error;
/// use askalono::{ScanStrategy, Store};
///
/// # fn main() -> Result<(), Box<Error>> {
/// let store = Store::new();
/// // [...]
/// let strategy = ScanStrategy::new(&store)
///     .confidence_threshold(0.9)
///     .optimize(true);
/// let results = strategy.scan(&"my text to scan".into())?;
/// # Ok(())
/// # }
/// ```
pub struct ScanStrategy<'a> {
    store: &'a Store,
    mode: ScanMode,
    confidence_threshold: f32,
    shallow_limit: f32,
    optimize: bool,
    max_passes: u16,
    step_size: usize,
}

/// Available scanning strategy modes.
pub enum ScanMode {
    /// Elimination is a general-purpose strategy that iteratively locates the
    /// highest license match in a file, then the next, and so on until not
    /// finding any more strong matches.
    Elimination,

    /// TopDown is a strategy intended for use with attribution documents, or
    /// text files containing multiple licenses (and not much else). It's more
    /// accurate than Elimination, but significantly slower.
    TopDown,
    // Smart, // TODO
}

impl<'a> ScanStrategy<'a> {
    /// Construct a new scanning strategy tied to the given `Store`.
    ///
    /// By default, the strategy has conservative defaults and won't perform
    /// any deeper investigaton into the contents of files.
    pub fn new(store: &'a Store) -> ScanStrategy<'a> {
        Self {
            store,
            mode: ScanMode::Elimination,
            confidence_threshold: 0.9,
            shallow_limit: 0.99,
            optimize: false,
            max_passes: 10,
            step_size: 5,
        }
    }

    pub fn mode(mut self, mode: ScanMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the confidence threshold for this strategy.
    ///
    /// The overall license match must meet this number in order to be
    /// reported. Additionally, if contained licenses are reported in the scan
    /// (when `optimize` is enabled), they'll also need to meet this bar.
    ///
    /// Set this to 1.0 for only exact matches, and 0.0 to report even the
    /// weakest match.
    pub fn confidence_threshold(mut self, confidence_threshold: f32) -> Self {
        self.confidence_threshold = confidence_threshold;
        self
    }

    /// Set a fast-exit parameter that allows the strategy to skip the rest of
    /// a scan for strong matches.
    ///
    /// This should be set higher than the confidence threshold; ideally close
    /// to 1.0. If the overall match score is above this limit, the scanner
    /// will return early and not bother performing deeper checks.
    ///
    /// This is really only useful in conjunction with `optimize`. A value of
    /// 0.0 will fast-return on any match meeting the confidence threshold,
    /// while a value of 1.0 will only stop on a perfect match.
    pub fn shallow_limit(mut self, shallow_limit: f32) -> Self {
        self.shallow_limit = shallow_limit;
        self
    }

    /// Indicate whether a deeper scan should be performed.
    ///
    /// This is ignored if the shallow limit is met. It's not enabled by
    /// default, however, so if you want deeper results you should set
    /// `shallow_limit` fairly high and enable this.
    pub fn optimize(mut self, optimize: bool) -> Self {
        self.optimize = optimize;
        self
    }

    /// The maximum number of identifications to perform before exiting a scan
    /// of a single text.
    ///
    /// This is largely to prevent misconfigurations and infinite loop
    /// scenarios, but if you have a document with a large number of licenses
    /// then you may want to tune this to a value above the number of licenses
    /// you expect to be identified.
    pub fn max_passes(mut self, max_passes: u16) -> Self {
        self.max_passes = max_passes;
        self
    }

    pub fn step_size(mut self, step_size: usize) -> Self {
        self.step_size = step_size;
        self
    }

    /// Scan the given text content using this strategy's configured
    /// preferences.
    ///
    /// Returns a `ScanResult` containing all discovered information.
    pub fn scan(&self, text: &TextData) -> Result<ScanResult, Error> {
        match self.mode {
            ScanMode::Elimination => self.scan_elimination(text),
            ScanMode::TopDown => self.scan_topdown(text),
            // ScanMode::Smart => unimplemented!(), // TODO... and make this the default
        }
    }

    fn scan_elimination(&self, text: &TextData) -> Result<ScanResult, Error> {
        let mut analysis = self.store.analyze(text)?;
        let score = analysis.score;
        let mut license = None;
        let mut containing = Vec::new();

        // meets confidence threshold? record that
        if analysis.score > self.confidence_threshold {
            license = Some(IdentifiedLicense {
                name: analysis.name.clone(),
                kind: analysis.license_type,
            });

            // above the shallow limit -> exit
            if analysis.score > self.shallow_limit {
                return Ok(ScanResult {
                    score,
                    license,
                    containing,
                });
            }
        }

        if self.optimize {
            // repeatedly try to dig deeper
            // this loop effectively iterates once for each license it finds
            let mut current_text: Cow<TextData> = Cow::Borrowed(text);
            for _n in 0..self.max_passes {
                let (optimized, optimized_score) = current_text.optimize_bounds(analysis.data);

                // stop if we didn't find anything acceptable
                if optimized_score < self.confidence_threshold {
                    break;
                }

                // otherwise, save it
                containing.push(ContainedResult {
                    score: optimized_score,
                    license: IdentifiedLicense {
                        name: analysis.name,
                        kind: analysis.license_type,
                    },
                    line_range: optimized.lines_view(),
                });

                // and white-out + reanalyze for next iteration
                current_text = Cow::Owned(optimized.white_out().expect("optimized must have text"));
                analysis = self.store.analyze(&current_text)?;
            }
        }

        Ok(ScanResult {
            score,
            license,
            containing,
        })
    }

    fn scan_topdown(&self, text: &TextData) -> Result<ScanResult, Error> {
        let (_, text_end) = text.lines_view();
        let mut containing = Vec::new();

        // find licenses working down thru the text's lines
        let mut current_start = 0usize;
        while current_start < text_end {
            let result = self.topdown_find_contained_license(text, current_start)?;

            let contained = match result {
                Some(c) => c,
                None => break,
            };

            current_start = contained.line_range.1 + 1;
            containing.push(contained);
        }

        Ok(ScanResult {
            score: 0.0,
            license: None,
            containing,
        })
    }

    fn topdown_find_contained_license(
        &self,
        text: &TextData,
        starting_at: usize,
    ) -> Result<Option<ContainedResult>, Error> {
        let (_, text_end) = text.lines_view();
        let mut found: (usize, usize, Option<Match>) = (0, 0, None);

        trace!(
            "topdown_find_contained_license starting at line {}",
            starting_at
        );

        // TODO: areas for improvement
        // - use something different from the confidence threshold for the start/end
        // loops, something more relaxed
        // - once above the relaxed threshold, change the step interval to
        // line-by-line to ensure we don't skip over anything

        // speed: only start tracking once conf is met, and bail out after
        let mut hit_threshold = false;

        // move the start of window...
        'start: for start in (starting_at..text_end).step_by(self.step_size) {
            // ...and also the end of window to find high scores.
            'end: for end in (start..=text_end).step_by(self.step_size) {
                let view = text.with_view(start, end).expect("view missing text");
                let analysis = self.store.analyze(&view)?;

                // just getting a feel for the data at this point, not yet
                // optimizing the view.

                // entering threshold: save the starting location
                if !hit_threshold && analysis.score >= self.confidence_threshold {
                    hit_threshold = true;
                    trace!(
                        "hit_threshold at ({}, {}) with score {}",
                        start,
                        end,
                        analysis.score
                    );
                }

                if hit_threshold {
                    if analysis.score < self.confidence_threshold {
                        // exiting threshold
                        trace!(
                            "exiting threshold at ({}, {}) with score {}",
                            start,
                            end,
                            analysis.score
                        );
                        break 'start;
                    } else {
                        // maintaining threshold (also true for entering)
                        found = (start, end, Some(analysis));
                    }
                }
            }
        }

        // at this point we have a *rough* bounds for a match.
        // now we can optimize to find the best one
        let matched = match found.2 {
            Some(m) => m,
            None => return Ok(None),
        };
        let check = matched.data;
        let view = text.with_view(found.0, found.1).expect("view missing text");
        let (optimized, optimized_score) = view.optimize_bounds(check);

        trace!(
            "optimized {} {} at ({:?})",
            optimized_score,
            matched.name,
            optimized.lines_view()
        );

        if optimized_score < self.confidence_threshold {
            return Ok(None);
        }

        Ok(Some(ContainedResult {
            score: optimized_score,
            license: IdentifiedLicense {
                name: matched.name,
                kind: matched.license_type,
            },
            line_range: optimized.lines_view(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate env_logger;

    #[test]
    fn can_construct() {
        let store = Store::new();
        ScanStrategy::new(&store);
        ScanStrategy::new(&store).confidence_threshold(0.5);
        ScanStrategy::new(&store)
            .shallow_limit(0.99)
            .optimize(true)
            .max_passes(100);
    }

    #[test]
    fn shallow_scan() {
        let store = create_dummy_store();
        let test_data = TextData::new("lorem ipsum\naaaaa bbbbb\nccccc\nhello");

        // the above text should have a result with a confidence minimum of 0.5
        let strategy = ScanStrategy::new(&store)
            .confidence_threshold(0.5)
            .shallow_limit(0.0);
        let result = strategy.scan(&test_data).unwrap();
        assert!(
            result.score > 0.5,
            format!("score must meet threshold; was {}", result.score)
        );
        assert_eq!(
            result.license.expect("result has a license").name,
            "license-1"
        );

        // but it won't pass with a threshold of 0.8
        let strategy = ScanStrategy::new(&store)
            .confidence_threshold(0.8)
            .shallow_limit(0.0);
        let result = strategy.scan(&test_data).unwrap();
        assert!(result.license.is_none(), "result license is None");
    }

    #[test]
    fn single_optimize() {
        let store = create_dummy_store();
        // this TextData matches license-2 with an overall score of ~0.46 and optimized
        // score of ~0.57
        let test_data =
            TextData::new("lorem\nipsum abc def ghi jkl\n1234 5678 1234\n0000\n1010101010\n\n8888 9999\nwhatsit hello\narst neio qwfp colemak is the best keyboard layout");

        // check that we can spot the gibberish license in the sea of other gibberish
        let strategy = ScanStrategy::new(&store)
            .confidence_threshold(0.5)
            .optimize(true)
            .shallow_limit(1.0);
        let result = strategy.scan(&test_data).unwrap();
        assert!(result.license.is_none(), "result license is None");
        assert_eq!(result.containing.len(), 1);
        let contained = &result.containing[0];
        assert_eq!(contained.license.name, "license-2");
        assert!(
            contained.score > 0.5,
            "contained score is greater than threshold"
        );
    }

    #[test]
    fn find_multiple_licenses_elimination() {
        let store = create_dummy_store();
        // this TextData matches license-2 with an overall score of ~0.46 and optimized
        // score of ~0.57
        let test_data =
            TextData::new("lorem\nipsum abc def ghi jkl\n1234 5678 1234\n0000\n1010101010\n\n8888 9999\nwhatsit hello\narst neio qwfp colemak is the best keyboard layout\naaaaa\nbbbbb\nccccc");

        // check that we can spot the gibberish license in the sea of other gibberish
        let strategy = ScanStrategy::new(&store)
            .mode(ScanMode::Elimination)
            .confidence_threshold(0.5)
            .optimize(true)
            .shallow_limit(1.0);
        let result = strategy.scan(&test_data).unwrap();
        assert!(result.license.is_none(), "result license is None");
        assert_eq!(2, result.containing.len());

        // inspect the array and ensure we got both licenses
        let mut found1 = 0;
        let mut found2 = 0;
        for (_, ref contained) in result.containing.iter().enumerate() {
            match contained.license.name.as_ref() {
                "license-1" => {
                    assert!(contained.score > 0.5, "license-1 score meets threshold");
                    found1 += 1;
                }
                "license-2" => {
                    assert!(contained.score > 0.5, "license-2 score meets threshold");
                    found2 += 1;
                }
                _ => {
                    panic!("somehow got an unknown license name");
                }
            }
        }

        assert!(
            found1 == 1 && found2 == 1,
            "found both licenses exactly once"
        );
    }

    #[test]
    fn find_multiple_licenses_topdown() {
        env_logger::init();

        let store = create_dummy_store();
        // this TextData matches license-2 with an overall score of ~0.46 and optimized
        // score of ~0.57
        let test_data =
            TextData::new("lorem\nipsum abc def ghi jkl\n1234 5678 1234\n0000\n1010101010\n\n8888 9999\nwhatsit hello\narst neio qwfp colemak is the best keyboard layout\naaaaa\nbbbbb\nccccc");

        // check that we can spot the gibberish license in the sea of other gibberish
        let strategy = ScanStrategy::new(&store)
            .mode(ScanMode::TopDown)
            .confidence_threshold(0.5)
            .step_size(1);
        let result = strategy.scan(&test_data).unwrap();
        assert!(result.license.is_none(), "result license is None");
        println!("{:?}", result);
        assert_eq!(2, result.containing.len());

        // inspect the array and ensure we got both licenses
        let mut found1 = 0;
        let mut found2 = 0;
        for (_, ref contained) in result.containing.iter().enumerate() {
            match contained.license.name.as_ref() {
                "license-1" => {
                    assert!(contained.score > 0.5, "license-1 score meets threshold");
                    found1 += 1;
                }
                "license-2" => {
                    assert!(contained.score > 0.5, "license-2 score meets threshold");
                    found2 += 1;
                }
                _ => {
                    panic!("somehow got an unknown license name");
                }
            }
        }

        assert!(
            found1 == 1 && found2 == 1,
            "found both licenses exactly once"
        );
    }

    fn create_dummy_store() -> Store {
        let mut store = Store::new();
        store.add_license("license-1".into(), "aaaaa\nbbbbb\nccccc".into());
        store.add_license(
            "license-2".into(),
            "1234 5678 1234\n0000\n1010101010\n\n8888 9999".into(),
        );
        store
    }
}
