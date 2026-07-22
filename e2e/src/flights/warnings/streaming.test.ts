import { expect, test } from "vitest";
import { getBaseUrl } from "../../../lib/backend.js";

const messageTimeoutMs = 60_000;

test(
  "GET /api/flights/warnings/streaming streams all flight validation results",
  async () => {
    const url = new URL("/api/flights/warnings/streaming", getBaseUrl());
    url.protocol = url.protocol === "https:" ? "wss:" : "ws:";

    const socket = new WebSocket(url);

    try {
      const message = await firstTextMessage(socket);
      const snapshot: unknown = JSON.parse(message);

      expect(snapshot).toBeTypeOf("object");
      expect(snapshot).not.toBeNull();
      expect(Array.isArray(snapshot)).toBe(false);

      for (const [callsign, warnings] of Object.entries(
        snapshot as Record<string, unknown>,
      )) {
        expect(callsign).not.toBe("");
        expect(Array.isArray(warnings)).toBe(true);
      }
    } finally {
      socket.close();
    }
  },
  90_000,
);

function firstTextMessage(socket: WebSocket): Promise<string> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup();
      reject(
        new Error(
          `WebSocket did not receive a message within ${messageTimeoutMs}ms`,
        ),
      );
    }, messageTimeoutMs);

    const cleanup = () => {
      clearTimeout(timeout);
      socket.removeEventListener("message", onMessage);
      socket.removeEventListener("error", onError);
      socket.removeEventListener("close", onClose);
    };

    const onMessage = (event: MessageEvent) => {
      cleanup();
      if (typeof event.data !== "string") {
        reject(new Error("WebSocket returned a non-text message"));
        return;
      }
      resolve(event.data);
    };

    const onError = () => {
      cleanup();
      reject(new Error("WebSocket connection failed"));
    };

    const onClose = (event: CloseEvent) => {
      cleanup();
      reject(
        new Error(
          `WebSocket closed before its first message (${event.code}: ${event.reason})`,
        ),
      );
    };

    socket.addEventListener("message", onMessage);
    socket.addEventListener("error", onError);
    socket.addEventListener("close", onClose);
  });
}
