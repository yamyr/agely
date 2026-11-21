import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const read = (path: string): string => readFileSync(new URL(path, import.meta.url), "utf8");
const versionOf = (json: string): string => (JSON.parse(json) as { version: string }).version;

const packageVersion = versionOf(read("../package.json"));
const cargoVersion = /^version = "([^"]+)"/m.exec(read("../desktop/Cargo.toml"))?.[1];

describe("release manifests", () => {
  it("carry one version in package.json and the native Cargo.toml", () => {
    expect(packageVersion.length).toBeGreaterThan(0);
    expect(cargoVersion).toBe(packageVersion);
  });
});
