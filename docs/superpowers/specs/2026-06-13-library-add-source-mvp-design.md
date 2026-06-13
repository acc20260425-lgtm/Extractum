# Library Add Source MVP Design

Date: 2026-06-13

## Goal

Design the first real Add Source flow for `/projects/library`.

The feature turns the current placeholder `Add` button into a modal workflow
that can create new Library sources using the source APIs that already exist
where possible. This slice should be useful, but still conservative: YouTube URL
add, YouTube playlist-to-video add, and Telegram dialog add.

## Confirmed Brief

- The Add Source entry point lives in Library, opened from the existing
  `LibraryWorkspace` `Add` button.
- Use a centered modal, not the right Inspector panel and not a drawer.
- The modal top-level choice is provider-first:
  - `YouTube`;
  - `Telegram`.
- The modal should use shadcn-svelte tabs through `extractum-ui` wrappers, not
  direct shadcn imports from Library files.
- YouTube has two inner modes:
  - `Smart import`;
  - `From existing data`.
- Telegram has no smart URL import in this MVP.
- Telegram sources are added only from an authorized account's visible dialogs.
- YouTube channel URLs are detected and shown as `Not supported yet`.
- YouTube `From existing data` supports only existing playlist sources in the
  first MVP. Existing YouTube channels are not supported yet.
- Selecting videos from a YouTube playlist creates standalone YouTube video
  sources in Library.

## Chosen UX Model

The Add Source modal uses provider tabs at the top:

```text
Add source

[ YouTube ] [ Telegram ]
```

The YouTube tab contains a second tab group:

```text
YouTube

[ Smart import ] [ From existing data ]
```

The Telegram tab is a single account-dialog picker. It does not expose a smart
URL box yet, because Telegram add depends on an authorized account and current
backend resolution requires `account_id`.

This model makes the user's first decision simple: choose the provider. Import
method becomes a YouTube-specific choice instead of a global modal concept.

## Non-Goals

- Do not implement direct Telegram URL import.
- Do not support Telegram private invite links in this slice.
- Do not implement YouTube channel ingestion.
- Do not add a bulk backend command in this slice. Playlist video import should
  use repeated `addYoutubeSource(canonicalUrl)` calls first; a bulk command can
  be designed later if the MVP proves it is needed.
- Do not redesign the Library shell, filter rail, table, or Inspector.
- Do not introduce direct shadcn/SVAR imports in Library feature files.
- Do not change the existing Connect from library workflow.

## Component Boundaries

Create the Library Add Source UI as Library-owned components, but keep generic
UI primitives inside `extractum-ui`.

Expected component shape:

- `LibraryAddSourceDialog.svelte`
  - owns modal open state, provider tab state, and top-level status;
  - emits `onSourcesChanged(sourceId?: number)` after successful adds;
  - emits status messages to the parent Library workflow if needed.
- `LibraryYoutubeAddPanel.svelte`
  - owns YouTube provider tab content;
  - contains Smart import and From existing data modes.
- `LibraryYoutubeSmartImport.svelte`
  - can reuse or extract behavior from the existing
    `youtube-source-add-panel.svelte`;
  - calls `previewYoutubeSource` and `addYoutubeSource`.
- `LibraryYoutubePlaylistImport.svelte`
  - lists existing Library YouTube playlist sources;
  - calls `getYoutubePlaylistDetail`;
  - lets the user select playlist videos;
  - creates standalone video sources through `addYoutubeSource`.
- `LibraryTelegramDialogImport.svelte`
  - lists accounts and statuses;
  - calls `listTelegramSources(accountId)`;
  - calls `addTelegramSource`.

Use existing `ExtractumTabs`, `ExtractumTabsList`, `ExtractumTabsTrigger`, and
`ExtractumTabsContent` exports for tabs. If the wrapper is too low-level for the
Library dialog, add a small `extractum-ui` wrapper that still owns all direct
shadcn-svelte imports.

Use a centered dialog primitive through an `extractum-ui` wrapper as well. The
project already has shadcn-svelte dialog primitives under
`$lib/components/ui/dialog`, but Library feature files should not import those
directly. If no `ExtractumDialog` wrapper exists when implementation starts,
create one in `src/lib/components/extractum-ui` and export its needed parts from
`src/lib/components/extractum-ui/index.ts`.

## YouTube Smart Import

The Smart import mode has one URL input.

Supported outcomes:

- YouTube video URL:
  - detect as `video`;
  - preview through `previewYoutubeSource(url)`;
  - add through `addYoutubeSource(url)`.
- YouTube playlist URL:
  - detect as `playlist`;
  - preview through `previewYoutubeSource(url)`;
  - add through `addYoutubeSource(url)`;
  - adding a playlist should also preserve playlist item rows, matching current
    backend behavior.
- YouTube channel URL:
  - detect common channel shapes in the frontend, such as `/@handle` and
    `/channel/UC...`;
  - do not call preview/add;
  - show `Not supported yet`.
- Non-YouTube or Telegram input:
  - do not attempt provider switching automatically;
  - show a clear validation message in the active YouTube tab.

The existing backend URL parser currently supports video, shorts, live, and
playlist URL shapes. Channel `Not supported yet` should be handled before
calling the backend.

## YouTube From Existing Data

The From existing data mode lets a user create standalone video sources from an
existing Library playlist.

Flow:

1. Show existing Library sources where `provider = "youtube"` and
   `source_subtype = "playlist"`.
2. User selects one playlist.
3. Load details with `getYoutubePlaylistDetail(sourceId)`.
4. Render playlist items with checkbox selection.
5. Disable rows that cannot be added:
   - missing `canonicalUrl`;
   - already represented by `videoSourceId`.
6. The user selects one or more addable videos.
7. `Add selected` calls `addYoutubeSource(item.canonicalUrl)` for each selected
   item.
8. After completion, refresh Library data and show a result summary.

Limit the first MVP batch to 10 selected videos per run. If the user selects
more than 10 addable videos, keep `Add selected` disabled and show a scoped
message asking them to reduce the selection. This keeps the repeated
`yt-dlp`-based path predictable until a bulk backend command exists.

Result summary should include:

- added count;
- skipped count for already-existing or disabled rows;
- failed count with per-row error messages.

Count a successful `addYoutubeSource` response as `added` even if the backend
upsert returns an existing source that was not linked to this playlist item
before the operation. Rows with `videoSourceId` already present before the
operation are `skipped`.

For the MVP, run adds sequentially or with very small concurrency. This avoids
creating too many simultaneous `yt-dlp` operations. A later backend command can
optimize bulk import once the product behavior is proven.

## Telegram Dialog Import

Telegram has one MVP path: import from account-visible dialogs.

Flow:

1. Load Telegram accounts and runtime statuses.
   - Use `listAccounts()`.
   - Use `getAccountRuntimeStatuses(accountIds)`.
2. User selects an account.
3. If the account is not ready, show the sign-in-required state.
4. User loads dialogs with `listTelegramSources(accountId)`.
5. User can filter/search by:
   - all;
   - channels;
   - supergroups;
   - groups.
6. User selects a dialog row.
7. `Add selected` calls `addTelegramSource({ accountId, sourceRef,
   expectedSubtype })`.
   - Use `String(selectedTelegramDialog.id)` as `sourceRef`.
   - Use the selected `TelegramDialogSource.sourceSubtype` as
     `expectedSubtype`.
8. Refresh Library data and show the added source.

This deliberately avoids Telegram smart import. Public usernames and private
links can be revisited in a later slice after the product decides how account
selection, invite links, and access errors should behave.

## Modal State And Selection

The modal should keep state local and predictable:

- default provider tab: `YouTube`;
- default YouTube mode: `Smart import`;
- switching provider keeps each provider's local draft state while the modal is
  open;
- closing and reopening the modal resets drafts unless a later UX decision says
  drafts should be preserved;
- after successful add, keep the modal open and show the result summary;
- `Refresh` in the main Library table should still work independently.

The parent Library screen should refresh data after any successful add. If a
single source id is available, selecting it after refresh is allowed, but not
required for the MVP. The safer MVP behavior is to keep the current Library
selection and report the result in the modal.

## Error Handling

Errors should be shown at the smallest useful scope:

- YouTube Smart import preview/add errors stay in the Smart import panel.
- YouTube playlist item failures stay attached to the item result list.
- Telegram account readiness problems stay in the Telegram panel.
- Dialog load failures stay near the dialog list.
- A modal-level status is reserved for cross-panel failures or completion
  summaries.

Partial success is expected for playlist import. If some videos add and others
fail, do not roll back successful additions. Show a summary and leave failed
rows visible.

## Accessibility

- Use shadcn-svelte/Bits UI tab semantics through `ExtractumTabs*`.
- The dialog must support focus trapping, Escape close, and Tab navigation.
- Provider tabs and YouTube inner tabs must be keyboard-accessible.
- Playlist video rows and Telegram dialog rows must support keyboard selection.
- Disabled rows must expose why they cannot be selected.
- The `Add selected` button must stay disabled until there is at least one
  valid selected row.

## Testing

Recommended coverage:

- Contract/import-boundary test proving Library Add Source files import
  `ExtractumTabs*` from `extractum-ui`, not direct shadcn/SVAR primitives.
- Unit tests for YouTube URL classification:
  - video;
  - playlist;
  - channel `Not supported yet`;
  - unsupported provider.
- Workflow tests for YouTube Smart import:
  - preview success;
  - add success;
  - preview failure;
  - add failure.
- Workflow tests for YouTube playlist import:
  - lists only playlist sources;
  - loads playlist detail;
  - disables already-linked videos;
  - disables videos without canonical URL;
  - blocks selection batches above the MVP limit;
  - sequentially adds selected videos;
  - reports partial success.
- Workflow tests for Telegram dialog import:
  - no accounts;
  - account not ready;
  - dialog load success;
  - add selected dialog success;
  - add failure.
- Svelte check for modal/tabs binding types.

## Open Risks

- Playlist bulk import through repeated `addYoutubeSource` calls may be slow for
  large selections. Keep the first UI selection count modest or add clear
  progress feedback.
- Existing YouTube detail data may include unlinked playlist items. Those rows
  must be disabled or added via canonical URL only if the URL exists.
- Channel URL detection happens in the frontend for this MVP because backend
  channel ingestion is not supported.
- The existing analysis add-source dialog uses older `$lib/components/ui/*`
  imports. New Library code should follow the newer `extractum-ui` boundary even
  if older analysis code has not been migrated.

## Acceptance Criteria

- Clicking Library `Add` opens a centered Add Source modal.
- Top-level provider tabs are `YouTube` and `Telegram`.
- YouTube contains `Smart import` and `From existing data` inner tabs.
- YouTube Smart import can preview and add video and playlist URLs.
- YouTube channel URLs are detected and shown as `Not supported yet`.
- YouTube From existing data can add selected videos from an existing playlist
  as standalone YouTube video sources.
- Telegram can add a source from a selected account dialog.
- Library refreshes after successful adds.
- Direct shadcn/SVAR imports remain blocked from Library feature files.
