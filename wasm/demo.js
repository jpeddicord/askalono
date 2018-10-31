// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

const wasm = import('./wasm/askalono_wasm_optimized');

console.log('im here!');

wasm.then(({ AskalonoStore }) => {
  console.log('im wasm!');
  const store = new AskalonoStore();

  const nameText = document.getElementById('result_name');
  const nameScore = document.getElementById('result_score');
  document.getElementById('input').addEventListener('input', (e) => {
    const val = e.currentTarget.value;
    console.time('identify');
    const result = store.identify(val);
    console.timeEnd('identify');

    if (result.score() < 0.01) {
      nameText.innerText = '???';
      nameScore.innerText = '???';
    } else {
      nameText.innerText = result.name();
      nameScore.innerText = result.score().toString();
    }
  });
});

