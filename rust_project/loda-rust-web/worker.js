function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function commandRange(parameters) {
    // self.postMessage([`commandRange ${parameters}`]);
    console.log("commandRange");
    const rangeStart = parameters.rangeStart;
    const rangeLength = parameters.rangeLength;
    for (var i = rangeStart; i < rangeLength; i++) {
        console.log("step", i);
        await sleep(100);
        self.postMessage({
            fn: 'result', 
            value: i
        });
    }
}

addEventListener('message', async (e) => {
    console.log("worker.js addEventListener message. before");
    switch (e.data.fn) {
    case "range":
        await commandRange(e.data);
        break;
    case "setup":
        console.log("setup:", e.data.instanceId);
        break;
    default:
        console.error(`worker.message: unknown: ${e.data}`);
        break;
    }
    console.log("worker.js addEventListener message. after");
});
