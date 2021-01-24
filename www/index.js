import { Universe } from "contrust/contrust_bg.js";
import { memory } from "contrust/contrust_bg";

const infoDiv = document.getElementById("info");

const universe = Universe.new();

const universeCanvas = document.getElementById("universe-canvas");
const canvasContext = universeCanvas.getContext("2d");

universeCanvas.width = 800;
universeCanvas.height = 600;

let displayOffsetX = -universeCanvas.width / 2;
let displayOffsetY = -universeCanvas.height / 2;
let expantionRatio = 0;

/**
 * Draw current universe state.
 */
const drawUniverse = () => {
    canvasContext.beginPath();
    canvasContext.clearRect(0, 0, universeCanvas.clientWidth, universeCanvas.height);

    canvasContext.fillStyle = "#FFFFFF";

    const count = universe.mass_count();
    const massPtr = universe.mass_ptr();
    const ms = new Float64Array(memory.buffer, massPtr, count);
    const xPtr = universe.position_x_ptr();
    const xs = new Float64Array(memory.buffer, xPtr, count);
    const yPtr = universe.position_y_ptr();
    const ys = new Float64Array(memory.buffer, yPtr, count);

    const ratio = Math.pow(2, expantionRatio);


    for (let i = 0; i < count; i++) {
        // 画面に表示するときの座標を求める
        const m = ms[i] * ratio;
        const radius = m ** (1 / 3.5);
        const x = (xs[i] + displayOffsetX) * ratio - radius / 2 + universeCanvas.width / 2;
        const y = (ys[i] + displayOffsetY) * ratio - radius / 2 + universeCanvas.height / 2;

        canvasContext.fillRect(x, y, radius, radius);
    }

    canvasContext.stroke();

    infoDiv.textContent = `${count} stars`;
};

const fps = new class {
    constructor() {
        this.fps = document.getElementById("fps");
        this.frames = [];
        this.lastFrameTimeStamp = performance.now();
        this.capacity = 100;
    }

    render() {
        const now = performance.now();
        const duration = now - this.lastFrameTimeStamp;
        this.lastFrameTimeStamp = now;
        const fps = 1 / duration * 1000;

        this.frames.push(fps);
        if (this.frames.length > this.capacity) { this.frames.shift(); }

        let min = Infinity;
        let max = -Infinity;
        let sum = 0;
        for (let i = 0; i < this.frames.length; i++) {
            sum += this.frames[i];
            min = Math.min(min, this.frames[i]);
            max = Math.max(max, this.frames[i]);

        }
        const mean = sum / this.frames.length;

        this.fps.textContent = `
        FPS:
            latest = ${Math.round(fps)}
            avg   of last ${this.capacity} = ${Math.round(mean)}
            worst of last ${this.capacity} = ${Math.round(min)}
            best  of last ${this.capacity} = ${Math.round(max)}
        `.trim();
    }
};

const renderLoop = () => {
    fps.render();

    universe.tick(0.05);

    drawUniverse();

    requestAnimationFrame(renderLoop);
};

for (let theta = 0; theta < 360; theta += 10) {
    const radian = theta / 180.0 * Math.PI;
    for (let r = 100; r < 200; r += 2) {
        const m = 1;
        const x = r * Math.cos(radian) + universeCanvas.width / 2.0;
        const y = r * Math.sin(radian) + universeCanvas.height / 2.0;

        const V = 250;

        const u = Math.pow(r + 100, -1 / 1.5) * Math.cos(radian + Math.PI / 2.0) * V;
        const v = Math.pow(r + 100, -1 / 1.5) * Math.sin(radian + Math.PI / 2.0) * V;

        universe.add_mass(m, x, y, u, v);
    }
}

universe.add_mass(10000, universeCanvas.width / 2.0, universeCanvas.height / 2.0, 0, 0);

universe.set_minimum_ratio_for_integration(2.0);

// start rendering
renderLoop();

document.body.addEventListener('keydown', e => {
    const offset = 10;
    const shift = offset * Math.pow(2, -expantionRatio);
    switch (e.key) {
        case 'a': displayOffsetX += shift; break;
        case 'd': displayOffsetX -= shift; break;
        case 'w': displayOffsetY += shift; break;
        case 's': displayOffsetY -= shift; break;
        case 'z': expantionRatio++; break;
        case 'x': expantionRatio--; break;
    }
});
