const puppeteer = require('puppeteer');
const http = require('http');
const nodeStatic = require('node-static');
const path = require('path');

const file = new nodeStatic.Server(path.join(__dirname, 'web-dist'));

const server = http.createServer((req, res) => {
    req.addListener('end', () => file.serve(req, res)).resume();
}).listen(8080);

(async () => {
    console.log("--- 🌐 STARTING WEB SMOKE TEST ---");
    const browser = await puppeteer.launch({
        headless: "new",
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    const page = await browser.newPage();

    let runtimeError = false;

    page.on('console', msg => {
        const text = msg.text();
        console.log(`[BROWSER CONSOLE] ${text}`);
        if (text.toLowerCase().includes("panic") || text.toLowerCase().includes("error") || text.toLowerCase().includes("fatal")) {
            runtimeError = true;
        }
    });

    page.on('pageerror', err => {
        console.error(`[BROWSER ERROR] ${err.toString()}`);
        runtimeError = true;
    });

    try {
        await page.goto('http://localhost:8080', { waitUntil: 'networkidle0' });
        console.log("Page loaded, waiting 5 seconds for panics...");
        await new Promise(r => setTimeout(r, 5000));
    } catch (e) {
        console.error("Navigation failed:", e);
        runtimeError = true;
    }

    await browser.close();
    server.close();

    if (runtimeError) {
        console.error("--- ❌ WEB SMOKE TEST FAILED ---");
        process.exit(1);
    } else {
        console.log("--- ✅ WEB SMOKE TEST PASSED ---");
        process.exit(0);
    }
})();
