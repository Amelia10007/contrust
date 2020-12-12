import { Universe } from "contrust/contrust_bg.js";
import { memory } from "contrust/contrust_bg";

const universe = Universe.new();

const universeCanvas = document.getElementById("universe-canvas");
const canvasContext = universeCanvas.getContext("2d");

universeCanvas.width = 800;
universeCanvas.height = 600;

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
        const x = xs[i];
        const y = ys[i];

        const radius = m ** (1 / 3);

        canvasContext.fillRect(x, y, radius, radius);
    }

    canvasContext.stroke();
};

const renderLoop = () => {
    universe.tick(1);

    drawUniverse();

    requestAnimationFrame(renderLoop);
};

universe.add_mass(10, 300, 300, 0.1, 0);
universe.add_mass(10, 200, 400, -0.1, 0);

// start rendering
renderLoop();
