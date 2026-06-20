import readline from "node:readline";
import { chromium } from "@playwright/test";
import { GeminiBrowserAdapter } from "./adapter.js";
import { parseEnvelope, type SidecarResponse } from "./protocol.js";

async function runPlaywrightSmoke() {
  const profileDirArg = process.argv.find((arg) => arg.startsWith("--profile-dir="));
  const profileDir =
    profileDirArg?.slice("--profile-dir=".length) ??
    "artifacts/gemini-browser-playwright-smoke-profile";
  const context = await chromium.launchPersistentContext(profileDir, {
    headless: true,
    viewport: { width: 800, height: 600 },
  });
  const page = context.pages()[0] ?? (await context.newPage());
  await page.goto("data:text/html,<title>Gemini Sidecar Smoke</title><main>ok</main>");
  const title = await page.title();
  await context.close();
  process.stdout.write(`${JSON.stringify({ ok: true, title })}\n`);
}

if (process.argv.includes("--playwright-smoke")) {
  runPlaywrightSmoke()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
} else {
  const adapter = new GeminiBrowserAdapter();

  function writeResponse(id: string, response: SidecarResponse) {
    process.stdout.write(`${JSON.stringify({ id, response })}\n`);
  }

  const rl = readline.createInterface({
    input: process.stdin,
    crlfDelay: Infinity,
  });

  rl.on("line", async (line) => {
    let id = "unknown";
    try {
      const envelope = parseEnvelope(line);
      id = envelope.id;
      const command = envelope.command;
      if (command.type === "status") {
        writeResponse(id, {
          type: "status",
          status: await adapter.status(command.browser_profile_dir),
        });
        return;
      }
      if (command.type === "open_browser") {
        writeResponse(id, {
          type: "status",
          status: await adapter.openBrowser(command.browser_profile_dir),
        });
        return;
      }
      if (command.type === "send_single") {
        writeResponse(id, {
          type: "run_result",
          result: await adapter.sendSingle({
            request: command.request,
            browserProfileDir: command.browser_profile_dir,
            artifactDir: command.artifact_dir,
          }),
        });
        return;
      }
      if (command.type === "resume") {
        writeResponse(id, { type: "ack" });
        return;
      }
      if (command.type === "stop") {
        await adapter.stop();
        writeResponse(id, { type: "ack" });
      }
    } catch (error) {
      writeResponse(id, { type: "error", message: String(error) });
    }
  });
}
