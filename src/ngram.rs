// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{
    cmp::min,
    collections::{hash_map::Iter, HashMap, VecDeque},
};

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct NgramSet {
    map: HashMap<String, u32>,
    // once Rust supports it, it'd be nice to make this
    // a type parameter & specialize
    n: u8,
    size: usize,
}

impl NgramSet {
    pub fn new(n: u8) -> NgramSet {
        NgramSet {
            map: HashMap::new(),
            n,
            size: 0,
        }
    }

    pub fn from_str(s: &str, n: u8) -> NgramSet {
        let mut set = NgramSet::new(n);
        set.analyze(s);
        set
    }

    pub fn analyze(&mut self, s: &str) {
        let words = s.split(' ');

        let mut deque: VecDeque<&str> = VecDeque::with_capacity(self.n as usize);
        for w in words {
            deque.push_back(w);
            if deque.len() == self.n as usize {
                let parts = deque.iter().cloned().collect::<Vec<&str>>();
                self.add_gram(parts.join(" "));
                deque.pop_front();
            }
        }
    }

    fn add_gram(&mut self, gram: String) {
        let n = self.map.entry(gram).or_insert(0);
        *n += 1;
        self.size += 1;
    }

    pub fn get(&self, gram: &str) -> u32 {
        if let Some(count) = self.map.get(gram) {
            *count
        } else {
            0
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn dice(&self, other: &NgramSet) -> f32 {
        // no sense comparing sets of different sizes
        if other.n != self.n {
            return 0f32;
        }

        // there's obviously no match if either are empty strings;
        // if we don't check here we could end up with NaN below
        // when both are empty
        if self.is_empty() || other.is_empty() {
            return 0f32;
        }

        // choose the smaller map to iterate
        let (x, y) = if self.len() < other.len() {
            (self, other)
        } else {
            (other, self)
        };

        let mut matches = 0;
        for (gram, count) in x {
            matches += min(*count, y.get(gram));
        }

        (2.0 * matches as f32) / ((self.len() + other.len()) as f32)
    }
}

impl<'a> IntoIterator for &'a NgramSet {
    type Item = (&'a String, &'a u32);
    type IntoIter = Iter<'a, String, u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // this is a pretty banal test, but it's a starting point :P
    #[test]
    fn can_construct() {
        let set = NgramSet::new(2);
        assert_eq!(set.size, 0);
        assert_eq!(set.n, 2);
    }

    #[test]
    fn no_nan() {
        let a = NgramSet::from_str("", 2);
        let b = NgramSet::from_str("", 2);

        let score = a.dice(&b);

        assert!(!score.is_nan());
    }

    #[test]
    fn same_size() {
        let a = NgramSet::from_str("", 2);
        let b = NgramSet::from_str("", 3);

        let score = a.dice(&b);

        assert_eq!(0f32, score);
    }

    #[test]
    fn identical() {
        let a = NgramSet::from_str("one two three apple banana", 2);
        let b = NgramSet::from_str("one two three apple banana", 2);

        let score = a.dice(&b);

        assert_eq!(1f32, score);
    }
}
