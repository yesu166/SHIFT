import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  server: { host: "127.0.0.1", port: 1420, strictPort: true },
  clearScreen: false,
  build: {
    rollupOptions: {
      input: {
        main: "index.html",
        wheel: "wheel.html",
      },
    },
  },
  test: { environment: "node", include: ["src/**/*.test.ts"] },
});
