import init, * as wasmModule from './pkg/loda_lab.js';

function computeAndYield(remaining, index, dm) {
    if (!gPageController.isRunningProgram()) {
        console.log("prematurely abort");
        dm.clone().print_stats();
        return;
    }
    if (remaining <= 0) {
        console.log("all terms have been computed.");
        dm.clone().print_stats();
        gPageController.finishedComputingTheLastTerm();
        return;
    }
    (async() => {
        await dm.clone().execute_current_program(index);
        setTimeout(function() { computeAndYield(remaining - 1, index + 1, dm); }, 0);
    })();
}

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

    callbackExecuteProgramId = (programId, termCount) => {
        console.log(`execute: ${programId}  termCount: ${termCount}`);
        wasmModule.run_program_id(programId);
    };

    callbackExecuteSourceCode = (sourceCode, termCount) => {
        console.log(`execute sourceCode, termCount: ${termCount}`);
        (async() => {
            console.log('before start');
       
            dm.increment();
            await dm.clone().run_source_code(sourceCode);
            computeAndYield(termCount, 0, dm);

            console.log('after start');
        })();
    };

    callbackFinishedWasmLoading();
    // wasmModule.run_program_id(45);
    // callbackExecuteSourceCode("mov $1,2\npow $1,$0");
};
runWasm();
