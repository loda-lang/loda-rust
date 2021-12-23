importScripts('https://unpkg.com/promise-worker/dist/promise-worker.register.js');
importScripts('./pkg/loda_rust_web.js');

delete WebAssembly.instantiateStreaming;

const {WebDependencyManager} = wasm_bindgen;

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function randomInt(max) {
    return Math.floor(Math.random() * max);
}

class OperationComputeTerm {
    constructor(index) {
        this.mIndex = index;
    }

    index() {
        return this.mIndex;
    }

    accept(visitor) {
        visitor.visit_compute_term(this);
    }
}

class MyWorker {
    constructor(dependencyManager, workerId) {
        this.mDependencyManager = dependencyManager;
        this.mWorkerId = workerId;
        this.mRangeStart = 0;
        this.mRangeLength = 100;
        this.mResults = [];
        this.mPendingOperations = [];
        this.mIsExecutingPendingOperations = false;
    }

    commandSetRange(parameters) {
        console.log("commandSetRange");
        this.mRangeStart = parameters.rangeStart;
        this.mRangeLength = parameters.rangeLength;
    }

    commandExecuteRange(parameters) {
        console.log("worker", this.mWorkerId, "- commandExecuteRange before");
        this.mIsExecutingPendingOperations = false;
        this.mResults = [];
        this.mPendingOperations = [];

        // Enqueue operations for this range
        const index0 = this.mRangeStart;
        const index1 = this.mRangeStart + this.mRangeLength;
        for (var i = index0; i < index1; i++) {
            this.mPendingOperations.push(new OperationComputeTerm(i));
        }

        this.mIsExecutingPendingOperations = true;
        var self = this;
        setTimeout(function() { self.commandExecuteRangePost(); }, 0);
        console.log("worker", this.mWorkerId, "- commandExecuteRange after");
    }

    commandExecuteRangePost() {
        // console.log("commandExecuteRangePost - start executing pending operations");
        // TODO: Execute pending items from queue

        this.pickFirstPendingOperation();
    }

    pickFirstPendingOperation() {
        // console.log("pickFirstPendingOperation");
        if (!this.mIsExecutingPendingOperations) {
            console.log("worker", this.mWorkerId, "- stop running");
            return;
        }
        const operation = this.mPendingOperations.shift();
        if (typeof (operation) === 'undefined') {
            // console.log("pickFirstPendingOperation - no more pending operations - stopping");
            this.mIsExecutingPendingOperations = false;
            return;
        }
        // console.log("pickFirstPendingOperation - will execute", operation);

        // this.mIsExecutingPendingOperations = true;
        operation.accept(this);

        var self = this;
        setTimeout(function() { self.pickFirstPendingOperation(); }, 0);
    }

    visit_compute_term(operation_compute_term) {
        const index = operation_compute_term.index();
        const valueString = this.executeIndex(index);
        var dict = {};
        dict["index"] = index;
        dict["value"] = valueString;
        this.mResults.push(dict);
    }

    commandTakeResult(parameters) {
        // console.log("commandTakeResult");
        const termsArray = this.mResults;
        this.mResults = [];

        var responseDictionary = {};
        responseDictionary["terms"] = termsArray;

        // If still executing, then set true, so the UI knows there is more data to come.
        // If execute has stopped, then set to false, so the UI stops refreshing.
        responseDictionary["isExecuting"] = this.mIsExecutingPendingOperations;
        return responseDictionary;
    }

    executeIndex(index) {
        // console.log(`executeIndex before step ${index}`);

        // await sleep(100);

        try {
            const valueString = this.mDependencyManager.clone().execute_current_program(index);
            // console.log("computed value", valueString, "for index", index);
            return valueString;
        }
        catch(err) {
            console.log("Exception inside execute_current_program: ", err);
            return "ERROR";
        }    
        
        // console.log("executeNext after");
    }

    stopExecuteAndDiscardResults() {
        this.mResults = [];
        this.mIsExecutingPendingOperations = false;
    }

    async commandCompile(parameters) {
        console.log("worker", this.mWorkerId, "- commandCompile before");
        // discard old results
        this.mResults = [];

        // indicate that a new execute is going on
        this.mIsExecutingPendingOperations = false;

        // Remove pending operations
        this.mPendingOperations = [];

        const sourceCode = parameters.sourceCode;
        await this.mDependencyManager.clone().run_source_code(sourceCode);
        console.log("worker", this.mWorkerId, "- commandCompile after");
    }
}

async function init_worker() {
    const workerId = randomInt(1000000);
    console.log("init_worker 1", workerId);

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

    const myWorker = new MyWorker(dm, workerId);

    registerPromiseWorker(async function (e) {
        switch (e.fn) {
        case "setrange":
            myWorker.commandSetRange(e);
            break;
        case "executerange":
            myWorker.commandExecuteRange(e);
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