"use strict";

function pixelOnMouseOver(element) {
    const infoid = element.dataset.infoid;
    // console.log("pixelOnMouseOver", element, infoid);
    const el2 = document.getElementById(infoid);
    el2.style.display = "block";
}
    
function pixelOnMouseOut(element) {
    const infoid = element.dataset.infoid;
    // console.log("pixelOnMouseOut", element, infoid);
    const el2 = document.getElementById(infoid);
    el2.style.display = "none";
}
    
function edgeOnMouseOver(element) {
    const infoid = element.dataset.infoid;
    // console.log("edgeOnMouseOver", element, infoid);
    const el2 = document.getElementById(infoid);
    el2.style.display = "block";
}
    
function edgeOnMouseOut(element) {
    const infoid = element.dataset.infoid;
    // console.log("edgeOnMouseOut", element, infoid);
    const el2 = document.getElementById(infoid);
    el2.style.display = "none";
}
