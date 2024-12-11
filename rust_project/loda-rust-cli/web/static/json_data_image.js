function onloadRenderAllJsonDataImages() {
  const imageDivs = document.querySelectorAll('[data-image]');
  Array.from(imageDivs).forEach((imageDiv) => {
    renderJsonDataImage(imageDiv);
  });
}

function renderJsonDataImage(imageDiv) {
    const datasetImageValue = imageDiv.dataset.image;
    const divs = imageDiv.querySelectorAll('.json-data');
    const div = divs[0];
    const json = div.innerText;
    const imageRows = JSON.parse(json);

    const spanRowContainer = document.createElement('span');
    spanRowContainer.className = 'themearc image rows';

    imageRows.forEach((row, rowIndex) => {
      const spanRow = document.createElement('span');
      spanRow.className = 'themearc image row';

      row.forEach((pixel, columnIndex) => {
        const spanPixel = document.createElement('span');
        spanPixel.className = `themearc symbol_${pixel}`;
        spanPixel.textContent = pixel;
        spanPixel.addEventListener('click', () => {
          const host = window.location.href;
          const url = `${host}/find-node-pixel?x=${columnIndex}&y=${rowIndex}&id=${datasetImageValue}`;
          window.open(url, '_blank');
        });
        spanRow.appendChild(spanPixel);
      });

      spanRowContainer.appendChild(spanRow);
    });

    const divRowContainer = document.createElement('div');
    divRowContainer.className = 'themearc image rows-container';
    divRowContainer.appendChild(spanRowContainer);

    imageDiv.appendChild(divRowContainer);

    div.style.display = 'none';
}
