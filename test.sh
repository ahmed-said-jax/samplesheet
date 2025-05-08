#! /usr/bin/env sh

set -uo pipefail

gpg  --decrypt config.test.toml.gpg > config.test.toml
cargo run -- --config-path config.test.toml stage-xenium test-data/20250508__foo__XR25001
rm config.test.toml
