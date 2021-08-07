import wasmInit from "./pkg/loda_lab.js";

const runWasm = async () => {
    const wasmModule = await wasmInit("./pkg/loda_lab_bg.wasm");

    wasmModule.setup_log();

    wasmModule.perform_selfcheck();

    wasmModule.fetch_from_repo();

    const x = 1234;
    document.body.textContent = `Hello World! x: ${x}`;
};
runWasm();
