// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import {
  isSourceKeyboardEditableTarget,
  sourceGridRowIdsFromElement,
  sourceKeyboardCommand,
} from "./research-projects-source-keyboard";

describe("research project source keyboard navigation", () => {
  const orderedSourceIds = ["10", "20", "30"];

  it("moves the active source through the ordered visible rows", () => {
    expect(
      sourceKeyboardCommand({
        key: "ArrowDown",
        orderedSourceIds,
        activeSourceId: "10",
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: true, kind: "activate", sourceId: "20" });

    expect(
      sourceKeyboardCommand({
        key: "ArrowUp",
        orderedSourceIds,
        activeSourceId: "20",
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: true, kind: "activate", sourceId: "10" });
  });

  it("clamps movement at the first and last visible rows", () => {
    expect(
      sourceKeyboardCommand({
        key: "ArrowUp",
        orderedSourceIds,
        activeSourceId: "10",
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: true, kind: "activate", sourceId: "10" });

    expect(
      sourceKeyboardCommand({
        key: "ArrowDown",
        orderedSourceIds,
        activeSourceId: "30",
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: true, kind: "activate", sourceId: "30" });
  });

  it("starts from the edge when no source is active yet", () => {
    expect(
      sourceKeyboardCommand({
        key: "ArrowDown",
        orderedSourceIds,
        activeSourceId: null,
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: true, kind: "activate", sourceId: "10" });

    expect(
      sourceKeyboardCommand({
        key: "ArrowUp",
        orderedSourceIds,
        activeSourceId: null,
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: true, kind: "activate", sourceId: "30" });
  });

  it("opens and toggles the active row from keyboard commands", () => {
    expect(
      sourceKeyboardCommand({
        key: "Enter",
        orderedSourceIds,
        activeSourceId: "20",
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: true, kind: "inspect", sourceId: "20" });

    expect(
      sourceKeyboardCommand({
        key: " ",
        orderedSourceIds,
        activeSourceId: "20",
        selectedSourceIds: ["10"],
      }),
    ).toEqual({
      handled: true,
      kind: "toggleSelection",
      sourceId: "20",
      selectedSourceIds: ["10", "20"],
    });

    expect(
      sourceKeyboardCommand({
        key: " ",
        orderedSourceIds,
        activeSourceId: "20",
        selectedSourceIds: ["10", "20"],
      }),
    ).toEqual({
      handled: true,
      kind: "toggleSelection",
      sourceId: "20",
      selectedSourceIds: ["10"],
    });
  });

  it("treats Escape as a close-surface command", () => {
    expect(
      sourceKeyboardCommand({
        key: "Escape",
        orderedSourceIds,
        activeSourceId: "20",
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: true, kind: "escape" });
  });

  it("ignores keys when there are no rows or no active row for row actions", () => {
    expect(
      sourceKeyboardCommand({
        key: "ArrowDown",
        orderedSourceIds: [],
        activeSourceId: null,
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: false });

    expect(
      sourceKeyboardCommand({
        key: "Enter",
        orderedSourceIds,
        activeSourceId: null,
        selectedSourceIds: [],
      }),
    ).toEqual({ handled: false });
  });

  it("detects editable targets that should keep keyboard input", () => {
    const input = document.createElement("input");
    const textarea = document.createElement("textarea");
    const contentEditable = document.createElement("div");
    contentEditable.setAttribute("contenteditable", "true");
    const button = document.createElement("button");

    expect(isSourceKeyboardEditableTarget(input)).toBe(true);
    expect(isSourceKeyboardEditableTarget(textarea)).toBe(true);
    expect(isSourceKeyboardEditableTarget(contentEditable)).toBe(true);
    expect(isSourceKeyboardEditableTarget(button)).toBe(false);
  });

  it("extracts rendered grid row ids in DOM order", () => {
    const host = document.createElement("div");
    host.innerHTML = `
      <div class="wx-row" data-id=":30"></div>
      <div class="wx-row" data-id=":10"></div>
      <div class="wx-row" data-id="20"></div>
    `;

    expect(sourceGridRowIdsFromElement(host)).toEqual(["30", "10", "20"]);
  });
});
