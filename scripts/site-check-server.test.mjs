import { afterAll, beforeAll, expect, test } from "vitest";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createSiteCheckServer } from "./site-check-server.mjs";

const media = "0123456789";
let root;
let server;
let base;

beforeAll(async () => {
  root = await mkdtemp(join(tmpdir(), "agely-check-server-"));
  await writeFile(join(root, "demo.mp4"), media);
  await writeFile(join(root, "index.html"), "<h1>Agely</h1>");
  server = createSiteCheckServer(root);
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  base = `http://127.0.0.1:${server.address().port}`;
});

afterAll(async () => {
  if (server) await new Promise((resolve) => server.close(resolve));
  if (root) await rm(root, { recursive: true, force: true });
});

test.each([
  ["bytes=3-6", "bytes 3-6/10", "3456"],
  ["bytes=8-", "bytes 8-9/10", "89"],
  ["bytes=-3", "bytes 7-9/10", "789"],
])("streams the requested media bytes for %s", async (range, contentRange, body) => {
  const response = await fetch(`${base}/demo.mp4`, { headers: { Range: range } });
  expect(response.status).toBe(206);
  expect(response.headers.get("accept-ranges")).toBe("bytes");
  expect(response.headers.get("content-range")).toBe(contentRange);
  expect(response.headers.get("content-length")).toBe(String(body.length));
  expect(await response.text()).toBe(body);
});

test("reports unsatisfiable ranges and still serves the media and page normally", async () => {
  const invalid = await fetch(`${base}/demo.mp4`, { headers: { Range: "bytes=10-" } });
  expect(invalid.status).toBe(416);
  expect(invalid.headers.get("content-range")).toBe("bytes */10");
  const full = await fetch(`${base}/demo.mp4`);
  expect(full.status).toBe(200);
  expect(await full.text()).toBe(media);
  const page = await fetch(base);
  expect(page.headers.get("content-type")).toBe("text/html");
  expect(await page.text()).toBe("<h1>Agely</h1>");
  expect((await fetch(`${base}/missing`)).status).toBe(404);
  expect((await fetch(`${base}/index.html/missing`)).status).toBe(404);
  expect((await fetch(`${base}/%00`)).status).toBe(400);
});
