const expression = process.argv[2] || "1+1";

const targets = await (await fetch("http://127.0.0.1:9222/json/list")).json();
const page = targets.find((t) => t.type === "page" && (t.url.includes("1420") || t.url.includes("tauri.localhost")));
if (!page) {
  console.error("no page target", targets.map((t) => [t.type, t.url]));
  process.exit(1);
}

const ws = new WebSocket(page.webSocketDebuggerUrl);
let id = 0;
const pending = new Map();

ws.addEventListener("message", (event) => {
  const message = JSON.parse(event.data);
  if (message.id && pending.has(message.id)) {
    pending.get(message.id)(message);
    pending.delete(message.id);
  }
});

await new Promise((resolve, reject) => {
  ws.addEventListener("open", resolve);
  ws.addEventListener("error", reject);
});

function send(method, params = {}) {
  const requestId = ++id;
  return new Promise((resolve) => {
    pending.set(requestId, resolve);
    ws.send(JSON.stringify({ id: requestId, method, params }));
  });
}

const result = await send("Runtime.evaluate", {
  expression,
  returnByValue: true,
  awaitPromise: true,
});

console.log(JSON.stringify(result.result?.result?.value ?? result, null, 2));
ws.close();
