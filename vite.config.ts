/// <reference types="vitest/config" />
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { defineConfig, normalizePath, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import { configDefaults } from "vitest/config";
import { compactCatalogModule } from "./scripts/compact-catalogs.mjs";

const host = process.env.TAURI_DEV_HOST;

const { version } = JSON.parse(
  readFileSync(fileURLToPath(new URL("./package.json", import.meta.url)), "utf-8"),
) as { version: string };

// Website build (agely.dev): the landing page in site/ is served at / and the app at /demo/.
// Cloudflare Pages sets CF_PAGES=1; locally use `bun run build:site`. Without the flag the build
// is the plain app in dist/, which is what Tauri's frontendDist expects.
const siteBuild = process.env.AGELY_SITE_BUILD === "1" || process.env.CF_PAGES === "1";

const rootPath = (relative: string) => fileURLToPath(new URL(relative, import.meta.url));
const sourceCatalogDir = rootPath("./site/i18n/catalogs");
const catalogIndex = normalizePath(rootPath("./site/i18n/catalogs/index.js"));

function productionCatalogs(code: string): Plugin {
  return {
    name: "agely-production-catalogs",
    load(id) {
      if (id === catalogIndex) return code;
    },
  };
}

function landingPage(catalogCode: string): Plugin {
  return {
    name: "agely-landing-page",
    // Clear all of dist/, not only dist/demo, so a previous app build can't leak into the site.
    buildStart() {
      rmSync(rootPath("./dist"), { recursive: true, force: true });
    },
    closeBundle() {
      cpSync(rootPath("./site"), rootPath("./dist"), {
        recursive: true,
        // Unit tests live beside the simulator's model but are not part of the website.
        filter: (source) => source !== sourceCatalogDir && !source.endsWith(".test.js"),
      });
      mkdirSync(rootPath("./dist/i18n/catalogs"), { recursive: true });
      writeFileSync(rootPath("./dist/i18n/catalogs/index.js"), catalogCode);
    },
  };
}

async function productionPlugins() {
  const { catalogs } = await import("./site/i18n/catalogs/index.js");
  const code = compactCatalogModule(catalogs);
  return [productionCatalogs(code), ...(siteBuild ? [landingPage(code)] : [])];
}

// https://vite.dev/config/
export default defineConfig(async ({ command }) => ({
  base: siteBuild ? "/demo/" : "/",
  build: siteBuild ? { outDir: "dist/demo", emptyOutDir: true } : {},
  plugins: [react(), ...(command === "build" ? await productionPlugins() : [])],
  define: {
    __APP_VERSION__: JSON.stringify(version),
  },
  test: {
    exclude: [...configDefaults.exclude, "e2e/**"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
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
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
