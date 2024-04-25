import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'
import { nodePolyfills } from 'vite-plugin-node-polyfills'
import path from "path";
import { loadGMSOLDeployment } from "./utils/load-deployment";
import { loadHttpsOptions } from './utils/load-https-options';

export default defineConfig(async ({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
  return {
    plugins: [
      nodePolyfills({
        include: ['buffer'],
      }),
      react({
        babel: {
          plugins: ['macros'],
          compact: mode == "development" ? false : undefined,
        }
      }),
    ],
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      }
    },
    define: {
      __GMSOL_DEPLOYMENT__: await loadGMSOLDeployment(env.GMSOL_DEPLOYMENT ? path.resolve(__dirname, env.GMSOL_DEPLOYMENT) : undefined),
    },
    server: {
      https: await loadHttpsOptions(env.GMSOL_SSL_DIR ? path.resolve(__dirname, env.GMSOL_SSL_DIR) : undefined),
    }
  }
})
