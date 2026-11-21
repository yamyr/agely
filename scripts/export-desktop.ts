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
import { validateDesktopFormatting } from "./desktop-formatting.ts";

const assets = new URL("../desktop/assets/", import.meta.url);
const check = process.argv.includes("--check");

// The native renderer does not pass through the web parser, so audit its own message arguments before embedding catalogs.
const nativeMessages = new Set<string>();
for (const name of ["app", "screens", "model", "widgets"]) {
  const path = new URL(`../desktop/src/${name}.rs`, import.meta.url);
  for (const message of findDesktopMessages(await readFile(path, "utf8"), path.pathname)) nativeMessages.add(message);
}
const nativeErrors = auditCatalogs([...nativeMessages], catalogs, LOCALES.filter(({ id }) => id !== "en").map(({ id }) => id));
if (nativeErrors.length) throw new Error(`Native translations are incomplete:\n${nativeErrors.slice(0, 25).join("\n")}`);

// Host ICU data differs between macOS and Linux. Embed the reviewed canonical source unchanged.
const formatting: unknown = JSON.parse(await readFile(new URL("./desktop-formatting.json", import.meta.url), "utf8"));
validateDesktopFormatting(formatting);

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
