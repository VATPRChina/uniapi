import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import path from "path";
import { defineConfig, loadEnv, splitVendorChunkPlugin } from "vite";

const env = loadEnv("development", process.cwd(), "");

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [TanStackRouterVite(), react(), splitVendorChunkPlugin()],
  server: {
    proxy: {
      "/api": {
        target: env.VITE_PROXY_API ?? "https://localhost:5001",
        secure: false,
        changeOrigin: true,
      },
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
