importScripts('./pkg/loda_rust_web.js');

const {WebDependencyManager} = wasm_bindgen;

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

class MyWorker {
    constructor(workerOwner, dependencyManager) {
        this.mWorkerOwner = workerOwner;
        this.mDependencyManager = dependencyManager;
        this.mRangeStart = 0;
        this.mRangeLength = 10;
    }

    debug(message) {
        this.mWorkerOwner.postMessage({
            fn: 'debug',
            message: message
        });
    }

    async commandSetRange(parameters) {
        this.debug("commandSetRange");
        this.mRangeStart = parameters.rangeStart;
        this.mRangeLength = parameters.rangeLength;
    }

    async commandExecuteRange(parameters) {
        console.log("commandExecuteRange before");
        const index0 = this.mRangeStart;
        const index1 = this.mRangeStart + this.mRangeLength;
        for (var i = index0; i < index1; i++) {
            await this.executeIndex(i);
        }
        console.log("commandExecuteRange after");
    }

    async executeIndex(index) {
        console.log(`executeIndex before step ${index}`);

        // await sleep(100);

        try {
            const valueString = await this.mDependencyManager.clone().execute_current_program(index);
            // console.log("computed value: ", valueString);
            this.mWorkerOwner.postMessage({
                fn: 'result', 
                valueString: valueString
            });
        }
        catch(err) {
            console.log("Exception inside execute_current_program: ", err);
        }    
        
        console.log("executeNext after");
    }

    async commandCompile(parameters) {
        console.log("commandCompile before");
        const sourceCode = parameters.sourceCode;
        await this.mDependencyManager.clone().run_source_code(sourceCode);
        console.log("commandCompile after");
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

    // dm.increment();
    // await dm.clone().run_source_code("mov $1,2\npow $1,$0");
    // await dm.clone().run_source_code("seq $0,40\nmul $0,-1");

    // const index = 2;
    // const value = await dm.clone().execute_current_program(index);
    // console.log("computed value: ", value);

    // dm.clone().print_stats();

    const myWorker = new MyWorker(owner, dm);
  
    owner.addEventListener('message', async (e) => {
        switch (e.data.fn) {
        case "setrange":
            myWorker.commandSetRange(e.data);
            break;
        case "executerange":
            await myWorker.commandExecuteRange(e.data);
            break;
        case "compile":
            await myWorker.commandCompile(e.data);
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
