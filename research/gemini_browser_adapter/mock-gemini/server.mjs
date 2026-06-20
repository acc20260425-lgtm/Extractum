import http from "node:http";
import { renderMockGeminiPage } from "./variants.mjs";

export async function startMockGeminiServer() {
  const server = http.createServer((request, response) => {
    const url = new URL(request.url || "/", "http://127.0.0.1");
    if (url.pathname === "/mock-gemini-event") {
      response.writeHead(204);
      response.end();
      return;
    }

    if (url.pathname !== "/mock-gemini") {
      response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      response.end("not found");
      return;
    }

    const variant = url.searchParams.get("variant") || "happy-path";
    response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    response.end(renderMockGeminiPage(variant));
  });

  await new Promise((resolve) => {
    server.listen(0, "127.0.0.1", resolve);
  });

  const address = server.address();
  if (!address || typeof address === "string") {
    throw new Error("mock_server_address_unavailable");
  }

  return {
    port: address.port,
    url(variant) {
      return `http://127.0.0.1:${address.port}/mock-gemini?variant=${encodeURIComponent(variant)}`;
    },
    async stop() {
      await new Promise((resolve, reject) => {
        server.close((error) => {
          if (error) reject(error);
          else resolve();
        });
      });
    },
  };
}
