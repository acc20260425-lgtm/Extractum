// @ts-nocheck
import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

export const bridgePortStart = 9223;
export const bridgePortEnd = 9322;
export const expectedAppIdentifier = "org.ai.extractum";
export const artifactRoot = path.join("tmp", "analysis-smoke");

export const fixtureLabels = {
  telegramChannel: "__analysis_redesign_fixture__ Telegram Channel",
  telegramSupergroup: "__analysis_redesign_fixture__ Telegram Supergroup",
  youtubeVideo: "__analysis_redesign_fixture__ YouTube Video",
  youtubePlaylist: "__analysis_redesign_fixture__ YouTube Playlist",
  telegramSourceGroup: "__analysis_redesign_fixture__ Telegram Source Group",
  completedSnapshotRun: "__analysis_redesign_fixture__ Completed Snapshot Run",
  missingSnapshotRun: "__analysis_redesign_fixture__ Missing Snapshot Run",
  failedRun: "__analysis_redesign_fixture__ Failed Run",
  cancelledRun: "__analysis_redesign_fixture__ Cancelled Run",
  captureFailedSnapshotRun: "__analysis_redesign_fixture__ Capture Failed Snapshot Run",
  groupSnapshotRun: "__analysis_redesign_fixture__ Group Snapshot Run",
};

export const expectedFixtureLabels = Object.values(fixtureLabels);

export const fixtureSummaryKeys = [
  "accounts",
  "chatMessages",
  "llmProfiles",
  "promptTemplates",
  "runs",
  "snapshotMessages",
  "sourceGroups",
  "sources",
  "youtubePlaylistItems",
  "youtubeTranscriptSegments",
];

export class SmokeAssertionError extends Error {
  constructor(message, details = {}) {
    super(message.startsWith("ASSERT:") ? message : `ASSERT: ${message}`);
    this.name = "SmokeAssertionError";
    this.kind = "assertion";
    this.details = details;
  }
}

export class SmokeBridgeError extends Error {
  constructor(message, kind = "bridge-error", details = {}) {
    super(message);
    this.name = "SmokeBridgeError";
    this.kind = kind;
    this.details = details;
  }
}

export function bridgePortCandidates(start = bridgePortStart, end = bridgePortEnd) {
  return Array.from({ length: end - start + 1 }, (_, index) => start + index);
}

export function sanitizeArtifactName(value) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    || "analysis-smoke-step";
}

export function assertTabOrderLabels(actual, expected) {
  const actualJoined = actual.join(" | ");
  const expectedJoined = expected.join(" | ");
  if (actualJoined !== expectedJoined) {
    throw new SmokeAssertionError(`tab order mismatch: expected ${expectedJoined}, got ${actualJoined}`, {
      actual,
      expected,
    });
  }
  return true;
}

export function validateFixtureLabels(labels) {
  const missing = expectedFixtureLabels.filter((label) => !labels.includes(label));
  if (missing.length > 0) {
    throw new SmokeAssertionError(`missing fixture labels: ${missing.join(", ")}`, { missing, labels });
  }
  return labels;
}

export function validateFixtureSummary(summary) {
  const expected = {
    accounts: 1,
    chatMessages: 2,
    llmProfiles: 1,
    promptTemplates: 1,
    runs: 7,
    snapshotMessages: 4,
    sourceGroups: 1,
    sources: 4,
    youtubePlaylistItems: 2,
    youtubeTranscriptSegments: 3,
  };
  const failures = Object.entries(expected)
    .filter(([key, value]) => Number(summary?.[key] ?? 0) < value)
    .map(([key, value]) => `${key}>=${value}`);
  if (failures.length > 0) {
    throw new SmokeAssertionError(`fixture summary below expected minimums: ${failures.join(", ")}`, {
      summary,
      expected,
    });
  }
  return true;
}

export function assertEmptyFixtureSummary(summary) {
  const missing = fixtureSummaryKeys.filter((key) => !Object.prototype.hasOwnProperty.call(summary ?? {}, key));
  if (missing.length > 0) {
    throw new SmokeAssertionError(`fixture cleanup summary missing keys: ${missing.join(", ")}`, {
      summary,
      missing,
    });
  }

  const nonEmpty = fixtureSummaryKeys
    .filter((key) => Number(summary[key] ?? 0) !== 0)
    .map((key) => `${key}=${summary[key]}`);
  const unexpectedNonEmpty = Object.entries(summary ?? {})
    .filter(([key, value]) => !fixtureSummaryKeys.includes(key) && Number(value ?? 0) !== 0)
    .map(([key, value]) => `${key}=${value}`);
  nonEmpty.push(...unexpectedNonEmpty);
  if (nonEmpty.length > 0) {
    throw new SmokeAssertionError(`fixture cleanup verification found remaining rows: ${nonEmpty.join(", ")}`, {
      summary,
    });
  }
  return true;
}

export function classifyBridgeFailure(error) {
  if (error instanceof SmokeAssertionError) return { kind: "assertion", message: error.message };
  if (error instanceof SmokeBridgeError) return { kind: error.kind, message: error.message };
  const message = error instanceof Error ? error.message : String(error);
  if (message.startsWith("ASSERT:")) return { kind: "assertion", message };
  if (message.includes("Script execution timeout")) return { kind: "script-timeout", message };
  if (message.includes("WebSocket") || message.includes("socket")) return { kind: "bridge-disconnect", message };
  return { kind: "app-contract", message };
}

export function createRequestId(prefix = "analysis-smoke") {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

export async function bridgeRequest(socket, command, args = {}, timeoutMs = 5000) {
  const id = createRequestId(command);
  const request = JSON.stringify({ id, command, args });
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup();
      reject(new SmokeBridgeError(`bridge timeout waiting for ${command}`, "bridge-timeout", { command }));
    }, timeoutMs);

    function cleanup() {
      clearTimeout(timeout);
      socket.removeEventListener("message", onMessage);
      socket.removeEventListener("close", onClose);
      socket.removeEventListener("error", onError);
    }

    function onMessage(event) {
      const text = typeof event.data === "string" ? event.data : "";
      let response;
      try {
        response = JSON.parse(text);
      } catch {
        return;
      }
      if (response.id !== id) return;
      cleanup();
      resolve(response);
    }

    function onClose() {
      cleanup();
      reject(new SmokeBridgeError(`bridge disconnected during ${command}`, "bridge-disconnect", { command }));
    }

    function onError() {
      cleanup();
      reject(new SmokeBridgeError(`bridge socket error during ${command}`, "bridge-disconnect", { command }));
    }

    socket.addEventListener("message", onMessage);
    socket.addEventListener("close", onClose);
    socket.addEventListener("error", onError);
    socket.send(request);
  });
}

export async function executeJs(socket, script, timeoutMs = 8000) {
  const response = await bridgeRequest(socket, "execute_js", { script, windowLabel: "main" }, timeoutMs);
  if (!response.success) {
    const message = response.error ?? "execute_js failed";
    if (message.startsWith("ASSERT:")) {
      throw new SmokeAssertionError(message, { response });
    }
    if (message.includes("Script execution timeout")) {
      throw new SmokeBridgeError(message, "script-timeout", { response });
    }
    throw new SmokeBridgeError(message, "script-failure", { response });
  }
  return response.data;
}

export function waitForSocketOpen(socket, timeoutMs = 1500) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup();
      reject(new SmokeBridgeError("bridge unavailable", "bridge-unavailable"));
    }, timeoutMs);
    function cleanup() {
      clearTimeout(timeout);
      socket.removeEventListener("open", onOpen);
      socket.removeEventListener("error", onError);
    }
    function onOpen() {
      cleanup();
      resolve(socket);
    }
    function onError() {
      cleanup();
      reject(new SmokeBridgeError("bridge unavailable", "bridge-unavailable"));
    }
    socket.addEventListener("open", onOpen);
    socket.addEventListener("error", onError);
  });
}

export async function discoverBridge({
  WebSocketCtor = globalThis.WebSocket,
  ports = bridgePortCandidates(),
  startupTimeoutMs = 90000,
} = {}) {
  if (typeof WebSocketCtor !== "function") {
    throw new SmokeBridgeError("Node runtime does not provide globalThis.WebSocket", "missing-websocket");
  }

  const deadline = Date.now() + startupTimeoutMs;
  let lastIdentifierMismatch = null;

  while (Date.now() < deadline) {
    for (const port of ports) {
      const socket = new WebSocketCtor(`ws://127.0.0.1:${port}`);
      try {
        await waitForSocketOpen(socket);
        const backend = await bridgeRequest(socket, "invoke_tauri", {
          command: "plugin:mcp-bridge|get_backend_state",
          args: { windowLabel: "main" },
        });
        if (backend.success && backend.data?.app?.identifier === expectedAppIdentifier) {
          return { socket, port, backendState: backend.data };
        }
        if (backend.success && backend.data?.app?.identifier) {
          lastIdentifierMismatch = {
            port,
            identifier: backend.data.app.identifier,
          };
        }
        socket.close();
      } catch {
        try {
          socket.close();
        } catch {
          // Ignore failed close while probing unavailable ports.
        }
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  if (lastIdentifierMismatch) {
    throw new SmokeBridgeError(
      `MCP bridge app identifier mismatch on port ${lastIdentifierMismatch.port}: ${lastIdentifierMismatch.identifier}`,
      "app-identifier-mismatch",
      lastIdentifierMismatch,
    );
  }

  throw new SmokeBridgeError(`No ${expectedAppIdentifier} MCP bridge found on ports ${ports[0]}-${ports.at(-1)}`, "bridge-unavailable");
}

export async function resizeWindow(socket, width = 1280, height = 860) {
  const response = await bridgeRequest(socket, "resize_window", {
    width,
    height,
    windowId: "main",
    logical: true,
  });
  if (!response.success || response.data?.success === false) {
    throw new SmokeBridgeError(response.error ?? response.data?.error ?? "resize_window failed", "bridge-command-failed", { response });
  }
  return response.data;
}

export async function captureNativeScreenshot(socket, maxWidth = 1280) {
  return bridgeRequest(socket, "capture_native_screenshot", {
    format: "png",
    maxWidth,
    windowLabel: "main",
  }, 8000);
}

export async function getVisibleTextSummary(socket) {
  return executeJs(socket, `
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    const chunks = [];
    while (walker.nextNode()) {
      const value = walker.currentNode.textContent.trim().replace(/\\s+/g, " ");
      if (value) chunks.push(value);
    }
    return chunks.join("\\n").slice(0, 12000);
  `);
}

export async function waitForText(socket, text, timeoutMs = 8000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const found = await executeJs(socket, `
      return document.body.innerText.includes(${JSON.stringify(text)});
    `);
    if (found) return true;
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  throw new SmokeAssertionError(`text not found: ${text}`);
}

export async function clickBySmokeId(socket, smokeId) {
  return executeJs(socket, `
    const element = document.querySelector('[data-smoke-id="${smokeId}"]');
    if (!element) throw new Error('ASSERT: missing smoke id ${smokeId}');
    element.click();
    return true;
  `);
}

export async function clickByText(socket, text) {
  return executeJs(socket, `
    const targetText = ${JSON.stringify(text)};
    const candidates = Array.from(document.querySelectorAll('button, [role="button"], a, summary'));
    const match = candidates.find((element) => element.innerText.trim().includes(targetText)
      || element.getAttribute('aria-label')?.includes(targetText)
      || element.getAttribute('title')?.includes(targetText));
    if (!match) throw new Error('ASSERT: clickable text not found: ' + targetText);
    match.click();
    return true;
  `);
}

export async function clickByTextWithinSmokeId(socket, smokeId, text) {
  return executeJs(socket, `
    const container = document.querySelector('[data-smoke-id="${smokeId}"]');
    if (!container) throw new Error('ASSERT: missing smoke id ${smokeId}');
    const targetText = ${JSON.stringify(text)};
    const candidates = Array.from(container.querySelectorAll('button, [role="button"], a, summary'));
    const match = candidates.find((element) => element.innerText.trim().includes(targetText)
      || element.getAttribute('aria-label')?.includes(targetText)
      || element.getAttribute('title')?.includes(targetText));
    if (!match) throw new Error('ASSERT: scoped clickable text not found: ' + targetText);
    match.click();
    return true;
  `);
}

export async function clickRowActionByText(socket, containerSmokeId, rowText, actionText) {
  return executeJs(socket, `
    const container = document.querySelector('[data-smoke-id="${containerSmokeId}"]');
    if (!container) throw new Error('ASSERT: missing smoke id ${containerSmokeId}');
    const rowText = ${JSON.stringify(rowText)};
    const actionText = ${JSON.stringify(actionText)};
    const rowCandidates = Array.from(container.querySelectorAll('li, article, .source-row, .group-row, button, [role="row"]'));
    const row = rowCandidates.find((candidate) => candidate.innerText.includes(rowText));
    if (!row) throw new Error('ASSERT: row not found: ' + rowText);
    const action = Array.from(row.querySelectorAll('button, [role="button"], a'))
      .find((candidate) => candidate.innerText.trim().includes(actionText)
        || candidate.getAttribute('aria-label')?.includes(actionText)
        || candidate.getAttribute('title')?.includes(actionText));
    if (!action) throw new Error('ASSERT: row action not found: ' + actionText + ' in ' + rowText);
    action.click();
    return true;
  `);
}

export async function fillByLabel(socket, label, value) {
  return executeJs(socket, `
    const labelText = ${JSON.stringify(label)};
    const value = ${JSON.stringify(value)};
    const controls = Array.from(document.querySelectorAll('input, textarea'));
    const control = controls.find((element) => element.getAttribute('aria-label') === labelText
      || element.closest('label')?.innerText.includes(labelText)
      || element.getAttribute('placeholder') === labelText);
    if (!control) throw new Error('ASSERT: input not found: ' + labelText);
    control.focus();
    control.value = value;
    control.dispatchEvent(new Event('input', { bubbles: true }));
    control.dispatchEvent(new Event('change', { bubbles: true }));
    return true;
  `);
}

export async function readTabLabels(socket, smokeId = "source-browser-tabs") {
  return executeJs(socket, `
    const container = document.querySelector('[data-smoke-id="${smokeId}"]');
    if (!container) throw new Error('ASSERT: missing tab container ${smokeId}');
    return Array.from(container.querySelectorAll('[role="tab"], button')).map((element) => element.innerText.trim()).filter(Boolean);
  `);
}

export async function assertSelectedTab(socket, expectedTab, smokeId = "source-browser-tabs") {
  return executeJs(socket, `
    const container = document.querySelector('[data-smoke-id="${smokeId}"]');
    if (!container) throw new Error('ASSERT: missing tab container ${smokeId}');
    const selected = Array.from(container.querySelectorAll('[role="tab"], button'))
      .find((element) => element.getAttribute('aria-selected') === 'true'
        || element.classList.contains('active')
        || element.classList.contains('selected'));
    if (!selected) throw new Error('ASSERT: no selected tab in ${smokeId}');
    if (!selected.innerText.trim().includes(${JSON.stringify(expectedTab)})) {
      throw new Error('ASSERT: selected tab mismatch: ' + selected.innerText.trim());
    }
    return true;
  `);
}

export async function assertDisabledWithReason(socket, buttonText, reasonText) {
  return executeJs(socket, `
    const buttonText = ${JSON.stringify(buttonText)};
    const reasonText = ${JSON.stringify(reasonText)};
    const button = Array.from(document.querySelectorAll('button'))
      .find((candidate) => candidate.innerText.includes(buttonText));
    if (!button) throw new Error('ASSERT: disabled button missing: ' + buttonText);
    if (!button.disabled) throw new Error('ASSERT: button is not disabled: ' + buttonText);
    button.click();
    const describedBy = button.getAttribute('aria-describedby');
    if (!describedBy) throw new Error('ASSERT: disabled button missing aria-describedby');
    const reason = document.getElementById(describedBy);
    if (!reason) throw new Error('ASSERT: described disabled reason missing: ' + describedBy);
    if (reason.dataset.smokeId !== 'notebooklm-export-disabled-reason') {
      throw new Error('ASSERT: disabled reason missing smoke id');
    }
    if (!reason.innerText.includes(reasonText)) {
      throw new Error('ASSERT: disabled reason mismatch: ' + reason.innerText);
    }
    return true;
  `);
}

export async function captureArtifacts({ socket, artifactDir, stepName, error }) {
  await mkdir(artifactDir, { recursive: true });
  const failure = classifyBridgeFailure(error);
  await writeFile(path.join(artifactDir, "failure.json"), JSON.stringify({
    stepName,
    failure,
    error: error instanceof Error ? { name: error.name, message: error.message, stack: error.stack } : String(error),
  }, null, 2));

  try {
    const summary = await getVisibleTextSummary(socket);
    await writeFile(path.join(artifactDir, "visible-text.txt"), summary);
  } catch (summaryError) {
    await writeFile(path.join(artifactDir, "visible-text-error.txt"), String(summaryError?.message ?? summaryError));
  }

  try {
    const dom = await executeJs(socket, `
      return Array.from(document.querySelectorAll('[data-smoke-id]')).map((element) => ({
        smokeId: element.dataset.smokeId,
        tag: element.tagName.toLowerCase(),
        text: element.innerText?.trim().slice(0, 500) ?? '',
      }));
    `);
    await writeFile(path.join(artifactDir, "smoke-dom.json"), JSON.stringify(dom, null, 2));
  } catch (domError) {
    await writeFile(path.join(artifactDir, "smoke-dom-error.txt"), String(domError?.message ?? domError));
  }

  try {
    const screenshot = await captureNativeScreenshot(socket);
    if (screenshot.success && typeof screenshot.data === "string") {
      await writeFile(path.join(artifactDir, "screenshot.data-url.txt"), screenshot.data);
    } else {
      await writeFile(path.join(artifactDir, "screenshot-error.txt"), screenshot.error ?? "screenshot unavailable");
    }
  } catch (screenshotError) {
    await writeFile(path.join(artifactDir, "screenshot-error.txt"), String(screenshotError?.message ?? screenshotError));
  }
}

export function spawnTauriDev({ command, args, cwd }) {
  return spawn(command, args, { cwd, shell: false, stdio: "inherit" });
}

export function killProcessTree(child) {
  if (!child?.pid) return Promise.resolve();
  if (process.platform === "win32") {
    return new Promise((resolve) => {
      const killer = spawn("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" });
      killer.on("close", () => resolve());
      killer.on("error", () => resolve());
    });
  }
  child.kill("SIGTERM");
  return Promise.resolve();
}
