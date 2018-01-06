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

use std::cmp::min;
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use std::collections::VecDeque;

#[derive(Debug, Serialize, Deserialize)]
pub struct NgramSet {
    map: HashMap<String, u32>,
    // once Rust supports it, it'd be nice to make this
    // a type parameter & specialize
    n: u8,
    size: usize,
}

impl<'a> NgramSet {
    pub fn new(n: u8) -> NgramSet {
        NgramSet {
            map: HashMap::new(),
            n: n,
            size: 0,
        }
    }

    pub fn from_str(s: &str, n: u8) -> NgramSet {
        let mut set = NgramSet::new(n);
        set.analyze(s);
        set
    }

    pub fn three_from_str(s: &str) -> (NgramSet, NgramSet, NgramSet) {
        let mut uni = NgramSet::new(1);
        let mut bi = NgramSet::new(2);
        let mut tri = NgramSet::new(3);
        NgramSet::analyze_three(&mut uni, &mut bi, &mut tri, s);
        (uni, bi, tri)
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

    /// An optimized version of analyze that fills three ngram sets at once:
    /// unigram, bigram, and trigram
    fn analyze_three(uni: &mut NgramSet, bi: &mut NgramSet, tri: &mut NgramSet, s: &str) {
        let words = s.split(' ');

        let mut deque: VecDeque<&str> = VecDeque::with_capacity(3);
        for w in words {
            deque.push_back(w);
            match deque.len() {
                1 => {
                    uni.add_gram(deque[0].to_owned());
                }
                2 => {
                    uni.add_gram(deque[0].to_owned());
                    bi.add_gram([deque[0], deque[1]].join(" "));
                }
                3 => {
                    uni.add_gram(deque[0].to_owned());
                    bi.add_gram([deque[0], deque[1]].join(" "));
                    tri.add_gram([deque[0], deque[1], deque[2]].join(" "));
                    deque.pop_front();
                }
                _ => unreachable!(),
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
        if other.n != self.n {
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
