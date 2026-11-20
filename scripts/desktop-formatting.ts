import { LOCALES } from "../site/i18n/core.js";

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringArray(value: unknown, length: number): value is string[] {
  return Array.isArray(value) && value.length === length && value.every((entry) => typeof entry === "string");
}

function fail(locale: string, field: string): never {
  throw new Error(`Invalid desktop formatting for ${locale}: ${field}`);
}

/** Validate the canonical source at export time so malformed metadata cannot reach native formatting. */
export function validateDesktopFormatting(value: unknown): void {
  if (!Array.isArray(value) || value.length !== LOCALES.length) {
    throw new Error("Desktop formatting must contain every registered locale exactly once, in registry order.");
  }
  for (const [index, locale] of LOCALES.entries()) {
    const language: unknown = value[index];
    if (!record(language)) fail(locale.id, "expected an object");
    for (const key of ["id", "label", "intl"] as const) {
      if (language[key] !== locale[key]) fail(locale.id, `${key} must match the shared locale registry`);
    }
    if (!stringArray(language.digits, 10) || language.digits.some((digit) => !digit) || new Set(language.digits).size !== 10) {
      fail(locale.id, "digits must contain ten distinct, nonempty values");
    }
    for (const [key, length] of [["months", 12], ["weekdays", 7]] as const) {
      if (!stringArray(language[key], length) || language[key].some((entry) => !entry.trim())) {
        fail(locale.id, `${key} must contain ${length} nonempty values`);
      }
    }
    const grouping = language.grouping;
    if (!record(grouping) || typeof grouping.separator !== "string") fail(locale.id, "grouping separator must be a string");
    for (const key of ["primary", "secondary", "minimum_digits"]) {
      if (typeof grouping[key] !== "number" || !Number.isSafeInteger(grouping[key]) || grouping[key] <= 0) {
        fail(locale.id, `grouping ${key} must be a positive safe integer`);
      }
    }
    for (const key of ["number", "money"]) {
      const pattern = language[key];
      if (!record(pattern)) fail(locale.id, `${key} must contain positive and negative affixes`);
      for (const sign of ["positive", "negative"]) {
        const affixes = pattern[sign];
        if (!record(affixes) || typeof affixes.prefix !== "string" || typeof affixes.suffix !== "string") {
          fail(locale.id, `${key} ${sign} prefix and suffix must be strings`);
        }
      }
    }
    if (typeof language.hour_cycle !== "string" || !["h11", "h12", "h23", "h24"].includes(language.hour_cycle)) {
      fail(locale.id, "unsupported hour cycle");
    }
    if (typeof language.hour_two_digits !== "boolean") fail(locale.id, "hour_two_digits must be a boolean");
    const twelveHour = ["h11", "h12"].includes(language.hour_cycle);
    if (!stringArray(language.day_periods, 2) || (twelveHour && language.day_periods.some((period) => !period.trim()))) {
      fail(locale.id, "day_periods must contain two values, nonempty for a 12-hour clock");
    }
    if (!Array.isArray(language.date_pattern)) fail(locale.id, "date_pattern must be an array");
    const dateFields = new Set<string>();
    for (const part of language.date_pattern) {
      if (!record(part) || typeof part.kind !== "string" || typeof part.value !== "string"
          || !["literal", "weekday", "month", "day", "year", "hour", "minute", "dayPeriod"].includes(part.kind)) {
        fail(locale.id, "date_pattern contains an unsupported part");
      }
      if (part.kind !== "literal") {
        if (dateFields.has(part.kind)) fail(locale.id, `date_pattern repeats ${part.kind}`);
        dateFields.add(part.kind);
      }
    }
    for (const field of ["weekday", "month", "day", "hour", "minute"]) {
      if (!dateFields.has(field)) fail(locale.id, `date_pattern is missing ${field}`);
    }
    if (dateFields.has("dayPeriod") !== twelveHour) fail(locale.id, "date_pattern dayPeriod must match the hour cycle");
    for (const [field, slots] of [["list_pair", ["{0}", "{1}"]], ["list_three", ["{0}", "{1}", "{2}"]]] as const) {
      const pattern = language[field];
      if (typeof pattern !== "string" || (pattern.match(/\{\d+\}/g) ?? []).sort().join() !== [...slots].sort().join()) {
        fail(locale.id, `${field} must contain each list slot exactly once`);
      }
      // Native lists longer than three reuse the separator before {1} and the ending after it.
      if (field === "list_three" && (!pattern.startsWith("{0}") || pattern.indexOf("{1}") > pattern.indexOf("{2}"))) {
        fail(locale.id, "list_three must keep slots in native list order");
      }
    }
  }
}
