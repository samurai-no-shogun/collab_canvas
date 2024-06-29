import init, { Canvas } from './pkg/collaborative_canvas_client.js';

let canvas;
let currentColor = '#000000';
const totalPixels = 50 * 500;
let colorCounts = {};

async function initCanvas() {
    await init();
    canvas = new Canvas(50, 500);
    createPixelGrid();
    loadState();
    setupEventListeners();
}

function createPixelGrid() {
    const pixelCanvas = document.getElementById('pixelCanvas');
    const fragment = document.createDocumentFragment();
    for (let i = 0; i < totalPixels; i++) {
        const pixel = document.createElement('div');
        pixel.className = 'pixel';
        pixel.dataset.index = i;
        fragment.appendChild(pixel);
    }
    pixelCanvas.appendChild(fragment);
}

function loadState() {
    const state = localStorage.getItem('canvasState');
    if (state) {
        JSON.parse(state).forEach((color, index) => {
            if (color !== '#FFFFFF') {
                updatePixel(index, color);
            }
        });
    }
    updateStats();
}

function setupEventListeners() {
    document.getElementById('pixelCanvas').addEventListener('click', handlePixelClick);
    document.getElementById('colorPicker').addEventListener('change', (event) => {
        currentColor = event.target.value;
    });
}

function handlePixelClick(event) {
    if (event.target.classList.contains('pixel')) {
        const index = parseInt(event.target.dataset.index);
        updatePixel(index, currentColor);
        canvas.update_pixel(index % 50, Math.floor(index / 50), currentColor);
        saveState();
    }
}

function updatePixel(index, color) {
    const pixel = document.getElementById('pixelCanvas').children[index];
    const oldColor = pixel.style.backgroundColor || '#FFFFFF';
    pixel.style.backgroundColor = color;
    updateColorCount(oldColor, -1);
    updateColorCount(color, 1);
    updateStats();
}

function updateColorCount(color, change) {
    if (color === 'white' || color === '') color = '#FFFFFF';
    colorCounts[color] = (colorCounts[color] || 0) + change;
    if (colorCounts[color] <= 0) delete colorCounts[color];
}

function updateStats() {
    const stats = document.getElementById('stats');
    const coloredPixels = Object.values(colorCounts).reduce((a, b) => a + b, 0);
    const uncolored = totalPixels - coloredPixels;
    const topColors = Object.entries(colorCounts)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 3);

    let statsHtml = `Colored: ${(coloredPixels/totalPixels*100).toFixed(2)}% | Uncolored: ${(uncolored/totalPixels*100).toFixed(2)}% | Top colors: `;
    
    topColors.forEach(([color, count]) => {
        statsHtml += `<span class="colorSwatch" style="background-color:${color};"></span>${(count/totalPixels*100).toFixed(2)}% `;
    });

    stats.innerHTML = statsHtml;
}

function saveState() {
    const pixels = Array.from(document.getElementById('pixelCanvas').children).map(pixel => pixel.style.backgroundColor || '#FFFFFF');
    localStorage.setItem('canvasState', JSON.stringify(pixels));
}

initCanvas();