import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  root: 'src',
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL('./src/index.html', import.meta.url)),
        floating: fileURLToPath(new URL('./src/floating.html', import.meta.url)),
      },
    },
  },
});
