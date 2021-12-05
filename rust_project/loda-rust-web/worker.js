function commandRange(parameters) {
    self.postMessage(`commandRange ${parameters}`);
}

self.onmessage = e => {
    // command = the first element of the input array
    // parameters = except the first element of the input array
    var parameters = e.data.slice();
    const command = parameters.shift();
    switch (command) {
    case "range":
        commandRange(parameters);
        break;
    default:
        self.postMessage(`unknown: ${e.data}`);
        break;
    }
};

postMessage('started');
