import wasmInit from "./pkg/loda_lab.js";

const runWasm = async () => {
    const wasmModule = await wasmInit("./pkg/loda_lab_bg.wasm");

    const addResult = wasmModule.add(24, 24);
    document.body.textContent = `Hello World! addResult: ${addResult}`;

    wasmModule.console_log_from_wasm();

    wasmModule.myjsfunc_from_wasm();
};
runWasm();
