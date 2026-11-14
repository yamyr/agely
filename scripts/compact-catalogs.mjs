/** Emit the browser module with each English source stored once across all locales. */
export function compactCatalogModule(catalogs) {
  const messages = [...new Set(Object.values(catalogs).flatMap(Object.keys))];
  const translations = Object.entries(catalogs).map(([locale, catalog]) => {
    const values = messages.map((source) => {
      if (!Object.hasOwn(catalog, source)) return null;
      const value = catalog[source];
      if (typeof value !== "string") throw new TypeError(`Expected a string for ${locale}: ${source}`);
      return value;
    });
    return [locale, values];
  });

  return `// Generated from site/i18n/catalogs.
const messages = ${JSON.stringify(messages)};
const translations = ${JSON.stringify(translations)};

export const catalogs = Object.fromEntries(translations.map(([locale, values]) => {
  const entries = [];
  for (const [index, source] of messages.entries()) {
    const value = values[index];
    if (value !== null) entries.push([source, value]);
  }
  return [locale, Object.fromEntries(entries)];
}));
`;
}
