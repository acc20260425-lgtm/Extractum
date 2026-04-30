<script lang="ts">
  import DesktopDialog from "$lib/components/desktop-dialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import CheckboxRow from "$lib/components/ui/CheckboxRow.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";

  let {
    groups,
    selectedGroupId,
    selectedGroup,
    groupName,
    groupMemberSourceIds,
    sources,
    savingGroup,
    deletingGroup,
    formatTimestamp,
    isGroupSourceSelected,
    onChangeSelectedGroupId,
    onChangeGroupName,
    onToggleSource,
    onStartNewGroup,
    onSaveGroupCopy,
    onSaveGroupChanges,
    onDeleteGroup,
  }: {
    groups: AnalysisSourceGroup[];
    selectedGroupId: string;
    selectedGroup: AnalysisSourceGroup | null;
    groupName: string;
    groupMemberSourceIds: number[];
    sources: AnalysisSourceOption[];
    savingGroup: boolean;
    deletingGroup: boolean;
    formatTimestamp: (timestamp: number | null) => string;
    isGroupSourceSelected: (sourceId: number) => boolean;
    onChangeSelectedGroupId: (value: string) => void;
    onChangeGroupName: (value: string) => void;
    onToggleSource: (sourceId: number) => void;
    onStartNewGroup: () => void;
    onSaveGroupCopy: () => void | Promise<void>;
    onSaveGroupChanges: () => void | Promise<void>;
    onDeleteGroup: () => void | Promise<void>;
  } = $props();

  let editorOpen = $state(false);

  function openNewGroupEditor() {
    onStartNewGroup();
    editorOpen = true;
  }

  function openSelectedGroupEditor() {
    editorOpen = true;
  }

  function closeEditor() {
    editorOpen = false;
  }
</script>

<Card>
  <div class="groups">
    <div class="panel-header">
      <div>
        <h3>Source Groups</h3>
        <p class="sub">Save reusable named sets of synced sources for future cross-source reports.</p>
      </div>
      <div class="group-actions">
        <Button variant="secondary" onclick={openNewGroupEditor} disabled={savingGroup || deletingGroup}>
          New group
        </Button>
        <Button
          variant="secondary"
          onclick={openSelectedGroupEditor}
          disabled={savingGroup || deletingGroup || (!selectedGroup && !groupName.trim() && groupMemberSourceIds.length === 0)}
        >
          {selectedGroup ? "Edit group" : "Open editor"}
        </Button>
        <Button variant="danger-soft" onclick={onDeleteGroup} disabled={savingGroup || deletingGroup || !selectedGroup}>
          {deletingGroup ? "Deleting..." : "Delete"}
        </Button>
      </div>
    </div>

    <div class="group-grid">
      <div class="group-form">
        <label>Saved groups
          <select
            value={selectedGroupId}
            onchange={(event) => onChangeSelectedGroupId((event.currentTarget as HTMLSelectElement).value)}
          >
            <option value="">Create a new group</option>
            {#each groups as group (group.id)}
              <option value={String(group.id)}>
                {group.name} - {group.members.length} sources
              </option>
            {/each}
          </select>
        </label>

        <label>Group name
          <Input type="text" value={groupName} placeholder="Core channels" readonly />
        </label>

        {#if selectedGroup}
          <p class="sub">
            Updated {formatTimestamp(selectedGroup.updated_at)}
          </p>
        {:else if groupName || groupMemberSourceIds.length > 0}
          <p class="sub">Unsaved draft group</p>
        {/if}
      </div>

      <div class="group-members">
        <div class="members-header">
          <h4>{selectedGroup ? "Saved Members" : "Draft Members"}</h4>
          <span class="selected-count">{groupMemberSourceIds.length} selected</span>
        </div>

        {#if sources.length === 0}
          <p class="empty">No synced sources available for grouping yet.</p>
        {:else}
          <div class="member-list">
            {#each sources as source (source.id)}
              <CheckboxRow
                checked={isGroupSourceSelected(source.id)}
                disabled={true}
                title={source.title ?? `Source ${source.id}`}
                description={`${source.item_count} messages`}
              />
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</Card>

<DesktopDialog
  open={editorOpen}
  title={selectedGroup ? "Edit Source Group" : "New Source Group"}
  description="Build reusable source sets for recurring cross-source reports."
  labelledBy="group-editor-title"
  width="44rem"
  onClose={closeEditor}
>
  <div class="editor-grid">
    <label>Group name
      <Input
        type="text"
        value={groupName}
        placeholder="Core channels"
        oninput={(event) => onChangeGroupName((event.currentTarget as HTMLInputElement).value)}
      />
    </label>

    <div class="group-members modal-members">
      <div class="members-header">
        <h4>Members</h4>
        <span class="selected-count">{groupMemberSourceIds.length} selected</span>
      </div>

      {#if sources.length === 0}
        <p class="empty">No synced sources available for grouping yet.</p>
      {:else}
        <div class="member-list">
          {#each sources as source (source.id)}
            <CheckboxRow
              checked={isGroupSourceSelected(source.id)}
              title={source.title ?? `Source ${source.id}`}
              description={`${source.item_count} messages`}
              onchange={() => onToggleSource(source.id)}
            />
          {/each}
        </div>
      {/if}
    </div>

    <footer class="modal-actions">
      <Button variant="secondary" type="button" onclick={closeEditor}>
        Cancel
      </Button>
      <Button variant="secondary" type="button" onclick={onSaveGroupCopy} disabled={savingGroup || deletingGroup}>
        {savingGroup ? "Saving..." : "Save as new"}
      </Button>
      <Button
        type="button"
        onclick={onSaveGroupChanges}
        disabled={savingGroup || deletingGroup || !selectedGroup}
      >
        {savingGroup ? "Saving..." : "Save changes"}
      </Button>
    </footer>
  </div>
</DesktopDialog>

<style>
  .groups {
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .sub,
  .empty,
  .selected-count {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .group-actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .group-grid {
    display: grid;
    grid-template-columns: minmax(260px, 360px) minmax(0, 1fr);
    gap: 1rem;
    align-items: start;
  }

  .group-form,
  .group-members {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .members-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .members-header h4 {
    margin: 0;
  }

  .member-list {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    max-height: 24rem;
    overflow: auto;
    padding-right: 0.25rem;
  }

  .group-form :global(input[readonly]) {
    cursor: default;
    color: var(--text);
    background: var(--panel-strong);
  }

  .editor-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .modal-members {
    min-height: 0;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding-top: 0.25rem;
    border-top: 1px solid var(--border);
    margin-top: 0.25rem;
  }

  @media (max-width: 1080px) {
    .group-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .modal-actions {
      flex-direction: column-reverse;
    }

    .modal-actions :global(button) {
      width: 100%;
    }
  }
</style>
