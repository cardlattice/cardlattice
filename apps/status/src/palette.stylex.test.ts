// @vitest-environment node
import { fileURLToPath } from "node:url";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import stylex from "@stylexjs/unplugin";
import { createServer } from "vite";
import { expect, it } from "vitest";

it("serves valid light and dark palette values through Vite's development CSS endpoint", async () => {
  const cacheDir = await mkdtemp(join(tmpdir(), "status-palette-test-"));
  const server = await createServer({
    root: fileURLToPath(new URL("../", import.meta.url)),
    configFile: false,
    cacheDir,
    optimizeDeps: { noDiscovery: true },
    logLevel: "silent",
    plugins: [stylex.vite({ useCSSLayers: true, devPersistToDisk: false })],
    server: { host: "127.0.0.1", port: 0, strictPort: false },
  });
  try {
    await server.listen();
    await server.transformRequest("/src/palette.stylex.ts");
    const address = server.httpServer?.address();
    if (!address || typeof address === "string") {
      throw new Error("Test Vite server did not bind a TCP port");
    }
    const response = await fetch(`http://127.0.0.1:${address.port}/virtual:stylex.css`);
    expect(response.ok).toBe(true);
    const css = await response.text();
    expect(css).toContain("prefers-color-scheme: dark");
    expect(css).toContain("#e8e8e8");
    expect(css).toContain("#2a2a2a");
    // Vite's raw development reset does not define Lightning CSS's helper variables.
    // Lowering light-dark() to those helpers concatenated both fallback colours.
    expect(css).not.toContain("--lightningcss-light");
    expect(css).not.toContain("--lightningcss-dark");
  } finally {
    await server.close();
    await rm(cacheDir, { recursive: true, force: true });
  }
});
