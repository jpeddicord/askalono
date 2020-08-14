build: init
    cargo build

init:
    [ -f datasets/spdx-json/MIT.json ] || git submodule update --init

# run a Cargo command across all packages
all +cmds: init
    cargo {{cmds}}
    cd cli && cargo {{cmds}}
    cd extras/lambda && cargo {{cmds}}
    cd extras/wasm && cargo {{cmds}}

lint:
    just all clippy
    just all fmt

# run the CLI in release mode
cli +args="": init
    cd cli && cargo build --release
    ./target/release/askalono {{args}}

# test askalono against a license file and show a diff
diag +args="": init
    cd cli && cargo build --release --features diagnostics
    ./target/release/askalono id --diff {{args}}

# update the gh-pages branch with generated documentation
update-docs:
    #!/bin/bash
    set -euxo pipefail

    cargo doc --no-deps

    rev=$(git rev-parse --short HEAD)
    git clone . -b gh-pages gh-pages
    cp -rv target/doc/. gh-pages/doc

    pushd gh-pages
        git add doc
        git commit -m "Documentation update from master commit $rev"
        git push
    popd

# update the wasm-demo stuff
update-wasm-demo:
    #!/bin/bash
    set -euxo pipefail

    rev=$(git rev-parse --short HEAD)

    pushd extras/wasm
        wasm-pack build --out-name askalono
        pushd demo
            npm install
            rm -rf dist
            npm run build
        popd
    popd

    git clone . -b gh-pages gh-pages
    rm -rf gh-pages/wasm-demo
    cp -rv extras/wasm/demo/dist/. gh-pages/wasm-demo

    pushd gh-pages
        git add wasm-demo
        git commit -m "wasm-demo update from master commit $rev"
        git push
    popd