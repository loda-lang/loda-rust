
self.onmessage = e => {
    self.postMessage('onmessage');
};

postMessage('started');
