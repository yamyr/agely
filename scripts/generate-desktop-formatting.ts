/**
 * Explicit maintenance tool; ordinary builds and freshness checks never call host Intl formatting.
 * The initial canonical source was reviewed on 2026-11-21 using Bun 1.4.0 on macOS (reported ICU 78.1).
 * Preview: bun scripts/generate-desktop-formatting.ts > /tmp/desktop-formatting.json
 * Replace the source: bun scripts/generate-desktop-formatting.ts --write
 * Review all locale changes, then run bun run desktop:assets and the native formatting tests.
 */
import { writeFile } from "node:fs/promises";
import { LOCALES } from "../site/i18n/core.js";
import { validateDesktopFormatting } from "./desktop-formatting.ts";

const args = process.argv.slice(2);
if (args.length > 1 || (args.length === 1 && args[0] !== "--write")) {
  throw new Error("Usage: bun scripts/generate-desktop-formatting.ts [--write]");
}
const sample = new Date(Date.UTC(2024, 10, 21, 13, 5));

function affixes(formatter: Intl.NumberFormat, value: number) {
  const parts = formatter.formatToParts(value);
  const first = parts.findIndex(({ type }) => type === "integer");
  const last = parts.findLastIndex(({ type }) => type === "integer");
  return { prefix: parts.slice(0, first).map(({ value }) => value).join(""), suffix: parts.slice(last + 1).map(({ value }) => value).join("") };
}

function numberPattern(locale: string, money: boolean) {
  const formatter = new Intl.NumberFormat(locale, money ? { style: "currency", currency: "USD", maximumFractionDigits: 0 } : {});
  const positive = affixes(formatter, 123456789012345);
  const negative = affixes(formatter, -123456789012345);
  return { positive, negative };
}

const formatting = LOCALES.map(({ id, label, intl }) => {
  const number = new Intl.NumberFormat(intl);
  const grouped = number.formatToParts(123456789012345);
  const groups = grouped.filter(({ type }) => type === "integer").map(({ value }) => [...value].length);
  let minimumDigits = 1;
  while (minimumDigits < 8 && !number.formatToParts(10 ** (minimumDigits - 1)).some(({ type }) => type === "group")) minimumDigits += 1;
  const dates = new Intl.DateTimeFormat(intl, { timeZone: "UTC", weekday: "long", month: "long", day: "numeric", hour: "numeric", minute: "2-digit" });
  const datePattern = dates.formatToParts(sample).map(({ type, value }) => ({ kind: type, value }));
  const monthName = (month: number) => dates.formatToParts(new Date(Date.UTC(2024, month, 21))).find(({ type }) => type === "month")?.value ?? "";
  const weekdayName = (day: number) => dates.formatToParts(new Date(Date.UTC(2024, 10, 17 + day))).find(({ type }) => type === "weekday")?.value ?? "";
  const dayPeriod = (hour: number) => dates.formatToParts(new Date(Date.UTC(2024, 10, 21, hour))).find(({ type }) => type === "dayPeriod")?.value ?? "";
  const lists = new Intl.ListFormat(intl, { style: "long", type: "conjunction" });
  return {
    id, label, intl,
    digits: Array.from({ length: 10 }, (_, n) => new Intl.NumberFormat(intl, { useGrouping: false }).format(n)),
    grouping: { separator: grouped.find(({ type }) => type === "group")?.value ?? "", primary: groups.at(-1) ?? 3, secondary: groups.at(-2) ?? 3, minimum_digits: minimumDigits },
    number: numberPattern(intl, false), money: numberPattern(intl, true),
    date_pattern: datePattern, months: Array.from({ length: 12 }, (_, month) => monthName(month)), weekdays: Array.from({ length: 7 }, (_, day) => weekdayName(day)),
    hour_cycle: dates.resolvedOptions().hourCycle ?? "h23", hour_two_digits: dates.resolvedOptions().hour === "2-digit", day_periods: [dayPeriod(1), dayPeriod(13)],
    list_pair: lists.format(["{0}", "{1}"]), list_three: lists.format(["{0}", "{1}", "{2}"]),
  };
});

validateDesktopFormatting(formatting);
const text = JSON.stringify(formatting, null, 2) + "\n";
if (args[0] === "--write") {
  await writeFile(new URL("./desktop-formatting.json", import.meta.url), text);
  console.error("Updated canonical desktop formatting. Review the locale changes before exporting native assets.");
} else {
  process.stdout.write(text);
}
const runtime = process.versions.bun ? `Bun ${process.versions.bun}` : `Node ${process.version}`;
console.error(`Formatting source: ${runtime}, reported ICU ${process.versions.icu ?? "unknown"}.`);
