import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [svelte()],
    server: {
        cors: true,
        proxy: {
            "/api": {
                target: "http://127.0.0.1:3002",
                changeOrigin: true,
                secure: false,
            },
        },
    },
});
