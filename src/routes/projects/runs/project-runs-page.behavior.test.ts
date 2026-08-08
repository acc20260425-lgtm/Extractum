import { describe, expect, it } from "vitest";

describe("project runs screen", () => {
  it("adds a dedicated project runs route to the icon rail", async () => {
    const modulePath = "./project-runs-page";
    const { PROJECT_ICON_RAIL_ROUTES, PROJECT_RUNS_PAGE } = await import(/* @vite-ignore */ modulePath);

    expect(PROJECT_RUNS_PAGE).toEqual({
      id: "project-runs",
      href: "/projects/runs",
      label: "Runs",
      screen: "ProjectRunsScreen",
    });
    expect(PROJECT_ICON_RAIL_ROUTES.find((item: { id: string }) => item.id === "project-runs"))
      .toBe(PROJECT_RUNS_PAGE);
  });
});
