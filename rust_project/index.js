import init, * as wasmModule from './pkg/loda_lab.js';

const runWasm = async () => {
    await init('./pkg/loda_lab_bg.wasm');
    window.wasmModule = wasmModule;

    wasmModule.setup_log();
    // wasmModule.perform_selfcheck();

    const dm = new wasmModule.WebDependencyManager();
    // dm.increment();
    // dm.clone().run_source_code("mov $1,2\npow $1,$0");
    // dm.clone().run_source_code("mov $1,3\npow $1,$0");
    // dm.clone().run_source_code("mov $1,4\npow $1,$0");

    callbackExecuteProgramId = (programId) => {
        console.log(`execute: ${programId}`);
        wasmModule.run_program_id(programId);
    };

    callbackExecuteSourceCode = (sourceCode) => {
        console.log(`execute sourceCode`);
        (async() => {
            console.log('before start');
       
            dm.increment();
            await dm.clone().run_source_code(sourceCode);
            dm.clone().print_stats();

            console.log('after start');
        })();
    };

    callbackFinishedWasmLoading();
    // wasmModule.run_program_id(45);
    // callbackExecuteSourceCode("mov $1,2\npow $1,$0");
};
runWasm();
