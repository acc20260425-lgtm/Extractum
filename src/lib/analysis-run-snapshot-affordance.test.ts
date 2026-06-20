import { describe, expect, it } from "vitest";
import {
  isActiveRunStatus,
  isTerminalRunStatus,
  snapshotAffordanceForRun,
  snapshotProbeStateFromAvailability,
  type SnapshotAffordanceInput,
} from "./analysis-run-snapshot-affordance";

function input(overrides: Partial<SnapshotAffordanceInput> = {}): SnapshotAffordanceInput {
  return {
    snapshotState: "captured",
    snapshotCapturedAt: "2026-05-18T10:00:00Z",
    snapshotError: null,
    probeState: "available",
    runStatus: "completed",
    surface: "run-details",
    ...overrides,
  };
}

describe("analysis run snapshot affordance", () => {
  it("marks captured available snapshots as available without degraded copy", () => {
    expect(snapshotAffordanceForRun(input())).toMatchObject({
      state: "available",
      severity: "none",
      compactLabel: null,
      badgeVariant: null,
      headerWarning: null,
      detailTitle: "Snapshot available",
      disabledReason: null,
      sanitizedError: null,
    });
  });

  it("distinguishes capture failures with sanitized backend errors", () => {
    const affordance = snapshotAffordanceForRun(input({
      snapshotState: "capture_failed",
      snapshotCapturedAt: null,
      snapshotError: "  provider stack trace\nline 2  ",
      probeState: "unavailable",
      surface: "run-details",
    }));

    expect(affordance).toMatchObject({
      state: "capture_failed_with_error",
      severity: "error",
      compactLabel: "Snapshot capture failed",
      badgeVariant: "danger",
      detailTitle: "Snapshot capture failed",
      sanitizedError: "provider stack trace line 2",
    });
  });

  it("uses softer copy when capture_failed has no error and the run failed or was cancelled", () => {
    for (const status of ["failed", "cancelled"]) {
      expect(snapshotAffordanceForRun(input({
        snapshotState: "capture_failed",
        snapshotCapturedAt: null,
        snapshotError: null,
        probeState: "unavailable",
        runStatus: status,
      }))).toMatchObject({
        state: "not_captured_before_terminal",
        compactLabel: "Snapshot not captured",
        detailTitle: "Snapshot was not captured before the run ended",
      });
    }
  });

  it("keeps capture_failed without error distinct when the terminal cause is unknown", () => {
    expect(snapshotAffordanceForRun(input({
      snapshotState: "capture_failed",
      snapshotCapturedAt: null,
      snapshotError: null,
      probeState: "unavailable",
      runStatus: "completed",
    }))).toMatchObject({
      state: "capture_failed_without_error_unknown",
      compactLabel: "Snapshot not captured",
      detailTitle: "Saved snapshot is unavailable",
    });
  });

  it("maps failed and cancelled null snapshot states with unavailable rows to not captured before terminal", () => {
    for (const status of ["failed", "cancelled"]) {
      expect(snapshotAffordanceForRun(input({
        snapshotState: null,
        snapshotCapturedAt: null,
        probeState: "unavailable",
        runStatus: status,
      }))).toMatchObject({
        state: "not_captured_before_terminal",
        detailTitle: "Snapshot was not captured before the run ended",
      });
    }
  });

  it("marks captured snapshots with unavailable rows as inconsistent", () => {
    expect(snapshotAffordanceForRun(input({
      snapshotState: "captured",
      probeState: "unavailable",
    }))).toMatchObject({
      state: "inconsistent",
      compactLabel: "Snapshot rows unavailable",
      detailTitle: "Snapshot rows are unavailable",
      disabledReason: "Exact source resolution is unavailable because the run is marked captured but saved snapshot rows are unavailable.",
    });
  });

  it("marks captured snapshots with probe errors as verification failures", () => {
    expect(snapshotAffordanceForRun(input({
      snapshotState: "captured",
      probeState: "error",
    }))).toMatchObject({
      state: "verification_failed",
      compactLabel: "Snapshot check failed",
      detailTitle: "Saved snapshot could not be verified",
      disabledReason: "Exact source resolution is unavailable because Extractum could not verify the saved snapshot rows.",
    });
  });

  it("handles the null snapshot state matrix", () => {
    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      snapshotCapturedAt: null,
      probeState: "unknown",
      runStatus: "running",
    })).state).toBe("pending");

    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      snapshotCapturedAt: null,
      probeState: "available",
      runStatus: "completed",
    })).state).toBe("available");

    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      snapshotCapturedAt: null,
      probeState: "unavailable",
      runStatus: "completed",
    }))).toMatchObject({
      state: "unknown",
      detailTitle: "Saved snapshot is unavailable",
    });

    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      snapshotCapturedAt: null,
      probeState: "error",
      runStatus: "completed",
    })).state).toBe("verification_failed");
  });

  it("classifies active, terminal, checking, and loading states", () => {
    expect(isActiveRunStatus("queued")).toBe(true);
    expect(isActiveRunStatus("running")).toBe(true);
    expect(isActiveRunStatus("completed")).toBe(false);

    expect(isTerminalRunStatus("completed")).toBe(true);
    expect(isTerminalRunStatus("failed")).toBe(true);
    expect(isTerminalRunStatus("cancelled")).toBe(true);
    expect(isTerminalRunStatus("running")).toBe(false);

    expect(snapshotAffordanceForRun(input({
      probeState: "loading",
      runStatus: "running",
    })).state).toBe("checking");
    expect(snapshotAffordanceForRun(input({
      snapshotState: null,
      probeState: "unknown",
      runStatus: "queued",
    })).state).toBe("pending");
  });

  it("derives probe state from route snapshot availability, loading, and error signals", () => {
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "available",
      loadingRunSnapshotMessages: false,
      runSnapshotError: "",
    })).toBe("available");
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "unavailable",
      loadingRunSnapshotMessages: false,
      runSnapshotError: "load failed",
    })).toBe("error");
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "unknown",
      loadingRunSnapshotMessages: true,
      runSnapshotError: "",
    })).toBe("loading");
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "capturing",
      loadingRunSnapshotMessages: false,
      runSnapshotError: "",
    })).toBe("unknown");
    expect(snapshotProbeStateFromAvailability({
      snapshotAvailability: "unknown",
      loadingRunSnapshotMessages: false,
      runSnapshotError: "",
    })).toBe("unknown");
  });
});
