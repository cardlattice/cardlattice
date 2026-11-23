import stylex from "@stylexjs/unplugin";
import { defineConfig } from "vitest/config";
export default defineConfig({
  plugins: [stylex.vite({ dev: true, test: true, runtimeInjection: false, useCSSLayers: true })],
  test: {
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/**/*.test.{ts,tsx}"],
      thresholds: { perFile: true, statements: 100, branches: 100, functions: 100, lines: 100 },
    },
  },
});
