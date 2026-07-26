/// <reference types="vitest" />
import path from 'node:path';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  resolve: {
    alias: {
      '~': path.resolve('./src'),
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
  },
});
