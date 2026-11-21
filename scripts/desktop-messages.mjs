/** Inventory native message arguments without mistaking comments, IDs or interpolation values for UI copy. */
export function findDesktopMessages(code, filename) {
  const tokens = [];
  for (let index = 0; index < code.length;) {
    const remaining = code.slice(index);
    if (/^\s/.test(remaining)) { index += 1; continue; }
    if (remaining.startsWith("//")) { index = code.indexOf("\n", index); if (index < 0) break; continue; }
    if (remaining.startsWith("/*")) {
      let depth = 1;
      index += 2;
      while (depth && index < code.length) {
        if (code.startsWith("/*", index)) { depth += 1; index += 2; }
        else if (code.startsWith("*/", index)) { depth -= 1; index += 2; }
        else index += 1;
      }
      if (depth) throw new Error(`Unclosed Rust comment in ${filename}`);
      continue;
    }
    const raw = remaining.match(/^r(#+)?"/);
    if (raw) {
      const end = code.indexOf(`"${raw[1] ?? ""}`, index + raw[0].length);
      if (end < 0) throw new Error(`Unclosed Rust raw string in ${filename}`);
      tokens.push({ kind: "string", value: code.slice(index + raw[0].length, end) });
      index = end + 1 + (raw[1]?.length ?? 0);
      continue;
    }
    const string = remaining.match(/^"(?:\\[\s\S]|[^"\\])*"/);
    if (string) {
      tokens.push({ kind: "quoted", value: string[0] });
      index += string[0].length;
      continue;
    }
    if (remaining.startsWith('"')) throw new Error(`Unclosed Rust string in ${filename}`);
    const character = remaining.match(/^'(?:\\.|[^'\\])'/u);
    if (character) { index += character[0].length; continue; }
    const token = remaining.match(/^(?:[A-Za-z_]\w*|::|\S)/u)?.[0];
    if (!token) throw new Error(`Cannot inventory Rust source in ${filename}`);
    tokens.push({ kind: "token", value: token });
    index += token.length;
  }

  const messages = new Set();
  for (let index = 0; index < tokens.length; index += 1) {
    const name = tokens[index].value;
    const member = tokens[index - 1]?.value === ".";
    const notice = tokens[index - 1]?.value === "::" && tokens[index - 2]?.value === "Notice";
    const argument = member && ["decision", "story_list"].includes(name) ? 1 : 0;
    if (!(member && ["t", "message", "decision", "story_list"].includes(name))
        && !(notice && ["error", "info", "with_args"].includes(name))) continue;
    if (tokens[index + 1]?.value !== "(") continue;
    let depth = 0;
    let current = 0;
    for (let cursor = index + 2; cursor < tokens.length; cursor += 1) {
      const token = tokens[cursor];
      if (token.kind === "token") {
        if (depth === 0 && token.value === ")") break;
        if (depth === 0 && token.value === ",") { current += 1; continue; }
        if (["(", "[", "{"].includes(token.value)) depth += 1;
        if ([")", "]", "}"].includes(token.value)) depth -= 1;
      } else if (current === argument) {
        // Native UI uses ordinary UTF-8 messages; unsupported Rust-only escapes fail this gate.
        try { messages.add(token.kind === "string" ? token.value : JSON.parse(token.value)); }
        catch { throw new Error(`Unsupported message escape in ${filename}: ${token.value}`); }
      }
    }
  }
  return [...messages].sort();
}
