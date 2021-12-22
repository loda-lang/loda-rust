importScripts('https://unpkg.com/promise-worker/dist/promise-worker.register.js');
importScripts('./pkg/loda_rust_web.js');

delete WebAssembly.instantiateStreaming;

// const {WebDependencyManager} = wasm_bindgen;

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

class MyWorker {
    constructor(dependencyManager) {
        this.mDependencyManager = dependencyManager;
        this.mRangeStart = 0;
        this.mRangeLength = 10;
        this.mResults = {};
    }

    commandSetRange(parameters) {
        console.log("commandSetRange");
        this.mRangeStart = parameters.rangeStart;
        this.mRangeLength = parameters.rangeLength;
    }

    async commandExecuteRange(parameters) {
        console.log("commandExecuteRange before");
        const index0 = this.mRangeStart;
        const index1 = this.mRangeStart + this.mRangeLength;
        for (var i = index0; i < index1; i++) {
            mResults[i] = await this.executeIndex(i);
        }
        console.log("commandExecuteRange after");
    }

    commandTakeResult(parameters) {
        console.log("commandTakeResult");
        const result = this.mResults;
        this.mResults = {};
        return result;
    }

    async executeIndex(index) {
        // console.log(`executeIndex before step ${index}`);

        // await sleep(100);

        try {
            const valueString = await this.mDependencyManager.clone().execute_current_program(index);
            // console.log("computed value: ", valueString);
            return valueString;
        }
        catch(err) {
            console.log("Exception inside execute_current_program: ", err);
            return "ERROR";
        }    
        
        // console.log("executeNext after");
    }

    async commandCompile(parameters) {
        console.log("commandCompile before");
        const sourceCode = parameters.sourceCode;
        await this.mDependencyManager.clone().run_source_code(sourceCode);
        console.log("commandCompile after");
    }
}

const {WebDependencyManager} = wasm_bindgen;


async function init_worker() {
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

    // throw new Error("Demo of an exception");

    const myWorker = new MyWorker(dm);

    registerPromiseWorker(async function (e) {
        switch (e.fn) {
        case "setrange":
            myWorker.commandSetRange(e);
            break;
        case "executerange":
            await myWorker.commandExecuteRange(e);
            // I imagine "await" will block the communication channel.
            // TODO: start a loop, so it doesn't block the communication
            break;
        case "compile":
            await myWorker.commandCompile(e);
            break;
        case "takeresult":
            return myWorker.commandTakeResult(e);
        default:
            throw Error(`worker.message: unknown: ${e}`);
        }
    });

    // let things know that we are ready to accept commands:
    postMessage({
        fn: "init",
        value: true
    });
    
    postMessage({
        fn: 'ready'
    });
}

init_worker()
    .catch(e => {
        console.log('There has been a problem: ' + e.message);

        // let things know that we failed to initialise the WASM:
        postMessage({
            fn: "init",
            value: false,
            reason: "failed to fetch and instantiate the WASM"
        });
    });