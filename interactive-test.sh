#! /usr/bin/env sh

set -uo pipefail

gpg  --decrypt ../.secrets/scbl-utils.config.test.toml.gpg > ../.secrets/scbl-utils.config.test.toml

mkdir -p test-data/20250508__FOO__XR25001/output-MACHINE__0041619__REGION__20250508__FOO && \
cd test-data/20250508__FOO__XR25001/ &&
mkdir output-MACHINE__0041617__REGION__20250508__FOO && \
touch output-MACHINE__0041617__REGION__20250508__FOO/data && \
touch output-MACHINE__0041619__REGION__20250508__FOO/data && \
cd .. && \
mkdir -p staging/elise_courtois

cd ..
cargo run -- --config-path ../.secrets/scbl-utils.config.test.toml stage-xenium test-data/20250508__FOO__XR25001
rm -r ../.secrets/scbl-utils.config.test.toml test-data
