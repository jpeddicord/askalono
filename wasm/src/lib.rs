// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![feature(use_extern_macros)]

extern crate wasm_bindgen;

extern crate askalono;

use askalono::*;
use wasm_bindgen::prelude::*;

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
    AskalonoStore {
      store: Store::new(),
    }
  }

  pub fn add_license(&mut self, name: String, text: &str) {
    self.store.add_license(name, text.into());
  }

  pub fn identify(&self, text: &str) -> MatchResult {
    let matched = self.store.analyze(&text.into()).unwrap();
    MatchResult {
      name: matched.name,
      score: matched.score,
    }
  }
}
