import { defineConfig } from "vite";

// Configuração recomendada pelo Tauri: porta fixa, sem limpar a tela do
// terminal (para ver os erros do Rust) e alvo compatível com as WebViews.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    target: "es2021",
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
