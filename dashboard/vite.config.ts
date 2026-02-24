import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    // In development, proxy all /api requests to the Rust backend.
    // The React code calls the backend directly via VITE_API_BASE_URL in
    // production; the proxy is purely a convenience for local development
    // (avoids CORS issues when running `npm run dev`).
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})
