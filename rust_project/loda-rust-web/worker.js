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
    switch (e.data.fn) {
    case "range":
        await commandRange(e.data);
        break;
    default:
        console.error(`worker.message: unknown: ${e.data}`);
        break;
    }
});
