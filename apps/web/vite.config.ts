import { defineConfig } from "vitest/config";
import { createLogger } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const logger = createLogger();

export default defineConfig({
  customLogger: logger,
  plugins: [
    react(),
    tailwindcss(),
    {
      name: "fail-on-build-warnings",
      apply: "build",
      closeBundle() {
        if (logger.hasWarned)
          throw new Error("Build emitted warnings. Fix them before publishing.");
      },
    },
  ],
  server: {
    proxy: Object.fromEntries(
      ["/v1", "/healthz", "/readyz", "/openapi.json", "/datasets/"].map((path) => [
        path,
        { target: process.env.API_PROXY_TARGET || "http://localhost:8080", changeOrigin: true },
      ]),
    ),
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
    exclude: ["e2e/**", "node_modules/**"],
  },
});
