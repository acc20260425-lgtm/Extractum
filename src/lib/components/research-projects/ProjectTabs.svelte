<script module lang="ts">
  export type ProjectSectionId =
    | "overview"
    | "sources"
    | "evidence"
    | "reports"
    | "runs"
    | "prompts";

  export const PROJECT_SECTIONS: { id: ProjectSectionId; label: string }[] = [
    { id: "overview", label: "Обзор" },
    { id: "sources", label: "Источники" },
    { id: "evidence", label: "Факты" },
    { id: "reports", label: "Отчёты" },
    { id: "runs", label: "Запуски" },
    { id: "prompts", label: "Промпты" },
  ];
</script>

<script lang="ts">
  let {
    active,
    onSelect,
  }: {
    active: ProjectSectionId;
    onSelect?: (id: ProjectSectionId) => void;
  } = $props();

  function focusTab(event: KeyboardEvent, index: number) {
    const tabs = Array.from(
      (event.currentTarget as HTMLElement)
        .closest('[role="tablist"]')
        ?.querySelectorAll<HTMLButtonElement>('[role="tab"]') ?? [],
    );
    tabs[index]?.focus();
  }

  function handleTabKeydown(event: KeyboardEvent, index: number) {
    let nextIndex: number | null = null;
    if (event.key === "ArrowRight") {
      nextIndex = (index + 1) % PROJECT_SECTIONS.length;
    } else if (event.key === "ArrowLeft") {
      nextIndex = (index - 1 + PROJECT_SECTIONS.length) % PROJECT_SECTIONS.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = PROJECT_SECTIONS.length - 1;
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelect?.(PROJECT_SECTIONS[index].id);
      return;
    }

    if (nextIndex === null) return;
    event.preventDefault();
    focusTab(event, nextIndex);
    onSelect?.(PROJECT_SECTIONS[nextIndex].id);
  }
</script>

<div class="project-tabs" role="tablist" aria-label="Разделы проекта">
  {#each PROJECT_SECTIONS as section, index (section.id)}
    <button
      type="button"
      role="tab"
      class="project-tabs__tab"
      aria-selected={active === section.id}
      tabindex={active === section.id ? 0 : -1}
      onclick={() => onSelect?.(section.id)}
      onkeydown={(event) => handleTabKeydown(event, index)}
    >
      {section.label}
    </button>
  {/each}
</div>

<style>
  .project-tabs {
    height: 40px;
    flex-shrink: 0;
    display: flex;
    align-items: stretch;
    gap: 20px;
    padding: 0 16px;
    background: var(--extractum-surface);
    border-bottom: 1px solid var(--extractum-border);
  }

  /* scoped override глобального button-правила */
  .project-tabs .project-tabs__tab {
    display: flex;
    align-items: center;
    padding: 0;
    border: none;
    background: transparent;
    font: 600 13px/1 var(--extractum-font);
    color: var(--extractum-muted);
    cursor: pointer;
  }

  .project-tabs .project-tabs__tab[aria-selected="true"] {
    font-weight: 700;
    color: var(--extractum-primary);
    box-shadow: inset 0 -2px 0 var(--extractum-primary);
  }

  .project-tabs .project-tabs__tab:hover:not([aria-selected="true"]) {
    color: var(--extractum-text);
  }
</style>
