import { readFileSync } from "node:fs";
import { gzipSync } from "node:zlib";
import { describe, expect, it } from "vitest";
import { catalogs } from "../site/i18n/catalogs/index.js";
import { compactCatalogModule } from "./compact-catalogs.mjs";

async function loadCatalogs(code) {
  const moduleUrl = `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
  const module = await import(/* @vite-ignore */ moduleUrl);
  return module.catalogs;
}

describe("production translation catalogs", () => {
  it("loads as one module and preserves every draft without repeated keys", async () => {
    const compact = compactCatalogModule(catalogs);
    await expect(loadCatalogs(compact)).resolves.toEqual(catalogs);

    const rawModules = ["index", ...Object.keys(catalogs)];
    const originalGzipBytes = rawModules.reduce((bytes, file) => {
      const module = readFileSync(new URL(`../site/i18n/catalogs/${file}.js`, import.meta.url));
      return bytes + gzipSync(module).length;
    }, 0);
    expect(gzipSync(compact).length).toBeLessThan(originalGzipBytes);
  });

  it("keeps missing keys absent without losing a different locale's value", async () => {
    const source = {
      fr: { "Book {0}": "Réserver {0}" },
      de: { "More\ntext": "Mehr\nText", ["__proto__"]: "Ein eigener Schlüssel" },
    };
    const result = await loadCatalogs(compactCatalogModule(source));
    expect(result).toEqual(source);
    expect(Object.hasOwn(result.de, "Book {0}")).toBe(false);
    expect(Object.hasOwn(result.fr, "__proto__")).toBe(false);
    expect(Object.hasOwn(result.de, "__proto__")).toBe(true);
  });
});
