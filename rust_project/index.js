import wasmInit from "./pkg/loda_lab.js";

const runWasm = async () => {
    const wasmModule = await wasmInit("./pkg/loda_lab_bg.wasm");

    const addResult = wasmModule.add(24, 24);
    document.body.textContent = `Hello World! addResult: ${addResult}`;

    wasmModule.setup_log();

    wasmModule.myjsfunc_from_wasm();

    wasmModule.fetch_from_repo();
};
runWasm();
