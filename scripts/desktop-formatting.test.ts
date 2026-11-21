import { describe, expect, it } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import formatting from "./desktop-formatting.json";
import { validateDesktopFormatting } from "./desktop-formatting.ts";

function replaceEnglish(value: unknown) {
  return [value, ...formatting.slice(1)];
}

describe("canonical desktop formatting", () => {
  it("exports fresh native assets without consulting host locale formatting", () => {
    const directory = mkdtempSync(join(tmpdir(), "agely-formatting-"));
    try {
      const preload = join(directory, "no-host-formatting.js");
      writeFileSync(preload, `
        for (const name of ["NumberFormat", "DateTimeFormat", "ListFormat"]) {
          Intl[name] = function () { throw new Error("Host locale formatting must not affect native assets"); };
        }
      `);
      const result = spawnSync("bun", ["--preload", preload, "scripts/export-desktop.ts", "--check"], {
        cwd: fileURLToPath(new URL("../", import.meta.url)), encoding: "utf8",
      });
      expect(result.status, result.stderr || result.error?.message).toBe(0);
      expect(result.stdout).toContain("Checked desktop content");
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });

  it("accepts the reviewed formatting for every advertised language", () => {
    expect(() => validateDesktopFormatting(formatting)).not.toThrow();
  });

  it("rejects a missing, duplicated or misidentified locale before native fallback can hide it", () => {
    expect(() => validateDesktopFormatting(formatting.slice(1))).toThrow(/every registered locale/);
    expect(() => validateDesktopFormatting([formatting[1], ...formatting.slice(1)])).toThrow(/en: id/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], intl: "en-GB" }))).toThrow(/en: intl/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], label: "Changed" }))).toThrow(/en: label/);
  });

  it("rejects incomplete number data that would lose digits, grouping or currency affixes", () => {
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], digits: ["0"] }))).toThrow(/digits/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], digits: Array(10).fill("0") }))).toThrow(/digits/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], grouping: { ...formatting[0].grouping, primary: 0 } }))).toThrow(/grouping primary/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], money: { positive: {} } }))).toThrow(/money positive/);
  });

  it("rejects date data that would omit a calendar value or confuse morning and afternoon", () => {
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], months: [] }))).toThrow(/months/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], weekdays: [] }))).toThrow(/weekdays/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], hour_cycle: "h25" }))).toThrow(/hour cycle/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], day_periods: ["", ""] }))).toThrow(/day_periods/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], date_pattern: formatting[0].date_pattern.filter(({ kind }) => kind !== "minute") }))).toThrow(/missing minute/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], date_pattern: [...formatting[0].date_pattern, { kind: "unsupported", value: "?" }] }))).toThrow(/unsupported part/);
  });

  it("rejects lost or reordered list slots that native list expansion cannot render", () => {
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], list_pair: "{0} and {0}" }))).toThrow(/list_pair/);
    expect(() => validateDesktopFormatting(replaceEnglish({ ...formatting[0], list_three: "{2}, {1}, {0}" }))).toThrow(/native list order/);
  });
});
