import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  server: { host: "127.0.0.1", port: 1420, strictPort: true },
  clearScreen: false,
  build: {
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL("./index.html", import.meta.url)),
        wheel: fileURLToPath(new URL("./wheel.html", import.meta.url)),
      },
    },
  },
  test: { environment: "node", include: ["src/**/*.test.ts"] },
});
