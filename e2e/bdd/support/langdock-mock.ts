import http from "node:http";

const MOCK_PORT = 18080;

const STREAM_CHUNK_SIZE = 4;
const STREAM_CHUNK_DELAY_MS = 25;

type ChatMessage = { role?: string; content?: string };

type CompletionsBody = {
  stream?: boolean;
  messages?: ChatMessage[];
};

function parseJsonBody(req: http.IncomingMessage): Promise<unknown> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    req.on("data", (chunk) => chunks.push(chunk));
    req.on("end", () => {
      try {
        const raw = Buffer.concat(chunks).toString("utf8");
        resolve(raw ? JSON.parse(raw) : {});
      } catch (err) {
        reject(err);
      }
    });
    req.on("error", reject);
  });
}

function requireBearerAuth(
  req: http.IncomingMessage,
  res: http.ServerResponse,
): boolean {
  const auth = req.headers.authorization;
  if (!auth?.startsWith("Bearer ")) {
    res.writeHead(401);
    res.end();
    return false;
  }
  return true;
}

function lastUserMessageText(body: CompletionsBody): string {
  const messages = body.messages ?? [];
  for (let i = messages.length - 1; i >= 0; i--) {
    const msg = messages[i];
    if (msg?.role === "user" && typeof msg.content === "string") {
      return msg.content;
    }
  }
  return "";
}

function chunkText(text: string): string[] {
  const chunks: string[] = [];
  for (let i = 0; i < text.length; i += STREAM_CHUNK_SIZE) {
    chunks.push(text.slice(i, i + STREAM_CHUNK_SIZE));
  }
  return chunks.length > 0 ? chunks : [""];
}

function streamVercelCompletion(
  res: http.ServerResponse,
  replyText: string,
  req: http.IncomingMessage,
): void {
  let aborted = false;

  const onAbort = () => {
    aborted = true;
    if (!res.writableEnded) {
      res.end();
    }
  };

  req.on("aborted", onAbort);
  req.on("close", onAbort);

  res.writeHead(200, { "Content-Type": "text/plain; charset=utf-8" });

  const deltas = chunkText(replyText);
  let index = 0;

  const writeNext = () => {
    if (aborted || res.writableEnded) {
      return;
    }
    if (index >= deltas.length) {
      res.write(`d:${JSON.stringify({ finishReason: "stop" })}\n`);
      res.end();
      return;
    }
    res.write(`0:${JSON.stringify(deltas[index])}\n`);
    index += 1;
    setTimeout(writeNext, STREAM_CHUNK_DELAY_MS);
  };

  writeNext();
}

export function startLangDockMock(): Promise<http.Server> {
  return new Promise((resolve) => {
    const server = http.createServer(async (req, res) => {
      const path = req.url?.split("?")[0];

      if (req.method === "GET" && path === "/agent/v1/models") {
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ data: [{ id: "mock-model" }] }));
        return;
      }

      if (req.method === "POST" && path === "/agent/v1/chat/completions") {
        if (!requireBearerAuth(req, res)) {
          return;
        }

        try {
          const body = (await parseJsonBody(req)) as CompletionsBody;
          const reply = `mock-reply:${lastUserMessageText(body)}`;

          if (body.stream === true) {
            streamVercelCompletion(res, reply, req);
            return;
          }

          res.writeHead(200, { "Content-Type": "application/json" });
          res.end(
            JSON.stringify({
              choices: [{ message: { role: "assistant", content: reply } }],
            }),
          );
        } catch {
          res.writeHead(400);
          res.end();
        }
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
