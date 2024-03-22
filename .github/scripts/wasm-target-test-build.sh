#!/bin/sh

GIT_ROOT=$(pwd)

cd /tmp

# create test project
cargo new foobar
cd foobar

# set rust-toolchain same as "halo2"
cp "${GIT_ROOT}/rust-toolchain" .

# add wasm32-* targets
rustup target add wasm32-unknown-unknown wasm32-wasi

# add dependencies
cargo add --path "${GIT_ROOT}/halo2_proofs" --features batch,dev-graph,gadget-traces
cargo add getrandom --features js --target wasm32-unknown-unknown

# test build for wasm32-* targets
cargo build --release --target wasm32-unknown-unknown
cargo build --release --target wasm32-wasi

# delete test project
cd ../
rm -rf foobar
