import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), tailwindcss()],
  server: {
    // During development the backend runs separately on port 3000.
    proxy: {
      '/api': 'http://127.0.0.1:3000',
    },
  },
})
