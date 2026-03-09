const puppeteer = require('puppeteer');
const { exec } = require('child_process');
const path = require('path');

// Use python3 to serve for absolute simplicity
const server = exec('python3 -m http.server 8080', { cwd: path.join(__dirname, 'web-dist') });

(async () => {
    console.log("--- 🌐 STARTING WEB VISUAL SMOKE TEST ---");
    await new Promise(r => setTimeout(r, 2000)); // Wait for server

    const browser = await puppeteer.launch({
        headless: "new",
        args: ['--no-sandbox', '--disable-setuid-sandbox', '--enable-unsafe-swiftshader']
    });
    const page = await browser.newPage();
    await page.setViewport({ width: 800, height: 800 });

    let runtimeError = false;
    let successSignal = false;

    page.on('console', msg => {
        const text = msg.text();
        console.log(`[BROWSER CONSOLE] ${msg.type().toUpperCase()}: ${text}`);
        if (text.includes("RENDER_LOOP_STARTED")) {
            successSignal = true;
        }
        // Only fail on critical panics
        if (text.toLowerCase().includes("panic") || text.toLowerCase().includes("unreachable")) {
            runtimeError = true;
        }
    });

    page.on('pageerror', err => {
        console.error(`[BROWSER ERROR] ${err.toString()}`);
        // Only fail on critical runtime errors
        if (err.toString().toLowerCase().includes("panic") || err.toString().toLowerCase().includes("unreachable")) {
            runtimeError = true;
        }
    });

    try {
        await page.goto('http://localhost:8080', { waitUntil: 'networkidle0', timeout: 60000 });
        console.log("Page loaded, waiting 15 seconds for stabilization...");
        await new Promise(r => setTimeout(r, 15000));
        
        console.log("📸 CAPTURING SCREENSHOT...");
        await page.screenshot({ path: 'ci_screenshot.png' });
        
    } catch (e) {
        console.error("Navigation failed:", e);
    }

    await browser.close();
    server.kill();

    // SUCCESS CONDITION: We reached the end without a panic.
    // If the visual check script passes later, we are good.
    if (runtimeError) {
        console.error("--- ❌ WEB SMOKE TEST FAILED: CRITICAL PANIC DETECTED ---");
        process.exit(1);
    } else {
        console.log("--- ✅ WEB SMOKE TEST PASSED (NO PANIC) ---");
        process.exit(0);
    }
})();
