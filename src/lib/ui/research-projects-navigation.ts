export const PROJECT_RUNS_PAGE = Object.freeze({
  id: "project-runs",
  href: "/projects/runs",
  label: "Runs",
  screen: "ProjectRunsScreen",
});

export const PROJECT_ICON_RAIL_ROUTES = Object.freeze([
  Object.freeze({ id: "projects", href: "/projects", label: "Projects" }),
  Object.freeze({ id: "library-prototype", href: "/projects/library", label: "Library" }),
  PROJECT_RUNS_PAGE,
  Object.freeze({ id: "diagnostics", href: "/diagnostics", label: "Diagnostics" }),
  Object.freeze({ id: "settings", href: "/settings", label: "Settings" }),
]);
