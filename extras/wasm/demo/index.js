// Copyright 2018-2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

import { diffWords } from "diff";
const wasm = import("askalono");

const resultInfo = document.getElementById("resultinfo");
const diffBox = document.getElementById("diff");

wasm.then(({ AskalonoStore, normalize_text }) => {
  const store = new AskalonoStore();

  const field = document.getElementById("identify");
  let timeout = null;
  field.addEventListener("input", (e) => {
    // debounce to not kill the UI
    if (timeout) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(() => {
      analyze(store, normalize_text(field.value));
      timeout = null;
    }, 200);
  });

  const licenselist = document.getElementById("licenselist");
  fillLicenses(store, licenselist);
  licenselist.addEventListener("change", (e) => {
    fillLicenseText(store, e.currentTarget.value, field);
    e.currentTarget.value = "";
  });

  // analyze on startup, because browsers tend to keep textbox text on reload
  field.dispatchEvent(new Event("input"));
});

function fillLicenses(store, select) {
  const licenses = store.licenses().sort();
  for (const license of licenses) {
    const opt = document.createElement("option");
    opt.value = license;
    opt.text = license;
    select.appendChild(opt);
  }
}

function fillLicenseText(store, name, target) {
  const info = store.get_license(name);
  target.value = info.text();
  target.dispatchEvent(new Event("input"));
}

function analyze(store, input) {
  const startTime = performance.now();
  const result = store.identify(input);
  const endTime = performance.now();

  renderResults(input, result, endTime - startTime);
}

function renderResults(input, result, time) {
  // reset
  clearChildren(diffBox);
  clearChildren(resultInfo);

  if (result.score() < 0.1) {
    return;
  }

  // punch in stats
  renderInfo(result, time);

  // show a diff
  const diffFrag = generateDiff(input, result.license_text());
  diffBox.appendChild(diffFrag);
}

function clearChildren(node) {
  while (diffBox.hasChildNodes()) {
    diffBox.removeChild(diffBox.lastChild);
  }
}

function renderInfo(result, time) {
  if (result.score() > 0.1) {
    resultInfo.innerHTML = `
      askalono thinks this is <strong>${result.name()}</strong><br/>with
      <strong>${(result.score() * 100).toFixed(1)}%</strong> confidence
      <small>(took ${time.toFixed(1)}ms)</small>
    `;
  }
}

function generateDiff(identify, original) {
  const changes = diffWords(original, identify);
  const frag = document.createDocumentFragment();
  for (const chunk of changes) {
    const span = document.createElement("span");
    span.innerText = chunk.value;
    if (chunk.added) {
      span.style.backgroundColor = "#c9fccb";
      span.style.fontWeight = "bold";
    } else if (chunk.removed) {
      span.style.backgroundColor = "#fcc9c9";
      span.style.textDecoration = "line-through";
    }
    frag.appendChild(span);
  }
  return frag;
}
