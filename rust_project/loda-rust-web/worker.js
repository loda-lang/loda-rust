function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function commandRange(parameters) {
    self.postMessage(`commandRange ${parameters}`);
    const rangeStart = parameters[0];
    const rangeLength = parameters[1];
    for (var i = rangeStart; i < rangeLength; i++) {
        console.log("step", i);
        await sleep(100);
        self.postMessage(['result', i]);
    }
}

addEventListener('message', async (e) => {
    // command = the first element of the input array
    // parameters = except the first element of the input array
    var parameters = e.data.slice();
    const command = parameters.shift();
    switch (command) {
    case "range":
        await commandRange(parameters);
        break;
    default:
        self.postMessage(`unknown: ${e.data}`);
        break;
    }
});

postMessage('started');
