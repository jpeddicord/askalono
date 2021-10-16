build toolchain="": init
    cargo {{toolchain}} build

init:
    [ -f datasets/modules/spdx-license-list-data/json/details/MIT.json ] || git submodule update --init

# run a Cargo command across all packages
all toolchain="" +cmds="": init
    cargo {{toolchain}} {{cmds}}
    cd cli && cargo {{toolchain}} {{cmds}}
    cd extras/lambda && cargo {{toolchain}} {{cmds}}
    cd extras/wasm && cargo {{toolchain}} {{cmds}}

lint toolchain="":
    just all {{toolchain}} clippy
    just all {{toolchain}} fmt

# run the CLI in release mode
# doesn't support rustup toolchain selection
cli +args="": init
    cd cli && cargo build --release
    ./target/release/askalono {{args}}

# test askalono against a license file and show a diff
# doesn't support rustup toolchain selection
diag +args="": init
    cd cli && cargo build --release --features diagnostics
    ./target/release/askalono id --diff {{args}}

# update the gh-pages branch with generated documentation
update-docs toolchain="":
    #!/bin/bash
    set -euxo pipefail

    cargo {{toolchain}} doc --no-deps

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
