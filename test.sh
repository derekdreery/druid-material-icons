#!/bin/bash

set -e

pushd generate-icons
cargo run --release
popd
mv generate-icons/icons.rs src/icons.rs.in
rustfmt src/icons.rs.in
cargo check
