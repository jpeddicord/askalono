// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

extern crate wasm_bindgen;

extern crate askalono;

use askalono::*;
use wasm_bindgen::prelude::*;

static CACHE_DATA: &'static [u8] = include_bytes!(env!("ASKALONO_WASM_EMBEDDED_CACHE"));

#[wasm_bindgen]
pub struct AskalonoStore {
    store: Store,
}

#[wasm_bindgen]
pub struct MatchResult {
    name: String,
    score: f32,
}

#[wasm_bindgen]
impl MatchResult {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn score(&self) -> f32 {
        self.score
    }
}

#[wasm_bindgen]
impl AskalonoStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AskalonoStore {
        let store = Store::from_cache(CACHE_DATA).unwrap();
        AskalonoStore {
            store,
        }
    }

    pub fn identify(&self, text: &str) -> MatchResult {
        let matched = self.store.analyze(&text.into()).unwrap();
        MatchResult {
            name: matched.name,
            score: matched.score,
        }
    }
}
