"use strict";

function hideVisibleByDefault() {
    const elements = document.getElementsByClassName("visiblebydefault");
    for (var i = 0; i < elements.length; i++) { 
        elements[i].style.display="none";
    }
}

function showVisibleByDefault() {
    const elements = document.getElementsByClassName("visiblebydefault");
    for (var i = 0; i < elements.length; i++) { 
        elements[i].style.display="block";
    }
}

function showInfoOnMouseOver(element) {
    hideVisibleByDefault();

    const infoid = element.dataset.infoid;
    // console.log("pixelOnMouseOver", element, infoid);
    const el2 = document.getElementById(infoid);
    el2.style.display = "block";
}
    
function hideInfoOnMouseOut(element) {
    const infoid = element.dataset.infoid;
    // console.log("pixelOnMouseOut", element, infoid);
    const el2 = document.getElementById(infoid);
    el2.style.display = "none";

    showVisibleByDefault();
}
