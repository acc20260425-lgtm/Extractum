import readline from "node:readline";
import { GeminiBrowserAdapter } from "./adapter";
import { parseEnvelope, type SidecarResponse } from "./protocol";

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
