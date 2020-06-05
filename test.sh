#!/bin/bash

set -e

pushd generate-icons
cargo run
popd
mv generate-icons/icons.rs src/icons.rs.in
cargo check
