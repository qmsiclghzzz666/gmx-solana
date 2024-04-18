import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from "path";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react({
    babel: {
      plugins: ['macros'],
    }
  })],
  resolve: {
    alias: {
      "styles": path.resolve(__dirname, "./src/styles"),
      "img": path.resolve(__dirname, "./src/img"),
      "components": path.resolve(__dirname, "./src/components"),
    }
  }
})
