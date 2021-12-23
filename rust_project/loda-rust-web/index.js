// Construct url for a program id (eg A112088), like the following
// https://raw.githubusercontent.com/loda-lang/loda-programs/main/oeis/112/A112088.asm
function urlFromProgramId(programId) {
    const zeroPad = (num, places) => String(num).padStart(places, '0');
    const dir_index_string = zeroPad(Math.floor(programId/1000), 3);
    const filename_string = "A" + zeroPad(programId, 6) + ".asm";
    let baseurl = "https://raw.githubusercontent.com/loda-lang/loda-programs/main/oeis";
    let url = `${baseurl}/${dir_index_string}/${filename_string}`;
    return url;
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

class PageController {
    constructor() {
        console.log("PageController.ctor");

        this.mWorkerIsReady = false;
        this.mDidLoadProgram = false;
        this.mIdenticalToOriginal = true;
        this.mOriginalText = "";
        this.setupWorker();
        this.mEditor = this.configureEditor();
        this.mOutputChart = this.configureChart();
        this.configureKeyboardShortcuts();
        this.configureOutputCount();
        this.prepareProgram();
        // this.rebuildChart();
    }
  
    setupWorker() {
        const worker = new Worker('worker.js');
        this.mPromiseWorker = new PromiseWorker(worker);
        const eventListener = (e) => {
            this.workerOnMessage(e);
        };
        worker.addEventListener('message', eventListener);
        this.mWorker = worker;
        this.mWorkerTemporaryEventListener = eventListener;
    }
  
    workerOnMessage(e) {
        // console.log("onMessage", e.data);
        switch (e.data.fn) {
        case "init":
            this.commandInit(e.data);
            break;
        default:
            console.error(`workerOnMessage.unknown: ${e.data}`);
            this.outputArea_appendError("workerOnMessage received unknown message");
            break;
        }
    }
  
    commandInit(parameters) {
        // console.log("worker init", parameters);

        // Remove the temporary event listener.
        this.mWorker.removeEventListener("message", this.mWorkerTemporaryEventListener);
        this.mWorkerTemporaryEventListener = null;
        this.mWorker = null;

        // Show an error if the worker failed to load.
        if(!parameters.value) {
            console.error("failed to initialize worker", parameters);
            this.outputArea_clear();
            this.outputArea_appendError(`Failed to initialize worker. reason: ${parameters.reason}`);
            return;
        }

        // Show that the worker has been loaded successfully.
        // console.log("worker initialized successful", parameters);
        this.outputArea_clear();
        this.outputArea_appendTerm("Worker loaded OK.");

        this.mWorkerIsReady = true;
        this.proceedIfAllThingsAreReady();
    }

    proceedIfAllThingsAreReady() {
        if (!this.mDidLoadProgram) {
            return;
        }
        if (!this.mWorkerIsReady) {
            return;
        }
        (async () => {
            this.hideOverlay();
            await this.workerCompileAndExecute();
        })();
    }

    async tellWorkerToStopExecuting() {
        console.log("stop executing BEFORE");
        await this.mPromiseWorker.postMessage({
            fn: "stop"
        });
        console.log("stop executing AFTER");
    }
  
    async compileEditorCode() {
        console.log("compile editor code BEFORE");
        let sourceCode = this.mEditor.getValue();
        await this.mPromiseWorker.postMessage({
            fn: "compile", 
            sourceCode: sourceCode
        });
        console.log("compile editor code AFTER");
    }
    
    outputArea_clear() {
        const div = document.getElementById("output-inner");
        div.innerHTML = '';
    }
  
    outputArea_appendTerm(termValueString) {
        const parentDiv = document.getElementById("output-inner");    
        if (parentDiv.hasChildNodes()) {
            const a0 = document.createElement("span");
            a0.className = "separator";
            const a1 = document.createTextNode(",");
            a0.appendChild(a1);        
            parentDiv.appendChild(a0);
        }
        const b0 = document.createElement("span");
        b0.className = "term";
        const b1 = document.createTextNode(termValueString);
        b0.appendChild(b1);
        parentDiv.appendChild(b0);
    }
  
    outputArea_appendError(message) {
        const parentDiv = document.getElementById("output-inner");    
        if (parentDiv.hasChildNodes()) {
            const a0 = document.createElement("span");
            a0.className = "separator";
            const a1 = document.createTextNode(",");
            a0.appendChild(a1);        
            parentDiv.appendChild(a0);
        }
        const b0 = document.createElement("span");
        b0.className = "error";
        const b1 = document.createTextNode(message);
        b0.appendChild(b1);        
        parentDiv.appendChild(b0);
    }
  
    async setRange() {
        let rangeLength = this.getNumberOfTerms();
        await this.mPromiseWorker.postMessage({
            fn: "setrange", 
            rangeStart: 0,
            rangeLength: rangeLength
        });
    }
  
    async executeRange() {
        // Usecase:
        // There is nothing executing in the web worker.
        // The user hits Ctrl+Enter a single time.
        // Causing the code to run.
        //
        // Usecase:
        // The user may be hitting Ctrl+Enter several times in rapid succession.
        // This happens surprisingly often.
        //
        // Usecase:
        // There is currently executing stuff in the web worker.
        // The user hits Ctrl+Enter a single time.
        // Causing the current executing to stop.
        // And causing the new code to run.

        await this.mPromiseWorker.postMessage({
            fn: "executerange", 
        });
        this.outputArea_clear();
        await this.pullWorkerResults();
    }

    async pullWorkerResults() {
        const responseDictionary = await this.mPromiseWorker.postMessage({
            fn: "takeresult", 
        });
        // console.log("responseDictionary", responseDictionary);
        const termsArray = responseDictionary.terms;
        const isExecuting = responseDictionary.isExecuting;

        var arrayLength = termsArray.length;
        for (var i = 0; i < arrayLength; i++) {
            const item = termsArray[i];
            if (item.value != null) {
                this.outputArea_appendTerm(item.value);
                continue;
            }
            if (item.error != null) {
                console.error("Unable to compute term", item.error);
                this.outputArea_appendError(item.error);
                break;
            }
            console.error("Encountered an integrity error. Expected either 'value' or 'error', but got something else.");
            this.outputArea_appendError("Integrity error");
            break;
        }

        this.rebuildChart();

        // Check if the worker is still computing. Stop if all terms have been computed.
        if (!isExecuting) {
            // Stop fetching, when all the data have been fetched.
            return;
        }

        // console.log("pull - before sleep");
        await sleep(30);
        // console.log("pull - after sleep");
        // this.outputArea_appendTerm("zzz");

        // Fetch more results
        await this.pullWorkerResults();
    }
  
    configureEditor() {
        const editor = CodeMirror.fromTextArea(document.getElementById("editor-inner"), {
            lineNumbers: true,
            lineWrapping: false,
            styleActiveLine: true,
            theme: "idea",
            mode: "loda",
            showTrailingSpace: true,
            tabSize: 2,
            indentWithTabs: false,
        });
        return editor;
    }
  
    configureChart() {
        var chart_config = {
            type: 'scatter',
            data: {
                datasets: []
            },
            options: {
                animation: false,
                responsive: true,
                maintainAspectRatio: false,
                tooltips: {
                    mode: 'point',
                    callbacks: {
                        label: function(tooltipItem, data) {
                            var pointItem = data.datasets[tooltipItem.datasetIndex].data[tooltipItem.index];
                            var s = pointItem.label;
                            var is_string = (typeof s == 'string') || (s instanceof String);
                            if (is_string) {
                                return s;
                            } else {
                                return "x: " + pointItem.x + " y: " + pointItem.y;
                            }
                        }
                    },
                },
                legend: {
                    display: false,
                }
            }
        };
    
        var ctx = document.getElementById('output-chart').getContext('2d');
        return new Chart(ctx, chart_config);
    }
  
    hideOverlay() {
        document.getElementById("overlay").style.display = "none";
    }
  
    showOverlay() {
        document.getElementById("overlay").style.display = "block";
    }
  
    getNumberOfTerms() {
        var radios = document.getElementsByName("outputcountname");
        const length = radios.length;
        var value = 10;
        for (var i = 0; i < length; i++) {
            if (radios[i].checked) {
            value = radios[i].value;
            break;
            }
        }
        return value;
    }
  
    configureOutputCount() {
        const element = document.getElementById('output-count');
        var self = this;
        element.addEventListener('change', function(e) {
            self.outputCountAction();
        }, false);
    }

    outputCountAction() {
        (async () => {
            await this.workerCompileAndExecute();
        })();
    }
  
    prepareProgram() {
        let params = new URLSearchParams(window.location.search);
        if (params.has('source')) {
            let sourceCode = params.get('source');
            this.prepareProgramSourceCode(sourceCode);
            return;
        }
        if (params.has('oeis')) {
            var programId = params.get('oeis');
            programId = programId.replace(/^A0*/i, '');
            this.prepareProgramId(programId);
            return;
        }
        console.log("Missing or unrecognized url parameters. Showing an empty program.");
        this.prepareProgramEmpty();
    }
  
    prepareProgramEmpty() {
        const sourceCode = "";
        this.mIdenticalToOriginal = true;
        this.mOriginalText = sourceCode;
        this.mEditor.setValue(sourceCode);
        this.mEditor.focus();
        this.didLoadProgram();
    }
  
    prepareProgramSourceCode(sourceCode) {
        console.log("prepareProgramSourceCode", sourceCode);
        this.mIdenticalToOriginal = true;
        this.mOriginalText = sourceCode;
        this.mEditor.setValue(sourceCode);
        this.mEditor.focus();
        this.didLoadProgram();
    }
  
    prepareProgramId(programId) {
        console.log("prepareProgramId", programId);
    
        let url = urlFromProgramId(programId);
    
        // TODO: deal with status code when there is no 404 and show error message
        fetch(url)
            .then(response => response.text())
            .then(textdata => {
                console.log('Did fetch program');
                this.mIdenticalToOriginal = true;
                this.mOriginalText = textdata;
                this.mEditor.setValue(textdata);
                this.mEditor.focus();
                this.didLoadProgram();
            })
            .catch((error) => {
                console.error('Error:', error);
                const textdata = "Unable to load program!";
                this.mIdenticalToOriginal = true;
                this.mOriginalText = textdata;
                this.mEditor.setValue(textdata);
                this.mEditor.focus();
                this.hideOverlay();
            });
    }
  
    async workerCompileAndExecute() {
        console.log("compile and execute");
        await this.setRange();
        await this.compileEditorCode();
        this.outputArea_clear();
        await this.executeRange();
    }

    didLoadProgram() {
        this.mDidLoadProgram = true;
        this.proceedIfAllThingsAreReady();
    }
  
    configureKeyboardShortcuts() {
        let self = this;
        let keydownHandler = function(event) {
            if(event.defaultPrevented) {
                return; // Should do nothing if the default action has been cancelled
            }
            const isMetaKey = event.metaKey || event.ctrlKey;
            const isEnterKeyCode = (event.keyCode == 10) || (event.keyCode == 13);
            const isEscapeKeyCode = (event.keyCode == 27);
            // intercept CTRL+ENTER, and run the program.
            if(isEnterKeyCode && isMetaKey) {
                console.log("ctrl+enter: submit form");
                event.preventDefault(); // Suppress "double action"
                self.runAction();
                return;
            }
            // intercept ESCape key, and stop a running program.
            if(isEscapeKeyCode) {
                console.log("escape: stop running");
                event.preventDefault(); // Suppress "double action"
                self.stopAction();
                return;
            }
        };
        window.addEventListener('keydown', keydownHandler, true);
    }
  
    runAction() {
        (async () => {
            await this.workerCompileAndExecute();
        })();
    }
  
    stopAction() {
        (async () => {
            await this.tellWorkerToStopExecuting();
        })();
    }
  
    showInfo() {
        console.log("Show info");
        window.open(
            "https://loda-lang.org/",
            '_blank' // Open in a new window.
        );
    }
  
    programURLString() {
        let sourceCode = this.mEditor.getValue();
        var url = new URL(window.location.href);
        url.search = `source=${encodeURIComponent(sourceCode)}`;
        return url.href;
    }
  
    copyProgramURLToClipboard() {
        let urlString = this.programURLString();
        navigator.clipboard.writeText(urlString);
        let byteCount = urlString.length;
        let tooltip = document.getElementById("copy-program-link-to-clipboard-tooltip-text");
        tooltip.innerHTML = `Copied ${byteCount} bytes to clipboard`;
    }
  
    hideCopyToClipboardTooltip() {
        let tooltip = document.getElementById("copy-program-link-to-clipboard-tooltip-text");
        tooltip.innerHTML = "Copy to clipboard";
    }
  
    rebuildChart() {
        var chart = this.mOutputChart;
        
        var count = 100;
        var dataAll = [];
        // for ( var i = 0; i < count; i+=1 ) {
        //     const value = i;
        //     const y = Math.floor(value);
        //     const dict = {
        //         x: i,
        //         y: y,
        //         label: `a(${i}) = ${y}`
        //     };
        //     dataAll.push(dict);
        // }
  
        const div = document.getElementById("output-inner");
        const text = div.innerText;
        // console.log("text", text);
        const textItems = text.split(",");
        for (var i = 0; i < textItems.length; i += 1) {
            const textItem = textItems[i];
            var value = parseInt(textItem);
            if (isNaN(value)) { 
                value = 0;
            }
            const y = Math.floor(value);
            const dict = {
                x: i,
                y: y,
                label: `a(${i}) = ${y}`
            };
            dataAll.push(dict);
        }

        var pointRadius = 1;
        if (dataAll.length <= 10) {
            pointRadius = 3;
        }
        
        const datasetAll = {
            label: 'All',
            backgroundColor: 'rgba(25,25,25,1.0)',
            pointRadius: pointRadius,
            pointHitRadius: 5,
            borderWidth: 0,
            data: dataAll,
        };
        
        while (chart.data.datasets.length > 0) {
            chart.data.datasets.pop();
        }
        chart.data.datasets.push(datasetAll);
        
        chart.update();
    }
}
  
var gPageController = null;
  
function body_onload() {
    gPageController = new PageController();
}
  
function body_onbeforeunload() {
    if (gPageController.mIdenticalToOriginal) {
        return undefined;
    } else {
        return "The data on this page will be lost if you leave";
    }
}
