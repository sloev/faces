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
        if (text.toLowerCase().includes("panic") || text.toLowerCase().includes("runtimeerror")) {
            runtimeError = true;
        }
    });

    page.on('pageerror', err => {
        console.error(`[BROWSER ERROR] ${err.toString()}`);
        runtimeError = true;
    });

    try {
        await page.goto('http://localhost:8080', { waitUntil: 'networkidle0', timeout: 30000 });
        console.log("Page loaded, waiting 5 seconds for stability...");
        await new Promise(r => setTimeout(r, 5000));
        
        console.log("📸 CAPTURING SCREENSHOT...");
        await page.screenshot({ path: 'ci_screenshot.png' });
        console.log("Screenshot saved to ci_screenshot.png");
        
    } catch (e) {
        console.error("Navigation failed:", e);
        runtimeError = true;
    }

    await browser.close();
    server.kill();

    if (runtimeError || !successSignal) {
        if (!successSignal) console.error("--- ❌ WEB SMOKE TEST FAILED: Never reached RENDER_LOOP_STARTED ---");
        else console.error("--- ❌ WEB SMOKE TEST FAILED ---");
        process.exit(1);
    } else {
        console.log("--- ✅ WEB SMOKE TEST PASSED ---");
        process.exit(0);
    }
})();
