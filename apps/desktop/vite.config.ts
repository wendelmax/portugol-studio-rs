import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const repositoryRoot = fileURLToPath(new URL("../..", import.meta.url));

// Configuração recomendada pelo Tauri: porta fixa, sem limpar a tela do
// terminal (para ver os erros do Rust) e alvo compatível com as WebViews.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    fs: { allow: [repositoryRoot] },
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    rolldownOptions: {
      input: {
        ide: fileURLToPath(new URL("./index.html", import.meta.url)),
        graphics: fileURLToPath(new URL("./graphics.html", import.meta.url)),
      },
    },
    target: "es2021",
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
