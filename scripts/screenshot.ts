/**
 * Generate a screenshot of the app in demo mode for the landing page.
 *
 * Usage: npx ts-node scripts/screenshot.ts
 * Or: npm run screenshot
 */

import { chromium } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import * as path from 'path';

const PORT = 4200;
const URL = `http://localhost:${PORT}?demo`;
const OUTPUT_PATH = path.join(__dirname, '../website/assets/screenshot.png');
const VIEWPORT = { width: 1200, height: 800 };

async function waitForServer(url: string, timeout = 60000): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Server not ready yet
    }
    await new Promise(r => setTimeout(r, 500));
  }
  throw new Error(`Server did not start within ${timeout}ms`);
}

async function main() {
  let server: ChildProcess | null = null;

  try {
    console.log('Starting development server...');
    server = spawn('npm', ['run', 'start', '--', '--port', PORT.toString()], {
      cwd: path.join(__dirname, '..'),
      stdio: 'pipe',
    });

    // Log server output
    server.stdout?.on('data', (data) => {
      const line = data.toString().trim();
      if (line) console.log(`[server] ${line}`);
    });
    server.stderr?.on('data', (data) => {
      const line = data.toString().trim();
      if (line) console.log(`[server] ${line}`);
    });

    console.log(`Waiting for server at ${URL}...`);
    await waitForServer(URL);
    console.log('Server is ready!');

    // Wait a bit more for Angular to fully initialize
    await new Promise(r => setTimeout(r, 2000));

    console.log('Launching browser...');
    const browser = await chromium.launch();
    const context = await browser.newContext({
      viewport: VIEWPORT,
      deviceScaleFactor: 2, // Retina screenshot
    });
    const page = await context.newPage();

    console.log('Loading page...');
    await page.goto(URL);

    // Wait for the app to be fully loaded
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000); // Extra time for animations

    // Hide any overlays that might interfere with the screenshot
    await page.evaluate(() => {
      // Hide debug console toggle button
      const debugToggle = document.querySelector('.debug-toggle');
      if (debugToggle) (debugToggle as HTMLElement).style.display = 'none';
    });

    console.log(`Taking screenshot...`);
    await page.screenshot({
      path: OUTPUT_PATH,
      type: 'png',
    });

    console.log(`Screenshot saved to: ${OUTPUT_PATH}`);

    await browser.close();
  } finally {
    if (server) {
      console.log('Stopping server...');
      server.kill('SIGTERM');
    }
  }
}

main().catch((error) => {
  console.error('Error:', error);
  process.exit(1);
});
