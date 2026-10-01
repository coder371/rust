import http from "node:http";

import { config } from "./config.js";
import { counters } from "./fakedb.js";
import { renderStorefrontPage } from "./pipeline.js";

const server = http.createServer(async (req, res) => {
  if (req.url === "/health") {
    res.writeHead(200, { "Content-Type": "text/plain" });
    return res.end("ok");
  }

  if (req.url === "/stats") {
    counters.reset();
    const started = process.hrtime.bigint();
    const html = await renderStorefrontPage();
    const ms = Number(process.hrtime.bigint() - started) / 1e6;

    res.writeHead(200, { "Content-Type": "application/json" });
    return res.end(
      JSON.stringify({ mode: config.label, ms: +ms.toFixed(1), htmlBytes: html.length, ...counters.snapshot() }),
    );
  }

  try {
    const html = await renderStorefrontPage();
    res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
    res.end(html);
  } catch (error) {
    res.writeHead(500, { "Content-Type": "text/plain" });
    res.end(String(error?.message ?? error));
  }
});

server.listen(config.port, "127.0.0.1", () => {
  console.log(`storefront-sim [${config.label}] on http://127.0.0.1:${config.port}`);
});
