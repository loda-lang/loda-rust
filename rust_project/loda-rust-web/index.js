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
  
class PageController {
    constructor(dict) {
        console.log("PageController.ctor");

        // Install `runSourceCode` callback
        this.runSourceCode = dict['runSourceCode'] || function(sourceCode, termCount, runId) {
            console.error("PageController.runSourceCode() callback not installed");
        };

        this.mWorkerIsReady = false;
        this.mTick = 0;
        this.mUpdateTick = false;
        this.mRunId = 0;
        this.mIdenticalToOriginal = true;
        this.mOriginalText = "";
        this.mEditor = this.configureEditor();
        this.mOutputChart = this.configureChart();
        this.configureKeyboardShortcuts();
        this.configureOutputCount();
        this.prepareProgram();
        // this.rebuildChart();
        this.mWorker = this.configureWorker();
    }
  
    configureWorker() {
        const worker = new Worker('worker.js');
        worker.addEventListener('message', (e) => {
            this.workerOnMessage(e);
        }, false);
        worker.addEventListener('error', (e) => {
            this.workerOnError(e);
        }, false);
        return worker;
    }
  
    workerOnError(e) {
        const errorMessage = [
            'ERROR: Line ', e.lineno, ' in ', e.filename, ': ', e.message
        ].join('');
        this.outputArea_appendError(errorMessage);
    }
    
    workerOnMessage(e) {
        switch (e.data.fn) {
        case "result":
            this.commandResult(e.data);
            break;
        case "debug":
            this.commandDebug(e.data);
            break;
        case "ready":
            this.commandReady(e.data);
            break;
        default:
            console.error(`workerOnMessage.unknown: ${e.data}`);
            this.outputArea_appendError("unknown message, see log");
            break;
        }
    }
  
    compileEditorCode() {
        console.log("compile editor code");
        let sourceCode = this.mEditor.getValue();
        this.mWorker.postMessage({
            fn: "compile", 
            sourceCode: sourceCode
        });
    }
  
    workerCompileAndExecute() {
        console.log("compile and execute");
        this.compileEditorCode();
        this.outputArea_clear();
        this.executeRange();
    }
  
    commandResult(parameters) {
        // console.log("worker result", parameters.valueString);
        this.outputArea_appendTerm(parameters.valueString);
        this.rebuildChart();
    }
  
    commandDebug(parameters) {
        console.log("worker debug", parameters);
    }
  
    commandReady(parameters) {
        console.log("worker is ready", parameters);
        this.mWorkerIsReady = true;
        // this.setRange();
        // this.outputArea_clear();
        // this.executeRange();
    }
  
    outputArea_clear() {
        const div = document.getElementById("output-inner2");
        div.innerHTML = '';
    }
  
    outputArea_appendTerm(termValueString) {
        const div = document.getElementById("output-inner2");
    
        const a0 = document.createElement("span");
        a0.className = "separator";
        const a1 = document.createTextNode(",");
        a0.appendChild(a1);
    
        const b0 = document.createElement("span");
        b0.className = "term";
        const b1 = document.createTextNode(termValueString);
        b0.appendChild(b1);
    
        div.appendChild(a0);
        div.appendChild(b0);
    }
  
    outputArea_appendError(message) {
        const div = document.getElementById("output-inner2");
    
        const a0 = document.createElement("span");
        a0.className = "separator";
        const a1 = document.createTextNode(",");
        a0.appendChild(a1);
    
        const b0 = document.createElement("span");
        b0.className = "error";
        const b1 = document.createTextNode(message);
        b0.appendChild(b1);
    
        div.appendChild(a0);
        div.appendChild(b0);
    }
  
    setRange() {
        let rangeLength = this.getNumberOfTerms();
        this.mWorker.postMessage({
            fn: "setrange", 
            rangeStart: 0,
            rangeLength: rangeLength
        });
    }
  
    executeRange() {
        this.mWorker.postMessage({
            fn: "executerange", 
        });
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
        const el = document.getElementById('output-count');
        var self = this;
        el.addEventListener('change', function(e) {
            self.setRange();
            self.outputArea_clear();
            self.executeRange();
            // self.runAction();
        }, false);
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
        this.hideOverlay();
        var self = this;
        setTimeout(function() { self.didLoadProgram(); }, 100);
    }
  
    prepareProgramSourceCode(sourceCode) {
        console.log("prepareProgramSourceCode", sourceCode);
        this.mIdenticalToOriginal = true;
        this.mOriginalText = sourceCode;
        this.mEditor.setValue(sourceCode);
        this.mEditor.focus();
        this.hideOverlay();
        var self = this;
        setTimeout(function() { self.didLoadProgram(); }, 100);
    }
  
    prepareProgramId(programId) {
        console.log("prepareProgramId", programId);
    
        let url = urlFromProgramId(programId);
    
        var output = document.getElementById("output-inner");
        output.innerText = 'Downloading';
    
        // TODO: make this fetch happen in the worker.js, so there is less redundant code.
        // TODO: deal with status code when there is no 404 and show error message
        fetch(url)
            .then(response => response.text())
            .then(textdata => {
                console.log('Did fetch program');
                this.mIdenticalToOriginal = true;
                this.mOriginalText = textdata;
                this.mEditor.setValue(textdata);
                this.mEditor.focus();
                this.hideOverlay();
        
                var self = this;
                setTimeout(function() { self.didLoadProgram(); }, 100);
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
  
    workerCompileAndExecute() {
        console.log("compile and execute");
        this.compileEditorCode();
        this.outputArea_clear();
        this.executeRange();
    }
  
    didLoadProgram() {
        console.log("didLoadProgram");
        if (!this.mWorkerIsReady) {
            console.log("worker is not yet ready");
            return;
        }
        this.setRange();
        this.compileEditorCode();
        this.outputArea_clear();
        this.executeRange();
        // var output = document.getElementById("output-inner");
        // output.innerText = 'Computing';
        // this.runAction();
    }
  
    outputInnerAppendErrorMessage(message) {
        var output = document.getElementById("output-inner");
        var el0 = document.createElement('span');
        el0.className = "separator";
        el0.innerText = ",";
        output.appendChild(el0);
        var el1 = document.createElement('span');
        el1.className = "error";
        el1.innerText = message;
        output.appendChild(el1);
    }
  
    executeTick() {
        if (!this.mUpdateTick) {
            return;
        }
        // console.log(`tick: ${this.mTick}`);
        if (this.mTick >= 100) {
            this.mUpdateTick = false;
            this.outputInnerAppendErrorMessage("Stopped - exceeded 10 second time limit.");
            return;
        }
        this.mTick += 1;
        var self = this;
        setTimeout(function() { self.executeTick(); }, 100);
    }
  
    configureKeyboardShortcuts() {
        let pageControllerInstance = this;
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
                // pageControllerInstance.runAction();
                pageControllerInstance.workerCompileAndExecute();
                return;
            }
            // intercept ESCape key, and stop a running program.
            if(isEscapeKeyCode) {
                console.log("escape: stop running");
                event.preventDefault(); // Suppress "double action"
                pageControllerInstance.stopAction();
                return;
            }
        };
        window.addEventListener('keydown', keydownHandler, true);
    }
  
    runAction() {
        this.mTick = 0;
        this.mUpdateTick = true;
        this.mRunId = (this.mRunId + 1) % 256;
        const runIdClone0 = this.mRunId * 10;
        const runIdClone1 = runIdClone0 / 10;
        this.executeTick();
        let sourceCode = this.mEditor.getValue();
        let termCount = this.getNumberOfTerms();
        this.runSourceCode(sourceCode, termCount, runIdClone1);
    }
  
    stopAction() {
        console.log("Stop button");
        this.mUpdateTick = false;
    }
  
    showInfo() {
        console.log("Show info");
        window.open(
            "https://loda-lang.org/",
            '_blank' // Open in a new window.
        );
    }
  
    isRunningProgram(runId) {
        if (!this.mUpdateTick) {
            return false;
        }
        if (this.mRunId != runId) {
            return false;
        }
        return true;
    }
  
    finishedComputingTheLastTerm() {
        console.log("finished computing the last term");
        this.mUpdateTick = false;
        this.rebuildChart();
    }
  
    exceptionOccurredWhileRunning(message) {
        this.mUpdateTick = false;
        this.outputInnerAppendErrorMessage(message);
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
  
        const div = document.getElementById("output-inner2");
        const text = div.innerText;
        console.log("text", text);
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
        
        const datasetAll = {
            label: 'All',
            backgroundColor: 'rgba(25,25,25,1.0)',
            pointRadius: 1,
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
  
var callbackExecuteSourceCode = (sourceCode, termCount, runId) => {
    console.log("callbackExecuteSourceCode is not installed");
}
var callbackFinishedWasmLoading = () => {
    console.log("callbackFinishedWasmLoading invoked");
}
  
function body_onload() {
    const runSourceCode = function(sourceCode, termCount, runId) {
        callbackExecuteSourceCode(sourceCode, termCount, runId);
    };
    const dict = {
        'runSourceCode': runSourceCode
    };
    gPageController = new PageController(dict);
}
  
function body_onbeforeunload() {
    if (gPageController.mIdenticalToOriginal) {
        return undefined;
    } else {
        return "The data on this page will be lost if you leave";
    }
}
