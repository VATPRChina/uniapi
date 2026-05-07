import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { once } from "node:events";
import process from "node:process";
import { setTimeout as delay } from "node:timers/promises";
import { resolve } from "node:path";
import type { SharedWorker } from "ava/plugin";

export type BackendWorkerMessage =
  | { type: "getBaseUrl" }
  | { type: "baseUrl"; baseUrl: string }
  | { type: "stop" }
  | { type: "stopped" };

type BackendHandle = {
  baseUrl: string;
  process: ChildProcessWithoutNullStreams;
  stop: () => Promise<void>;
};

const startupTimeoutMs = Number(
  process.env.E2E_BACKEND_STARTUP_TIMEOUT_MS ?? 60_000,
);
const backendPort = Number(process.env.E2E_BACKEND_PORT ?? 3010);
const backendHost = process.env.E2E_BACKEND_HOST ?? "127.0.0.1";
const baseUrl = `http://${backendHost}:${backendPort}`;

export default function factory(options: SharedWorker.FactoryOptions): void {
  const protocol = options.negotiateProtocol<BackendWorkerMessage>(["ava-4"]);

  startBackend()
    .then((backend) => {
      void handleMessages(protocol, backend);
      protocol.ready();
    })
    .catch((error) => {
      setImmediate(() => {
        throw error;
      });
    });
}

async function handleMessages(
  protocol: SharedWorker.Protocol<BackendWorkerMessage>,
  backend: BackendHandle,
): Promise<void> {
  for await (const message of protocol.subscribe()) {
    if (message.data.type === "getBaseUrl") {
      message.reply({ type: "baseUrl", baseUrl: backend.baseUrl });
    }

    if (message.data.type === "stop") {
      await backend.stop();
      message.reply({ type: "stopped" });
    }
  }
}

async function startBackend(): Promise<BackendHandle> {
  const repoRoot = resolve(process.cwd(), "..");
  const rustDir = resolve(repoRoot, "rust");
  const child = spawn("cargo", ["run"], {
    cwd: rustDir,
    detached: process.platform !== "win32",
    env: {
      ...process.env,
      APP_BIND_ADDRESS: `${backendHost}:${backendPort}`,
    },
  });

  const output: string[] = [];
  child.stdout.on("data", (chunk) => output.push(String(chunk)));
  child.stderr.on("data", (chunk) => output.push(String(chunk)));

  let stopped = false;
  const stop = async (): Promise<void> => {
    if (stopped) {
      return;
    }

    stopped = true;
    await terminate(child);
  };

  process.once("exit", () => {
    terminateSync(child);
  });

  await waitForHealth(child, output);

  return {
    baseUrl,
    process: child,
    stop,
  };
}

async function waitForHealth(
  child: ChildProcessWithoutNullStreams,
  output: string[],
): Promise<void> {
  const deadline = Date.now() + startupTimeoutMs;

  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(
        `Backend exited before becoming healthy.\n${output.join("")}`,
      );
    }

    try {
      const response = await fetch(`${baseUrl}/health`, {
        signal: AbortSignal.timeout(1_000),
      });

      if (response.ok) {
        return;
      }
    } catch {
      // The server may still be compiling or binding its socket.
    }

    await delay(500);
  }

  await terminate(child);
  throw new Error(
    `Backend did not become healthy within ${startupTimeoutMs}ms.\n${output.join("")}`,
  );
}

async function terminate(child: ChildProcessWithoutNullStreams): Promise<void> {
  if (child.exitCode !== null || child.signalCode !== null) {
    return;
  }

  terminateSync(child);

  const exited = once(child, "exit");
  await Promise.race([
    exited,
    delay(5_000).then(() => {
      if (child.exitCode === null && child.signalCode === null) {
        kill(child, "SIGKILL");
      }
    }),
  ]);

  await exited.catch(() => undefined);
}

function terminateSync(child: ChildProcessWithoutNullStreams): void {
  kill(child, "SIGTERM");
}

function kill(
  child: ChildProcessWithoutNullStreams,
  signal: NodeJS.Signals,
): void {
  if (child.pid === undefined) {
    return;
  }

  try {
    if (process.platform === "win32") {
      child.kill(signal);
    } else {
      process.kill(-child.pid, signal);
    }
  } catch (error) {
    const code = (error as NodeJS.ErrnoException).code;
    if (code !== "ESRCH") {
      throw error;
    }
  }
}
