importScripts('./pkg/loda_rust_web.js');

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


async function init_worker() {
    // console.log("init_worker 1");

    const module = await wasm_bindgen('./pkg/loda_rust_web_bg.wasm');

    // console.log("init_worker 2");

    module.setup_lib();

    // console.log("init_worker 3");

    const myWorker = new MyWorker(this);
  
    addEventListener('message', async (e) => {
        switch (e.data.fn) {
        case "range":
            await myWorker.commandRange(e.data);
            break;
        default:
            console.error(`worker.message: unknown: ${e.data.fn} ${e.data}`);
            break;
        }
    }, false);    
}

init_worker();
