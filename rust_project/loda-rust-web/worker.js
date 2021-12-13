// import * as wasmModule from './pkg/loda_rust_web.js';
// 
// importScripts('./pkg/loda_rust_web.js');

console.log('Initializing worker');

// const {setup_lib} = wasm_bindgen;
// const module = await import("./pkg/loda_rust_web.js");

async function init_wasm_in_worker() {
    console.log('Worker A');
    // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`.
    // await wasm_bindgen('./pkg/wasm_in_web_worker_bg.wasm');
    // await init('./pkg/loda_rust_web_bg.wasm');
    // window.wasmModule = wasmModule;

    // wasmModule.setup_lib();

    // setup_lib();

    // Create a new object of the `NumberEval` struct.
    // var num_eval = NumberEval.new();

    // // Set callback to handle messages passed to the worker.
    // self.onmessage = async event => {
    //     // By using methods of a struct as reaction to messages passed to the
    //     // worker, we can preserve our state between messages.
    //     var worker_result = num_eval.is_even(event.data);

    //     // Send response back to be handled by callback in main thread.
    //     self.postMessage(worker_result);
    // };
};

// init_wasm_in_worker();


function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

class MyWorker {
    constructor(workerOwner) {
        this.mWorkerOwner = workerOwner;
    }

    debug(message) {
        this.mWorkerOwner.postMessage({
            fn: 'debug',
            message: message
        });
    }

    async commandRange(parameters) {
        this.debug("commandRange");
        const rangeStart = parameters.rangeStart;
        const rangeLength = parameters.rangeLength;
        for (var i = rangeStart; i < rangeLength; i++) {
            // this.debug(`step ${i}`);
            await sleep(100);
            this.mWorkerOwner.postMessage({
                fn: 'result', 
                value: i
            });
        }
    }
}

const myWorker = new MyWorker(this);
  
addEventListener('message', async (e) => {
    switch (e.data.fn) {
    case "setup":
        const module = await import("./pkg/loda_rust_web.js");

        break;
    case "range":
        await myWorker.commandRange(e.data);
        break;
    default:
        console.error(`worker.message: unknown: ${e.data.fn} ${e.data}`);
        break;
    }
}, false);
