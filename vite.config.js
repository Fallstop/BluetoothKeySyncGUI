import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from '@tailwindcss/vite';
import topLevelAwait from "vite-plugin-top-level-await";

// @ts-ignore
import path from "path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
		tailwindcss(),
		sveltekit(),
    // topLevelAwait({
    //   // The export name of top-level await promise for each chunk module
    //   promiseExportName: "__tla",
    //   // The function to generate import names of top-level await promise in each chunk module
    //   promiseImportName: i => `__tla_${i}`
    // })
		// w
	],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },

		fs: {
			allow: ["./bindings.ts"]
		}
  },
  resolve: {
    alias: {
      $lib: path.resolve("./src/lib"),
			"#root": path.resolve("./"),
    },
  },
}));
