import wasmInit from "./pkg/loda_lab.js";

const runWasm = async () => {
    const wasmModule = await wasmInit("./pkg/loda_lab_bg.wasm");

    wasmModule.setup_log();

    wasmModule.perform_selfcheck();

    var output = document.getElementById("output");
    output.innerText = 'Loading';

    wasmModule.fetch_from_repo();
};
runWasm();
