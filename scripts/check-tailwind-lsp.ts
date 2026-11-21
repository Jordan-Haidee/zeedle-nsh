const childProcessModule = "node:child_process";
const fsModule = "node:fs";
const pathModule = "node:path";
const urlModule = "node:url";
const processModule = "node:process";
const bufferModule = "node:buffer";

const { execFileSync, spawn } = await import(childProcessModule);
const { readFileSync, readdirSync, statSync } = await import(fsModule);
const path = (await import(pathModule)).default;
const { fileURLToPath, pathToFileURL } = await import(urlModule);
const process = (await import(processModule)).default;
const { Buffer } = await import(bufferModule);

type LspMessage = Record<string, any>;
type LspDiagnostic = {
  code?: string | number;
  message?: string;
  range?: { start?: { line?: number } };
  severity?: number;
};

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const configuration = JSON.parse(readFileSync(path.join(root, "tailwind-lsp.config.json"), "utf8"));
const omittedDirectories = new Set([".git", ".nsh", "node_modules", "target", "docs"]);

function sourceFiles(directory: string): string[] {
  const files: string[] = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!omittedDirectories.has(entry.name)) files.push(...sourceFiles(path.join(directory, entry.name)));
    } else if (entry.isFile() && /\.tsx?$/.test(entry.name) && !entry.name.endsWith(".d.ts")) {
      files.push(path.join(directory, entry.name));
    }
  }
  return files.sort();
}

function languageServerEntry() {
  const local = path.join(root, "node_modules", "@tailwindcss", "language-server", "bin", "tailwindcss-language-server");
  if (statSyncSafe(local)) return local;

  const npmRootCommand = process.platform === "win32" ? (process.env.ComSpec ?? "cmd.exe") : "npm";
  const npmRootArgs = process.platform === "win32" ? ["/d", "/s", "/c", "npm root -g"] : ["root", "-g"];
  const npmRoot = execFileSync(npmRootCommand, npmRootArgs, {
    cwd: root,
    encoding: "utf8",
  }).trim();
  const global = path.join(npmRoot, "@tailwindcss", "language-server", "bin", "tailwindcss-language-server");
  if (statSyncSafe(global)) return global;
  throw new Error(`Could not find @tailwindcss/language-server under ${npmRoot}`);
}

function statSyncSafe(file: string) {
  try {
    return statSync(file).isFile();
  } catch {
    return false;
  }
}

const nodeExecutable: string = process.env.TAILWINDCSS_NODE ?? "node";
let input = Buffer.alloc(0);
const queued: LspMessage[] = [];
const waiters: ((message: LspMessage) => void)[] = [];
let stderr = "";
const server = spawn(nodeExecutable, [languageServerEntry(), "--stdio"], {
  cwd: root,
  stdio: ["pipe", "pipe", "pipe"],
});

server.stdout.on("data", (chunk: any) => {
  input = Buffer.concat([input, chunk]);
  while (true) {
    const headerEnd = input.indexOf("\r\n\r\n");
    if (headerEnd < 0) break;
    const header = input.subarray(0, headerEnd).toString("ascii");
    const contentLength = Number(header.match(/content-length:\s*(\d+)/i)?.[1]);
    if (!Number.isFinite(contentLength)) throw new Error(`Invalid LSP header: ${header}`);
    const bodyStart = headerEnd + 4;
    if (input.length < bodyStart + contentLength) break;
    const message = JSON.parse(input.subarray(bodyStart, bodyStart + contentLength).toString("utf8"));
    input = input.subarray(bodyStart + contentLength);
    const waiter = waiters.shift();
    if (waiter) waiter(message);
    else queued.push(message);
  }
});
server.stderr.on("data", (chunk: any) => { stderr += chunk.toString("utf8"); });

function send(message: LspMessage): void {
  const body = Buffer.from(JSON.stringify(message), "utf8");
  server.stdin.write(Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, "ascii"));
  server.stdin.write(body);
}

function nextMessage(timeoutMs = 10000): Promise<LspMessage> {
  if (queued.length) return Promise.resolve(queued.shift()!);
  return new Promise((resolve, reject) => {
    const waiter = (message: LspMessage) => {
      clearTimeout(timer);
      resolve(message);
    };
    const timer = setTimeout(() => {
      const index = waiters.indexOf(waiter);
      if (index >= 0) waiters.splice(index, 1);
      reject(new Error("Timed out waiting for Tailwind language server"));
    }, timeoutMs);
    waiters.push(waiter);
  });
}

const clientInfo = { name: "zeedle-tailwind-lsp-check", version: "1.0" };
const rootUri = pathToFileURL(`${root}${path.sep}`).href;
send({
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: {
    processId: process.pid,
    clientInfo,
    rootUri,
    workspaceFolders: [{ uri: rootUri, name: path.basename(root) }],
    capabilities: {
      workspace: { configuration: true, workspaceFolders: true },
      textDocument: { publishDiagnostics: { relatedInformation: true, versionSupport: true } },
      general: { positionEncodings: ["utf-16"] },
    },
  },
});

let initializeResponse: LspMessage | undefined;
while (!initializeResponse) {
  const message = await nextMessage();
  if (message.id === 1) initializeResponse = message;
  else if (message.method && message.id !== undefined) {
    send({ jsonrpc: "2.0", id: message.id, result: null });
  }
}
if (initializeResponse.error) throw new Error(JSON.stringify(initializeResponse.error));

send({ jsonrpc: "2.0", method: "initialized", params: {} });
send({ jsonrpc: "2.0", method: "workspace/didChangeConfiguration", params: { settings: { tailwindCSS: configuration } } });

const files = sourceFiles(root);
const fileByUri = new Map<string, string>();
for (const file of files) {
  const uri = pathToFileURL(file).href;
  fileByUri.set(uri, file);
  send({
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: {
      textDocument: {
        uri,
        languageId: file.endsWith(".tsx") ? "typescriptreact" : "typescript",
        version: 1,
        text: readFileSync(file, "utf8"),
      },
    },
  });
}

const diagnosticsByUri = new Map<string, LspDiagnostic[]>();
let quietPeriods = 0;
while (quietPeriods < 2) {
  let message;
  try {
    message = await nextMessage(1500);
  } catch {
    quietPeriods++;
    continue;
  }
  quietPeriods = 0;
  if (message.method === "textDocument/publishDiagnostics") {
    diagnosticsByUri.set(message.params.uri, message.params.diagnostics ?? []);
  } else if (message.method && message.id !== undefined) {
    const items = message.params?.items ?? [];
    const result = items.map((item: any) => {
      if (item.section === "tailwindCSS") return configuration;
      if (item.section === "editor") return { tabSize: 2 };
      return null;
    });
    send({ jsonrpc: "2.0", id: message.id, result });
  }
}

const diagnostics: { file: string; line: number; severity?: number; code?: string | number; message?: string }[] = [];
for (const [uri, items] of diagnosticsByUri) {
  for (const item of items) {
    diagnostics.push({
      file: path.relative(root, fileByUri.get(uri) ?? uri),
      line: (item.range?.start?.line ?? 0) + 1,
      severity: item.severity,
      code: item.code,
      message: item.message,
    });
  }
}

send({ jsonrpc: "2.0", id: 2, method: "shutdown", params: {} });
while ((await nextMessage()).id !== 2) {}
send({ jsonrpc: "2.0", method: "exit", params: {} });
await new Promise((resolve) => server.once("exit", resolve));

if (stderr.trim()) process.stderr.write(stderr);
if (diagnostics.length) {
  for (const diagnostic of diagnostics) {
    process.stderr.write(`${diagnostic.file}:${diagnostic.line}: ${diagnostic.message}\n`);
  }
  process.exitCode = 1;
} else {
  process.stdout.write(`Tailwind LSP: ${files.length} TS/TSX files checked; no diagnostics.\n`);
}
