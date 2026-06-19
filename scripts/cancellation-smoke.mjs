// @ts-nocheck
import {
  SmokeAssertionError,
  discoverBridge,
  executeJs,
  expectedAppIdentifier,
} from "./analysis-smoke-helpers.mjs";

const runningAnalysisLabel = "__analysis_redesign_fixture__ Running Run";

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function field(record, snakeName, camelName = snakeName) {
  if (!record || typeof record !== "object") return undefined;
  return record[snakeName] ?? record[camelName];
}

function runId(record) {
  return field(record, "run_id", "runId") ?? field(record, "id", "id");
}

function jobId(record) {
  return field(record, "job_id", "jobId");
}

function runStatus(record) {
  return field(record, "run_status", "runStatus") ?? field(record, "status", "status");
}

function assert(condition, message, details = {}) {
  if (!condition) {
    throw new SmokeAssertionError(message, details);
  }
}

async function invoke(ctx, command, args = {}) {
  return executeJs(ctx.socket, `
    return await window.__TAURI__.core.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)});
  `, 30000);
}

async function poll(label, action, predicate, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  let lastValue = null;
  while (Date.now() < deadline) {
    lastValue = await action();
    if (predicate(lastValue)) return lastValue;
    await sleep(100);
  }
  throw new SmokeAssertionError(`${label} did not reach expected state`, { lastValue });
}

async function runStep(name, action) {
  process.stdout.write(`STEP ${name}\n`);
  try {
    const detail = await action();
    console.log(`PASS ${name}`);
    if (detail) {
      console.log(`     ${detail}`);
    }
    return true;
  } catch (error) {
    console.error(`FAIL ${name}`);
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    return false;
  }
}

async function analysisCancellation(ctx) {
  let seeded = false;
  try {
    await invoke(ctx, "clear_analysis_redesign_fixtures");
    await invoke(ctx, "seed_analysis_redesign_fixtures");
    seeded = true;

    const activeRuns = await invoke(ctx, "list_active_analysis_runs");
    const runningRun = activeRuns.find((run) => field(run, "scope_label_snapshot", "scopeLabelSnapshot") === runningAnalysisLabel)
      ?? activeRuns.find((run) => runStatus(run) === "running")
      ?? activeRuns[0];
    const id = runId(runningRun);
    assert(id, "analysis smoke did not seed an active running run", { activeRuns });

    await invoke(ctx, "cancel_analysis_run", { runId: id });
    const cancelled = await poll(
      "analysis run cancellation",
      () => invoke(ctx, "get_analysis_run", { runId: id }),
      (run) => runStatus(run) === "cancelled",
    );
    const activeAfter = await invoke(ctx, "list_active_analysis_runs");
    assert(!activeAfter.some((run) => runId(run) === id), "analysis run remained active after cancellation", { activeAfter });
    return `run ${id} -> ${runStatus(cancelled)}`;
  } finally {
    if (seeded) {
      await invoke(ctx, "clear_analysis_redesign_fixtures").catch((error) => {
        console.error(`WARN analysis cleanup failed: ${error?.message ?? error}`);
      });
    }
  }
}

async function promptPackCancellation(ctx) {
  let touched = false;
  try {
    await invoke(ctx, "clear_prompt_pack_cancellation_smoke_fixture");
    const seeded = await invoke(ctx, "seed_prompt_pack_cancellation_smoke_fixture");
    touched = true;
    const id = runId(seeded);
    assert(id, "prompt pack smoke did not return run id", { seeded });
    assert(runStatus(seeded) === "running", "prompt pack smoke run was not running", { seeded });

    await invoke(ctx, "cancel_prompt_pack_run", { runId: id });
    const cancelled = await poll(
      "prompt pack run cancellation",
      () => invoke(ctx, "list_prompt_pack_runs", { limit: 100 }),
      (runs) => runs.some((run) => runId(run) === id && runStatus(run) === "cancelled"),
    );
    const run = cancelled.find((candidate) => runId(candidate) === id);
    return `run ${id} -> ${runStatus(run)}`;
  } finally {
    if (touched) {
      await invoke(ctx, "clear_prompt_pack_cancellation_smoke_fixture").catch((error) => {
        console.error(`WARN prompt pack cleanup failed: ${error?.message ?? error}`);
      });
    }
  }
}

async function sourceJobCancellation(ctx) {
  let touched = false;
  try {
    await invoke(ctx, "clear_source_job_cancellation_smoke_fixture");
    const seeded = await invoke(ctx, "seed_source_job_cancellation_smoke_fixture");
    touched = true;
    const id = jobId(seeded);
    assert(id, "source job smoke did not return job id", { seeded });
    assert(field(seeded, "status", "status") === "running", "source job smoke was not running", { seeded });

    await invoke(ctx, "cancel_source_job", { jobId: id });
    const cancelled = await poll(
      "source job cancellation",
      () => invoke(ctx, "list_source_jobs", { filter: { limit: 100 } }),
      (jobs) => jobs.some((job) => jobId(job) === id && field(job, "status", "status") === "cancelled"),
    );
    const job = cancelled.find((candidate) => jobId(candidate) === id);
    return `job ${id} -> ${field(job, "status", "status")}`;
  } finally {
    if (touched) {
      await invoke(ctx, "clear_source_job_cancellation_smoke_fixture").catch((error) => {
        console.error(`WARN source job cleanup failed: ${error?.message ?? error}`);
      });
    }
  }
}

async function takeoutCancellation(ctx) {
  let touched = false;
  try {
    await invoke(ctx, "clear_takeout_cancellation_smoke_fixture");
    const seeded = await invoke(ctx, "seed_takeout_cancellation_smoke_fixture");
    touched = true;
    const id = jobId(seeded);
    assert(id, "takeout smoke did not return job id", { seeded });
    assert(field(seeded, "status", "status") === "running", "takeout smoke was not running", { seeded });

    const cancelResult = await invoke(ctx, "cancel_takeout_source_import", { jobId: id });
    assert(cancelResult?.cancelled === true, "takeout cancel command returned cancelled=false", { cancelResult });
    const cancelled = await poll(
      "takeout import cancellation",
      () => invoke(ctx, "list_takeout_source_import_jobs"),
      (jobs) => jobs.some((job) => jobId(job) === id && field(job, "status", "status") === "cancelled"),
    );
    const job = cancelled.find((candidate) => jobId(candidate) === id);
    return `job ${id} -> ${field(job, "status", "status")}`;
  } finally {
    if (touched) {
      await invoke(ctx, "clear_takeout_cancellation_smoke_fixture").catch((error) => {
        console.error(`WARN takeout cleanup failed: ${error?.message ?? error}`);
      });
    }
  }
}

async function main() {
  const bridge = await discoverBridge({ startupTimeoutMs: 15000 });
  const ctx = { socket: bridge.socket, port: bridge.port, backendState: bridge.backendState };
  try {
    assert(
      ctx.backendState.app.identifier === expectedAppIdentifier,
      `unexpected app identifier ${ctx.backendState.app.identifier}`,
      { backendState: ctx.backendState },
    );
    console.log(`Connected to ${ctx.backendState.app.identifier} on MCP Bridge port ${ctx.port}.`);

    const steps = [
      ["cancellation.analysis-run", analysisCancellation],
      ["cancellation.prompt-pack-run", promptPackCancellation],
      ["cancellation.youtube-source-job", sourceJobCancellation],
      ["cancellation.takeout-import", takeoutCancellation],
    ];
    let ok = true;
    for (const [name, action] of steps) {
      const passed = await runStep(name, () => action(ctx));
      ok = ok && passed;
    }

    if (!ok) {
      process.exit(1);
    }
    console.log("\nCancellation smoke passed.");
  } finally {
    ctx.socket.close();
  }
}

await main();
