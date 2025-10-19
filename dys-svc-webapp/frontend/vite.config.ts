import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vueJsx(),
    vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
      '%': fileURLToPath(new URL('./generated', import.meta.url))
    }
  },
  server: {
    proxy: {
      '/api': 'http://172.18.0.1:6050',
    },
    strictPort: true,
    hmr: {
      host: "localhost",
      protocol: "ws",
    },
    watch: {
      usePolling: true
    },
  }
})
