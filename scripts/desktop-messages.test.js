import { describe, expect, it } from "vitest";
import { findDesktopMessages } from "./desktop-messages.mjs";
import { auditCatalogs } from "./i18n-check.mjs";

describe("native catalog integrity gate", () => {
  it("finds native source labels and conditional copy while excluding IDs and authored values", () => {
    const code = `
      // self.t("Comment");
      /* nested /* self.t("Comment") */ comment */
      self.message("Book {0}", &["Margaret".into()]);
      self.t(if ready { "Ready" } else { r#"Say "yes""# });
      self.decision("publish-button", "Publish", Action::Publish, cx);
      self.story_list("workflow-checks", "Checks", &items);
      Notice::with_args("Budget: {0}", vec![Money(50)]);
      let example = "self.t(\\"Example\\")";
    `;
    expect(findDesktopMessages(code, "screen.rs")).toEqual([
      "Book {0}", "Budget: {0}", "Checks", "Publish", "Ready", 'Say "yes"',
    ]);
  });

  it("rejects a missing native message and a translation that loses the user's value", () => {
    const messages = findDesktopMessages('self.t("New decision"); self.message("Book {0}", &[name]);', "screen.rs");
    expect(auditCatalogs(messages, { fr: { "Book {0}": "Réserver" } }, ["fr"])).toEqual([
      "fr: interpolation mismatch: Book {0}", "fr: missing translation: New decision",
    ]);
  });

  it("fails closed on malformed source rather than silently losing native messages", () => {
    expect(() => findDesktopMessages('self.t("Broken)', "broken.rs")).toThrow(/broken\.rs/);
    expect(() => findDesktopMessages('/* unclosed', "broken.rs")).toThrow(/broken\.rs/);
  });
});
