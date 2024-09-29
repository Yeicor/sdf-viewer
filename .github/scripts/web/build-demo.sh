
#!/usr/bin/env bash

set -ex

# This script takes the generated .wasm and preparse it for the release.
wasm_file="$1"
shift 1
dirname="$(dirname "$wasm_file")"
mkdir -p "$dirname"
cd "$dirname"
wasm_file="$(basename "$wasm_file")"

# Also compile the SDF demo to an example small WASM file and include it in the distribution.
wasm_demo_file="demo_sdf.wasm"
cargo build --lib --no-default-features --features sdfdemoffi --target wasm32-unknown-unknown $@
cp "../wasm32-unknown-unknown/release/sdf_viewer.wasm" "$wasm_demo_file"

[ -f .gitignore ] && rm .gitignore # This may conflict with some deployments and it is already ignored in the main repo.

# Prepare the release .tar.gz, removing the original file
tar -czf "../${wasm_file%.*}.tar.gz" .
