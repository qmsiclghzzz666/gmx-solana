import { defineConfig } from 'vite';
import { resolve } from 'path';
import dts from 'vite-plugin-dts';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

export default defineConfig({
    build: {
        lib: { entry: resolve(__dirname, 'src/main.ts'), formats: ['es', 'cjs'] },
        outDir: "../../dist/gmsol",
        emptyOutDir: true,
    },
    resolve: { alias: { src: resolve('src/') } },
    plugins: [
        nodePolyfills({
            include: ['buffer', 'crypto', 'stream', 'vm'],
        }),
        dts()
    ],
});
