const puppeteer = require('puppeteer');
const { exec } = require('child_process');
const path = require('path');

const server = exec('python3 -m http.server 8080', { cwd: path.join(__dirname, 'web-dist') });

(async () => {
    console.log("--- 🌐 STARTING STRICT WEB SMOKE TEST ---");
    await new Promise(r => setTimeout(r, 2000));

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
        const type = msg.type().toUpperCase();
        console.log(`[BROWSER CONSOLE] ${type}: ${text}`);
        
        if (text.includes("RENDER_LOOP_STARTED")) successSignal = true;
        
        // FAIL ON ANY ERROR OR PANIC (Ignoring favicon.ico)
        if (type === "ERROR" && !text.includes("favicon.ico")) {
            console.error(`❌ CRITICAL ERROR DETECTED: ${text}`);
            runtimeError = true;
        }
        if (text.toLowerCase().includes("panic") || text.toLowerCase().includes("unreachable")) {
            console.error(`❌ CRITICAL PANIC DETECTED: ${text}`);
            runtimeError = true;
        }

    });

    page.on('pageerror', err => {
        console.error(`[BROWSER PAGE ERROR] ${err.toString()}`);
        runtimeError = true;
    });

    try {
        await page.goto('http://localhost:8080', { waitUntil: 'networkidle0', timeout: 30000 });
        console.log("Page loaded, waiting 10 seconds for stabilization...");
        await new Promise(r => setTimeout(r, 10000));
        
        console.log("📸 CAPTURING SCREENSHOT...");
        await page.screenshot({ path: 'ci_screenshot.png' });
        
    } catch (e) {
        console.error("Navigation failed:", e);
        runtimeError = true;
    }

    await browser.close();
    server.kill();

    if (runtimeError || !successSignal) {
        console.error("--- ❌ WEB SMOKE TEST FAILED ---");
        process.exit(1);
    } else {
        console.log("--- ✅ WEB SMOKE TEST PASSED ---");
        process.exit(0);
    }
})();
