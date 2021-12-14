importScripts('./pkg/loda_rust_web.js');

const {WebDependencyManager} = wasm_bindgen;

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

class MyWorker {
    constructor(workerOwner, dependencyManager) {
        this.mWorkerOwner = workerOwner;
        this.mDependencyManager = dependencyManager;
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

    async commandRun(parameters) {
        this.debug("commandRun");
        const sourceCode = parameters.sourceCode;
        await this.mDependencyManager.clone().run_source_code(sourceCode);
    }
}


async function init_worker(owner) {
    // console.log("init_worker 1");

    const wasmModule = await wasm_bindgen('./pkg/loda_rust_web_bg.wasm');

    // console.log("init_worker 2");

    wasmModule.setup_lib();

    // console.log("init_worker 3");

    wasmModule.perform_selfcheck();

    // console.log("init_worker 4");

    const dm = new WebDependencyManager();

    dm.increment();
    // await dm.clone().run_source_code("mov $1,2\npow $1,$0");
    await dm.clone().run_source_code("seq $0,40\nmul $0,-1");


    const myWorker = new MyWorker(owner, dm);
  
    owner.addEventListener('message', async (e) => {
        switch (e.data.fn) {
        case "range":
            await myWorker.commandRange(e.data);
            break;
        case "run":
            await myWorker.commandRun(e.data);
            break;
        default:
            console.error(`worker.message: unknown: ${e.data.fn} ${e.data}`);
            break;
        }
    }, false);

    owner.postMessage({
        fn: 'ready'
    });
}

init_worker(this);
