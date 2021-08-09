import init, * as wasmModule from './pkg/loda_lab.js';

const runWasm = async () => {
    await init('./pkg/loda_lab_bg.wasm');
    window.wasmModule = wasmModule;

    wasmModule.setup_log();

    wasmModule.perform_selfcheck();

    var output = document.getElementById("output-inner");
    output.innerText = 'Loading';

    wasmModule.run_program_id(45);
    
    executeProgramCallback = (programId) => {
        console.log(`execute: ${programId}`);
        wasmModule.run_program_id(programId);
    }
};
runWasm();
