import init, * as wasmModule from './pkg/lodalab_web.js';

function computeAndYield(remaining, index, dm, runId) {
    if (!gPageController.isRunningProgram(runId)) {
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
        try {
            await dm.clone().execute_current_program(index);
        }
        catch(err) {
            console.log("Exception inside execute_current_program: ", err);
            dm.clone().print_stats();
            gPageController.exceptionOccurredWhileRunning(`${err}`);
            return;
        }
        setTimeout(function() { computeAndYield(remaining - 1, index + 1, dm, runId); }, 0);
    })();
}

const runWasm = async () => {
    await init('./pkg/lodalab_web_bg.wasm');
    window.wasmModule = wasmModule;

    wasmModule.setup_lib();
    // wasmModule.perform_selfcheck();

    const dm = new wasmModule.WebDependencyManager();
    // dm.increment();
    // dm.clone().run_source_code("mov $1,2\npow $1,$0");
    // dm.clone().run_source_code("mov $1,3\npow $1,$0");
    // dm.clone().run_source_code("mov $1,4\npow $1,$0");

    callbackExecuteSourceCode = (sourceCode, termCount, runId) => {
        console.log(`execute sourceCode, termCount: ${termCount}`);
        (async() => {
            console.log('before start');
       
            dm.increment();
            await dm.clone().run_source_code(sourceCode);
            computeAndYield(termCount, 0, dm, runId);

            console.log('after start');
        })();
    };

    callbackFinishedWasmLoading();
    // wasmModule.run_program_id(45);
    // callbackExecuteSourceCode("mov $1,2\npow $1,$0");
};
runWasm();
