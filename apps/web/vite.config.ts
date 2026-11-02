import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    proxy: Object.fromEntries(['/v1', '/healthz', '/readyz', '/openapi.json', '/datasets/'].map((path) => [path, { target: process.env.API_PROXY_TARGET || 'http://localhost:8080', changeOrigin: true }])),
  },
  test: { environment: 'jsdom', setupFiles: ['./src/test-setup.ts'], exclude: ['e2e/**', 'node_modules/**'] },
});
