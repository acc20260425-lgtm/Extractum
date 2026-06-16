# Settings Redesign: Tabbed Layout & Hybrid Auth (Light Theme)

This document presents the finalized design plan for the **Settings** panel redesign in Projects mode, built as a tabbed layout in a clean **Light Theme**.

---

## 1. Page Architecture & Shared State

* **Unified Backend Storage:** Both the Legacy UI settings page and the new Projects UI settings page call the same Tauri commands (`get_llm_profiles`, `save_llm_profile`). Consequently, they share the same SQLite database (`app_settings` table) and OS credential storage. Any profile created, updated, or activated in one interface is immediately reflected in the other.
* **Top-Level Tabs:**
  1. **LLM Profiles:** Manage LLM configurations (CRUD Table + Modal Editor).
  2. **Telegram Sync:** Configure Telegram-specific synchronization behaviors (modes and values).
  3. **YouTube Sync:** Configure YouTube credentials/cookies and sync constraints (delay, language, parallelism).

---

## 2. LLM Profiles Tab

The LLM Profiles tab lists all saved profiles in a clean **CRUD Table**.

### CRUD Table Features
* **Columns:** Profile ID, Provider (Gemini / OpenAI), Default Model, API Key Configured (Badge), Actions (Edit, Set Active, Delete).
* **Add Profile Button:** Opens a clean dialog box to create a new profile.
* **Edit Button:** Opens the same dialog box populated with the profile's current values.
* **Active Profile Status:** Active profile setting remains global.
* **Delete Button (New Backend command needed):** Deletes the selected profile. Because the backend does not currently support deleting profiles, we will implement a new `delete_llm_profile` Tauri command (see Section 5).

---

## 3. Pop-up Editor Dialog (Modal)

When the user clicks "Add Profile" or the Edit icon in the table, a pop-up dialog opens.

### Editor Dialog Workflow
1. **Credentials Setup:** Input Profile ID, select Provider (Gemini / OpenAI-compatible), enter API Key (and optional Base URL).
   * *API Key Security:* Keep the old behavior where the saved API key is not returned to the form. Shows a text hint: *"Saved key configured. Leave blank to keep it."*
2. **Fetch Models:** Click "Fetch Models" button. The app connects to the provider using the input credentials and loads available models.
   * *Error Handling:* If connection fails, errors are rendered as a **local error banner** inside the dialog (so the user doesn't lose form data or get the window closed).
3. **Model Selection:** The dialog displays the fetched models list. The user selects one model as the default model.
4. **Save:** Clicking "Save" persists the Profile ID, Provider, and Default Model in the database, and stores the API Key in the OS keyring.

---

## 4. Telegram & YouTube Sync Tabs

General application settings are organized into these tabs:

* **Telegram Sync Tab:**
  * Configures the `InitialSyncMode` settings (`RecentMessages` or `RecentDays`) and the corresponding value.
* **YouTube Sync Tab:**
  * Displays YouTube Auth status.
  * **Hybrid Cookie Input:** Allows the user to either drag & drop / upload their exported `cookies.txt` file directly (usually downloaded to the "Downloads" folder via extensions like *Get cookies.txt*), or click a toggle to paste the raw cookies text into a standard `textarea` manually.
  * Configures captions language, delay between requests, and parallel video/comment sync counts.

---

## 5. New Backend Command: `delete_llm_profile`

To support the delete action in the CRUD table, a new Rust Tauri command `delete_llm_profile` will be added:

* **Signature:** `pub async fn delete_llm_profile(handle: AppHandle, profile_id: String) -> AppResult<LlmProfilesState>`
* **Implementation Details:**
  * Validates that the profile being deleted is not the `"default"` profile (which is protected).
  * Deletes keys from SQLite `app_settings` matching:
    * `profile.{profile_id}.provider`
    * `profile.{profile_id}.default_model`
    * `profile.{profile_id}.base_url`
  * Deletes the API Key secret from the OS secret store: `llm.profile.{profile_id}.api_key`.
  * If the deleted profile was the `active_profile`, resets the active profile to `"default"`.
  * Returns the updated `LlmProfilesState` to Svelte to refresh the UI.
