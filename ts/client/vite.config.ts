import { defineConfig } from 'vite';
import { resolve } from 'path';
import dts from 'vite-plugin-dts';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

const isNode = process.env.BUILD_TARGET === "node";

export default defineConfig({
    build: {
        lib: { entry: resolve(__dirname, 'src/main.ts'), formats: ['es', 'cjs'] },
        outDir: "../../dist/gmsol",
        emptyOutDir: true,
        rollupOptions: {
            external: isNode ? ['crypto', 'buffer'] : [],
        },
        target: isNode ? 'node19' : undefined,
    },
    resolve: { alias: { src: resolve('src/') } },
    plugins: [
        nodePolyfills({
            include: ['buffer', 'crypto', 'stream', 'vm'],
        }),
        dts(),
    ].filter(Boolean),
});
