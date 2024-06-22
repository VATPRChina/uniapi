import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv, splitVendorChunkPlugin } from "vite";

const env = loadEnv("development", process.cwd(), "");

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), splitVendorChunkPlugin()],
  server: {
    proxy: {
      "/api": {
        target: env.VITE_PROXY_API ?? "https://localhost:5001",
        secure: false,
        changeOrigin: true,
      },
    },
  },
});
