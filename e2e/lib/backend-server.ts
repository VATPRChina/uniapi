import {
  spawn,
  spawnSync,
  type ChildProcessWithoutNullStreams,
} from "node:child_process";
import { once } from "node:events";
import process from "node:process";
import { resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";

export type BackendHandle = {
  baseUrl: string;
  stop: () => Promise<void>;
};

const startupTimeoutMs = Number(
  process.env.E2E_BACKEND_STARTUP_TIMEOUT_MS ?? 60_000,
);
const backendPort = Number(process.env.E2E_BACKEND_PORT ?? 3010);
const backendHost = process.env.E2E_BACKEND_HOST ?? "127.0.0.1";
const baseUrl = `http://${backendHost}:${backendPort}`;

export async function startBackend(): Promise<BackendHandle> {
  const repoRoot = resolve(process.cwd(), "..");
  const rustDir = resolve(repoRoot, "src");
  const build = spawnSync("cargo", ["build"], {
    cwd: rustDir,
    encoding: "utf8",
    env: process.env,
  });

  if (build.status !== 0) {
    throw new Error(
      `Backend build failed.\n${build.stdout ?? ""}${build.stderr ?? ""}`,
    );
  }

  const binary = resolve(
    rustDir,
    "target",
    "debug",
    process.platform === "win32" ? "vatprc-uniapi.exe" : "vatprc-uniapi",
  );
  const child = spawn(binary, [], {
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
