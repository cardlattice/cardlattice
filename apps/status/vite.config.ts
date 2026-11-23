import stylex from "@stylexjs/unplugin";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
export default defineConfig({
  plugins: [stylex.vite({ useCSSLayers: { before: ["reset"] } }), react()],
  server: {
    host: "127.0.0.1",
    port: 8788,
    strictPort: true,
    allowedHosts: ["lake.tail3275e2.ts.net"],
  },
});
