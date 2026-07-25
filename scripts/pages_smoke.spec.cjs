const fs = require("node:fs");
const path = require("node:path");
const { chromium, expect, test } = require("@playwright/test");

const configuredUrl = process.env.PAGES_URL;
if (!configuredUrl) {
  throw new Error("PAGES_URL is required");
}

const baseUrl = new URL(configuredUrl.endsWith("/") ? configuredUrl : `${configuredUrl}/`);

const webglLaunchOptions = {
  executablePath: process.env.PLAYWRIGHT_EXECUTABLE_PATH || undefined,
  args: [
    "--use-gl=angle",
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
  ],
};

const webgpuLaunchOptions = {
  executablePath: process.env.PLAYWRIGHT_EXECUTABLE_PATH || undefined,
  args: [
    "--enable-unsafe-webgpu",
    "--use-webgpu-adapter=swiftshader",
    "--enable-dawn-features=allow_unsafe_apis",
    "--disable-dawn-features=use_dxc",
    "--enable-webgpu-developer-features",
    "--use-gpu-in-tests",
    "--enable-accelerated-2d-canvas",
    "--enable-unsafe-swiftshader",
  ],
};

async function withBrowserPage(launchOptions, callback) {
  const browser = await chromium.launch(launchOptions);
  try {
    const context = await browser.newContext({
      viewport: { width: 1280, height: 720 },
    });
    const page = await context.newPage();
    return await callback(page);
  } finally {
    await browser.close();
  }
}

test.describe("WebGL2 release smoke test", () => {
  test("both WASM files validate and a WebGL2 level starts", async () => {
    test.setTimeout(180_000);
    await withBrowserPage(webglLaunchOptions, async (page) => {
      const runtimeErrors = monitorRuntimeErrors(page);
      const screenshotPath = process.env.PAGES_SMOKE_SCREENSHOT || "test-results/pages-smoke.png";

      const { menuScreenshot, screenshot } = await launchLevel(page, "webgl2", screenshotPath);

      const wasmResults = await page.evaluate(async (rootUrl) => {
        const backends = ["webgl2", "webgpu"];
        return Promise.all(backends.map(async (backend) => {
          const wasmUrl = new URL(`bevy_open_siege_${backend}_bg.wasm`, rootUrl);
          wasmUrl.searchParams.set("browser_validation", `${Date.now()}`);
          const response = await fetch(wasmUrl);
          const bytes = await response.arrayBuffer();
          return {
            backend,
            ok: response.ok,
            status: response.status,
            contentType: response.headers.get("content-type"),
            byteLength: bytes.byteLength,
            valid: WebAssembly.validate(bytes),
          };
        }));
      }, baseUrl.href);

      for (const result of wasmResults) {
        expect(result.ok, `${result.backend} returned HTTP ${result.status}`).toBe(true);
        expect(result.contentType).toContain("application/wasm");
        expect(result.byteLength).toBeGreaterThan(1_000_000);
        expect(result.valid, `${result.backend} failed WebAssembly.validate`).toBe(true);
      }

      const moduleResults = await page.evaluate(async (rootUrl) => {
        const backends = ["webgl2", "webgpu"];
        return Promise.all(backends.map(async (backend) => {
          const moduleUrl = new URL(`bevy_open_siege_${backend}.js`, rootUrl);
          moduleUrl.searchParams.set("browser_validation", `${Date.now()}`);
          const module = await import(moduleUrl.href);
          return { backend, hasDefaultInitializer: typeof module.default === "function" };
        }));
      }, baseUrl.href);

      for (const result of moduleResults) {
        expect(result.hasDefaultInitializer, `${result.backend} loader has no default initializer`).toBe(true);
      }

      expect(runtimeErrors).toEqual([]);
      expect(screenshot.byteLength, "WebGL2 gameplay screenshot is blank").toBeGreaterThan(50_000);
      expect(screenshot.equals(menuScreenshot), "WebGL2 never left the title menu").toBe(false);
    });
  });
});

test.describe("WebGPU release smoke test", () => {
  test("the browser exposes a usable WebGPU adapter", async () => {
    test.setTimeout(60_000);
    await withBrowserPage(webgpuLaunchOptions, async (page) => {
      const probeUrl = new URL("web-build-info.txt", baseUrl);
      await page.goto(probeUrl.href, { waitUntil: "domcontentloaded" });
      const probe = await page.evaluate(async () => {
        const adapter = await navigator.gpu?.requestAdapter();
        if (!adapter) {
          return { hasAdapter: false };
        }

        const device = await adapter.requestDevice();
        const buffer = device.createBuffer({
          size: 4,
          usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE,
          mappedAtCreation: true,
        });
        buffer.unmap();
        buffer.destroy();

        return {
          hasAdapter: true,
          maxBufferSize: device.limits.maxBufferSize,
        };
      });

      expect(probe.hasAdapter, "Chromium did not provide the required WebGPU adapter").toBe(true);
      expect(probe.maxBufferSize).toBeGreaterThan(1_000_000);
    });
  });
});

function monitorRuntimeErrors(page) {
  const runtimeErrors = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      runtimeErrors.push(`console: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => {
    if (!error.message.startsWith("Using exceptions for control flow")) {
      runtimeErrors.push(`page: ${error.message}`);
    }
  });
  return runtimeErrors;
}

async function launchLevel(page, backend, screenshotPath) {
  const gameUrl = new URL(baseUrl);
  gameUrl.searchParams.set("backend", backend);
  gameUrl.searchParams.set("release_verifier", `${backend}-${Date.now()}`);
  await page.goto(gameUrl.href, { waitUntil: "domcontentloaded", timeout: 120_000 });

  await expect(page.locator("#loading")).toHaveCount(0, { timeout: 120_000 });
  const canvas = page.locator("canvas").first();
  await expect(canvas).toBeVisible({ timeout: 120_000 });

  const bounds = await canvas.boundingBox();
  expect(bounds).not.toBeNull();
  expect(bounds.width).toBeGreaterThan(320);
  expect(bounds.height).toBeGreaterThan(240);

  const menuScreenshot = await canvas.screenshot();
  await canvas.focus();
  await page.keyboard.press("Enter");
  await page.waitForTimeout(5_000);

  fs.mkdirSync(path.dirname(screenshotPath), { recursive: true });
  const screenshot = await canvas.screenshot({ path: screenshotPath });
  return { menuScreenshot, screenshot };
}
