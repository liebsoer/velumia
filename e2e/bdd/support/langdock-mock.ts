import http from "node:http";

const MOCK_PORT = 18080;

export function startLangDockMock(): Promise<http.Server> {
  return new Promise((resolve) => {
    const server = http.createServer((req, res) => {
      if (req.method === "GET" && req.url === "/agent/v1/models") {
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ data: [{ id: "mock-model" }] }));
        return;
      }
      res.writeHead(404);
      res.end();
    });
    server.listen(MOCK_PORT, "127.0.0.1", () => resolve(server));
  });
}

export function langDockMockUrl(): string {
  return `http://127.0.0.1:${MOCK_PORT}`;
}

export async function probeLangDockMock(): Promise<number> {
  const res = await fetch(`${langDockMockUrl()}/agent/v1/models`, {
    headers: { Authorization: "Bearer mock-key" },
  });
  return res.status;
}

export function stopLangDockMock(server: http.Server): Promise<void> {
  return new Promise((resolve, reject) => {
    server.close((err) => (err ? reject(err) : resolve()));
  });
}
