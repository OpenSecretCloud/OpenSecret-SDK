import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import derPlugin from "./vite-der-plugin";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), derPlugin()]
});
