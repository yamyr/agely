import { readFile, writeFile, mkdir } from "node:fs/promises";
import { MARGARET_DRAFT, INTERVIEW_TURNS, PRICE_SUGGESTION, PROFILE_REPLIES, PRICE_REPLIES } from "../src/demo/script/margaret.ts";
import { DEV_BRIEF, BRIEF_TURNS } from "../src/demo/script/dev.ts";
import { OTHER_EXPERTS } from "../src/demo/script/experts.ts";
import { PROPOSED_WORKFLOW, DEBRIEF_TURNS } from "../src/demo/script/debrief.ts";
import { BRIEFS } from "../src/demo/live/briefs.ts";
import { LOCALES } from "../site/i18n/core.js";
import { catalogs } from "../site/i18n/catalogs/index.js";
import { auditCatalogs } from "./i18n-check.mjs";
import { findDesktopMessages } from "./desktop-messages.mjs";

const assets = new URL("../desktop/assets/", import.meta.url);
const check = process.argv.includes("--check");
const sample = new Date(Date.UTC(2024, 10, 21, 13, 5));

// The native renderer does not pass through the web parser, so audit its own message arguments before embedding catalogs.
const nativeMessages = new Set<string>();
for (const name of ["app", "screens", "model", "widgets"]) {
  const path = new URL(`../desktop/src/${name}.rs`, import.meta.url);
  for (const message of findDesktopMessages(await readFile(path, "utf8"), path.pathname)) nativeMessages.add(message);
}
const nativeErrors = auditCatalogs([...nativeMessages], catalogs, LOCALES.filter(({ id }) => id !== "en").map(({ id }) => id));
if (nativeErrors.length) throw new Error(`Native translations are incomplete:\n${nativeErrors.slice(0, 25).join("\n")}`);

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

const outputs = {
  "content.json": {
    interview_turns: INTERVIEW_TURNS, margaret: MARGARET_DRAFT, price_suggestion: PRICE_SUGGESTION,
    profile_replies: PROFILE_REPLIES, price_replies: PRICE_REPLIES, brief_turns: BRIEF_TURNS,
    brief: DEV_BRIEF, other_experts: OTHER_EXPERTS, debrief_turns: DEBRIEF_TURNS, workflow: PROPOSED_WORKFLOW,
  },
  "catalogs.json": { en: {}, ...catalogs },
  "formatting.json": formatting,
  "live_briefs.json": BRIEFS,
};

await mkdir(assets, { recursive: true });
for (const [name, value] of Object.entries(outputs)) {
  const text = JSON.stringify(value) + "\n";
  const path = new URL(name, assets);
  if (check) {
    if (await readFile(path, "utf8") !== text) throw new Error(`${name} is stale. Run bun scripts/export-desktop.ts.`);
  } else {
    await writeFile(path, text);
  }
}
console.log(`${check ? "Checked" : "Exported"} desktop content, catalogs, locale formatting and live briefs.`);
