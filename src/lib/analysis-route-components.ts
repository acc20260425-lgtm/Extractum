import CompactSourceRail from "$lib/components/analysis/compact-source-rail.svelte";
import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";
import RunCompanionTabs from "$lib/components/analysis/run-companion-tabs.svelte";

export const analysisRouteComponents = {
  sourceRail: CompactSourceRail,
  reportCanvas: ReportCanvas,
  runCompanion: RunCompanionTabs,
};
