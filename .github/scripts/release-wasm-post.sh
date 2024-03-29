
#!/usr/bin/env bash

set -ex

# This script takes the generated .wasm and preparse it for the release.
wasm_file="$1"
dirname="$(dirname "$wasm_file")"
mkdir -p "$dirname"
cd "$dirname"
wasm_file="$(basename "$wasm_file")"

cat >index.html <<EOF
<html lang="en">
<head>
    <meta content="text/html;charset=utf-8" http-equiv="Content-Type"/>
    <meta name="viewport" content="initial-scale=1.0,minimum-scale=1.0,maximum-scale=1.0,width=320,height=device-height,user-scalable=yes"/>
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
    import * as SDFViewer from './sdf_viewer.js';

    SDFViewer.default().then( // Async initialization
        () => SDFViewer.run_app("sdf-viewer")); // Run the actual App

	  // Start a toggleable web console for mobile devices only (to help with debugging)
    if (/Mobi|Android/i.test(navigator.userAgent)) {
    	var src = '//cdn.jsdelivr.net/npm/eruda';
      document.write('<scr' + 'ipt src="' + src + '"></scr' + 'ipt>');
      document.write('<scr' + 'ipt>eruda.init();</scr' + 'ipt>');
    }
</script>
</body>
</html>
EOF

# Also compile the SDF demo to an example small WASM file and include it in the distribution.
wasm_demo_file="demo_sdf.wasm"
cargo build --lib --no-default-features --features sdfdemoffi --release --target wasm32-unknown-unknown
cp "../wasm32-unknown-unknown/release/sdf_viewer.wasm" "$wasm_demo_file"

[ -f .gitignore ] && rm .gitignore # This may conflict with some deployments and it is already ignored in the main repo.

# Prepare the release .tar.gz, removing the original file
tar -czf "../${wasm_file%.*}.tar.gz" .
