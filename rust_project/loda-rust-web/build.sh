#!/bin/sh

# Print debug info before executing commands.
set -ex

# In the past I used `--target web`, but that only allowed me to run wasm in the main thread, causing the UI to hang.
#
# Now I use the flag `--target no-modules`, so I can run the wasm within a Web Worker.
wasm-pack build --target no-modules
