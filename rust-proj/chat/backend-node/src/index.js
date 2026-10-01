import Fastify from "fastify";

import { config } from "./config.js";
import { registerRoutes } from "./routes.js";
import { attachWebSocket } from "./ws.js";

const app = Fastify({ logger: false });

app.addHook("onSend", async (request, reply) => {
  const origin = request.headers.origin;

  if (origin && config.allowedOrigins.includes(origin)) {
    reply.header("Access-Control-Allow-Origin", origin);
    reply.header("Access-Control-Allow-Methods", "GET, POST");
    reply.header("Access-Control-Allow-Headers", "Content-Type, Authorization");
  }
});

app.options("/*", async (_request, reply) => reply.code(204).send());

app.get("/", async () => ({
  service: "chat-backend-node",
  note: "ده الـ API. واجهة التطبيق بتشتغل على منفذ تاني (5173 في التطوير).",
}));

registerRoutes(app);

await app.listen({ host: config.bindHost, port: config.bindPort });
attachWebSocket(app.server);

console.log(`listening on http://${config.bindHost}:${config.bindPort}`);
