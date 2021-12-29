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

// Construct an OEIS id from a program id (eg 40), like the following
// "A000040".
function oeisIdFromProgramId(programId) {
    const zeroPad = (num, places) => String(num).padStart(places, '0');
    return "A" + zeroPad(programId, 6);
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

const ENUM_SCALEMODE_AUTO = 0;
const ENUM_SCALEMODE_LINEAR = 1;
const ENUM_SCALEMODE_LOGARITHMIC = 2;

class PageController {
    constructor() {
        this.mWorkerIsReady = false;
        this.mDidLoadProgram = false;
        this.mIdenticalToOriginal = true;
        this.mOriginalText = "";
        this.mWasUnableToFetchProgram = false;
        this.mScaleMode = ENUM_SCALEMODE_AUTO;
        this.setupWorker();
        this.setupEditor();
        this.setupChart();
        this.setupKeyboardShortcuts();
        this.setupRangePicker();
        this.prepareProgram();
        this.setupInteractiveArea();
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

        // The worker has been initialized successfully
        // console.log("worker initialized successful", parameters);
        this.mWorkerIsReady = true;

        // Mechanism that prevents the "Worker loaded OK" message to get shown, 
        // if there have been an error during initialization.
        if (this.mWasUnableToFetchProgram) {
            console.log("An error is already shown, that has higher priority.");
            return;
        }

        // Show that the worker has been loaded successfully.
        this.outputArea_clear();
        this.outputArea_appendTerm("Worker loaded OK.");

        // Run the program 
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

        // Fetch more results
        await this.pullWorkerResults();
    }
  
    setupEditor() {
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
        this.mEditor = editor;
    }
  
    setupChart() {
        const plugin_tooltip = {
            mode: 'point',
            callbacks: {
                label: function(context) {
                    const pointItem = context.raw;
                    var s = pointItem.label || '';
                    var is_string = (typeof s == 'string') || (s instanceof String);
                    if (is_string) {
                        return s;
                    } else {
                        return "x: " + pointItem.x + " y: " + pointItem.y;
                    }
                }
            },
        };
        const plugin_legend = {
            display: false,
        };
        const options = {
            animation: false,
            maintainAspectRatio: false,
            plugins: {
                tooltip: plugin_tooltip,
                legend: plugin_legend,
            }
        };
        // Dataset with invisible points
        const dataAll = this.chartEmptyData();
        const datasetAll = {
            pointRadius: 0,
            pointHitRadius: 0,
            borderWidth: 0,
            data: dataAll,
        };
        const data = {
            datasets: [
                datasetAll
            ]
        };
        const config = {
            type: 'scatter',
            data: data,
            options: options
        };
        var ctx = document.getElementById('output-chart').getContext('2d');
        this.mOutputChart = new Chart(ctx, config);
    }

    setupInteractiveArea() {
        this.mGraphScalingAuto = document.getElementById("graph-scaling-auto");
        this.mGraphScalingLinear = document.getElementById("graph-scaling-linear");
        this.mGraphScalingLogarithmic = document.getElementById("graph-scaling-logarithmic");
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
  
    setupRangePicker() {
        const element = document.getElementById('output-count');
        var self = this;
        element.addEventListener('change', function(e) {
            self.rangePickerAction();
        }, false);
    }

    rangePickerAction() {
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
        fetch(url)
            .then(response => {
                if (response.status == 404) {
                    const oeisId = oeisIdFromProgramId(programId);
                    var error = new Error(`There exist no program for ${oeisId}`);
                    error.name = 'pretty-error-message';
                    throw error;
                }
                if (!response.ok) {
                    throw new Error(`Expected status 2xx, but got ${response.status}`);
                }
                return response.text();
            })
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
                const textdata = '';
                this.mIdenticalToOriginal = true;
                this.mOriginalText = textdata;
                this.mEditor.setValue(textdata);
                this.mEditor.focus();
                this.outputArea_clear();
                this.mWasUnableToFetchProgram = true;
                if (error.name == 'pretty-error-message') {
                    this.outputArea_appendError(error.message);
                } else {
                    this.outputArea_appendError(`Error - ${error.message}`);
                }
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
  
    setupKeyboardShortcuts() {
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

    chartEmptyData() {
        var count = 10;
        var dataAll = [];
        for ( var i = 0; i < count; i+=1 ) {
            const value = i;
            const y = Math.floor(value);
            const dict = {
                x: i,
                y: y,
                label: `a(${i}) = ${y}`
            };
            dataAll.push(dict);
        }
        return dataAll;
    }

    extractChartDataFromOutput() {
        var dataAll = [];
        const parentDiv = document.getElementById("output-inner");
        var children = parentDiv.children;
        var index = 0;
        for (var i = 0; i < children.length; i += 1) {
            const child = children[i];
            if (child.className != 'term') {
                // Ignore elements such as: separator, error
                continue;
            }
            const textItem = child.innerText;
            var value = parseInt(textItem);
            if (isNaN(value)) { 
                value = 0;
            }
            const y = Math.floor(value);
            const dict = {
                x: index,
                y: y,
                label: `a(${index}) = ${y}`
            };
            dataAll.push(dict);
            index += 1;
        }
        return dataAll;
    }
  
    rebuildChart() {
        var chart = this.mOutputChart;
        
        // const dataAll = this.chartEmptyData();
        const dataAll = this.extractChartDataFromOutput();

        if (dataAll.length < 1) {
            this.outputArea_appendError("length is empty");
            return;
        }

        var pointRadius = 1;
        if (dataAll.length <= 10) {
            pointRadius = 3;
        }

        var useLogarithmic = false;

        var minY = 0;
        var maxY = 0;
        if (dataAll.length >= 1) {
            const dataItem = dataAll[0];
            const y = dataItem.y;
            minY = y;
            maxY = y;
        }
        for (var i = 0; i < dataAll.length; i += 1) {
            const dataItem = dataAll[i];
            const y = dataItem.y;
            if (y < minY) {
                minY = y;
            }
            if (y > maxY) {
                maxY = y;
            }
        }
        const yRangeLength = maxY - minY + 1;

        var linearSum = 0.0;
        var logSum = 0.0;
        for (var i = 0; i < dataAll.length; i += 1) {
            const dataItem = dataAll[i];
            linearSum += dataItem.y;
            logSum += Math.log(dataItem.y - minY + 1);
        }
        var linearAverage = linearSum / dataAll.length;
        var logAverage = logSum / dataAll.length;

        var linearSumError = 0.0;
        var logSumError = 0.0;
        for (var i = 0; i < dataAll.length; i += 1) {
            const dataItem = dataAll[i];
            const linearDiff = dataItem.y - linearAverage;
            linearSumError += linearDiff * linearDiff;
            const logDiff = Math.log(dataItem.y - minY + 1) - logAverage;
            logSumError += logDiff * logDiff;
        }
        var linearAverageError = linearSumError / dataAll.length;
        var logAverageError = logSumError / dataAll.length;

        // useLogarithmic = linearAverageError < logAverageError;
        useLogarithmic = false;
        const binCount = 10;
        var bins = [];
        for (var i = 0; i < binCount; i += 1) {
            bins.push(0);
        }
        var total = 0;
        for (var i = 0; i < dataAll.length; i += 1) {
            const dataItem = dataAll[i];
            const y = dataItem.y - minY;
            const binIndex = Math.floor(y * (binCount-1) / yRangeLength);
            bins[binIndex] += 1;
            total += 1;
        }
        // console.log("total", total);
        console.log("bin count", bins.length);
        for (var i = 0; i < bins.length; i += 1) {
            const count = bins[i];
            console.log(`bin ${i} = ${count}`);
        }
        const target0 = Math.floor(dataAll.length * 0.8);
        console.log("target", target0, "bin0", bins[0]);
        if (bins[0] > target0) {
            useLogarithmic = true;
        }
        if (minY <= 0) {
            // chartjs cannot do logarithmic plots with negative numbers nor zeros.
            useLogarithmic = false;
        }
        if (yRangeLength < 20) {
            useLogarithmic = false;
        }

        if (this.mScaleMode == ENUM_SCALEMODE_AUTO) {
            // do nothing
        }
        if (this.mScaleMode == ENUM_SCALEMODE_LINEAR) {
            useLogarithmic = false;
        }
        if (this.mScaleMode == ENUM_SCALEMODE_LOGARITHMIC) {
            useLogarithmic = true;
        }


        const divStats = document.getElementById("output-stats");
        // divStats.innerText = `average: ${linearAverage} ${linearAverageError}`;
        divStats.innerText = `${linearAverageError} ${logAverageError}`;

        const divColor = document.getElementById("graph-scaling");
        if (useLogarithmic) {
            divColor.style = 'background:#f00';
        } else {
            divColor.style = '';
        }

        const datasetAll = {
            backgroundColor: 'rgba(25,25,25,1.0)',
            pointRadius: pointRadius,
            pointHitRadius: 5,
            borderWidth: 0,
            data: dataAll
        };
        
        while (chart.data.datasets.length > 0) {
            chart.data.datasets.pop();
        }
        chart.data.datasets.push(datasetAll);
        
        if (useLogarithmic) {
            chart.options.scales.y.type = 'logarithmic';
        } else {
            chart.options.scales.y.type = 'linear';
        }

        chart.update();
    }

    useAutoScalingAction() {
        // console.log("use auto scaling");
        this.mGraphScalingAuto.className = 'selected';
        this.mGraphScalingLinear.className = 'not-selected';
        this.mGraphScalingLogarithmic.className = 'not-selected';
        this.mScaleMode = ENUM_SCALEMODE_AUTO;
        this.rebuildChart();
    }

    useLinearScalingAction() {
        // console.log("use linear scaling");
        this.mGraphScalingAuto.className = 'not-selected';
        this.mGraphScalingLinear.className = 'selected';
        this.mGraphScalingLogarithmic.className = 'not-selected';
        this.mScaleMode = ENUM_SCALEMODE_LINEAR;
        this.rebuildChart();
    }

    useLogarithmicScalingAction() {
        // console.log("use logarithmic scaling");
        this.mGraphScalingAuto.className = 'not-selected';
        this.mGraphScalingLinear.className = 'not-selected';
        this.mGraphScalingLogarithmic.className = 'selected';
        this.mScaleMode = ENUM_SCALEMODE_LOGARITHMIC;
        this.rebuildChart();
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
