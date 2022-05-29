#!/usr/bin/env bash

set -ex

# This script takes the generated .wasm and preparse it for the release.
wasm_file="$1"
dirname="$(dirname "$wasm_file")"
cd "$dirname"
wasm_file="$(basename "$wasm_file")"

cat >index.html <<EOF
<html lang="en">
<head>
    <meta content="text/html;charset=utf-8" http-equiv="Content-Type"/>
    <meta name="viewport"
          content="initial-scale=1.0,minimum-scale=1.0,maximum-scale=1.0,width=320,height=device-height,target-densitydpi=device-dpi,user-scalable=yes"/>
    <title>SDF Viewer</title>
    <style>
        body {
            margin: 0;
            overflow: hidden;
        }
    </style>
</head>
<body>
<canvas id="sdf-viewer" style="position: absolute;top:0;bottom: 0;left: 0;right: 0;margin:auto;"></canvas>
<script type="module">
    import init from './sdf_viewer.js';

    async function run() {
        await init();
    }

    run();
</script>
</body>
</html>
EOF

# Prepare the release .tar.gz, removing the original file
tar -czf "../${wasm_file%.*}.tar.gz" .
