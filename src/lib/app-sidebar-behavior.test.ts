import { describe, expect, it } from "vitest";
import layoutSource from "../routes/+layout.svelte?raw";
import appSidebarSource from "./components/app-sidebar.svelte?raw";
import buttonSource from "./components/ui/Button.svelte?raw";

describe("app sidebar behavior", () => {
  it("extracts primary navigation into AppSidebar", () => {
    expect(layoutSource).toContain('import AppSidebar from "$lib/components/app-sidebar.svelte";');
    expect(layoutSource).toContain("<AppSidebar");
    expect(appSidebarSource).toContain('id="app-sidebar"');
    expect(appSidebarSource).toContain('aria-label="Primary navigation"');
    expect(appSidebarSource).toContain('class="sidebar-nav"');
    expect(appSidebarSource).toContain("{#each navItems as item (item.href)}");
  });

  it("persists desktop collapsed state separately from mobile drawer state", () => {
    expect(layoutSource).toContain('const SIDEBAR_COLLAPSED_KEY = "extractum.sidebar.collapsed";');
    expect(layoutSource).toContain("let sidebarCollapsed = $state(false);");
    expect(layoutSource).toContain("let mobileSidebarOpen = $state(false);");
    expect(layoutSource).toContain("localStorage.getItem(SIDEBAR_COLLAPSED_KEY)");
    expect(layoutSource).toContain("localStorage.setItem(SIDEBAR_COLLAPSED_KEY, String(collapsed))");
  });

  it("keeps the theme toggle in the topbar", () => {
    const topbarStart = layoutSource.indexOf('<div class="workspace-topbar">');
    const innerStart = layoutSource.indexOf('<div class="workspace-inner">');
    const sidebarStart = appSidebarSource.indexOf("<aside");
    const sidebarEnd = appSidebarSource.indexOf("</aside>");

    expect(topbarStart).toBeGreaterThanOrEqual(0);
    expect(innerStart).toBeGreaterThan(topbarStart);
    expect(layoutSource.slice(topbarStart, innerStart)).toContain('className="theme-toggle"');
    expect(layoutSource.slice(topbarStart, innerStart)).toContain("toggleTheme");
    expect(appSidebarSource.slice(sidebarStart, sidebarEnd)).not.toContain("toggleTheme");
  });

  it("supports mobile drawer controls and closes on navigation", () => {
    expect(layoutSource).toContain("mobileSidebarOpen = true");
    expect(layoutSource).toContain("mobileSidebarOpen = false");
    expect(layoutSource).toContain("handleShellKeydown");
    expect(layoutSource).toContain('event.key === "Escape"');
    expect(appSidebarSource).toContain("class:mobile-open={mobileOpen}");
    expect(appSidebarSource).toContain("handleNavClick");
    expect(appSidebarSource).toContain("onCloseMobile()");
    expect(appSidebarSource).toContain("sidebar-overlay");
    expect(appSidebarSource).toContain("sidebarElement?.focus()");
    expect(appSidebarSource).toContain("tabindex={mobileOpen ? -1 : undefined}");
  });

  it("supports icon-only collapsed desktop navigation accessibly", () => {
    expect(appSidebarSource).toContain("class:collapsed={!mobileOpen && collapsed}");
    expect(appSidebarSource).toContain("title={!mobileOpen && collapsed ? item.label : undefined}");
    expect(appSidebarSource).toContain("aria-label={item.label}");
    expect(appSidebarSource).toContain('aria-current={item.active(pathname) ? "page" : undefined}');
    expect(appSidebarSource).toContain(
      'ariaLabel={collapsed && !mobileOpen ? "Expand navigation" : "Collapse navigation"}',
    );
  });

  it("lets Button expose aria-expanded for the mobile menu", () => {
    expect(buttonSource).toContain("ariaExpanded");
    expect(buttonSource).toContain("aria-expanded={ariaExpanded}");
    expect(layoutSource).toContain("ariaExpanded={mobileSidebarOpen}");
    expect(layoutSource).toContain('ariaControls="app-sidebar"');
  });

  it("keeps the mobile menu button hidden in desktop layout despite Button defaults", () => {
    expect(layoutSource).toContain(":global(.mobile-menu-button.ui-button)");
    expect(layoutSource).toContain("display: none;");
    expect(layoutSource).toContain("display: inline-flex;");
  });
});
