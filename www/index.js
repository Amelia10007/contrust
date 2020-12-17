import { Universe } from "contrust/contrust_bg.js";
import { memory } from "contrust/contrust_bg";

const infoDiv = document.getElementById("info");

const universe = Universe.new();

const universeCanvas = document.getElementById("universe-canvas");
const canvasContext = universeCanvas.getContext("2d");

universeCanvas.width = 800;
universeCanvas.height = 600;

let display_offset_x = 0;
let display_offset_y = 0;

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

    for (let i = 0; i < count; i++) {
        const m = ms[i];
        const x = xs[i] + display_offset_x;
        const y = ys[i] + display_offset_y;

        const radius = m ** (1 / 3);

        canvasContext.fillRect(x - radius / 2, y - radius / 2, radius, radius);
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

for (let theta = 0; theta < 360; theta += 5) {
    const radian = theta / 180.0 * Math.PI;
    for (let r = 10; r < 200; r += 5) {
        const m = 10;
        const x = r * Math.cos(radian) + universeCanvas.width / 2.0;
        const y = r * Math.sin(radian) + universeCanvas.height / 2.0;

        const V = 0.2;
        const R = 3.5;

        const u = r * Math.cos(radian + Math.PI / 2.0) * V + (Math.random() - 0.5) * R;
        const v = r * Math.sin(radian + Math.PI / 2.0) * V + (Math.random() - 0.5) * R;

        universe.add_mass(m, x, y, u, v);
    }
}

universe.set_minimum_ratio_for_integration(2.0);

// start rendering
renderLoop();

document.body.addEventListener('keydown', e => {
    const OFFSET = 10;
    switch (e.key) {
        case 'a': display_offset_x += OFFSET; break;
        case 'd': display_offset_x -= OFFSET; break;
        case 'w': display_offset_y += OFFSET; break;
        case 's': display_offset_y -= OFFSET; break;
    }
});
