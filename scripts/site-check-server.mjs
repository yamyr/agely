import { createReadStream, statSync } from "node:fs";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";

const TYPES = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json", ".png": "image/png", ".jpg": "image/jpeg", ".svg": "image/svg+xml", ".woff2": "font/woff2", ".mp4": "video/mp4", ".txt": "text/plain" };

function byteRange(header, size) {
  const match = /^bytes=(\d*)-(\d*)$/i.exec(header);
  if (match === null || (match[1] === "" && match[2] === "")) return undefined;
  const [begin, finish] = match.slice(1);
  const start = begin === "" ? Math.max(0, size - Number(finish)) : Number(begin);
  const end = begin === "" || finish === "" ? size - 1 : Math.min(Number(finish), size - 1);
  return start <= end ? { start, end } : null;
}

export function createSiteCheckServer(root) {
  const directory = resolve(root);
  return createServer((req, res) => {
    let pathname;
    try {
      pathname = decodeURIComponent(new URL(req.url, "http://localhost").pathname);
    } catch {
      return res.writeHead(400).end();
    }
    if (!pathname.startsWith("/") || pathname.includes("\0")) return res.writeHead(400).end();
    let file = resolve(directory, `.${pathname}`);
    if (file !== directory && !file.startsWith(`${directory}${sep}`)) return res.writeHead(403).end();
    let entry = statSync(file, { throwIfNoEntry: false });
    if (entry?.isDirectory()) {
      file = resolve(file, "index.html");
      entry = statSync(file, { throwIfNoEntry: false });
    }
    if (!entry?.isFile()) return res.writeHead(404).end();

    const size = entry.size;
    const ext = extname(file);
    const headers = { "content-type": TYPES[ext] ?? "application/octet-stream", "content-length": size };
    const range = ext === ".mp4" && req.method === "GET" ? byteRange(req.headers.range, size) : undefined;
    if (ext === ".mp4") headers["accept-ranges"] = "bytes";
    if (range === null) return res.writeHead(416, { "content-range": `bytes */${size}` }).end();
    if (range) {
      headers["content-range"] = `bytes ${range.start}-${range.end}/${size}`;
      headers["content-length"] = range.end - range.start + 1;
    }
    res.writeHead(range ? 206 : 200, headers);
    if (req.method === "HEAD") return res.end();
    const stream = createReadStream(file, range);
    stream.on("error", (error) => res.destroy(error));
    res.on("close", () => stream.destroy());
    stream.pipe(res);
  });
}
