import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { compression } from 'vite-plugin-compression2';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    tailwindcss(),
    sveltekit(),
    compression({ algorithms: ['brotliCompress'] }),
  ],
  server: {
    proxy: {
      '/api': 'http://localhost:4174',
    }
  },
});
