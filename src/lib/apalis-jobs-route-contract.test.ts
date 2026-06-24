import { describe, expect, it } from "vitest";
import layoutSource from "../routes/+layout.svelte?raw";
import jobsPageSource from "../routes/jobs/+page.svelte?raw";
import jobsPanelSource from "./components/jobs/ApalisJobsPanel.svelte?raw";

describe("apalis jobs inspector frontend source contracts", () => {
  it("adds Jobs as a separate top-level navigation item in both modes", () => {
    expect(layoutSource).toContain("ListChecks");
    expect(layoutSource.match(/label: "Jobs"/g)?.length).toBe(2);
    expect(layoutSource.match(/caption: "Apalis queue"/g)?.length).toBe(2);
    expect(layoutSource.match(/pathname.startsWith\("\/jobs"\)/g)?.length).toBe(3);
    expect(layoutSource).toContain("Jobs");
  });

  it("keeps Tauri invocation inside the Apalis jobs API wrapper", () => {
    expect(jobsPageSource).toContain("ApalisJobsPanel");
    expect(jobsPanelSource).toContain(
      'import { loadApalisJobs, pruneOldTerminalApalisJobs } from "$lib/api/apalis-jobs";',
    );
    expect(jobsPageSource).not.toContain("invoke(");
    expect(jobsPanelSource).not.toContain("invoke(");
  });

  it("uses the shared SVAR DataGrid adapter for the jobs table", () => {
    expect(jobsPanelSource).toContain("ExtractumDataGrid");
    expect(jobsPanelSource).toContain("ExtractumDataGridColumn");
    expect(jobsPanelSource).toContain("selectedRowIds");
    expect(jobsPanelSource).toContain("onSelectedRowIdsChange");
    expect(jobsPanelSource).not.toContain('role="table"');
    expect(jobsPanelSource).not.toContain("jobs-table");
    expect(jobsPanelSource).not.toContain('from "@svar-ui/svelte-grid"');
  });

  it("implements manual refresh and guarded pruning without auto polling", () => {
    expect(jobsPanelSource).toMatch(/onMount\s*\(\s*\(\)\s*=>/);
    expect(jobsPanelSource).toContain("refreshJobs(true)");
    expect(jobsPanelSource).toContain("refreshJobs(false)");
    expect(jobsPanelSource).toContain("pruneOldTerminalApalisJobs");
    expect(jobsPanelSource).toContain("confirm(");
    expect(jobsPanelSource).toContain("Delete old finished jobs");
    expect(jobsPanelSource).toContain("Trash2");
    expect(jobsPanelSource).not.toContain("setInterval");
    expect(jobsPanelSource).not.toContain("retry");
    expect(jobsPanelSource).not.toContain("cancel");
    expect(jobsPanelSource).not.toContain("kill");
    expect(jobsPanelSource).not.toContain("copy");
  });

  it("reloads through the backend when filters change", () => {
    expect(jobsPanelSource).toContain("function handleFilterChange");
    expect(jobsPanelSource).toContain("void refreshJobs(false)");
    expect(jobsPanelSource).toContain("onchange={() => handleFilterChange()}");
    expect(jobsPanelSource).toContain("function statusFilterOptions");
    expect(jobsPanelSource).toContain("response?.statusCounts");
    expect(jobsPanelSource).toContain("statusFilterOptions(response?.statusCounts ?? [], statusFilter)");
    expect(jobsPanelSource).not.toContain('const statusOptions = ["", "Pending"');
    expect(jobsPanelSource).toContain("searchDebounce");
    expect(jobsPanelSource).toContain("refreshSequence");
    expect(jobsPanelSource).toContain("sequence !== refreshSequence");
    expect(jobsPanelSource).not.toContain("onchange={handleFilterChange}");
    expect(jobsPanelSource).not.toContain(".filter((job");
    expect(jobsPanelSource).not.toContain(".filter(job");
  });

  it("uses the user's locale and time zone for display formatting", () => {
    expect(jobsPanelSource).toContain('formatDataGridDateTimeValue(value, "datetime")');
    expect(jobsPanelSource).not.toContain('"en-US", "UTC"');
  });

  it("renders split inspector pieces and safe payload labels", () => {
    expect(jobsPanelSource).toContain('return "danger"');
    expect(jobsPanelSource).not.toContain('return "error"');
    expect(jobsPanelSource).toContain("selectedJobId ? response?.jobs.find");
    expect(jobsPanelSource).not.toContain("?? response?.jobs[0]");

    for (const token of [
      "Status",
      "Job type",
      "Search",
      "Limit",
      "Refresh",
      "Delete old finished jobs",
      "Job payload",
      "Last result",
      "Metadata",
      "truncated",
      "redacted",
      "No Apalis jobs match these filters.",
      "Select a job",
    ]) {
      expect(jobsPanelSource).toContain(token);
    }
  });
});
