# Settings Redesign: Tabbed Layout & Hybrid Auth Implementation Plan

This document details the step-by-step tasks required to implement the redesigned **Settings** page for the new Projects interface of Extractum.

---

## Technical Architecture

* **Backend Changes (Rust/Tauri):** Add a new `delete_llm_profile` command to delete settings keys from the SQLite database and clean up OS keyring secrets.
* **Frontend Changes (SvelteKit):**
  * Modify `src/routes/settings/+page.svelte` to render different components depending on the current `uiMode` (`legacy` vs `projects`).
  * Create `src/lib/components/settings/ProjectsSettings.svelte` to house the new Tabbed Layout.
  * Integrate Telegram Sync settings (`sync.initial.mode`/`value`) and YouTube Sync settings (including a new hybrid cookie text/file input).

---

## Implementation Steps

### Task 1: Rust Backend Command `delete_llm_profile`

- [ ] **Step 1: Implement database deletion logic in `profiles.rs`**
  Modify `src-tauri/src/llm/profiles.rs` to delete profile keys:
  ```rust
  pub(crate) async fn delete_profile_from_pool(
      pool: &Pool<Sqlite>,
      secret_store: &SecretStoreState,
      profile_id: &str,
  ) -> AppResult<()> {
      let profile_id = normalize_profile_id(profile_id)?;
      if profile_id == "default" {
          return Err(AppError::validation("Cannot delete the default profile"));
      }

      delete_setting(pool, &profile_provider_key(&profile_id)).await?;
      delete_setting(pool, &profile_model_key(&profile_id)).await?;
      delete_setting(pool, &profile_base_url_key(&profile_id)).await?;

      let key = llm_profile_api_key_secret(&profile_id);
      secret_store.delete_secret(key).await?;

      // Reset active profile to default if it was the deleted one
      let active = read_setting(pool, active_profile_key())
          .await?
          .unwrap_or_else(|| "default".to_string());
      if normalize_profile_id(&active)? == profile_id {
          write_setting(pool, active_profile_key(), "default").await?;
      }
      Ok(())
  }
  ```

- [ ] **Step 2: Add Tauri command handler in `mod.rs`**
  In `src-tauri/src/llm/mod.rs`, add the command:
  ```rust
  #[tauri::command]
  pub async fn delete_llm_profile(
      handle: AppHandle,
      secret_store: tauri::State<'_, SecretStoreState>,
      profile_id: String,
  ) -> AppResult<LlmProfilesState> {
      let pool = get_pool(&handle).await?;
      delete_profile_from_pool(&pool, &secret_store, &profile_id).await?;
      load_profiles_state_from_pool(&pool, &secret_store).await
  }
  ```

- [ ] **Step 3: Register command in `lib.rs`**
  In `src-tauri/src/lib.rs`, import and register `delete_llm_profile`.

- [ ] **Step 4: Add tests in `profiles.rs`**
  Write a test verifying profile settings and secret deletion.

---

### Task 2: Split Svelte Settings page by UI Mode

- [ ] **Step 1: Modify `src/routes/settings/+page.svelte`**
  Update the main settings route page to check `uiMode` and render conditionally:
  ```svelte
  {#if uiMode === "projects"}
    <ProjectsSettings />
  {:else}
    <!-- Render legacy Settings UI -->
  {/if}
  ```

---

### Task 3: Create `ProjectsSettings.svelte` component

- [ ] **Step 1: Create Svelte 5 component**
  Create `src/lib/components/settings/ProjectsSettings.svelte` with:
  * A tab navigation bar (LLM Profiles, Telegram Sync, YouTube Sync).
  * State variables to track active tab and profile list.

- [ ] **Step 2: Implement LLM Profiles CRUD Table**
  * Render a Svelte table displaying profile entries.
  * Implement an "Add Profile" button and "Edit"/"Delete" action buttons.
  * Clicking "Delete" invokes `delete_llm_profile` and refreshes table state.

- [ ] **Step 3: Implement Pop-up Dialog for Editing Profiles**
  * Create/reuse a modal dialog (`DesktopDialog`).
  * Contain input fields for Profile ID, Provider (Select), API Key (Password), and Base URL.
  * Integrate "Fetch Models" button that calls `list_llm_provider_models`.
  * Display a **local error banner** inside the dialog on failure.
  * List fetched models and let user select one as the default model.
  * "Save" button invokes `save_llm_profile`.

---

### Task 4: Implement Telegram & YouTube Sync Tabs

- [ ] **Step 1: Telegram Sync Tab**
  * Call `get_sync_settings` on mount.
  * Display Select dropdown for mode (`RecentMessages` / `RecentDays`) and numeric input for value.
  * "Save" button calls `save_sync_settings`.

- [ ] **Step 2: YouTube Sync Tab & Hybrid Cookie Loader**
  * Embed / refactor `youtube-settings-panel.svelte` to support hybrid cookie loader.
  * **File Upload / Drag & Drop:** Allow selecting/dropping a `cookies.txt` file. Use JavaScript `FileReader.readAsText` to retrieve the contents, then call `saveYoutubeCookies`.
  * **Text Input Fallback:** A text area toggle to paste manually.

---

### Task 5: Compilation and Verification

- [ ] **Step 1: Cargo check**
  Run `cargo check` inside `src-tauri/` to ensure Rust code compiles.

- [ ] **Step 2: Frontend build**
  Run `npm run build` or Vite verify to ensure TypeScript and Svelte components compile without errors.
