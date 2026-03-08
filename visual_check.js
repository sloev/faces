const fs = require('fs');
const { PNG } = require('pngjs');

function analyzeImage(filePath) {
    console.log(`--- 🔬 ANALYZING ${filePath} ---`);
    if (!fs.existsSync(filePath)) {
        console.error("❌ File not found!");
        process.exit(1);
    }

    const data = fs.readFileSync(filePath);
    const png = PNG.sync.read(data);
    const pixels = png.data;
    const colorCounts = new Map();

    for (let i = 0; i < pixels.length; i += 4) {
        const r = pixels[i];
        const g = pixels[i + 1];
        const b = pixels[i + 2];
        const key = `${r},${g},${b}`;
        colorCounts.set(key, (colorCounts.get(key) || 0) + 1);
    }

    const totalPixels = png.width * png.height;
    const sortedColors = [...colorCounts.entries()].sort((a, b) => b[1] - a[1]);
    const dominantColor = sortedColors[0];
    const dominantPercentage = (dominantColor[1] / totalPixels) * 100;

    console.log(`Unique colors found: ${colorCounts.size}`);
    console.log(`Dominant color: ${dominantColor[0]} (${dominantPercentage.toFixed(2)}%)`);

    if (dominantPercentage > 95) {
        console.error("❌ FAIL: Image is over 95% a single color. Rendering is likely broken!");
        process.exit(1);
    }

    if (colorCounts.size < 100) {
        console.error("❌ FAIL: Low color variance detected. Image looks artificial.");
        process.exit(1);
    }

    console.log("✅ SUCCESS: High visual variance detected.");
    process.exit(0);
}

const target = process.argv[2] || 'ci_screenshot.png';
analyzeImage(target);
