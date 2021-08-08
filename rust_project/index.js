import wasmInit from "./pkg/loda_lab.js";

const runWasm = async () => {
    const wasmModule = await wasmInit("./pkg/loda_lab_bg.wasm");

    wasmModule.setup_log();

    wasmModule.perform_selfcheck();

    var output = document.getElementById("output");
    output.innerText = 'Loading';

    wasmModule.run_program(45);
    
    executeProgramCallback = (programId) => {
        console.log(`execute: ${programId}`);
        wasmModule.run_program(programId);
    }
};
runWasm();
