# Extractum Telegram Phase 8C Extraction Verification

**Result: implemented and retained.**

## Retained commits

- `BASE_COMMIT`: `525569d6219458e6edb4556d232b9858040b54df`
- `EXTRACTION_COMMIT`: `3e2a3b841cacfea2c4ce3cfdd27977dee569a403`
- extraction parent equals `BASE_COMMIT`: `true`
- extraction subject: `refactor: extract Telegram integration crate`
- extraction tree equals the fully verified staged tree: `true`

## Frozen 8B authority and BASE_COMMIT

- frozen 8B commit: `c4c446b1169733d8623f84bbda5e028c2e7fa365`
- frozen 8B is an ancestor of `BASE_COMMIT`: `true`

```text
src/lib/telegram-8b-staging-sha256.json 12e99b10aaaccc471ae4c950b4a3ea0331ae68db45618823ea2aa58bae29d1a9
src/lib/telegram-grammers-feature-baseline.json 774e2b979d5cdc8185a85488c965548cf09cdf1ef0ab4b9ecad58246283cf5b3
src-tauri/Cargo.lock 720e38ea632d7b932b2a23d1481528845ec9304376035b1c851c546ea402e43c
src/lib/telegram-8b-test-identities.json 507f09f4fab76bee4360185eca3fbef17fb1563e784f7654bebb430cf7f08a95
src/lib/telegram-8b-symbol-map.json f978e80cd58303fd9cd6402ba17deef1df817d22fdbf821830e9e0a5968c13b3
src-tauri/Cargo.toml ee323d7b613573918d4ad3777b238bc7e107d049588ddcfa0959dacfd1e2cf69
```

The retained Grammers feature artifact stayed byte-identical:

```json
{
  "schemaVersion": 1,
  "revision": "1f901ce6e973fdcf0e74267f3d8efad5c729daaa",
  "packages": [
    {
      "name": "grammers-client",
      "required": [],
      "forbidden": [
        "default",
        "fs",
        "html",
        "html5ever",
        "markdown",
        "parse_invite_link",
        "proxy",
        "pulldown-cmark",
        "url"
      ],
      "universe": [
        "default",
        "fs",
        "html",
        "html5ever",
        "markdown",
        "parse_invite_link",
        "proxy",
        "pulldown-cmark",
        "url"
      ]
    },
    {
      "name": "grammers-mtsender",
      "required": [],
      "forbidden": [
        "hickory-resolver",
        "proxy",
        "tokio-socks",
        "url"
      ],
      "universe": [
        "hickory-resolver",
        "proxy",
        "tokio-socks",
        "url"
      ]
    },
    {
      "name": "grammers-session",
      "required": [
        "serde"
      ],
      "forbidden": [
        "default",
        "sqlite-storage"
      ],
      "universe": [
        "default",
        "serde",
        "sqlite-storage"
      ]
    },
    {
      "name": "grammers-tl-types",
      "required": [
        "default",
        "deserializable-functions",
        "impl-debug",
        "impl-from-enum",
        "impl-from-type",
        "tl-api",
        "tl-mtproto"
      ],
      "forbidden": [
        "impl-serde"
      ],
      "universe": [
        "default",
        "deserializable-functions",
        "impl-debug",
        "impl-from-enum",
        "impl-from-type",
        "impl-serde",
        "tl-api",
        "tl-mtproto"
      ]
    }
  ]
}
```

## Transient fixture-boundary RED

The primary facade re-export failed with `E0432` and named both fixture imports.
The secondary consumer cascade was absent from this rustc emission. The command was
`cargo check`; no test executable was produced and no test ran.

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At line:9 char:5
+     cargo test --color never --manifest-path src-tauri/Cargo.toml -p  ...
+     ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `extractum-prompt-packs` (lib) generated 1 warning
   Compiling extractum-telegram v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-telegram)
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
   Compiling extractum v0.2.0 (G:\Develop\Extractum\src-tauri)
error[E0432]: unresolved imports `extractum_telegram::takeout_attempt_fixture`, `extractum_telegram::takeout_fallback_f
ixture`
  --> src\telegram_impl\lib.rs:14:5
   |
14 |     takeout_attempt_fixture,
   |     ^^^^^^^^^^^^^^^^^^^^^^^ no `takeout_attempt_fixture` in the root
15 |     takeout_fallback_fixture,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^ no `takeout_fallback_fixture` in the root
   |
note: found an item that was configured out
  --> crates\extractum-telegram\src\lib.rs:25:44
   |
24 | #[cfg(test)]
   |       ---- the item is gated here
25 | pub(crate) use takeout::attempt_fixture as takeout_attempt_fixture;
   |                                            ^^^^^^^^^^^^^^^^^^^^^^^
note: found an item that was configured out
  --> crates\extractum-telegram\src\lib.rs:27:45
   |
26 | #[cfg(test)]
   |       ---- the item is gated here
27 | pub(crate) use takeout::fallback_fixture as takeout_fallback_fixture;
   |                                             ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0432`.
warning: `extractum` (lib test) generated 5 warnings
error: could not compile `extractum` (lib test) due to 1 previous error; 5 warnings emitted
```

## File disposition and content identity

The retained extraction moved exactly 19 prepared source paths. Seventeen are
Git-blob identical; only `lib.rs` and `takeout/mod.rs` have the two approved
complete zero-context diffs below.

### 17 unchanged Git blob pairs

```text
dto.rs 04ded173495431c07c130559a1e71a521923612f 04ded173495431c07c130559a1e71a521923612f
error.rs d402d14960eac01378221856d1ac92fc7df5dfcd d402d14960eac01378221856d1ac92fc7df5dfcd
live/avatar.rs 7756c10010f0dc89857d786b002b0768b793fb5a 7756c10010f0dc89857d786b002b0768b793fb5a
live/messages.rs aa836dff6e9d3e8198d435b61164b52c75d77515 aa836dff6e9d3e8198d435b61164b52c75d77515
live/mod.rs 57fdcfe5bc97f7a5248fb0daef0dd04fd143f76f 57fdcfe5bc97f7a5248fb0daef0dd04fd143f76f
live/peer.rs ec0347712c9a90c697c7fd812ea9aa856622c9b3 ec0347712c9a90c697c7fd812ea9aa856622c9b3
live/topics.rs f2b4be2c0e37878682869938e0ac040b13ca8a5a f2b4be2c0e37878682869938e0ac040b13ca8a5a
media.rs 3424091f47a6dd2394aea9b652c406835ff3d13c 3424091f47a6dd2394aea9b652c406835ff3d13c
runtime.rs 8a699973782481cf63224d4f855b9758f5ad5642 8a699973782481cf63224d4f855b9758f5ad5642
session.rs 058078bf9bbd857bda8296fdb6d25737fd37c521 058078bf9bbd857bda8296fdb6d25737fd37c521
takeout/export_dc.rs 9a2642f9b08a8871482005fe5f548ce18233ee03 9a2642f9b08a8871482005fe5f548ce18233ee03
takeout/forum_topics.rs 65b2ca35c73555e85f09f6cba7853cb151be2210 65b2ca35c73555e85f09f6cba7853cb151be2210
takeout/operations.rs 8879ba7cf2c6eb304a10c66387447f95445f6113 8879ba7cf2c6eb304a10c66387447f95445f6113
takeout/pagination.rs 2c0ddd6eb0f11dfb3cc92995bdd57cad091f8994 2c0ddd6eb0f11dfb3cc92995bdd57cad091f8994
takeout/raw_parse.rs 78c91f358afb42a2c5cd344ddb09dc2f5c49c770 78c91f358afb42a2c5cd344ddb09dc2f5c49c770
takeout/transport.rs 96de9f058bb9c2f98014341576cfc18d3c82f595 96de9f058bb9c2f98014341576cfc18d3c82f595
takeout/types.rs d613ae16cc6b19d65ad79cefaec666744caf76bf d613ae16cc6b19d65ad79cefaec666744caf76bf
```

### lib.rs approved complete diff

```diff
diff --git a/5736f2affdd46c60fea26765bf87e38996d975b5 b/7a906091a25e6cc318f61f5a0f89c3aab0a0c97b
index 5736f2af..7a906091 100644
--- a/5736f2affdd46c60fea26765bf87e38996d975b5
+++ b/7a906091a25e6cc318f61f5a0f89c3aab0a0c97b
@@ -24,4 +24,4 @@ pub use session::{
-#[cfg(test)]
-pub(crate) use takeout::attempt_fixture as takeout_attempt_fixture;
-#[cfg(test)]
-pub(crate) use takeout::fallback_fixture as takeout_fallback_fixture;
+#[cfg(feature = "app-test-support")]
+pub use takeout::attempt_fixture as takeout_attempt_fixture;
+#[cfg(feature = "app-test-support")]
+pub use takeout::fallback_fixture as takeout_fallback_fixture;
```

### takeout/mod.rs approved complete diff

```diff
diff --git a/c06b6557a5ad233993731e374777d54d9a101139 b/b99c91ed9503f1e293f465b2e96240c512f86c17
index c06b6557..b99c91ed 100644
--- a/c06b6557a5ad233993731e374777d54d9a101139
+++ b/b99c91ed9503f1e293f465b2e96240c512f86c17
@@ -74,2 +74,2 @@ pub(super) async fn takeout_forum_topics(
-#[cfg(test)]
-pub(crate) fn fallback_fixture(
+#[cfg(feature = "app-test-support")]
+pub fn fallback_fixture(
@@ -87,2 +87,2 @@ pub(crate) fn fallback_fixture(
-#[cfg(test)]
-pub(crate) fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {
+#[cfg(feature = "app-test-support")]
+pub fn attempt_fixture(home_dc_id: i32, export_dc_id: i32) -> TakeoutAttempt {
```

The two corrected source/destination blob pairs are:

```text
lib.rs 5736f2affdd46c60fea26765bf87e38996d975b5 7a906091a25e6cc318f61f5a0f89c3aab0a0c97b
takeout/mod.rs c06b6557a5ad233993731e374777d54d9a101139 b99c91ed9503f1e293f465b2e96240c512f86c17
```

## Cargo package and dependency ownership

```json
{
    "workspace_members":  [
                              "extractum",
                              "extractum-analysis",
                              "extractum-core",
                              "extractum-llm",
                              "extractum-gemini-browser",
                              "extractum-prompt-packs",
                              "extractum-telegram"
                          ],
    "app_to_extractum_telegram_dep_kinds":  [
                                                "dev",
                                                "normal"
                                            ],
    "removed_app_direct_roots":  [
                                     "chacha20poly1305",
                                     "grammers-client",
                                     "grammers-mtsender",
                                     "grammers-session",
                                     "grammers-tl-types",
                                     "rand_core"
                                 ],
    "producer_direct_roots":  [
                                  "base64",
                                  "chacha20poly1305",
                                  "extractum-core",
                                  "grammers-client",
                                  "grammers-mtsender",
                                  "grammers-session",
                                  "grammers-tl-types",
                                  "rand_core",
                                  "secrecy",
                                  "serde",
                                  "serde_json",
                                  "tokio"
                              ],
    "producer_to_app_edge":  0,
    "workspace_dependencies_equal_base":  true
}
```

The exact `[workspace.dependencies]` block is byte-identical to `BASE_COMMIT`:

```toml
[workspace.dependencies]
base64 = "0.22"
chacha20poly1305 = { version = "0.10", features = ["std"] }
grammers-client = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", default-features = false }
grammers-mtsender = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa" }
grammers-session = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", default-features = false, features = ["serde"] }
grammers-tl-types = { git = "https://codeberg.org/Lonami/grammers", rev = "1f901ce6e973fdcf0e74267f3d8efad5c729daaa", features = ["deserializable-functions"] }
parking_lot = "0.12"
rand_core = { version = "0.6", features = ["getrandom"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
secrecy = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
tempfile = "3"
time = { version = "0.3", features = ["formatting", "parsing", "macros"] }
tokio = "1"
tokio-util = "0.7"
url = "2"
zstd = "0.13"
```

The locked metadata and graph-cut contract prove seven members, exactly one
normal/dev app edge to the canonical producer package, removal of the six raw
Telegram roots from the app, the exact twelve producer roots, and no reverse
producer-to-app edge.

## Resolver-v2 fixture boundary

The first capture is the canonical feature-off proof. Producer-isolated
`--all-targets` captures are package checkpoints; consumer `--all-targets`
captures activate `app-test-support` through the app dev-dependency.

#### producer-feature-off-lib-check.txt

```text
cargo :    Compiling syn v2.0.117
At line:10 char:9
+         cargo check --color never --manifest-path src-tauri/Cargo.tom ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (   Compiling syn v2.0.117:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

    Checking zeroize v1.8.2
   Compiling serde_core v1.0.228
    Checking windows-sys v0.61.2
    Checking cipher v0.4.4
    Checking simd-adler32 v0.3.9
    Checking miniz_oxide v0.8.9
    Checking glass_pumpkin v1.10.0
    Checking aes v0.8.4
    Checking ctr v0.9.2
    Checking flate2 v1.1.9
    Checking grammers-crypto v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa#
1f901ce6)
    Checking slab v0.4.12
    Checking grammers-mtproto v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa
#1f901ce6)
    Checking futures-task v0.3.32
    Checking futures-util v0.3.32
    Checking serde_json v1.0.149
    Checking chacha20 v0.9.1
    Checking socket2 v0.6.3
    Checking mio v1.2.0
    Checking os_info v3.14.0
    Checking chrono v0.4.44
    Checking chacha20poly1305 v0.10.1
    Checking secrecy v0.8.0
   Compiling darling_core v0.23.0
   Compiling serde_derive v1.0.228
   Compiling tokio-macros v2.7.0
    Checking tokio v1.52.1
   Compiling darling_macro v0.23.0
   Compiling darling v0.23.0
   Compiling serde_with_macros v3.18.0
    Checking serde v1.0.228
    Checking extractum-core v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-core)
    Checking serde_with v3.18.0
    Checking grammers-session v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa
#1f901ce6)
    Checking grammers-mtsender v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daa
a#1f901ce6)
    Checking grammers-client v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa#
1f901ce6)
    Checking extractum-telegram v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-telegram)
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.03s
```

#### producer-isolated-all-targets-check.txt

```text
cargo :     Checking tokio v1.52.1
At line:13 char:9
+         cargo check --color never --manifest-path src-tauri/Cargo.tom ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (    Checking tokio v1.52.1:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

    Checking grammers-session v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa
#1f901ce6)
    Checking grammers-mtsender v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daa
a#1f901ce6)
    Checking grammers-client v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa#
1f901ce6)
    Checking extractum-telegram v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-telegram)
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
warning: `extractum-telegram` (lib test) generated 4 warnings (4 duplicates)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.78s
```

#### producer-isolated-all-targets-test.txt

```text
cargo :    Compiling zeroize v1.8.2
At line:16 char:9
+         cargo test --color never --manifest-path src-tauri/Cargo.toml ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (   Compiling zeroize v1.8.2:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

   Compiling serde_core v1.0.228
   Compiling windows-sys v0.61.2
   Compiling simd-adler32 v0.3.9
   Compiling cipher v0.4.4
   Compiling miniz_oxide v0.8.9
   Compiling ctr v0.9.2
   Compiling aes v0.8.4
   Compiling glass_pumpkin v1.10.0
   Compiling grammers-crypto v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa#
1f901ce6)
   Compiling flate2 v1.1.9
   Compiling grammers-mtproto v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa
#1f901ce6)
   Compiling slab v0.4.12
   Compiling futures-task v0.3.32
   Compiling serde v1.0.228
   Compiling serde_with v3.18.0
   Compiling serde_json v1.0.149
   Compiling futures-util v0.3.32
   Compiling chacha20 v0.9.1
   Compiling chrono v0.4.44
   Compiling chacha20poly1305 v0.10.1
   Compiling mio v1.2.0
   Compiling socket2 v0.6.3
   Compiling tokio v1.52.1
   Compiling os_info v3.14.0
   Compiling extractum-core v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-core)
   Compiling secrecy v0.8.0
   Compiling grammers-session v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa
#1f901ce6)
   Compiling grammers-mtsender v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daa
a#1f901ce6)
   Compiling grammers-client v0.9.0 (https://codeberg.org/Lonami/grammers?rev=1f901ce6e973fdcf0e74267f3d8efad5c729daaa#
1f901ce6)
   Compiling extractum-telegram v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-telegram)
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib test) generated 4 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.52s
     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_telegram-00e8d90229c5b1ac.exe)

running 71 tests
test dto::tests::telegram_item_kind_constant_matches_persisted_wire_value ... ok
test dto::tests::telegram_message_draft_has_single_persistence_shape ... ok
test dto::tests::telegram_message_identity_validation_rejects_invalid_values ... ok
test error::tests::channel_private_detection_reads_rpc_name_from_error_message ... ok
test error::tests::non_forum_topic_refresh_errors_are_detected ... ok
test live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure ... ok
test live::messages::tests::fallback_peer_identity_uses_telegram_history_peer_vocabulary ... ok
test live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload ... ok
test live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule ... ok
test live::messages::tests::reply_peer_context_uses_telegram_peer_kinds ... ok
test live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget ... ok
test live::peer::tests::dialog_lookup_misses_are_not_found ... ok
test live::peer::tests::dialog_lookup_not_found_message_explains_numeric_manual_limit ... ok
test live::peer::tests::peer_ref_from_identity_ignores_small_groups_without_supported_identity ... ok
test live::peer::tests::peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation ... ok
test live::peer::tests::peer_ref_from_identity_uses_channel_access_hash ... ok
test live::peer::tests::peer_ref_from_identity_uses_supergroup_access_hash ... ok
test live::peer::tests::typed_identity_builds_channel_peer_ref_when_access_hash_exists ... ok
test live::peer::tests::typed_identity_rejects_subtype_peer_kind_mismatch ... ok
test live::peer::tests::validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype ... ok
test live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor ... ok
test media::tests::derive_content_kind_tracks_text_and_media_presence ... ok
test media::tests::derive_document_media_kind_prefers_specific_signals ... ok
test runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors ... ok
test runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure ... ok
test runtime::tests::client_preserves_missing_account_error_without_authorization_check ... ok
test runtime::tests::failed_sign_in_retains_pending_attempt ... ok
test runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner ... ok
test runtime::tests::missing_account_authentication_is_false ... ok
test runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt ... ok
test runtime::tests::sign_in_without_code_request_preserves_auth_error ... ok
test runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt ... ok
test session::tests::encrypted_session_load_fails_for_wrong_account_id ... ok
test session::tests::encrypted_session_load_round_trips ... ok
test session::tests::generated_session_key_returns_write_only_encoded_secret ... ok
test session::tests::legacy_json_returns_rewrite_decision ... ok
test session::tests::missing_encrypted_key_preserves_auth_error ... ok
test session::tests::saving_session_writes_encrypted_envelope_not_plaintext ... ok
test session::tests::session_encryption_key_rejects_invalid_length ... ok
test takeout::export_dc::tests::export_dc_attempt_state_detects_first_fallback_transition ... ok
test takeout::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors ... ok
test takeout::export_dc::tests::export_dc_id_applies_tdesktop_shift ... ok
test takeout::export_dc::tests::export_dc_invoke_does_not_fallback_for_rpc_errors ... ok
test takeout::export_dc::tests::export_dc_invoke_falls_back_to_home_dc_on_local_error ... ok
test takeout::export_dc::tests::export_dc_invoke_uses_home_dc_directly_after_fallback ... ok
test takeout::export_dc::tests::takeout_init_request_uses_source_subtype_flags_and_file_limit ... ok
test takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots ... ok
test takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping ... ok
test takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome ... ok
test takeout::operations::tests::history_page_and_search_return_owned_takeout_messages ... ok
test takeout::operations::tests::only_my_messages_fallback_is_limited_to_channels ... ok
test takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity ... ok
test takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges ... ok
test takeout::pagination::tests::messages_not_modified_response_is_rejected_for_takeout_page ... ok
test takeout::pagination::tests::descending_fallback_keeps_raw_order_and_moves_to_min_message_id ... ok
test takeout::pagination::tests::messages_response_without_slice_is_terminal_page ... ok
test takeout::pagination::tests::split_selection_falls_back_when_telegram_returns_no_ranges ... ok
test takeout::pagination::tests::split_selection_uses_all_ranges_for_small_group ... ok
test takeout::pagination::tests::split_selection_uses_last_range_for_channel_and_supergroup ... ok
test takeout::pagination::tests::tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback ... ok
test takeout::pagination::tests::tdesktop_non_advancing_cursor_restarts_descending_fallback ... ok
test takeout::pagination::tests::tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id ... ok
test takeout::raw_parse::tests::parses_document_media_kind_filename_and_dimensions ... ok
test takeout::raw_parse::tests::parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids ... ok
test takeout::raw_parse::tests::parses_photo_message_metadata ... ok
test takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids ... ok
test takeout::raw_parse::tests::parses_text_message_with_reply_and_reactions ... ok
test takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id ... ok
test takeout::raw_parse::tests::skips_empty_raw_messages ... ok
test takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error ... ok
test live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes ... ok

test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

#### consumer-normal-lib-check.txt

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At line:19 char:9
+         cargo check --color never --manifest-path src-tauri/Cargo.tom ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `extractum-prompt-packs` (lib) generated 1 warning
    Checking extractum-telegram v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-telegram)
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
    Checking extractum v0.2.0 (G:\Develop\Extractum\src-tauri)
warning: unused imports: `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_GROUP`, and `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:12:43
   |
12 |     SourceSyncTarget, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                           ^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TELEGRAM_KIND_CHANNEL`
  --> src\sources\mod.rs:58:52
   |
58 |     NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                                    ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `GeminiBrowserArtifactMode`
 --> src\gemini_browser\jobs.rs:9:41
  |
9 |     DeliveredJobInput, DeliveryOutcome, GeminiBrowserArtifactMode, GeminiBrowserJob,
  |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, `GeminiBrowserRunStatus`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |     ^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib) generated 10 warnings (run `cargo fix --lib -p extractum` to apply 5 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 35.34s
```

#### consumer-feature-on-all-targets-check.txt

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At line:22 char:9
+         cargo check --color never --manifest-path src-tauri/Cargo.tom ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `extractum-prompt-packs` (lib) generated 1 warning
    Checking extractum-telegram v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-telegram)
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
    Checking extractum v0.2.0 (G:\Develop\Extractum\src-tauri)
warning: unused imports: `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_GROUP`, and `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:12:43
   |
12 |     SourceSyncTarget, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                           ^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TELEGRAM_KIND_CHANNEL`
  --> src\sources\mod.rs:58:52
   |
58 |     NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                                    ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `GeminiBrowserArtifactMode`
 --> src\gemini_browser\jobs.rs:9:41
  |
9 |     DeliveredJobInput, DeliveryOutcome, GeminiBrowserArtifactMode, GeminiBrowserJob,
  |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, `GeminiBrowserRunStatus`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |     ^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib) generated 10 warnings (run `cargo fix --lib -p extractum` to apply 5 suggestions)
warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: `extractum` (lib test) generated 14 warnings (5 duplicates) (run `cargo fix --lib -p extractum --tests` to app
ly 4 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 39.83s
```

#### consumer-feature-on-all-targets-test.txt

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At line:25 char:9
+         cargo test --color never --manifest-path src-tauri/Cargo.toml ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `extractum-prompt-packs` (lib) generated 1 warning
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
   Compiling extractum v0.2.0 (G:\Develop\Extractum\src-tauri)
warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib test) generated 14 warnings (run `cargo fix --lib -p extractum --tests` to apply 5 suggestion
s)
warning: unused imports: `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_GROUP`, and `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:12:43
   |
12 |     SourceSyncTarget, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                           ^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TELEGRAM_KIND_CHANNEL`
  --> src\sources\mod.rs:58:52
   |
58 |     NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                                    ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `GeminiBrowserArtifactMode`
 --> src\gemini_browser\jobs.rs:9:41
  |
9 |     DeliveredJobInput, DeliveryOutcome, GeminiBrowserArtifactMode, GeminiBrowserJob,
  |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, `GeminiBrowserRunStatus`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |     ^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `extractum` (lib) generated 10 warnings (5 duplicates) (run `cargo fix --lib -p extractum` to apply 4 suggesti
ons)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 53.69s
     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_lib-51295d1508d3c6a6.exe)

running 665 tests
test account_deletion::tests::active_source_job_on_owned_source_blocks_but_unowned_job_does_not ... ok
test account_deletion::tests::active_llm_request_for_completed_owned_run_blocks_but_provider_test_does_not ... ok
test account_deletion::tests::active_direct_source_analysis_run_blocks_owned_source_only ... ok
test account_deletion::tests::active_group_analysis_run_blocks_when_any_member_source_is_owned ... ok
test account_deletion::tests::active_takeout_job_on_owned_source_blocks ... ok
test account_deletion::tests::blocker_collection_keeps_multiple_categories_for_internal_diagnostics ... ok
test account_deletion::tests::missing_account_returns_not_found ... ok
test account_deletion::tests::existing_account_with_zero_sources_passes ... ok
test accounts::tests::deleting_account_removes_secret_after_database_row ... ok
test accounts::tests::creating_account_rolls_back_when_secret_write_fails ... ok
test accounts::tests::creating_account_writes_api_hash_to_secret_store_only ... ok
test account_deletion::tests::source_ingest_lock_on_owned_source_blocks_without_deleting_rows ... ok
test accounts::tests::deleting_missing_account_returns_not_found ... ok
test accounts::tests::secret_cleanup_failure_keeps_deleted_database_row_deleted ... ok
test analysis::corpus::tests::live::default_analysis_corpus_excludes_migrated_history_documents ... ok
test analysis::corpus::tests::live::description_mode_creates_synthetic_description_message ... ok
test analysis::corpus::tests::live::explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus ... ok
test analysis::corpus::tests::live::live_corpus_refs_use_local_item_ids ... ok
test analysis::corpus::tests::live::load_corpus_messages_filters_telegram_to_telegram_message ... ok
test analysis::corpus::tests::live::load_corpus_messages_filters_youtube_transcript_only_to_transcripts ... ok
test analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion ... ok
test analysis::corpus::tests::live::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref ... ok
test analysis::corpus::tests::live::load_corpus_messages_includes_youtube_comment_only_in_comments_mode ... ok
test analysis::corpus::tests::live::load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content ... ok
test analysis::corpus::tests::live::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight ... ok
test analysis::corpus::tests::live::preflight_ref_format_matches_corpus_loader_ref_format ... ok
test analysis::corpus::tests::live::youtube_description_missing_typed_metadata_skips_without_decoding_source_blob ... ok
test analysis::corpus::tests::live::source_group_opt_in_includes_only_members_with_migrated_rows ... ok
test analysis::corpus::tests::live::youtube_transcript_segment_evidence_uses_typed_source_context ... ok
test analysis::corpus::tests::live::youtube_description_rows_use_typed_metadata_with_corrupt_source_blob ... ok
test analysis::corpus::tests::preflight::preflight_count_matches_loader_for_youtube_corpus_modes ... ok
test analysis::corpus::tests::preflight::preflight_ignores_media_only_items_without_text_content ... ok
test analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows ... ok
test analysis::corpus::tests::preflight::preflight_counts_eligible_text_messages_for_sources ... ok
test analysis::corpus::tests::source_resolution::resolve_analysis_sources_rejects_mixed_provider_project ... ok
test analysis::corpus::tests::source_resolution::resolve_analysis_sources_loads_single_provider_project ... ok
test analysis::corpus::tests::source_resolution::resolve_analysis_sources_preserves_no_linked_youtube_error_message ... ok
test analysis::fixtures::tests::clear::clear_deletes_child_rows_through_fixture_parent_ids ... ok
test analysis::corpus::tests::source_resolution::resolve_run_source_ids_loads_project_sources_without_snapshot ... ok
test analysis::fixtures::tests::active_runs::fixture_cancel_waiter_marks_running_run_cancelled ... ok
test analysis::fixtures::tests::active_runs::fixture_active_state_tracks_seeded_running_run ... ok
test analysis::fixtures::tests::clear::clear_preserves_non_fixture_groups_and_members ... ok
test analysis::fixtures::tests::harness::fixture_test_pool_has_required_tables ... ok
test analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent ... ok
test analysis::fixtures::tests::seed::compressed_fixture_fields_are_readable ... ok
test analysis::fixtures::tests::seed::seed_creates_fixture_runs_with_statuses_templates_and_snapshots ... ok
test analysis::fixtures::tests::seed::seed_creates_post_sync_reader_content ... ok
test analysis::fixtures::tests::seed::seed_creates_safe_account_prompt_profile_sources_and_group ... ok
test analysis::fixtures::tests::seed::seed_creates_sources_that_pass_identity_repair ... ok
test analysis::fixtures::tests::seed::seed_creates_valid_typed_youtube_detail_metadata ... ok
test analysis::fixtures::tests::snapshot::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot ... ok
test analysis::fixtures::tests::snapshot::capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report ... ok
test analysis::fixtures::tests::summary::summary_serializes_with_camel_case_keys ... ok
test analysis::fixtures::tests::seed::seed_twice_keeps_one_deterministic_fixture_set ... ok
test analysis::groups::tests::prepare_analysis_source_group_input_preserves_baseline_error_precedence ... ok
test analysis::groups::tests::validate_group_source_type_rejects_mixed_provider_membership ... ok
test analysis::groups::tests::validate_group_source_type_accepts_matching_provider_membership ... ok
test analysis::groups::tests::validate_group_source_type_rejects_unknown_group_type ... ok
test analysis::store::tests::read_model::list_analysis_run_summaries_filters_project_runs ... ok
test analysis::store::tests::read_model::list_analysis_run_summaries_matches_all_query_terms_across_any_field ... ok
test analysis::report::tests::capture::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases ... ok
test analysis::store::tests::setup::ensure_sources_exist_returns_typed_not_found_error ... ok
test analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects ... ok
test analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit ... ok
test analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id ... ok
test analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot ... ok
test analysis::tests_application::report_profile_resolution_failure_prevents_run_creation ... ok
test analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads ... ok
test analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels ... ok
test analysis::fixtures::tests::snapshot::missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages ... ok
test analysis_documents::tests::document_metadata_envelopes_match_current_evidence_shape ... ok
test analysis_documents::tests::rebuild_analysis_documents_excludes_migrated_history_rows ... ok
test analysis::fixtures::tests::snapshot::seeded_snapshot_runs_expose_captured_snapshot_state ... ok
test analysis_documents::tests::rebuild_source_materializes_text_units_with_document_order ... ok
test analysis_documents::tests::schema_creates_analysis_documents_constraints_and_indexes ... ok
test analysis_documents::tests::rebuild_source_removes_stale_documents_and_is_idempotent ... ok
test apalis_jobs::tests::apalis_jobs_decode_failure_returns_redacted_preview_without_json ... ok
test apalis_jobs::tests::apalis_jobs_counts_ignore_their_own_active_filter ... ok
test apalis_jobs::tests::apalis_jobs_limit_excludes_large_payloads_outside_limited_rows ... ok
test apalis_jobs::tests::apalis_jobs_list_returns_empty_when_jobs_table_missing ... ok
test apalis_jobs::tests::apalis_jobs_list_clamps_limit ... ok
test apalis_jobs::tests::apalis_jobs_list_does_not_mutate_jobs ... ok
test apalis_jobs::tests::apalis_jobs_list_filters_by_status_job_type_and_search ... ok
test apalis_jobs::tests::apalis_jobs_list_returns_rfc3339_utc_timestamps ... ok
test apalis_jobs::tests::apalis_jobs_list_returns_rows_from_jobs_table ... ok
test apalis_jobs::tests::apalis_jobs_non_json_result_and_metadata_are_omitted_in_v1 ... ok
test apalis_jobs::tests::apalis_jobs_prune_terminal_returns_zero_when_jobs_table_missing ... ok
test apalis_jobs::tests::apalis_jobs_list_sorts_by_latest_activity_timestamp ... ok
test apalis_jobs::tests::apalis_jobs_row_shape_is_stable_when_optional_columns_are_absent ... ok
test apalis_jobs::tests::apalis_jobs_payloads_are_redacted_and_truncated ... ok
test archive_read_model::tests::create_schema_adds_state_and_item_tables ... ok
test archive_read_model::tests::current_ready_state_rejects_old_model_version ... ok
test apalis_jobs::tests::apalis_jobs_schema_probe_documents_local_jobs_table_shape ... ok
test child_process::tests::create_no_window_matches_win32_process_creation_flags ... ok
test apalis_jobs::tests::apalis_jobs_prune_terminal_deletes_only_old_done_killed_and_terminal_failed_jobs ... ok
test archive_read_model::tests::rebuild_source_excludes_migrated_history_rows ... ok
test diagnostics::database::tests::migration_status_reports_pending_and_failed_versions ... ok
test archive_read_model::tests::rebuild_source_materializes_archive_fidelity_fields ... ok
test diagnostics::dto::tests::diagnostic_summary_fixture_serializes_without_forbidden_sentinels ... ok
test diagnostics::redaction::tests::redact_json_value_redacts_sensitive_keys_recursively ... ok
test diagnostics::redaction::tests::redact_text_removes_secret_and_content_patterns ... ok
test diagnostics::redaction::tests::sanitized_error_message_bounds_unicode_by_chars ... ok
test diagnostics::runtime::tests::failed_runtime_check_uses_coarse_summary_without_os_error_text ... ok
test diagnostics::redaction::tests::sanitized_error_message_is_bounded ... ok
test diagnostics::runtime::tests::secure_storage_failure_does_not_expose_store_error_text ... ok
test diagnostics::tests::serialized_diagnostic_summary_preserves_allowed_data_and_excludes_forbidden_data ... ok
test external_process::tests::admission_wait_consumes_the_shared_graceful_budget ... ok
test external_process::tests::concurrent_watchdogs_invoke_exit_once ... ok
test external_process::tests::cleanup_tasks_start_concurrently_and_isolate_error_and_panic ... ok
test external_process::tests::exhausted_admission_budget_skips_the_cleanup_factory ... ok
test external_process::tests::injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback ... ok
test external_process::tests::permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown ... ok
test external_process::tests::repeated_start_does_not_replace_code_or_schedule_again ... ok
test external_process::tests::start_reports_completed_after_watchdog_claims_exit ... ok
test external_process::tests::start_returns_started_and_schedules_one_watchdog ... ok
test external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets ... ok
test external_process::tests::watchdog_exits_with_the_preserved_code_unless_cleanup_completed ... ok
test gemini_browser::cdp_chrome::tests::drop_falls_back_to_owned_child_shutdown ... ok
test gemini_browser::cdp_chrome::tests::explicit_shutdown_kills_and_reaps_the_owned_child_once ... ok
test gemini_browser::cdp_chrome::tests::shutdown_does_not_claim_or_kill_an_already_exited_child ... ok
test gemini_browser::cdp_chrome::tests::shutdown_reaps_when_the_child_has_already_exited_during_kill ... ok
test diagnostics::tests::sanitize_diagnostic_error_bounds_and_redacts_command_errors ... ok
test gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_accepts_json_version_response ... ok
test gemini_browser::executor::tests::cancelled_run_marks_the_sidecar_transport_tainted ... ok
test gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json ... ok
test external_process::tests::permits_acquired_before_shutdown_are_waited_for ... ok
test diagnostics::database::tests::database_diagnostics_groups_only_allow_listed_aggregates ... ok
test gemini_browser::jobs::app_tests::apalis_sqlite_status_probe_documents_actual_status_values ... ok
test gemini_browser::jobs::app_tests::apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job ... ok
test gemini_browser::jobs::app_tests::apalis_storage_uses_shared_main_extractum_db_identity ... ok
test gemini_browser::jobs::app_tests::apalis_storage_preserves_existing_sqlx_migration_history_table ... ok
test gemini_browser::jobs::app_tests::apalis_storage_shares_extractum_db_without_locking_app_pool ... ok
test gemini_browser::jobs::app_tests::enqueue_duplicate_run_id_returns_conflict ... ok
test gemini_browser::jobs::app_tests::gemini_browser_jobs_are_built_with_one_total_attempt ... ok
test gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_reports_unreachable_endpoint ... ok
test gemini_browser::jobs::app_tests::enqueue_persists_job_before_worker_startup ... ok
test gemini_browser::sidecar::tests::stderr_drain_consumes_sidecar_output_concurrently ... ok
test ingest_provenance::tests::completed_zero_observation_batch_is_complete_without_partial_flags ... ok
test gemini_browser::jobs::app_tests::restart_worker_processes_pending_job_after_runtime_restart ... ok
test ingest_provenance::tests::create_takeout_batch_inserts_generic_and_detail_rows_atomically ... ok
test gemini_browser::jobs::app_tests::failed_gemini_browser_job_is_not_retried ... ok
test ingest_provenance::tests::migrated_small_group_imported_allows_duplicate_only_success ... ok
test ingest_provenance::tests::migrated_history_deferred_scope_finalizes_partial_and_records_warning_once ... ok
test ingest_provenance::tests::migrated_small_group_scope_can_be_marked_running_and_completed ... ok
test job_helpers::tests::active_job_guards_track_and_release_scoped_jobs ... ok
test job_helpers::tests::cancellation_state_cancels_child_tokens ... ok
test job_helpers::tests::cancellation_state_marks_checks_and_clears_jobs ... ok
test library_sources::tests::catalog_status_for_input_keeps_failed_job_without_detail_empty ... ok
test ingest_provenance::tests::mixed_partial_scope_finalizes_as_partial ... ok
test ingest_provenance::tests::terminal_update_recalculates_counters_and_sanitizes_error ... ok
test library_sources::tests::list_library_catalog_returns_status_capabilities_and_filter_counts ... ok
test library_sources::tests::list_library_sources_keeps_sources_with_missing_provider_details ... ok
test library_sources::tests::list_library_sources_returns_youtube_and_telegram_metadata ... ok
test llm::profiles::tests::clear_profile_api_key_deletes_secret ... ok
test llm::profiles::tests::changing_key_scope_without_replacement_is_rejected ... ok
test llm::profiles::tests::active_profile_resolution_loads_key_from_secret_store ... ok
test llm::profiles::tests::credential_scope_uses_provider_origin_and_effective_port_but_not_path ... ok
test llm::profiles::tests::delete_profile_fails_if_secret_store_fails_leaving_db_settings_intact ... ok
test llm::profiles::tests::delete_profile_removes_settings_and_secret_and_resets_active ... ok
test llm::profiles::tests::empty_save_preserves_existing_secret ... ok
test llm::profiles::tests::materialization_write_failure_fails_closed_during_state_load ... ok
test llm::profiles::tests::legacy_remote_http_profile_is_rejected_before_request_configuration ... ok
test llm::profiles::tests::keyed_legacy_profile_materializes_effective_base_url_while_unkeyed_stays_blank ... ok
test llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store ... ok
test llm::profiles::tests::profile_state_lists_multiple_saved_profiles ... ok
test llm::profiles::tests::provider_access_resolution_uses_configured_key_with_saved_base_url ... ok
test llm::profiles::tests::validate_profile_id_rejects_invalid_characters ... ok
test llm::profiles::tests::set_active_profile_returns_typed_not_found_error ... ok
test llm::tests::llm_command_errors_and_failed_events_keep_distinct_json_shapes ... ok
test llm::tests::llm_stream_events_serialize_exact_lifecycle_contract ... ok
test llm::profiles::tests::provider_access_resolution_uses_saved_key_with_configured_base_url ... ok
test migrations::baseline_reset::tests::classifies_baseline_history_after_post_baseline_migrations ... ok
test gemini_browser::jobs::app_tests::worker_picks_up_job_quickly_after_idle ... ok
test migrations::baseline_reset::tests::classifies_baseline_history_only_when_checksum_matches ... ok
test llm::tests::provider_diagnostics_exclude_profile_ids_and_base_urls ... ok
test migrations::baseline_reset::tests::classifies_old_history_only_when_versions_one_through_twenty_six_are_successful ... ok
test migrations::baseline_reset::tests::rejects_baseline_history_with_wrong_checksum ... ok
test migrations::baseline_reset::tests::rejects_failed_migration_history ... ok
test migrations::baseline_reset::tests::rejects_partial_old_history_without_version_twenty_six ... ok
test migrations::tests::app_migrator_accepts_database_with_preexisting_apalis_migration_history ... ok
test migrations::tests::build_migrations_includes_apalis_sqlite_versions_for_shared_sqlx_history ... ok
test migrations::tests::analysis_telegram_history_scope_migration_adds_nullable_checked_column ... ok
test migrations::tests::build_migrations_includes_prompt_pack_runtime_provider_version_ten ... ok
test migrations::tests::build_migrations_starts_at_current_schema_baseline ... ok
test migrations::tests::current_schema_baseline_checksum_matches_frozen_reset_boundary ... ok
test migrations::tests::current_schema_baseline_migration_is_version_one ... ok
test migrations::baseline_reset::tests::backup_failure_prevents_migration_history_rewrite ... ok
test migrations::tests::fresh_schema_includes_analysis_snapshot_markers ... ok
test migrations::tests::fresh_schema_includes_analysis_documents_table_indexes_and_constraints ... ok
test migrations::baseline_reset::tests::old_history_cutover_backs_up_then_rewrites_only_migration_history ... ok
test migrations::tests::concurrent_test_migrations_publish_complete_apalis_schemas ... ok
test migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints ... ok
test migrations::tests::fresh_schema_includes_source_identity_tables_after_sql_managed_migrations ... ok
test migrations::tests::prepare_database_skips_cutover_when_database_file_is_missing ... ok
test migrations::tests::projects_mvp_migration_is_registered ... ok
test migrations::tests::fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints ... ok
test migrations::tests::post_baseline_migration_upgrades_frozen_baseline_for_migrated_history ... ok
test migrations::tests::fresh_schema_includes_projects_redesign_columns_index_and_defaults ... ok
test migrations::tests::test_migration_batch_rolls_back_schema_and_history_together ... ok
test migrations::tests::vendored_apalis_sqlite_migrations_match_pinned_dependency ... ok
test notebooklm_export::chunker::tests::accounts_for_document_overhead_when_splitting ... ok
test notebooklm_export::chunker::tests::falls_back_to_month_when_year_exceeds_limits ... ok
test notebooklm_export::chunker::tests::falls_back_to_topic_id_when_topic_title_slug_is_invalid ... ok
test notebooklm_export::chunker::tests::filters_short_text_without_other_signal ... ok
test notebooklm_export::chunker::tests::groups_chunks_by_topic_slug ... ok
test notebooklm_export::chunker::tests::keeps_yearly_group_when_within_limits ... ok
test notebooklm_export::chunker::tests::splits_by_word_and_byte_limits ... ok
test notebooklm_export::filename::tests::accepts_safe_relative_child_paths ... ok
test notebooklm_export::filename::tests::child_paths_stay_under_base ... ok
test notebooklm_export::filename::tests::rejects_reserved_components ... ok
test notebooklm_export::filename::tests::rejects_unsafe_relative_child_paths ... ok
test notebooklm_export::filename::tests::sanitizes_unsafe_filename_parts ... ok
test notebooklm_export::glossary::tests::aggregates_participants_by_author ... ok
test notebooklm_export::links::tests::detects_and_trims_http_urls ... ok
test notebooklm_export::media::tests::renders_numeric_only_media_metadata ... ok
test notebooklm_export::media::tests::renders_useful_media_placeholder_parts ... ok
test notebooklm_export::message_mapping::tests::reply_snippet_decode_failures_are_typed_internal_errors ... ok
test notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_bounded_periods ... ok
test notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_notebooklm_messages ... ok
test notebooklm_export::query::tests::corrupt_archive_reply_target_outside_period_fails_archive_loader ... ok
test notebooklm_export::query::tests::current_export_archive_loader_sets_scope_markers ... ok
test notebooklm_export::query::tests::export_fixture_rejects_null_published_at_before_loader_parity ... ok
test notebooklm_export::query::tests::load_export_messages_adds_local_reply_context_outside_period ... ok
test notebooklm_export::query::tests::load_export_messages_attaches_general_topic_when_topic_header_is_missing ... ok
test migrations::tests::projects_mvp_schema_applies_to_memory_pool ... ok
test notebooklm_export::query::tests::load_export_messages_does_not_root_match_non_numeric_external_ids ... ok
test migrations::tests::prompt_pack_mvp_migration_creates_library_and_run_tables ... ok
test notebooklm_export::query::tests::load_export_messages_attaches_topic_metadata_for_reply_and_root_messages ... ok
test migrations::tests::prompt_pack_mvp_migration_declares_required_integrity_constraints ... ok
test notebooklm_export::query::tests::load_export_messages_reads_materialized_topic_memberships ... ok
test notebooklm_export::query::tests::load_export_source_group_exposes_youtube_group_for_hard_validation ... ok
test notebooklm_export::query::tests::load_export_source_group_keeps_dirty_member_source_type_for_skip_logic ... ok
test notebooklm_export::query::tests::load_export_source_uses_canonical_subtype_not_legacy_kind ... ok
test notebooklm_export::query::tests::load_export_source_rejects_non_telegram_before_message_loader_selection ... ok
test notebooklm_export::query::tests::load_export_source_group_orders_members_by_title_then_id ... ok
test notebooklm_export::query::tests::migrated_export_reply_lookup_stays_inside_old_history_domain ... ok
test notebooklm_export::query::tests::notebooklm_archive_export_excludes_migrated_history_rows_even_if_materialized ... ok
test notebooklm_export::query::tests::notebooklm_default_export_excludes_migrated_history_rows ... ok
test notebooklm_export::query::tests::notebooklm_export_query_file_has_no_export_row_mapping ... ok
test notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_missing_and_old_version ... ok
test notebooklm_export::query::tests::notebooklm_export_loader_selection_uses_archive_for_ready_current_state ... ok
test notebooklm_export::query::tests::notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails ... ok
test notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_all_fallback_reasons ... ok
test notebooklm_export::renderer::tests::formats_metadata_as_rfc3339 ... ok
test notebooklm_export::renderer::tests::renders_json_compatible_yaml_string_scalars ... ok
test notebooklm_export::query::tests::notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection ... ok
test notebooklm_export::renderer::tests::renders_message_metadata_and_text ... ok
test notebooklm_export::renderer::tests::renders_migrated_history_scope_metadata ... ok
test notebooklm_export::renderer::tests::renders_reply_thread_and_reaction_metadata ... ok
test notebooklm_export::renderer::tests::renders_topic_aware_document_header ... ok
test notebooklm_export::tests::formats_timestamp_folder_suffix ... ok
test notebooklm_export::tests::group_member_manifest_records_source_scoped_generated_files ... ok
test notebooklm_export::tests::keeps_migrated_history_opt_in_in_validated_config ... ok
test notebooklm_export::query::tests::opted_in_export_loads_migrated_rows_separately_with_markers ... ok
test notebooklm_export::tests::load_group_export_inputs_rejects_youtube_group_for_hard_validation ... ok
test notebooklm_export::tests::marker_read_and_write_reject_existing_symlink_file ... ok
test notebooklm_export::tests::prefix_chunk_filename_adds_sources_directory_and_prefix ... ok
test notebooklm_export::tests::load_group_export_inputs_rejects_group_without_telegram_members ... ok
test notebooklm_export::tests::reads_legacy_single_source_manifest_after_manifest_expansion ... ok
test notebooklm_export::tests::marker_read_and_write_accept_normal_file ... ok
test notebooklm_export::tests::remove_generated_files_rejects_invalid_manifest_relative_path ... ok
test notebooklm_export::tests::remove_generated_files_rejects_symlink_parent_directory ... ok
test notebooklm_export::tests::render_source_export_filters_messages_and_clears_media_placeholders ... ok
test notebooklm_export::tests::render_source_export_tracks_empty_migrated_history_warning_and_section_prefix ... ok
test notebooklm_export::query::tests::notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states ... ok
test notebooklm_export::tests::render_source_group_export_errors_when_all_members_empty_after_filters ... ok
test notebooklm_export::tests::render_source_group_export_keeps_empty_member_skipped_reason_out_of_member_warnings ... ok
test notebooklm_export::tests::source_member_file_prefix_includes_index_id_and_slug ... ok
test notebooklm_export::tests::source_member_file_prefix_uses_fallback_slug_for_unsafe_title ... ok
test notebooklm_export::tests::treats_blank_export_id_as_missing ... ok
test notebooklm_export::tests::trims_optional_export_id ... ok
test notebooklm_export::tests::validates_exactly_one_export_scope ... ok
test notebooklm_export::tests::validates_period_order ... ok
test notebooklm_export::tests::validates_single_source_scope ... ok
test notebooklm_export::tests::validates_source_group_scope ... ok
test notebooklm_export::tests::write_export_file_rejects_symlink_parent_directory ... ok
test notebooklm_export::tests::write_export_file_creates_sources_parent_directory ... ok
test notebooklm_export::tests::removes_generated_files_in_sources_subdirectory ... ok
test process_tree::tests::creates_a_job_object ... ok
test process_tree::tests::assigns_a_directly_owned_std_child ... ok
test process_tree::tests::process_tree_guard_can_be_owned_by_async_application_state ... ok
test notebooklm_export::tests::write_group_export_package_records_group_manifest_and_source_files ... ok
test process_tree::tests::terminate_failure_remains_reportable_and_retryable ... ok
test process_tree::tests::dropping_the_guard_closes_the_job_and_kills_its_children ... ok
test process_tree::tests::terminate_is_idempotent ... ok
test projects::data_range::tests::project_data_range_includes_telegram_migrated_history_when_requested ... ok
test projects::data_range::tests::project_data_range_preserves_migrated_history_error_for_unknown_source_type ... ok
test projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources ... ok
test projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram ... ok
test projects::data_range::tests::project_data_range_rejects_mixed_provider_project ... ok
test projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project ... ok
test projects::data_range::tests::project_data_range_returns_nulls_for_empty_project ... ok
test projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project ... ok
test projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds ... ok
test projects::read_model::tests::list_research_projects_counts_playlist_linked_video_materials ... ok
test projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout ... ok
test projects::read_model::tests::list_research_projects_prioritizes_running_and_sorts_active_pinned_updated_first ... ok
test projects::tests::create_project_trims_and_rejects_duplicate_names_case_insensitively ... ok
test projects::tests::add_project_sources_is_idempotent_and_lists_ui_ready_rows ... ok
test projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources ... ok
test projects::tests::list_project_sources_counts_playlist_linked_video_materials ... ok
test projects::tests::list_project_sources_includes_catalog_status_last_sync_and_handle ... ok
test projects::tests::project_scoped_delete_blocks_other_active_and_archived_projects_without_mutation ... ok
test projects::tests::project_scoped_delete_rejects_invalid_sources_and_missing_links ... ok
test projects::tests::project_scoped_delete_caps_blocking_projects_and_reports_remaining_count ... ok
test projects::tests::project_scoped_delete_removes_youtube_video_and_cascaded_materials ... ok
test projects::tests::set_project_archived_toggles_timestamp_and_rejects_missing_project ... ok
test prompt_packs::browser_adapter::tests::browser_port_delegates_readiness_submission_and_cancellation_without_narrowing_result ... ok
test prompt_packs::event_adapter::tests::typed_events_map_to_exact_legacy_ipc_payloads ... ok
test prompt_packs::runtime_commands::tests::execution_adapter_resolves_api_profile_only_inside_spawned_task ... ok
test prompt_packs::runtime_commands::tests::execution_adapter_spawns_exactly_once_per_ticket ... ok
test projects::tests::project_scoped_delete_schema_source_foreign_keys_are_delete_safe ... ok
test prompt_packs::runtime_commands::tests::execution_task_reuses_start_pool_without_reacquisition ... ok
test projects::tests::set_project_pinned_toggles_flag_updates_timestamp_and_rejects_missing_project ... ok
test prompt_packs::source_adapter::tests::load_comment_body_performs_a_fresh_read_with_decompression_fallback ... ok
test prompt_packs::source_adapter::tests::load_source_preserves_caller_order_missing_rows_and_nullables ... ok
test prompt_packs::source_adapter::tests::load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows ... ok
test process_tree::tests::terminates_a_descendant_created_after_assignment ... ok
test prompt_packs::source_adapter::tests::load_transcript_segments_orders_segment_index_then_row_id ... ok
test prompt_packs::source_adapter::tests::select_comment_candidates_applies_limit_order_and_decompression_fallback ... ok
test readiness::tests::is_ready_current_requires_ready_status_and_current_version ... ok
test prompt_packs::source_adapter::tests::load_video_maps_full_nullable_metadata_and_missing_rows ... ok
test readiness::tests::mark_failed_returns_failed_state ... ok
test readiness::tests::mark_stale_only_changes_ready_state ... ok
test readiness::tests::readiness_status_roundtrips_wire_values ... ok
test secret_store::tests::in_memory_store_can_fail_each_operation ... ok
test secret_store::tests::secret_ids_are_stable ... ok
test secret_store::tests::state_reads_writes_and_deletes_secrets ... ok
test source_ingest::tests::active_kinds_for_sources_reports_matching_locks_only ... ok
test source_ingest::tests::lock_allows_different_sources ... ok
test source_ingest::tests::lock_rejects_concurrent_same_source_operations ... ok
test source_ingest::tests::lock_releases_when_guard_drops ... ok
test sources::identity::tests::canonical_external_id_rejects_malformed_values ... ok
test sources::identity::tests::load_telegram_identity_returns_typed_row ... ok
test sources::identity::tests::peer_kind_matches_telegram_subtype ... ok
test sources::identity::tests::username_normalization_removes_url_and_at_syntax ... ok
test sources::identity::tests::load_telegram_runtime_source_pairs_source_with_typed_identity ... ok
test sources::identity_repair::tests::apply_repair_creates_typed_identity_and_keeps_source_id ... ok
test prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled ... ok
test sources::identity_repair::tests::apply_repair_is_idempotent ... ok
test sources::identity_repair::tests::dry_run_reports_repair_without_writing_typed_rows ... ok
test sources::identity_repair::tests::duplicate_canonical_identity_reports_conflicting_source_ids ... ok
test sources::identity_repair::tests::duplicate_typed_peer_identity_reports_conflicting_source_ids ... ok
test sources::identity_repair::tests::fatal_repair_rolls_back_and_does_not_create_canonical_index ... ok
test sources::identity_repair::tests::missing_account_id_is_fatal ... ok
test sources::identity_repair::tests::repair_fails_on_conflicting_typed_projection_drift ... ok
test sources::identity_repair::tests::repair_creates_minimal_typed_identity_when_legacy_metadata_is_missing_or_malformed ... ok
test sources::identity_repair::tests::repair_fails_when_canonical_identity_is_invalid_even_with_legacy_peer_metadata ... ok
test sources::identity_repair::tests::repair_ignores_malformed_metadata_when_canonical_identity_is_present ... ok
test sources::identity_repair::tests::malformed_external_ids_fail_without_writing_typed_rows ... ok
test sources::identity_repair::tests::repair_ignores_optional_enrichment_gaps_when_typed_identity_is_valid ... ok
test sources::identity_repair::tests::repair_reads_post_v19_sources_without_legacy_column ... ok
test sources::identity_repair::tests::repair_rejects_zero_external_id ... ok
test sources::identity_repair::tests::repair_skips_malformed_metadata_when_typed_identity_is_valid ... ok
test sources::identity_repair::tests::repair_treats_typed_projection_mismatch_as_fatal ... ok
test sources::identity_repair::tests::source_identity_gate_blocks_while_running ... ok
test sources::identity_repair::tests::source_identity_gate_returns_startup_failure ... ok
test sources::identity_repair::tests::repair_updates_non_conflicting_typed_projection_drift ... ok
test sources::identity_repair::tests::repair_uses_canonical_subtype_without_legacy_kind ... ok
test sources::items::query::tests::archive_reader_matches_items_path_for_source_browsing_rows ... ok
test sources::items::query::tests::archive_reader_matches_topic_filter_and_around_item_semantics ... ok
test sources::items::query::tests::default_items_path_excludes_migrated_history_rows ... ok
test sources::items::query::tests::default_source_browsing_does_not_surface_migrated_rows_after_archive_ready ... ok
test prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer ... ok
test sources::items::query::tests::load_item_rows_attaches_topic_metadata_and_root_matches ... ok
test sources::items::query::tests::load_item_rows_can_start_at_selected_item ... ok
test sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_not_ready ... ok
test sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_stale ... ok
test sources::items::query::tests::merged_browsing_uses_full_cursor_tuple_for_equal_timestamps ... ok
test sources::items::query::tests::scoped_browsing_can_load_only_migrated_rows_with_labels ... ok
test sources::items::query::tests::scoped_browsing_defaults_to_current_rows ... ok
test sources::identity_repair::tests::youtube_sources_are_unaffected_by_source_identity_repair ... ok
test sources::items::query::tests::topic_filters_are_rejected_for_non_current_history_scope ... ok
test sources::items::tests::forum_topic_filter_deserializes_camel_case_topic_id ... ok
test sources::items::query::tests::telegram_load_item_rows_uses_items_path_when_archive_model_is_ready ... ok
test sources::items::query::tests::uncategorized_filter_returns_empty_when_topic_resolution_is_not_ready ... ok
test sources::items::tests::insert_telegram_source_item_resolves_topic_membership_only_for_new_item ... ok
test sources::items::tests::insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload ... ok
test sources::items::tests::insert_telegram_source_item_allows_same_message_id_in_different_history_domains ... ok
test sources::items::tests::insert_telegram_source_item_writes_payload_and_skips_duplicates ... ok
test sources::items::tests::media_metadata_roundtrip_through_zstd ... ok
test sources::items::tests::list_source_items_keeps_base_youtube_comment_when_raw_payload_is_malformed ... ok
test sources::items::tests::migrated_insert_idempotency_uses_old_chat_native_identity ... ok
test sources::items::tests::list_source_items_enriches_youtube_comment_rows_from_raw_payload ... ok
test sources::items::tests::migrated_small_group_insert_skips_current_history_derived_writes ... ok
test sources::items::tests::scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item ... ok
test sources::items::tests::single_telegram_insert_maintains_ready_archive_model ... ok
test sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build ... ok
test sources::items::tests::text_roundtrip_through_zstd ... ok
test sources::items::tests::telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate ... ok
test sources::items::tests::telegram_insert_writes_analysis_document_in_same_writer_transaction ... ok
test sources::items::tests::upsert_youtube_transcript_item_updates_existing_text_and_returns_id ... ok
test sources::items::tests::telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows ... ok
test sources::items::tests::upsert_youtube_comment_item_updates_existing_text_and_reaction_count ... ok
test sources::legacy_metadata_cleanup::tests::audit_ignores_non_telegram_and_null_metadata_rows ... ok
test sources::items::tests::youtube_comment_upsert_targets_non_telegram_partial_unique_index ... ok
test sources::items::tests::youtube_transcript_upsert_targets_non_telegram_partial_unique_index ... ok
test sources::items::tests::youtube_comment_upsert_writes_analysis_document_and_updates_content ... ok
test sources::legacy_metadata_cleanup::tests::audit_reports_eligible_legacy_telegram_metadata_without_mutating ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_invalid_typed_identity ... ok
test sources::legacy_metadata_cleanup::tests::candidate_skip_reason_rejects_unparseable_typed_identity_values ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_missing_typed_identity ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_unsupported_subtype_and_missing_account ... ok
test sources::legacy_metadata_cleanup::tests::clear_is_idempotent_after_eligible_metadata_is_removed ... ok
test sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_subtype_and_account_mismatches ... ok
test sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_empty_refs_as_validation ... ok
test sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_private_links ... ok
test sources::peer_resolution::manual_ref::tests::parse_username_accepts_username_and_t_me_links ... ok
test sources::peer_resolution::tests::add_source_resolution_strategy_distinguishes_username_and_dialog_flows ... ok
test sources::peer_resolution::tests::source_metadata_decode_failures_are_internal ... ok
test sources::peer_resolution::tests::source_metadata_decodes_old_username_only_payloads ... ok
test sources::peer_resolution::tests::source_peer_input_rejects_malformed_external_id_as_validation ... ok
test sources::peer_resolution::tests::source_metadata_decodes_old_dialog_payloads_into_peer_identity ... ok
test sources::peer_resolution::tests::source_metadata_decodes_typed_peer_identity_payloads ... ok
test sources::peer_resolution::tests::source_peer_input_rejects_unsupported_source_type_as_validation ... ok
test sources::peer_resolution::tests::source_peer_resolution_plan_prefers_explicit_strategy_order ... ok
test sources::peer_resolution::tests::typed_identity_plan_allows_username_resolution_without_access_hash ... ok
test sources::peer_resolution::tests::source_peer_resolution_failure_explains_small_group_dialog_dependency ... ok
test sources::peer_resolution::tests::typed_identity_plan_keeps_dialog_group_dependent_on_dialog_scan ... ok
test sources::legacy_metadata_cleanup::tests::clear_nulls_only_eligible_legacy_telegram_metadata ... ok
test sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_channel_stored_peer_when_access_hash_exists ... ok
test sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_supergroup_stored_peer_when_access_hash_exists ... ok
test sources::peer_resolution::tests::typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists ... ok
test sources::peer_resolution::tests::typed_identity_plan_skips_unusable_stored_peer_when_access_hash_is_missing ... ok
test sources::settings::tests::validate_sync_settings_rejects_out_of_range_values ... ok
test sources::settings::tests::initial_sync_policy_label_formats_messages_and_days ... ok
test sources::store::tests::avatar_cache_key_skips_non_telegram_metadata ... ok
test sources::settings::tests::sync_settings_default_when_app_settings_are_missing ... ok
test sources::settings::tests::sync_settings_roundtrip_through_app_settings ... ok
test sources::store::tests::delete_source_is_blocked_when_source_is_used_by_project ... ok
test sources::store::tests::dialog_picked_channel_writes_dialog_typed_identity_with_access_hash ... ok
test sources::store::tests::dialog_picked_group_writes_dialog_dependent_typed_identity_without_access_hash ... ok
test sources::store::tests::dialog_picked_supergroup_writes_dialog_typed_identity_with_access_hash ... ok
test sources::store::tests::list_sources_exposes_migrated_history_counts_without_old_chat_identity ... ok
test sources::store::tests::list_sources_exposes_sanitized_migrated_history_status_without_chat_id ... ok
test sources::store::tests::source_record_parts_allow_non_telegram_source ... ok
test sources::store::tests::load_source_returns_not_found_for_missing_source ... ok
test sources::store::tests::source_record_parts_emit_only_source_subtype ... ok
test sources::store::tests::telegram_identity_rejects_same_account_peer_conflict_at_typed_boundary ... ok
test sources::store::tests::telegram_identity_allows_same_peer_on_different_accounts ... ok
test sources::store::tests::telegram_source_upsert_inserts_null_metadata ... ok
test sources::store::tests::telegram_source_upsert_preserves_existing_legacy_metadata_blob ... ok
test sources::store::tests::telegram_source_upsert_rolls_back_source_when_typed_identity_fails ... ok
test sources::store::tests::telegram_source_upsert_writes_required_identity_and_available_optional_fields ... ok
test sources::store::tests::upsert_youtube_playlist_source_handles_legacy_not_null_telegram_kind ... ok
test sources::store::tests::upsert_youtube_playlist_source_writes_typed_row_and_null_source_metadata ... ok
test sources::store::tests::upsert_youtube_video_source_handles_legacy_not_null_telegram_kind ... ok
test sources::store::tests::upsert_youtube_video_source_conflict_clears_existing_legacy_blob ... ok
test sources::store::tests::upsert_youtube_video_source_rejects_invalid_canonical_url_without_source_row ... ok
test sources::sync::tests::determine_sync_policy_only_applies_initial_settings_on_first_sync ... ok
test sources::sync::tests::finalize_sync_preserves_existing_legacy_metadata_blob ... ok
test sources::store::tests::upsert_youtube_video_source_writes_typed_row_and_null_source_metadata ... ok
test sources::sync::tests::sync_provider_accepts_telegram_sources ... ok
test sources::sync::tests::sync_provider_rejects_manual_youtube_video_sources ... ok
test sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error ... ok
test sources::sync::tests::finalize_sync_updates_source_state_and_typed_avatar_cache ... ok
test sources::store::tests::delete_source_waits_for_temporary_database_write_lock ... ok
test sources::store::tests::delete_source_from_pool_enables_foreign_keys_and_cascades_dependents ... ok
test sources::topics::tests::forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists ... ok
test sources::test_support::tests::source_fixture_creates_expected_tables ... ok
test sources::topics::tests::forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind ... ok
test sources::types::tests::item_kind_constants_match_persisted_wire_values ... ok
test sources::types::tests::source_type_serializes_supported_provider_values ... ok
test sources::types::tests::telegram_source_subtype_parses_from_canonical_source_subtype ... ok
test sources::types::tests::telegram_source_subtype_parses_supported_values ... ok
test sources::types::tests::telegram_source_subtype_rejects_unknown_values_as_validation ... ok
test sources::topics::tests::topic_refresh_rebuilds_materialized_memberships ... ok
test sources::types::tests::telegram_source_subtype_rejects_unsupported_source_subtype ... ok
test sources::types::tests::telegram_source_subtype_serializes_as_existing_wire_value ... ok
test takeout_import::forum_topics::tests::completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups ... ok
test sql_helpers::tests::push_i64_bind_list_binds_values_in_order ... ok
test sources::topics::tests::upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted ... ok
test sources::topics::tests::list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket ... ok
test takeout_import::migrated_history::tests::capability_unavailable_keeps_reason_internal_and_clears_chat_hint ... ok
test takeout_import::migrated_history::tests::capability_available_is_source_level_and_restart_safe ... ok
test takeout_import::migrated_history::tests::migrated_history_errors_are_typed_for_frontend_behavior ... ok
test takeout_import::migrated_history::tests::migrated_small_group_identity_uses_native_old_chat_scope ... ok
test takeout_import::migrated_history::tests::validation_accepts_matching_revalidated_chat_id ... ok
test takeout_import::migrated_history::tests::validation_rejects_missing_or_changed_revalidated_chat_id ... ok
test takeout_import::forum_topics::tests::takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize ... ok
test takeout_import::forum_topics::tests::takeout_forum_topic_refresh_success_records_no_warning ... ok
test takeout_import::recovery::tests::takeout_recovery_ignores_non_takeout_batches ... ok
test takeout_import::recovery::tests::recovery_state_includes_migrated_history_scope_for_historical_batches ... ok
test takeout_import::recovery::tests::takeout_recovery_latest_complete_hides_older_failed ... ok
test takeout_import::recovery::tests::takeout_recovery_latest_failed_wins_over_older_complete ... ok
test takeout_import::recovery::tests::takeout_recovery_returns_partial_completed_and_hides_complete ... ok
test takeout_import::recovery::tests::takeout_recovery_running_with_active_job_is_hidden ... ok
test takeout_import::state::tests::active_jobs_for_sources_filters_non_terminal_jobs ... ok
test takeout_import::state::tests::job_state_can_cancel_and_finish_job ... ok
test takeout_import::state::tests::job_state_cancels_child_tokens ... ok
test takeout_import::state::tests::job_state_records_history_scope_for_frontend_labels ... ok
test takeout_import::state::tests::job_state_rejects_duplicate_active_source_jobs ... ok
test takeout_import::recovery::tests::takeout_recovery_running_without_active_job_is_interrupted ... ok
test takeout_import::state::tests::takeout_cancellation_smoke_fixture_finishes_cancelled_and_clears ... ok
test takeout_import::state::tests::takeout_cancellation_smoke_fixture_tracks_running_job ... ok
test takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact ... ok
test takeout_import::recovery::tests::takeout_recovery_source_filter_limits_results ... ok
test takeout_import::recovery::tests::takeout_recovery_warning_codes_are_unique_sorted_and_message_free ... ok
test takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation ... ok
test takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues ... ok
test takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize ... ok
test takeout_import::tests::migrated_history_detected_warning_is_sanitized ... ok
test takeout_import::tests::historical_batch_completion_does_not_advance_source_watermark ... ok
test takeout_import::tests::locked_start_allows_only_one_batch_for_same_source ... ok
test takeout_import::tests::locked_start_conflict_creates_no_provenance_rows ... ok
test takeout_import::tests::migrated_history_start_records_use_same_source_takeout_lock ... ok
test takeout_import::tests::migrated_history_start_requires_available_capability ... ok
test takeout_import::tests::takeout_step_cancel_wrapper_allows_completed_future ... ok
test takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future ... ok
test takeout_import::tests::takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists ... ok
test takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once ... ok
test takeout_import::tests::takeout_subtype_load_uses_typed_identity_not_legacy_kind ... ok
test takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_batch_summary_is_durable_and_sanitized ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_duplicate_after_normal_sync_summarizes_outcomes ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_caps_samples_deterministically ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_snapshot_delta_uses_explicit_snapshots ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_compares_batch_to_canonical_without_content ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_matched_observations_for_aggregates ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_missing_observations_by_identity ... ok
test telegram::tests::diagnostic_status_counts_do_not_return_account_ids_or_messages ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_source_snapshot_is_aggregate_and_sanitized ... ok
test telegram::tests::legacy_api_hash_migrates_to_secret_store_and_blanks_column ... ok
test telegram::tests::legacy_api_hash_remains_when_secret_write_fails ... ok
test telegram::tests::runtime_status_maps_to_existing_wire_strings ... ok
test telegram::tests::telegram_api_id_out_of_range_returns_typed_validation_error ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_excludes_non_latest_recovery_candidates ... ok
test telegram::tests::telegram_status_and_event_payload_contract_is_exact ... ok
test telegram::tests::missing_secure_api_hash_for_blank_legacy_account_is_auth_error ... ok
test telegram_session_store::tests::delete_session_from_path_removes_file_and_key ... ok
test telegram_session_store::tests::encrypted_session_load_fails_when_key_is_missing ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_is_durable_only ... ok
test telegram_session_store::tests::legacy_plaintext_session_remains_when_keyring_write_fails ... ok
test telegram_session_store::tests::legacy_plaintext_session_migrates_to_encrypted_file ... ok
test telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact ... ok
test topic_memberships::tests::rebuild_matches_retained_hidden_and_deleted_topics ... ok
test tx::tests::begin_immediate_commit_persists_changes ... ok
test topic_memberships::tests::rebuild_replaces_stale_memberships_and_versions ... ok
test topic_memberships::tests::rebuild_uses_legacy_root_only_without_typed_child ... ok
test topic_memberships::tests::rebuild_prioritizes_specific_topic_matches_before_general_fallback ... ok
test tx::tests::begin_immediate_rollback_discards_changes ... ok
test tx::tests::finish_manual_transaction_commits_success_result ... ok
test tx::tests::begin_immediate_with_foreign_keys_enforces_cascade ... ok
test tx::tests::sqlite_ignores_foreign_keys_pragma_inside_open_transaction ... ok
test youtube::captions::tests::caption_download_args_request_json3_and_vtt_without_media ... ok
test youtube::captions::tests::caption_selection_honors_explicit_override_before_original_language ... ok
test tx::tests::finish_manual_transaction_rolls_back_error_result ... ok
test youtube::captions::tests::caption_selection_prefers_original_then_preferred_then_english_then_any ... ok
test youtube::captions::tests::json3_parser_allows_missing_duration ... ok
test youtube::captions::tests::json3_parser_concatenates_segments_and_preserves_timing ... ok
test youtube::captions::tests::replace_transcript_segments_deletes_previous_rows_and_inserts_current_segments ... ok
test youtube::captions::tests::transcript_external_id_includes_language_and_track_kind ... ok
test youtube::captions::tests::vtt_parser_reads_cues_and_skips_blank_text ... ok
test youtube::captions::tests::vtt_parser_rejects_invalid_timing ... ok
test youtube::comments::tests::comment_published_at_accepts_numbers_strings_and_fallback ... ok
test youtube::comments::tests::comments_fetch_args_include_bounded_extractor_args ... ok
test youtube::comments::tests::comments_fetch_timeout_is_longer_than_metadata_preview_timeout ... ok
test youtube::comments::tests::default_comment_limit_is_bounded ... ok
test youtube::comments::tests::normalize_comments_flattens_replies_and_warns_for_timestamp_fallbacks ... ok
test youtube::comments::tests::normalize_comments_truncates_raw_comment_array_before_normalization ... ok
test youtube::cookies::tests::accepts_empty_cookie_values ... ok
test youtube::cookies::tests::accepts_http_only_cookie_rows ... ok
test youtube::cookies::tests::rejects_empty_cookie_text ... ok
test youtube::cookies::tests::rejects_files_without_cookie_rows ... ok
test youtube::cookies::tests::rejects_invalid_cookie_text_before_saving_secret ... ok
test youtube::captions::tests::replace_transcript_segments_rebuilds_analysis_documents_by_segment_order ... ok
test youtube::cookies::tests::validates_netscape_cookie_rows_without_exposing_values ... ok
test youtube::cookies::tests::stores_reads_and_clears_youtube_cookies_through_secret_store ... ok
test youtube::detail::tests::list_summaries_uses_source_id_order_and_marks_no_captions_unavailable ... ok
test youtube::detail::tests::playlist_detail_reports_ordered_items_and_summary_counts ... ok
test youtube::detail::tests::source_summary_missing_typed_metadata_uses_generic_title_without_blob_decode ... ok
test youtube::detail::tests::playlist_detail_uses_typed_linked_video_metadata_with_corrupt_source_blob ... ok
test tx::tests::deferred_read_then_write_hits_busy_snapshot_under_concurrent_writer ... ok
test youtube::detail::tests::summaries_use_typed_video_metadata_with_corrupt_source_blob ... ok
test youtube::detail::tests::video_detail_includes_safe_source_metadata_without_item_raw_payloads ... ok
test tx::tests::begin_immediate_read_then_write_survives_concurrent_writer ... ok
test youtube::dto::tests::availability_status_serializes_as_snake_case ... ok
test youtube::dto::tests::preview_kind_deserializes_snake_case ... ok
test youtube::dto::tests::video_form_serializes_short_value ... ok
test youtube::errors::tests::invalid_youtube_url_maps_to_validation_error ... ok
test youtube::detail::tests::video_detail_missing_typed_metadata_returns_controlled_error ... ok
test youtube::errors::tests::ytdlp_deleted_failures_map_to_not_found_error ... ok
test youtube::errors::tests::ytdlp_network_failures_map_to_network_error ... ok
test youtube::errors::tests::ytdlp_private_failures_map_to_auth_error ... ok
test youtube::jobs::tests::catalog_jobs_for_sources_includes_latest_failed_jobs ... ok
test youtube::jobs::tests::active_jobs_for_sources_filters_non_terminal_direct_and_related_sources ... ok
test youtube::jobs::tests::diagnostic_counts_group_source_jobs_without_ids_or_raw_errors ... ok
test youtube::jobs::tests::job_state_cancels_child_tokens ... ok
test youtube::jobs::tests::job_state_finishes_cancel_requested_jobs_as_cancelled ... ok
test youtube::jobs::tests::job_state_list_filters_before_limit_and_sorts_newest_first ... ok
test youtube::jobs::tests::job_state_rejects_duplicate_active_scope_but_allows_different_job_types ... ok
test youtube::jobs::tests::jobs_missing_typed_video_metadata_errors_after_failed_refresh ... ok
test youtube::jobs::tests::source_job_cancellation_smoke_fixture_finishes_cancelled_and_clears ... ok
test youtube::jobs::tests::source_job_cancellation_smoke_fixture_tracks_running_job ... ok
test youtube::jobs::tests::source_job_step_with_process_cancel_allows_completed_future ... ok
test youtube::jobs::tests::retryable_playlist_video_rows_excludes_auth_deleted_and_removed_entries ... ok
test youtube::jobs::tests::source_job_step_with_process_cancel_interrupts_pending_future ... ok
test youtube::jobs::tests::source_job_type_uses_comments_specific_type_for_comments_only_video_sync ... ok
test youtube::jobs::tests::jobs_reload_missing_typed_video_metadata_after_refresh_callback ... ok
test youtube::jobs::tests::source_job_workflow_file_has_no_tauri_command_adapters ... ok
test youtube::detail::tests::video_detail_reports_synced_transcript_comments_and_playlist_memberships ... ok
test youtube::metadata::tests::availability_values_map_to_statuses ... ok
test youtube::jobs::tests::source_jobs_no_longer_decode_source_metadata_blobs ... ok
test youtube::metadata::tests::video_fixture_maps_metadata_and_preview_fields ... ok
test youtube::metadata::tests::playlist_metadata_page_args_use_adjacent_playlist_range ... ok
test youtube::metadata::tests::playlist_fixture_maps_metadata_entries_and_preview_warning ... ok
test youtube::metadata::tests::video_fixture_missing_optional_fields_maps_to_none ... ok
test youtube::playlist::tests::playlist_item_video_source_upsert_writes_typed_video_metadata_not_source_blob ... ok
test youtube::playlist::tests::upsert_playlist_items_can_skip_video_source_materialization ... ok
test youtube::playlist::tests::upsert_playlist_items_marks_missing_rows_removed ... ok
test youtube::preview::tests::preview_from_playlist_json_returns_playlist_preview ... ok
test youtube::playlist::tests::upsert_playlist_items_reuses_existing_video_source_and_keeps_unavailable_null ... ok
test youtube::preview::tests::preview_from_video_json_uses_parsed_url_kind ... ok
test youtube::process_runtime::tests::cancellation_reaches_all_reserved_operations ... ok
test youtube::process_runtime::tests::cookie_guard_retains_file_until_detached_reaper_finishes ... ok
test youtube::process_runtime::tests::detached_reaper_keeps_cookie_until_the_stuck_child_releases ... ok
test youtube::playlist::tests::upsert_playlist_items_without_materialization_reuses_existing_video_source ... ok
test youtube::process_runtime::tests::dropped_caller_keeps_child_and_registry_owned_until_shutdown_reaps_it ... ok
test youtube::process_runtime::tests::finite_pipe_backpressure_requires_concurrent_drain ... ok
test youtube::process_runtime::tests::external_source_job_cancellation_reaps_its_managed_operation ... ok
test youtube::process_runtime::tests::injected_launcher_drains_backpressured_output_before_waiting_for_exit ... ok
test youtube::process_runtime::tests::registry_reserves_an_operation_before_spawn ... ok
test youtube::process_runtime::tests::injected_wait_error_reaps_the_child_before_releasing_registry ... ok
test youtube::process_runtime::tests::shutdown_rejects_new_ytdlp_admission_before_spawn ... ok
test youtube::process_runtime::tests::spawn_failure_rolls_back_the_registry_reservation ... ok
test youtube::runtime::tests::runtime_status_serializes_with_camel_case_keys ... ok
test youtube::process_runtime::tests::timeout_fallback_detaches_cookie_until_stuck_child_reaps ... ok
test youtube::settings::tests::invalid_stored_settings_return_validation_error_with_key ... ok
test youtube::settings::tests::invalid_youtube_settings_do_not_write_partial_values ... ok
test youtube::settings::tests::auth_cookies_load_only_when_auth_is_enabled ... ok
test youtube::settings::tests::validate_youtube_settings_normalizes_preferred_captions_language ... ok
test youtube::settings::tests::validate_youtube_settings_rejects_out_of_range_values ... ok
test youtube::settings::tests::saving_cookies_enables_auth_and_clear_disables_it ... ok
test youtube::settings::tests::youtube_settings_default_when_app_settings_are_missing ... ok
test youtube::settings::tests::youtube_settings_serializes_with_camel_case_keys ... ok
test youtube::source_metadata::tests::playlist_metadata_columns_are_versioned_and_secret_safe ... ok
test youtube::process_runtime::tests::injected_nonzero_exit_preserves_not_found_classification_and_releases_registry ... ok
test youtube::source_metadata::tests::video_metadata_columns_include_wire_values_arrays_caption_override_and_sanitized_raw ... ok
test youtube::source_metadata::tests::video_metadata_rejects_wrong_canonical_url_shape ... ok
test youtube::settings::tests::youtube_settings_roundtrip_through_app_settings ... ok
test youtube::thumbnail::tests::accepts_only_allowlisted_https_thumbnail_urls ... ok
test youtube::source_metadata::tests::video_source_metadata_restores_raw_caption_metadata_for_provider_sync ... ok
test youtube::thumbnail::tests::bounds_thumbnail_responses_to_one_mib ... ok
test youtube::thumbnail::tests::recognizes_supported_image_magic_bytes ... ok
test youtube::thumbnail::tests::builds_the_dedicated_thumbnail_client ... ok
test youtube::transcript_reader::tests::list_youtube_transcript_segments_can_start_at_selected_time ... ok
test youtube::transcript_reader::tests::list_youtube_transcript_segments_filters_by_search ... ok
test youtube::transcript_reader::tests::search_escapes_existing_backslashes_before_like_wildcards ... ok
test youtube::url::tests::parses_live_url ... ok
test youtube::url::tests::parses_playlist_url ... ok
test youtube::url::tests::parses_short_youtu_be_url ... ok
test youtube::transcript_reader::tests::list_youtube_transcript_segments_pages_by_time_and_id ... ok
test youtube::url::tests::parses_shorts_url ... ok
test youtube::url::tests::parses_watch_video_url ... ok
test youtube::url::tests::rejects_empty_input ... ok
test youtube::url::tests::rejects_invalid_host ... ok
test youtube::url::tests::watch_url_with_playlist_parameter_parses_selected_video ... ok
test youtube::ytdlp::tests::authenticated_command_args_include_cookie_file_path_without_cookie_content ... ok
test youtube::ytdlp::tests::cookie_file_content_adds_netscape_header_when_missing ... ok
test youtube::ytdlp::tests::cookie_file_content_preserves_existing_netscape_header ... ok
test youtube::ytdlp::tests::preview_playlist_args_limit_entries_to_first_fifty ... ok
test youtube::ytdlp::tests::preview_video_args_use_dump_json_without_shell_fragments ... ok
test youtube::source_metadata::tests::upsert_video_metadata_maintains_description_document ... ok
test youtube::process_runtime::tests::injected_timeout_reap_detaches_stuck_child_and_keeps_cookie_until_release ... ok

test result: ok. 665 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.42s

     Running unittests src\main.rs (src-tauri\target\debug\deps\extractum-4b630ead96220393.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

All four app-owned fixture consumers selected and passed exactly one test:

#### fixture-green-takeout_import__tests__takeout_step_cancel_wrapper_interrupts_pending_future.txt

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At C:\Users\Dima\AppData\Local\Temp\extractum-telegram-8c-05c4157a39374e788a3dbd2b3b91ee46\helpers.ps1:90 char:9
+         cargo test --color never --manifest-path src-tauri/Cargo.toml ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `extractum-prompt-packs` (lib) generated 1 warning
   Compiling extractum-telegram v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-telegram)
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
   Compiling extractum v0.2.0 (G:\Develop\Extractum\src-tauri)
warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib test) generated 14 warnings (run `cargo fix --lib -p extractum --tests` to apply 5 suggestion
s)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 03s
     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_lib-51295d1508d3c6a6.exe)

running 1 test
test takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 664 filtered out; finished in 0.01s
```

#### fixture-green-takeout_import__tests__channel_private_count_probe_records_fallback_before_search_continuation.txt

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At C:\Users\Dima\AppData\Local\Temp\extractum-telegram-8c-05c4157a39374e788a3dbd2b3b91ee46\helpers.ps1:90 char:9
+         cargo test --color never --manifest-path src-tauri/Cargo.toml ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `extractum-prompt-packs` (lib) generated 1 warning
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib test) generated 14 warnings (run `cargo fix --lib -p extractum --tests` to apply 5 suggestion
s)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.17s
     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_lib-51295d1508d3c6a6.exe)

running 1 test
test takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 664 filtered out; finished in 0.13s
```

#### fixture-green-takeout_import__tests__export_dc_fallback_provenance_records_once_before_finalize.txt

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At C:\Users\Dima\AppData\Local\Temp\extractum-telegram-8c-05c4157a39374e788a3dbd2b3b91ee46\helpers.ps1:90 char:9
+         cargo test --color never --manifest-path src-tauri/Cargo.toml ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-prompt-packs` (lib) generated 1 warning
warning: `extractum-telegram` (lib) generated 9 warnings
warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib test) generated 14 warnings (run `cargo fix --lib -p extractum --tests` to apply 5 suggestion
s)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.12s
     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_lib-51295d1508d3c6a6.exe)

running 1 test
test takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 664 filtered out; finished in 0.02s
```

#### fixture-green-takeout_import__tests__channel_private_validation_preflight_records_fallback_and_continues.txt

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At C:\Users\Dima\AppData\Local\Temp\extractum-telegram-8c-05c4157a39374e788a3dbd2b3b91ee46\helpers.ps1:90 char:9
+         cargo test --color never --manifest-path src-tauri/Cargo.toml ...
+         ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-prompt-packs` (lib) generated 1 warning
warning: `extractum-telegram` (lib) generated 9 warnings
warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib test) generated 14 warnings (run `cargo fix --lib -p extractum --tests` to apply 5 suggestion
s)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.07s
     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_lib-51295d1508d3c6a6.exe)

running 1 test
test takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 664 filtered out; finished in 0.01s
```

## Exact Rust test identity sets

The comparison is exact set equality: `736 BASE = 665 app + 71 producer`, and
prefixing each producer identity with `telegram_impl::` recreates the complete
736-entry BASE set.

### 736 pre-edit BASE identities

```text
account_deletion::tests::active_direct_source_analysis_run_blocks_owned_source_only
account_deletion::tests::active_group_analysis_run_blocks_when_any_member_source_is_owned
account_deletion::tests::active_llm_request_for_completed_owned_run_blocks_but_provider_test_does_not
account_deletion::tests::active_source_job_on_owned_source_blocks_but_unowned_job_does_not
account_deletion::tests::active_takeout_job_on_owned_source_blocks
account_deletion::tests::blocker_collection_keeps_multiple_categories_for_internal_diagnostics
account_deletion::tests::existing_account_with_zero_sources_passes
account_deletion::tests::missing_account_returns_not_found
account_deletion::tests::source_ingest_lock_on_owned_source_blocks_without_deleting_rows
accounts::tests::creating_account_rolls_back_when_secret_write_fails
accounts::tests::creating_account_writes_api_hash_to_secret_store_only
accounts::tests::deleting_account_removes_secret_after_database_row
accounts::tests::deleting_missing_account_returns_not_found
accounts::tests::secret_cleanup_failure_keeps_deleted_database_row_deleted
analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion
analysis::corpus::tests::live::default_analysis_corpus_excludes_migrated_history_documents
analysis::corpus::tests::live::description_mode_creates_synthetic_description_message
analysis::corpus::tests::live::explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus
analysis::corpus::tests::live::live_corpus_refs_use_local_item_ids
analysis::corpus::tests::live::load_corpus_messages_filters_telegram_to_telegram_message
analysis::corpus::tests::live::load_corpus_messages_filters_youtube_transcript_only_to_transcripts
analysis::corpus::tests::live::load_corpus_messages_includes_youtube_comment_only_in_comments_mode
analysis::corpus::tests::live::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref
analysis::corpus::tests::live::load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content
analysis::corpus::tests::live::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
analysis::corpus::tests::live::preflight_ref_format_matches_corpus_loader_ref_format
analysis::corpus::tests::live::source_group_opt_in_includes_only_members_with_migrated_rows
analysis::corpus::tests::live::youtube_description_missing_typed_metadata_skips_without_decoding_source_blob
analysis::corpus::tests::live::youtube_description_rows_use_typed_metadata_with_corrupt_source_blob
analysis::corpus::tests::live::youtube_transcript_segment_evidence_uses_typed_source_context
analysis::corpus::tests::preflight::preflight_count_matches_loader_for_youtube_corpus_modes
analysis::corpus::tests::preflight::preflight_counts_eligible_text_messages_for_sources
analysis::corpus::tests::preflight::preflight_ignores_media_only_items_without_text_content
analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows
analysis::corpus::tests::source_resolution::resolve_analysis_sources_loads_single_provider_project
analysis::corpus::tests::source_resolution::resolve_analysis_sources_preserves_no_linked_youtube_error_message
analysis::corpus::tests::source_resolution::resolve_analysis_sources_rejects_mixed_provider_project
analysis::corpus::tests::source_resolution::resolve_run_source_ids_loads_project_sources_without_snapshot
analysis::fixtures::tests::active_runs::fixture_active_state_tracks_seeded_running_run
analysis::fixtures::tests::active_runs::fixture_cancel_waiter_marks_running_run_cancelled
analysis::fixtures::tests::clear::clear_deletes_child_rows_through_fixture_parent_ids
analysis::fixtures::tests::clear::clear_preserves_non_fixture_groups_and_members
analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent
analysis::fixtures::tests::harness::fixture_test_pool_has_required_tables
analysis::fixtures::tests::seed::compressed_fixture_fields_are_readable
analysis::fixtures::tests::seed::seed_creates_fixture_runs_with_statuses_templates_and_snapshots
analysis::fixtures::tests::seed::seed_creates_post_sync_reader_content
analysis::fixtures::tests::seed::seed_creates_safe_account_prompt_profile_sources_and_group
analysis::fixtures::tests::seed::seed_creates_sources_that_pass_identity_repair
analysis::fixtures::tests::seed::seed_creates_valid_typed_youtube_detail_metadata
analysis::fixtures::tests::seed::seed_twice_keeps_one_deterministic_fixture_set
analysis::fixtures::tests::snapshot::capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report
analysis::fixtures::tests::snapshot::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot
analysis::fixtures::tests::snapshot::missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages
analysis::fixtures::tests::snapshot::seeded_snapshot_runs_expose_captured_snapshot_state
analysis::fixtures::tests::summary::summary_serializes_with_camel_case_keys
analysis::groups::tests::prepare_analysis_source_group_input_preserves_baseline_error_precedence
analysis::groups::tests::validate_group_source_type_accepts_matching_provider_membership
analysis::groups::tests::validate_group_source_type_rejects_mixed_provider_membership
analysis::groups::tests::validate_group_source_type_rejects_unknown_group_type
analysis::report::tests::capture::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
analysis::store::tests::read_model::list_analysis_run_summaries_filters_project_runs
analysis::store::tests::read_model::list_analysis_run_summaries_matches_all_query_terms_across_any_field
analysis::store::tests::setup::ensure_sources_exist_returns_typed_not_found_error
analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit
analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects
analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot
analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id
analysis::tests_application::report_profile_resolution_failure_prevents_run_creation
analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads
analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels
analysis_documents::tests::document_metadata_envelopes_match_current_evidence_shape
analysis_documents::tests::rebuild_analysis_documents_excludes_migrated_history_rows
analysis_documents::tests::rebuild_source_materializes_text_units_with_document_order
analysis_documents::tests::rebuild_source_removes_stale_documents_and_is_idempotent
analysis_documents::tests::schema_creates_analysis_documents_constraints_and_indexes
apalis_jobs::tests::apalis_jobs_counts_ignore_their_own_active_filter
apalis_jobs::tests::apalis_jobs_decode_failure_returns_redacted_preview_without_json
apalis_jobs::tests::apalis_jobs_limit_excludes_large_payloads_outside_limited_rows
apalis_jobs::tests::apalis_jobs_list_clamps_limit
apalis_jobs::tests::apalis_jobs_list_does_not_mutate_jobs
apalis_jobs::tests::apalis_jobs_list_filters_by_status_job_type_and_search
apalis_jobs::tests::apalis_jobs_list_returns_empty_when_jobs_table_missing
apalis_jobs::tests::apalis_jobs_list_returns_rfc3339_utc_timestamps
apalis_jobs::tests::apalis_jobs_list_returns_rows_from_jobs_table
apalis_jobs::tests::apalis_jobs_list_sorts_by_latest_activity_timestamp
apalis_jobs::tests::apalis_jobs_non_json_result_and_metadata_are_omitted_in_v1
apalis_jobs::tests::apalis_jobs_payloads_are_redacted_and_truncated
apalis_jobs::tests::apalis_jobs_prune_terminal_deletes_only_old_done_killed_and_terminal_failed_jobs
apalis_jobs::tests::apalis_jobs_prune_terminal_returns_zero_when_jobs_table_missing
apalis_jobs::tests::apalis_jobs_row_shape_is_stable_when_optional_columns_are_absent
apalis_jobs::tests::apalis_jobs_schema_probe_documents_local_jobs_table_shape
archive_read_model::tests::create_schema_adds_state_and_item_tables
archive_read_model::tests::current_ready_state_rejects_old_model_version
archive_read_model::tests::rebuild_source_excludes_migrated_history_rows
archive_read_model::tests::rebuild_source_materializes_archive_fidelity_fields
child_process::tests::create_no_window_matches_win32_process_creation_flags
diagnostics::database::tests::database_diagnostics_groups_only_allow_listed_aggregates
diagnostics::database::tests::migration_status_reports_pending_and_failed_versions
diagnostics::dto::tests::diagnostic_summary_fixture_serializes_without_forbidden_sentinels
diagnostics::redaction::tests::redact_json_value_redacts_sensitive_keys_recursively
diagnostics::redaction::tests::redact_text_removes_secret_and_content_patterns
diagnostics::redaction::tests::sanitized_error_message_bounds_unicode_by_chars
diagnostics::redaction::tests::sanitized_error_message_is_bounded
diagnostics::runtime::tests::failed_runtime_check_uses_coarse_summary_without_os_error_text
diagnostics::runtime::tests::secure_storage_failure_does_not_expose_store_error_text
diagnostics::tests::sanitize_diagnostic_error_bounds_and_redacts_command_errors
diagnostics::tests::serialized_diagnostic_summary_preserves_allowed_data_and_excludes_forbidden_data
external_process::tests::admission_wait_consumes_the_shared_graceful_budget
external_process::tests::cleanup_tasks_start_concurrently_and_isolate_error_and_panic
external_process::tests::concurrent_watchdogs_invoke_exit_once
external_process::tests::exhausted_admission_budget_skips_the_cleanup_factory
external_process::tests::injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback
external_process::tests::permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown
external_process::tests::permits_acquired_before_shutdown_are_waited_for
external_process::tests::repeated_start_does_not_replace_code_or_schedule_again
external_process::tests::start_reports_completed_after_watchdog_claims_exit
external_process::tests::start_returns_started_and_schedules_one_watchdog
external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets
external_process::tests::watchdog_exits_with_the_preserved_code_unless_cleanup_completed
gemini_browser::cdp_chrome::tests::drop_falls_back_to_owned_child_shutdown
gemini_browser::cdp_chrome::tests::explicit_shutdown_kills_and_reaps_the_owned_child_once
gemini_browser::cdp_chrome::tests::shutdown_does_not_claim_or_kill_an_already_exited_child
gemini_browser::cdp_chrome::tests::shutdown_reaps_when_the_child_has_already_exited_during_kill
gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_accepts_json_version_response
gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_reports_unreachable_endpoint
gemini_browser::executor::tests::cancelled_run_marks_the_sidecar_transport_tainted
gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json
gemini_browser::jobs::app_tests::apalis_sqlite_status_probe_documents_actual_status_values
gemini_browser::jobs::app_tests::apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job
gemini_browser::jobs::app_tests::apalis_storage_preserves_existing_sqlx_migration_history_table
gemini_browser::jobs::app_tests::apalis_storage_shares_extractum_db_without_locking_app_pool
gemini_browser::jobs::app_tests::apalis_storage_uses_shared_main_extractum_db_identity
gemini_browser::jobs::app_tests::enqueue_duplicate_run_id_returns_conflict
gemini_browser::jobs::app_tests::enqueue_persists_job_before_worker_startup
gemini_browser::jobs::app_tests::failed_gemini_browser_job_is_not_retried
gemini_browser::jobs::app_tests::gemini_browser_jobs_are_built_with_one_total_attempt
gemini_browser::jobs::app_tests::restart_worker_processes_pending_job_after_runtime_restart
gemini_browser::jobs::app_tests::worker_picks_up_job_quickly_after_idle
gemini_browser::sidecar::tests::stderr_drain_consumes_sidecar_output_concurrently
ingest_provenance::tests::completed_zero_observation_batch_is_complete_without_partial_flags
ingest_provenance::tests::create_takeout_batch_inserts_generic_and_detail_rows_atomically
ingest_provenance::tests::migrated_history_deferred_scope_finalizes_partial_and_records_warning_once
ingest_provenance::tests::migrated_small_group_imported_allows_duplicate_only_success
ingest_provenance::tests::migrated_small_group_scope_can_be_marked_running_and_completed
ingest_provenance::tests::mixed_partial_scope_finalizes_as_partial
ingest_provenance::tests::terminal_update_recalculates_counters_and_sanitizes_error
job_helpers::tests::active_job_guards_track_and_release_scoped_jobs
job_helpers::tests::cancellation_state_cancels_child_tokens
job_helpers::tests::cancellation_state_marks_checks_and_clears_jobs
library_sources::tests::catalog_status_for_input_keeps_failed_job_without_detail_empty
library_sources::tests::list_library_catalog_returns_status_capabilities_and_filter_counts
library_sources::tests::list_library_sources_keeps_sources_with_missing_provider_details
library_sources::tests::list_library_sources_returns_youtube_and_telegram_metadata
llm::profiles::tests::active_profile_resolution_loads_key_from_secret_store
llm::profiles::tests::changing_key_scope_without_replacement_is_rejected
llm::profiles::tests::clear_profile_api_key_deletes_secret
llm::profiles::tests::credential_scope_uses_provider_origin_and_effective_port_but_not_path
llm::profiles::tests::delete_profile_fails_if_secret_store_fails_leaving_db_settings_intact
llm::profiles::tests::delete_profile_removes_settings_and_secret_and_resets_active
llm::profiles::tests::empty_save_preserves_existing_secret
llm::profiles::tests::keyed_legacy_profile_materializes_effective_base_url_while_unkeyed_stays_blank
llm::profiles::tests::legacy_remote_http_profile_is_rejected_before_request_configuration
llm::profiles::tests::materialization_write_failure_fails_closed_during_state_load
llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store
llm::profiles::tests::profile_state_lists_multiple_saved_profiles
llm::profiles::tests::provider_access_resolution_uses_configured_key_with_saved_base_url
llm::profiles::tests::provider_access_resolution_uses_saved_key_with_configured_base_url
llm::profiles::tests::set_active_profile_returns_typed_not_found_error
llm::profiles::tests::validate_profile_id_rejects_invalid_characters
llm::tests::llm_command_errors_and_failed_events_keep_distinct_json_shapes
llm::tests::llm_stream_events_serialize_exact_lifecycle_contract
llm::tests::provider_diagnostics_exclude_profile_ids_and_base_urls
migrations::baseline_reset::tests::backup_failure_prevents_migration_history_rewrite
migrations::baseline_reset::tests::classifies_baseline_history_after_post_baseline_migrations
migrations::baseline_reset::tests::classifies_baseline_history_only_when_checksum_matches
migrations::baseline_reset::tests::classifies_old_history_only_when_versions_one_through_twenty_six_are_successful
migrations::baseline_reset::tests::old_history_cutover_backs_up_then_rewrites_only_migration_history
migrations::baseline_reset::tests::rejects_baseline_history_with_wrong_checksum
migrations::baseline_reset::tests::rejects_failed_migration_history
migrations::baseline_reset::tests::rejects_partial_old_history_without_version_twenty_six
migrations::tests::analysis_telegram_history_scope_migration_adds_nullable_checked_column
migrations::tests::app_migrator_accepts_database_with_preexisting_apalis_migration_history
migrations::tests::build_migrations_includes_apalis_sqlite_versions_for_shared_sqlx_history
migrations::tests::build_migrations_includes_prompt_pack_runtime_provider_version_ten
migrations::tests::build_migrations_starts_at_current_schema_baseline
migrations::tests::concurrent_test_migrations_publish_complete_apalis_schemas
migrations::tests::current_schema_baseline_checksum_matches_frozen_reset_boundary
migrations::tests::current_schema_baseline_migration_is_version_one
migrations::tests::fresh_schema_includes_analysis_documents_table_indexes_and_constraints
migrations::tests::fresh_schema_includes_analysis_snapshot_markers
migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints
migrations::tests::fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints
migrations::tests::fresh_schema_includes_projects_redesign_columns_index_and_defaults
migrations::tests::fresh_schema_includes_source_identity_tables_after_sql_managed_migrations
migrations::tests::post_baseline_migration_upgrades_frozen_baseline_for_migrated_history
migrations::tests::prepare_database_skips_cutover_when_database_file_is_missing
migrations::tests::projects_mvp_migration_is_registered
migrations::tests::projects_mvp_schema_applies_to_memory_pool
migrations::tests::prompt_pack_mvp_migration_creates_library_and_run_tables
migrations::tests::prompt_pack_mvp_migration_declares_required_integrity_constraints
migrations::tests::test_migration_batch_rolls_back_schema_and_history_together
migrations::tests::vendored_apalis_sqlite_migrations_match_pinned_dependency
notebooklm_export::chunker::tests::accounts_for_document_overhead_when_splitting
notebooklm_export::chunker::tests::falls_back_to_month_when_year_exceeds_limits
notebooklm_export::chunker::tests::falls_back_to_topic_id_when_topic_title_slug_is_invalid
notebooklm_export::chunker::tests::filters_short_text_without_other_signal
notebooklm_export::chunker::tests::groups_chunks_by_topic_slug
notebooklm_export::chunker::tests::keeps_yearly_group_when_within_limits
notebooklm_export::chunker::tests::splits_by_word_and_byte_limits
notebooklm_export::filename::tests::accepts_safe_relative_child_paths
notebooklm_export::filename::tests::child_paths_stay_under_base
notebooklm_export::filename::tests::rejects_reserved_components
notebooklm_export::filename::tests::rejects_unsafe_relative_child_paths
notebooklm_export::filename::tests::sanitizes_unsafe_filename_parts
notebooklm_export::glossary::tests::aggregates_participants_by_author
notebooklm_export::links::tests::detects_and_trims_http_urls
notebooklm_export::media::tests::renders_numeric_only_media_metadata
notebooklm_export::media::tests::renders_useful_media_placeholder_parts
notebooklm_export::message_mapping::tests::reply_snippet_decode_failures_are_typed_internal_errors
notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_bounded_periods
notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_notebooklm_messages
notebooklm_export::query::tests::corrupt_archive_reply_target_outside_period_fails_archive_loader
notebooklm_export::query::tests::current_export_archive_loader_sets_scope_markers
notebooklm_export::query::tests::export_fixture_rejects_null_published_at_before_loader_parity
notebooklm_export::query::tests::load_export_messages_adds_local_reply_context_outside_period
notebooklm_export::query::tests::load_export_messages_attaches_general_topic_when_topic_header_is_missing
notebooklm_export::query::tests::load_export_messages_attaches_topic_metadata_for_reply_and_root_messages
notebooklm_export::query::tests::load_export_messages_does_not_root_match_non_numeric_external_ids
notebooklm_export::query::tests::load_export_messages_reads_materialized_topic_memberships
notebooklm_export::query::tests::load_export_source_group_exposes_youtube_group_for_hard_validation
notebooklm_export::query::tests::load_export_source_group_keeps_dirty_member_source_type_for_skip_logic
notebooklm_export::query::tests::load_export_source_group_orders_members_by_title_then_id
notebooklm_export::query::tests::load_export_source_rejects_non_telegram_before_message_loader_selection
notebooklm_export::query::tests::load_export_source_uses_canonical_subtype_not_legacy_kind
notebooklm_export::query::tests::migrated_export_reply_lookup_stays_inside_old_history_domain
notebooklm_export::query::tests::notebooklm_archive_export_excludes_migrated_history_rows_even_if_materialized
notebooklm_export::query::tests::notebooklm_default_export_excludes_migrated_history_rows
notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_all_fallback_reasons
notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_missing_and_old_version
notebooklm_export::query::tests::notebooklm_export_loader_selection_uses_archive_for_ready_current_state
notebooklm_export::query::tests::notebooklm_export_query_file_has_no_export_row_mapping
notebooklm_export::query::tests::notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails
notebooklm_export::query::tests::notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states
notebooklm_export::query::tests::notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection
notebooklm_export::query::tests::opted_in_export_loads_migrated_rows_separately_with_markers
notebooklm_export::renderer::tests::formats_metadata_as_rfc3339
notebooklm_export::renderer::tests::renders_json_compatible_yaml_string_scalars
notebooklm_export::renderer::tests::renders_message_metadata_and_text
notebooklm_export::renderer::tests::renders_migrated_history_scope_metadata
notebooklm_export::renderer::tests::renders_reply_thread_and_reaction_metadata
notebooklm_export::renderer::tests::renders_topic_aware_document_header
notebooklm_export::tests::formats_timestamp_folder_suffix
notebooklm_export::tests::group_member_manifest_records_source_scoped_generated_files
notebooklm_export::tests::keeps_migrated_history_opt_in_in_validated_config
notebooklm_export::tests::load_group_export_inputs_rejects_group_without_telegram_members
notebooklm_export::tests::load_group_export_inputs_rejects_youtube_group_for_hard_validation
notebooklm_export::tests::marker_read_and_write_accept_normal_file
notebooklm_export::tests::marker_read_and_write_reject_existing_symlink_file
notebooklm_export::tests::prefix_chunk_filename_adds_sources_directory_and_prefix
notebooklm_export::tests::reads_legacy_single_source_manifest_after_manifest_expansion
notebooklm_export::tests::remove_generated_files_rejects_invalid_manifest_relative_path
notebooklm_export::tests::remove_generated_files_rejects_symlink_parent_directory
notebooklm_export::tests::removes_generated_files_in_sources_subdirectory
notebooklm_export::tests::render_source_export_filters_messages_and_clears_media_placeholders
notebooklm_export::tests::render_source_export_tracks_empty_migrated_history_warning_and_section_prefix
notebooklm_export::tests::render_source_group_export_errors_when_all_members_empty_after_filters
notebooklm_export::tests::render_source_group_export_keeps_empty_member_skipped_reason_out_of_member_warnings
notebooklm_export::tests::source_member_file_prefix_includes_index_id_and_slug
notebooklm_export::tests::source_member_file_prefix_uses_fallback_slug_for_unsafe_title
notebooklm_export::tests::treats_blank_export_id_as_missing
notebooklm_export::tests::trims_optional_export_id
notebooklm_export::tests::validates_exactly_one_export_scope
notebooklm_export::tests::validates_period_order
notebooklm_export::tests::validates_single_source_scope
notebooklm_export::tests::validates_source_group_scope
notebooklm_export::tests::write_export_file_creates_sources_parent_directory
notebooklm_export::tests::write_export_file_rejects_symlink_parent_directory
notebooklm_export::tests::write_group_export_package_records_group_manifest_and_source_files
process_tree::tests::assigns_a_directly_owned_std_child
process_tree::tests::creates_a_job_object
process_tree::tests::dropping_the_guard_closes_the_job_and_kills_its_children
process_tree::tests::process_tree_guard_can_be_owned_by_async_application_state
process_tree::tests::terminate_failure_remains_reportable_and_retryable
process_tree::tests::terminate_is_idempotent
process_tree::tests::terminates_a_descendant_created_after_assignment
projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources
projects::data_range::tests::project_data_range_includes_telegram_migrated_history_when_requested
projects::data_range::tests::project_data_range_preserves_migrated_history_error_for_unknown_source_type
projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram
projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project
projects::data_range::tests::project_data_range_rejects_mixed_provider_project
projects::data_range::tests::project_data_range_returns_nulls_for_empty_project
projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project
projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds
projects::read_model::tests::list_research_projects_counts_playlist_linked_video_materials
projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout
projects::read_model::tests::list_research_projects_prioritizes_running_and_sorts_active_pinned_updated_first
projects::tests::add_project_sources_is_idempotent_and_lists_ui_ready_rows
projects::tests::create_project_trims_and_rejects_duplicate_names_case_insensitively
projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources
projects::tests::list_project_sources_counts_playlist_linked_video_materials
projects::tests::list_project_sources_includes_catalog_status_last_sync_and_handle
projects::tests::project_scoped_delete_blocks_other_active_and_archived_projects_without_mutation
projects::tests::project_scoped_delete_caps_blocking_projects_and_reports_remaining_count
projects::tests::project_scoped_delete_rejects_invalid_sources_and_missing_links
projects::tests::project_scoped_delete_removes_youtube_video_and_cascaded_materials
projects::tests::project_scoped_delete_schema_source_foreign_keys_are_delete_safe
projects::tests::set_project_archived_toggles_timestamp_and_rejects_missing_project
projects::tests::set_project_pinned_toggles_flag_updates_timestamp_and_rejects_missing_project
prompt_packs::browser_adapter::tests::browser_port_delegates_readiness_submission_and_cancellation_without_narrowing_result
prompt_packs::event_adapter::tests::typed_events_map_to_exact_legacy_ipc_payloads
prompt_packs::runtime_commands::tests::execution_adapter_resolves_api_profile_only_inside_spawned_task
prompt_packs::runtime_commands::tests::execution_adapter_spawns_exactly_once_per_ticket
prompt_packs::runtime_commands::tests::execution_task_reuses_start_pool_without_reacquisition
prompt_packs::source_adapter::tests::load_comment_body_performs_a_fresh_read_with_decompression_fallback
prompt_packs::source_adapter::tests::load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows
prompt_packs::source_adapter::tests::load_source_preserves_caller_order_missing_rows_and_nullables
prompt_packs::source_adapter::tests::load_transcript_segments_orders_segment_index_then_row_id
prompt_packs::source_adapter::tests::load_video_maps_full_nullable_metadata_and_missing_rows
prompt_packs::source_adapter::tests::select_comment_candidates_applies_limit_order_and_decompression_fallback
prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled
prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer
readiness::tests::is_ready_current_requires_ready_status_and_current_version
readiness::tests::mark_failed_returns_failed_state
readiness::tests::mark_stale_only_changes_ready_state
readiness::tests::readiness_status_roundtrips_wire_values
secret_store::tests::in_memory_store_can_fail_each_operation
secret_store::tests::secret_ids_are_stable
secret_store::tests::state_reads_writes_and_deletes_secrets
source_ingest::tests::active_kinds_for_sources_reports_matching_locks_only
source_ingest::tests::lock_allows_different_sources
source_ingest::tests::lock_rejects_concurrent_same_source_operations
source_ingest::tests::lock_releases_when_guard_drops
sources::identity::tests::canonical_external_id_rejects_malformed_values
sources::identity::tests::load_telegram_identity_returns_typed_row
sources::identity::tests::load_telegram_runtime_source_pairs_source_with_typed_identity
sources::identity::tests::peer_kind_matches_telegram_subtype
sources::identity::tests::username_normalization_removes_url_and_at_syntax
sources::identity_repair::tests::apply_repair_creates_typed_identity_and_keeps_source_id
sources::identity_repair::tests::apply_repair_is_idempotent
sources::identity_repair::tests::dry_run_reports_repair_without_writing_typed_rows
sources::identity_repair::tests::duplicate_canonical_identity_reports_conflicting_source_ids
sources::identity_repair::tests::duplicate_typed_peer_identity_reports_conflicting_source_ids
sources::identity_repair::tests::fatal_repair_rolls_back_and_does_not_create_canonical_index
sources::identity_repair::tests::malformed_external_ids_fail_without_writing_typed_rows
sources::identity_repair::tests::missing_account_id_is_fatal
sources::identity_repair::tests::repair_creates_minimal_typed_identity_when_legacy_metadata_is_missing_or_malformed
sources::identity_repair::tests::repair_fails_on_conflicting_typed_projection_drift
sources::identity_repair::tests::repair_fails_when_canonical_identity_is_invalid_even_with_legacy_peer_metadata
sources::identity_repair::tests::repair_ignores_malformed_metadata_when_canonical_identity_is_present
sources::identity_repair::tests::repair_ignores_optional_enrichment_gaps_when_typed_identity_is_valid
sources::identity_repair::tests::repair_reads_post_v19_sources_without_legacy_column
sources::identity_repair::tests::repair_rejects_zero_external_id
sources::identity_repair::tests::repair_skips_malformed_metadata_when_typed_identity_is_valid
sources::identity_repair::tests::repair_treats_typed_projection_mismatch_as_fatal
sources::identity_repair::tests::repair_updates_non_conflicting_typed_projection_drift
sources::identity_repair::tests::repair_uses_canonical_subtype_without_legacy_kind
sources::identity_repair::tests::source_identity_gate_blocks_while_running
sources::identity_repair::tests::source_identity_gate_returns_startup_failure
sources::identity_repair::tests::youtube_sources_are_unaffected_by_source_identity_repair
sources::items::query::tests::archive_reader_matches_items_path_for_source_browsing_rows
sources::items::query::tests::archive_reader_matches_topic_filter_and_around_item_semantics
sources::items::query::tests::default_items_path_excludes_migrated_history_rows
sources::items::query::tests::default_source_browsing_does_not_surface_migrated_rows_after_archive_ready
sources::items::query::tests::load_item_rows_attaches_topic_metadata_and_root_matches
sources::items::query::tests::load_item_rows_can_start_at_selected_item
sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_not_ready
sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_stale
sources::items::query::tests::merged_browsing_uses_full_cursor_tuple_for_equal_timestamps
sources::items::query::tests::scoped_browsing_can_load_only_migrated_rows_with_labels
sources::items::query::tests::scoped_browsing_defaults_to_current_rows
sources::items::query::tests::telegram_load_item_rows_uses_items_path_when_archive_model_is_ready
sources::items::query::tests::topic_filters_are_rejected_for_non_current_history_scope
sources::items::query::tests::uncategorized_filter_returns_empty_when_topic_resolution_is_not_ready
sources::items::tests::forum_topic_filter_deserializes_camel_case_topic_id
sources::items::tests::insert_telegram_source_item_allows_same_message_id_in_different_history_domains
sources::items::tests::insert_telegram_source_item_resolves_topic_membership_only_for_new_item
sources::items::tests::insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload
sources::items::tests::insert_telegram_source_item_writes_payload_and_skips_duplicates
sources::items::tests::list_source_items_enriches_youtube_comment_rows_from_raw_payload
sources::items::tests::list_source_items_keeps_base_youtube_comment_when_raw_payload_is_malformed
sources::items::tests::media_metadata_roundtrip_through_zstd
sources::items::tests::migrated_insert_idempotency_uses_old_chat_native_identity
sources::items::tests::migrated_small_group_insert_skips_current_history_derived_writes
sources::items::tests::scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item
sources::items::tests::single_telegram_insert_maintains_ready_archive_model
sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build
sources::items::tests::telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate
sources::items::tests::telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows
sources::items::tests::telegram_insert_writes_analysis_document_in_same_writer_transaction
sources::items::tests::text_roundtrip_through_zstd
sources::items::tests::upsert_youtube_comment_item_updates_existing_text_and_reaction_count
sources::items::tests::upsert_youtube_transcript_item_updates_existing_text_and_returns_id
sources::items::tests::youtube_comment_upsert_targets_non_telegram_partial_unique_index
sources::items::tests::youtube_comment_upsert_writes_analysis_document_and_updates_content
sources::items::tests::youtube_transcript_upsert_targets_non_telegram_partial_unique_index
sources::legacy_metadata_cleanup::tests::audit_ignores_non_telegram_and_null_metadata_rows
sources::legacy_metadata_cleanup::tests::audit_reports_eligible_legacy_telegram_metadata_without_mutating
sources::legacy_metadata_cleanup::tests::audit_skips_invalid_typed_identity
sources::legacy_metadata_cleanup::tests::audit_skips_missing_typed_identity
sources::legacy_metadata_cleanup::tests::audit_skips_subtype_and_account_mismatches
sources::legacy_metadata_cleanup::tests::audit_skips_unsupported_subtype_and_missing_account
sources::legacy_metadata_cleanup::tests::candidate_skip_reason_rejects_unparseable_typed_identity_values
sources::legacy_metadata_cleanup::tests::clear_is_idempotent_after_eligible_metadata_is_removed
sources::legacy_metadata_cleanup::tests::clear_nulls_only_eligible_legacy_telegram_metadata
sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids
sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_empty_refs_as_validation
sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_private_links
sources::peer_resolution::manual_ref::tests::parse_username_accepts_username_and_t_me_links
sources::peer_resolution::tests::add_source_resolution_strategy_distinguishes_username_and_dialog_flows
sources::peer_resolution::tests::source_metadata_decode_failures_are_internal
sources::peer_resolution::tests::source_metadata_decodes_old_dialog_payloads_into_peer_identity
sources::peer_resolution::tests::source_metadata_decodes_old_username_only_payloads
sources::peer_resolution::tests::source_metadata_decodes_typed_peer_identity_payloads
sources::peer_resolution::tests::source_peer_input_rejects_malformed_external_id_as_validation
sources::peer_resolution::tests::source_peer_input_rejects_unsupported_source_type_as_validation
sources::peer_resolution::tests::source_peer_resolution_failure_explains_small_group_dialog_dependency
sources::peer_resolution::tests::source_peer_resolution_plan_prefers_explicit_strategy_order
sources::peer_resolution::tests::typed_identity_plan_allows_username_resolution_without_access_hash
sources::peer_resolution::tests::typed_identity_plan_keeps_dialog_group_dependent_on_dialog_scan
sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_channel_stored_peer_when_access_hash_exists
sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_supergroup_stored_peer_when_access_hash_exists
sources::peer_resolution::tests::typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists
sources::peer_resolution::tests::typed_identity_plan_skips_unusable_stored_peer_when_access_hash_is_missing
sources::settings::tests::initial_sync_policy_label_formats_messages_and_days
sources::settings::tests::sync_settings_default_when_app_settings_are_missing
sources::settings::tests::sync_settings_roundtrip_through_app_settings
sources::settings::tests::validate_sync_settings_rejects_out_of_range_values
sources::store::tests::avatar_cache_key_skips_non_telegram_metadata
sources::store::tests::delete_source_from_pool_enables_foreign_keys_and_cascades_dependents
sources::store::tests::delete_source_is_blocked_when_source_is_used_by_project
sources::store::tests::delete_source_waits_for_temporary_database_write_lock
sources::store::tests::dialog_picked_channel_writes_dialog_typed_identity_with_access_hash
sources::store::tests::dialog_picked_group_writes_dialog_dependent_typed_identity_without_access_hash
sources::store::tests::dialog_picked_supergroup_writes_dialog_typed_identity_with_access_hash
sources::store::tests::list_sources_exposes_migrated_history_counts_without_old_chat_identity
sources::store::tests::list_sources_exposes_sanitized_migrated_history_status_without_chat_id
sources::store::tests::load_source_returns_not_found_for_missing_source
sources::store::tests::source_record_parts_allow_non_telegram_source
sources::store::tests::source_record_parts_emit_only_source_subtype
sources::store::tests::telegram_identity_allows_same_peer_on_different_accounts
sources::store::tests::telegram_identity_rejects_same_account_peer_conflict_at_typed_boundary
sources::store::tests::telegram_source_upsert_inserts_null_metadata
sources::store::tests::telegram_source_upsert_preserves_existing_legacy_metadata_blob
sources::store::tests::telegram_source_upsert_rolls_back_source_when_typed_identity_fails
sources::store::tests::telegram_source_upsert_writes_required_identity_and_available_optional_fields
sources::store::tests::upsert_youtube_playlist_source_handles_legacy_not_null_telegram_kind
sources::store::tests::upsert_youtube_playlist_source_writes_typed_row_and_null_source_metadata
sources::store::tests::upsert_youtube_video_source_conflict_clears_existing_legacy_blob
sources::store::tests::upsert_youtube_video_source_handles_legacy_not_null_telegram_kind
sources::store::tests::upsert_youtube_video_source_rejects_invalid_canonical_url_without_source_row
sources::store::tests::upsert_youtube_video_source_writes_typed_row_and_null_source_metadata
sources::sync::tests::determine_sync_policy_only_applies_initial_settings_on_first_sync
sources::sync::tests::finalize_sync_preserves_existing_legacy_metadata_blob
sources::sync::tests::finalize_sync_updates_source_state_and_typed_avatar_cache
sources::sync::tests::sync_provider_accepts_telegram_sources
sources::sync::tests::sync_provider_rejects_manual_youtube_video_sources
sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error
sources::test_support::tests::source_fixture_creates_expected_tables
sources::topics::tests::forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists
sources::topics::tests::forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind
sources::topics::tests::list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket
sources::topics::tests::topic_refresh_rebuilds_materialized_memberships
sources::topics::tests::upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted
sources::types::tests::item_kind_constants_match_persisted_wire_values
sources::types::tests::source_type_serializes_supported_provider_values
sources::types::tests::telegram_source_subtype_parses_from_canonical_source_subtype
sources::types::tests::telegram_source_subtype_parses_supported_values
sources::types::tests::telegram_source_subtype_rejects_unknown_values_as_validation
sources::types::tests::telegram_source_subtype_rejects_unsupported_source_subtype
sources::types::tests::telegram_source_subtype_serializes_as_existing_wire_value
sql_helpers::tests::push_i64_bind_list_binds_values_in_order
takeout_import::forum_topics::tests::completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups
takeout_import::forum_topics::tests::takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize
takeout_import::forum_topics::tests::takeout_forum_topic_refresh_success_records_no_warning
takeout_import::migrated_history::tests::capability_available_is_source_level_and_restart_safe
takeout_import::migrated_history::tests::capability_unavailable_keeps_reason_internal_and_clears_chat_hint
takeout_import::migrated_history::tests::migrated_history_errors_are_typed_for_frontend_behavior
takeout_import::migrated_history::tests::migrated_small_group_identity_uses_native_old_chat_scope
takeout_import::migrated_history::tests::validation_accepts_matching_revalidated_chat_id
takeout_import::migrated_history::tests::validation_rejects_missing_or_changed_revalidated_chat_id
takeout_import::recovery::tests::recovery_state_includes_migrated_history_scope_for_historical_batches
takeout_import::recovery::tests::takeout_recovery_ignores_non_takeout_batches
takeout_import::recovery::tests::takeout_recovery_latest_complete_hides_older_failed
takeout_import::recovery::tests::takeout_recovery_latest_failed_wins_over_older_complete
takeout_import::recovery::tests::takeout_recovery_returns_partial_completed_and_hides_complete
takeout_import::recovery::tests::takeout_recovery_running_with_active_job_is_hidden
takeout_import::recovery::tests::takeout_recovery_running_without_active_job_is_interrupted
takeout_import::recovery::tests::takeout_recovery_source_filter_limits_results
takeout_import::recovery::tests::takeout_recovery_warning_codes_are_unique_sorted_and_message_free
takeout_import::state::tests::active_jobs_for_sources_filters_non_terminal_jobs
takeout_import::state::tests::job_state_can_cancel_and_finish_job
takeout_import::state::tests::job_state_cancels_child_tokens
takeout_import::state::tests::job_state_records_history_scope_for_frontend_labels
takeout_import::state::tests::job_state_rejects_duplicate_active_source_jobs
takeout_import::state::tests::takeout_cancellation_smoke_fixture_finishes_cancelled_and_clears
takeout_import::state::tests::takeout_cancellation_smoke_fixture_tracks_running_job
takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact
takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation
takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues
takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize
takeout_import::tests::historical_batch_completion_does_not_advance_source_watermark
takeout_import::tests::locked_start_allows_only_one_batch_for_same_source
takeout_import::tests::locked_start_conflict_creates_no_provenance_rows
takeout_import::tests::migrated_history_detected_warning_is_sanitized
takeout_import::tests::migrated_history_start_records_use_same_source_takeout_lock
takeout_import::tests::migrated_history_start_requires_available_capability
takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once
takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers
takeout_import::tests::takeout_step_cancel_wrapper_allows_completed_future
takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future
takeout_import::tests::takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists
takeout_import::tests::takeout_subtype_load_uses_typed_identity_not_legacy_kind
takeout_import::validation_diagnostics::tests::takeout_validation_batch_summary_is_durable_and_sanitized
takeout_import::validation_diagnostics::tests::takeout_validation_duplicate_after_normal_sync_summarizes_outcomes
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_caps_samples_deterministically
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_compares_batch_to_canonical_without_content
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_matched_observations_for_aggregates
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_missing_observations_by_identity
takeout_import::validation_diagnostics::tests::takeout_validation_snapshot_delta_uses_explicit_snapshots
takeout_import::validation_diagnostics::tests::takeout_validation_source_snapshot_is_aggregate_and_sanitized
takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_excludes_non_latest_recovery_candidates
takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_is_durable_only
telegram::tests::diagnostic_status_counts_do_not_return_account_ids_or_messages
telegram::tests::legacy_api_hash_migrates_to_secret_store_and_blanks_column
telegram::tests::legacy_api_hash_remains_when_secret_write_fails
telegram::tests::missing_secure_api_hash_for_blank_legacy_account_is_auth_error
telegram::tests::runtime_status_maps_to_existing_wire_strings
telegram::tests::telegram_api_id_out_of_range_returns_typed_validation_error
telegram::tests::telegram_status_and_event_payload_contract_is_exact
telegram_impl::dto::tests::telegram_item_kind_constant_matches_persisted_wire_value
telegram_impl::dto::tests::telegram_message_draft_has_single_persistence_shape
telegram_impl::dto::tests::telegram_message_identity_validation_rejects_invalid_values
telegram_impl::error::tests::channel_private_detection_reads_rpc_name_from_error_message
telegram_impl::error::tests::non_forum_topic_refresh_errors_are_detected
telegram_impl::live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure
telegram_impl::live::messages::tests::fallback_peer_identity_uses_telegram_history_peer_vocabulary
telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload
telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule
telegram_impl::live::messages::tests::reply_peer_context_uses_telegram_peer_kinds
telegram_impl::live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget
telegram_impl::live::peer::tests::dialog_lookup_misses_are_not_found
telegram_impl::live::peer::tests::dialog_lookup_not_found_message_explains_numeric_manual_limit
telegram_impl::live::peer::tests::peer_ref_from_identity_ignores_small_groups_without_supported_identity
telegram_impl::live::peer::tests::peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation
telegram_impl::live::peer::tests::peer_ref_from_identity_uses_channel_access_hash
telegram_impl::live::peer::tests::peer_ref_from_identity_uses_supergroup_access_hash
telegram_impl::live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes
telegram_impl::live::peer::tests::typed_identity_builds_channel_peer_ref_when_access_hash_exists
telegram_impl::live::peer::tests::typed_identity_rejects_subtype_peer_kind_mismatch
telegram_impl::live::peer::tests::validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype
telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor
telegram_impl::media::tests::derive_content_kind_tracks_text_and_media_presence
telegram_impl::media::tests::derive_document_media_kind_prefers_specific_signals
telegram_impl::runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors
telegram_impl::runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure
telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check
telegram_impl::runtime::tests::failed_sign_in_retains_pending_attempt
telegram_impl::runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner
telegram_impl::runtime::tests::missing_account_authentication_is_false
telegram_impl::runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt
telegram_impl::runtime::tests::sign_in_without_code_request_preserves_auth_error
telegram_impl::runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt
telegram_impl::session::tests::encrypted_session_load_fails_for_wrong_account_id
telegram_impl::session::tests::encrypted_session_load_round_trips
telegram_impl::session::tests::generated_session_key_returns_write_only_encoded_secret
telegram_impl::session::tests::legacy_json_returns_rewrite_decision
telegram_impl::session::tests::missing_encrypted_key_preserves_auth_error
telegram_impl::session::tests::saving_session_writes_encrypted_envelope_not_plaintext
telegram_impl::session::tests::session_encryption_key_rejects_invalid_length
telegram_impl::takeout::export_dc::tests::export_dc_attempt_state_detects_first_fallback_transition
telegram_impl::takeout::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors
telegram_impl::takeout::export_dc::tests::export_dc_id_applies_tdesktop_shift
telegram_impl::takeout::export_dc::tests::export_dc_invoke_does_not_fallback_for_rpc_errors
telegram_impl::takeout::export_dc::tests::export_dc_invoke_falls_back_to_home_dc_on_local_error
telegram_impl::takeout::export_dc::tests::export_dc_invoke_uses_home_dc_directly_after_fallback
telegram_impl::takeout::export_dc::tests::takeout_init_request_uses_source_subtype_flags_and_file_limit
telegram_impl::takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots
telegram_impl::takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping
telegram_impl::takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome
telegram_impl::takeout::operations::tests::history_page_and_search_return_owned_takeout_messages
telegram_impl::takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity
telegram_impl::takeout::operations::tests::only_my_messages_fallback_is_limited_to_channels
telegram_impl::takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges
telegram_impl::takeout::pagination::tests::descending_fallback_keeps_raw_order_and_moves_to_min_message_id
telegram_impl::takeout::pagination::tests::messages_not_modified_response_is_rejected_for_takeout_page
telegram_impl::takeout::pagination::tests::messages_response_without_slice_is_terminal_page
telegram_impl::takeout::pagination::tests::split_selection_falls_back_when_telegram_returns_no_ranges
telegram_impl::takeout::pagination::tests::split_selection_uses_all_ranges_for_small_group
telegram_impl::takeout::pagination::tests::split_selection_uses_last_range_for_channel_and_supergroup
telegram_impl::takeout::pagination::tests::tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback
telegram_impl::takeout::pagination::tests::tdesktop_non_advancing_cursor_restarts_descending_fallback
telegram_impl::takeout::pagination::tests::tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id
telegram_impl::takeout::raw_parse::tests::parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids
telegram_impl::takeout::raw_parse::tests::parses_document_media_kind_filename_and_dimensions
telegram_impl::takeout::raw_parse::tests::parses_photo_message_metadata
telegram_impl::takeout::raw_parse::tests::parses_text_message_with_reply_and_reactions
telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids
telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id
telegram_impl::takeout::raw_parse::tests::skips_empty_raw_messages
telegram_impl::takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error
telegram_session_store::tests::delete_session_from_path_removes_file_and_key
telegram_session_store::tests::encrypted_session_load_fails_when_key_is_missing
telegram_session_store::tests::legacy_plaintext_session_migrates_to_encrypted_file
telegram_session_store::tests::legacy_plaintext_session_remains_when_keyring_write_fails
telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact
topic_memberships::tests::rebuild_matches_retained_hidden_and_deleted_topics
topic_memberships::tests::rebuild_prioritizes_specific_topic_matches_before_general_fallback
topic_memberships::tests::rebuild_replaces_stale_memberships_and_versions
topic_memberships::tests::rebuild_uses_legacy_root_only_without_typed_child
tx::tests::begin_immediate_commit_persists_changes
tx::tests::begin_immediate_read_then_write_survives_concurrent_writer
tx::tests::begin_immediate_rollback_discards_changes
tx::tests::begin_immediate_with_foreign_keys_enforces_cascade
tx::tests::deferred_read_then_write_hits_busy_snapshot_under_concurrent_writer
tx::tests::finish_manual_transaction_commits_success_result
tx::tests::finish_manual_transaction_rolls_back_error_result
tx::tests::sqlite_ignores_foreign_keys_pragma_inside_open_transaction
youtube::captions::tests::caption_download_args_request_json3_and_vtt_without_media
youtube::captions::tests::caption_selection_honors_explicit_override_before_original_language
youtube::captions::tests::caption_selection_prefers_original_then_preferred_then_english_then_any
youtube::captions::tests::json3_parser_allows_missing_duration
youtube::captions::tests::json3_parser_concatenates_segments_and_preserves_timing
youtube::captions::tests::replace_transcript_segments_deletes_previous_rows_and_inserts_current_segments
youtube::captions::tests::replace_transcript_segments_rebuilds_analysis_documents_by_segment_order
youtube::captions::tests::transcript_external_id_includes_language_and_track_kind
youtube::captions::tests::vtt_parser_reads_cues_and_skips_blank_text
youtube::captions::tests::vtt_parser_rejects_invalid_timing
youtube::comments::tests::comment_published_at_accepts_numbers_strings_and_fallback
youtube::comments::tests::comments_fetch_args_include_bounded_extractor_args
youtube::comments::tests::comments_fetch_timeout_is_longer_than_metadata_preview_timeout
youtube::comments::tests::default_comment_limit_is_bounded
youtube::comments::tests::normalize_comments_flattens_replies_and_warns_for_timestamp_fallbacks
youtube::comments::tests::normalize_comments_truncates_raw_comment_array_before_normalization
youtube::cookies::tests::accepts_empty_cookie_values
youtube::cookies::tests::accepts_http_only_cookie_rows
youtube::cookies::tests::rejects_empty_cookie_text
youtube::cookies::tests::rejects_files_without_cookie_rows
youtube::cookies::tests::rejects_invalid_cookie_text_before_saving_secret
youtube::cookies::tests::stores_reads_and_clears_youtube_cookies_through_secret_store
youtube::cookies::tests::validates_netscape_cookie_rows_without_exposing_values
youtube::detail::tests::list_summaries_uses_source_id_order_and_marks_no_captions_unavailable
youtube::detail::tests::playlist_detail_reports_ordered_items_and_summary_counts
youtube::detail::tests::playlist_detail_uses_typed_linked_video_metadata_with_corrupt_source_blob
youtube::detail::tests::source_summary_missing_typed_metadata_uses_generic_title_without_blob_decode
youtube::detail::tests::summaries_use_typed_video_metadata_with_corrupt_source_blob
youtube::detail::tests::video_detail_includes_safe_source_metadata_without_item_raw_payloads
youtube::detail::tests::video_detail_missing_typed_metadata_returns_controlled_error
youtube::detail::tests::video_detail_reports_synced_transcript_comments_and_playlist_memberships
youtube::dto::tests::availability_status_serializes_as_snake_case
youtube::dto::tests::preview_kind_deserializes_snake_case
youtube::dto::tests::video_form_serializes_short_value
youtube::errors::tests::invalid_youtube_url_maps_to_validation_error
youtube::errors::tests::ytdlp_deleted_failures_map_to_not_found_error
youtube::errors::tests::ytdlp_network_failures_map_to_network_error
youtube::errors::tests::ytdlp_private_failures_map_to_auth_error
youtube::jobs::tests::active_jobs_for_sources_filters_non_terminal_direct_and_related_sources
youtube::jobs::tests::catalog_jobs_for_sources_includes_latest_failed_jobs
youtube::jobs::tests::diagnostic_counts_group_source_jobs_without_ids_or_raw_errors
youtube::jobs::tests::job_state_cancels_child_tokens
youtube::jobs::tests::job_state_finishes_cancel_requested_jobs_as_cancelled
youtube::jobs::tests::job_state_list_filters_before_limit_and_sorts_newest_first
youtube::jobs::tests::job_state_rejects_duplicate_active_scope_but_allows_different_job_types
youtube::jobs::tests::jobs_missing_typed_video_metadata_errors_after_failed_refresh
youtube::jobs::tests::jobs_reload_missing_typed_video_metadata_after_refresh_callback
youtube::jobs::tests::retryable_playlist_video_rows_excludes_auth_deleted_and_removed_entries
youtube::jobs::tests::source_job_cancellation_smoke_fixture_finishes_cancelled_and_clears
youtube::jobs::tests::source_job_cancellation_smoke_fixture_tracks_running_job
youtube::jobs::tests::source_job_step_with_process_cancel_allows_completed_future
youtube::jobs::tests::source_job_step_with_process_cancel_interrupts_pending_future
youtube::jobs::tests::source_job_type_uses_comments_specific_type_for_comments_only_video_sync
youtube::jobs::tests::source_job_workflow_file_has_no_tauri_command_adapters
youtube::jobs::tests::source_jobs_no_longer_decode_source_metadata_blobs
youtube::metadata::tests::availability_values_map_to_statuses
youtube::metadata::tests::playlist_fixture_maps_metadata_entries_and_preview_warning
youtube::metadata::tests::playlist_metadata_page_args_use_adjacent_playlist_range
youtube::metadata::tests::video_fixture_maps_metadata_and_preview_fields
youtube::metadata::tests::video_fixture_missing_optional_fields_maps_to_none
youtube::playlist::tests::playlist_item_video_source_upsert_writes_typed_video_metadata_not_source_blob
youtube::playlist::tests::upsert_playlist_items_can_skip_video_source_materialization
youtube::playlist::tests::upsert_playlist_items_marks_missing_rows_removed
youtube::playlist::tests::upsert_playlist_items_reuses_existing_video_source_and_keeps_unavailable_null
youtube::playlist::tests::upsert_playlist_items_without_materialization_reuses_existing_video_source
youtube::preview::tests::preview_from_playlist_json_returns_playlist_preview
youtube::preview::tests::preview_from_video_json_uses_parsed_url_kind
youtube::process_runtime::tests::cancellation_reaches_all_reserved_operations
youtube::process_runtime::tests::cookie_guard_retains_file_until_detached_reaper_finishes
youtube::process_runtime::tests::detached_reaper_keeps_cookie_until_the_stuck_child_releases
youtube::process_runtime::tests::dropped_caller_keeps_child_and_registry_owned_until_shutdown_reaps_it
youtube::process_runtime::tests::external_source_job_cancellation_reaps_its_managed_operation
youtube::process_runtime::tests::finite_pipe_backpressure_requires_concurrent_drain
youtube::process_runtime::tests::injected_launcher_drains_backpressured_output_before_waiting_for_exit
youtube::process_runtime::tests::injected_nonzero_exit_preserves_not_found_classification_and_releases_registry
youtube::process_runtime::tests::injected_timeout_reap_detaches_stuck_child_and_keeps_cookie_until_release
youtube::process_runtime::tests::injected_wait_error_reaps_the_child_before_releasing_registry
youtube::process_runtime::tests::registry_reserves_an_operation_before_spawn
youtube::process_runtime::tests::shutdown_rejects_new_ytdlp_admission_before_spawn
youtube::process_runtime::tests::spawn_failure_rolls_back_the_registry_reservation
youtube::process_runtime::tests::timeout_fallback_detaches_cookie_until_stuck_child_reaps
youtube::runtime::tests::runtime_status_serializes_with_camel_case_keys
youtube::settings::tests::auth_cookies_load_only_when_auth_is_enabled
youtube::settings::tests::invalid_stored_settings_return_validation_error_with_key
youtube::settings::tests::invalid_youtube_settings_do_not_write_partial_values
youtube::settings::tests::saving_cookies_enables_auth_and_clear_disables_it
youtube::settings::tests::validate_youtube_settings_normalizes_preferred_captions_language
youtube::settings::tests::validate_youtube_settings_rejects_out_of_range_values
youtube::settings::tests::youtube_settings_default_when_app_settings_are_missing
youtube::settings::tests::youtube_settings_roundtrip_through_app_settings
youtube::settings::tests::youtube_settings_serializes_with_camel_case_keys
youtube::source_metadata::tests::playlist_metadata_columns_are_versioned_and_secret_safe
youtube::source_metadata::tests::upsert_video_metadata_maintains_description_document
youtube::source_metadata::tests::video_metadata_columns_include_wire_values_arrays_caption_override_and_sanitized_raw
youtube::source_metadata::tests::video_metadata_rejects_wrong_canonical_url_shape
youtube::source_metadata::tests::video_source_metadata_restores_raw_caption_metadata_for_provider_sync
youtube::thumbnail::tests::accepts_only_allowlisted_https_thumbnail_urls
youtube::thumbnail::tests::bounds_thumbnail_responses_to_one_mib
youtube::thumbnail::tests::builds_the_dedicated_thumbnail_client
youtube::thumbnail::tests::recognizes_supported_image_magic_bytes
youtube::transcript_reader::tests::list_youtube_transcript_segments_can_start_at_selected_time
youtube::transcript_reader::tests::list_youtube_transcript_segments_filters_by_search
youtube::transcript_reader::tests::list_youtube_transcript_segments_pages_by_time_and_id
youtube::transcript_reader::tests::search_escapes_existing_backslashes_before_like_wildcards
youtube::url::tests::parses_live_url
youtube::url::tests::parses_playlist_url
youtube::url::tests::parses_short_youtu_be_url
youtube::url::tests::parses_shorts_url
youtube::url::tests::parses_watch_video_url
youtube::url::tests::rejects_empty_input
youtube::url::tests::rejects_invalid_host
youtube::url::tests::watch_url_with_playlist_parameter_parses_selected_video
youtube::ytdlp::tests::authenticated_command_args_include_cookie_file_path_without_cookie_content
youtube::ytdlp::tests::cookie_file_content_adds_netscape_header_when_missing
youtube::ytdlp::tests::cookie_file_content_preserves_existing_netscape_header
youtube::ytdlp::tests::preview_playlist_args_limit_entries_to_first_fifty
youtube::ytdlp::tests::preview_video_args_use_dump_json_without_shell_fragments
```

### 665 extractum identities

```text
account_deletion::tests::active_direct_source_analysis_run_blocks_owned_source_only
account_deletion::tests::active_group_analysis_run_blocks_when_any_member_source_is_owned
account_deletion::tests::active_llm_request_for_completed_owned_run_blocks_but_provider_test_does_not
account_deletion::tests::active_source_job_on_owned_source_blocks_but_unowned_job_does_not
account_deletion::tests::active_takeout_job_on_owned_source_blocks
account_deletion::tests::blocker_collection_keeps_multiple_categories_for_internal_diagnostics
account_deletion::tests::existing_account_with_zero_sources_passes
account_deletion::tests::missing_account_returns_not_found
account_deletion::tests::source_ingest_lock_on_owned_source_blocks_without_deleting_rows
accounts::tests::creating_account_rolls_back_when_secret_write_fails
accounts::tests::creating_account_writes_api_hash_to_secret_store_only
accounts::tests::deleting_account_removes_secret_after_database_row
accounts::tests::deleting_missing_account_returns_not_found
accounts::tests::secret_cleanup_failure_keeps_deleted_database_row_deleted
analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion
analysis::corpus::tests::live::default_analysis_corpus_excludes_migrated_history_documents
analysis::corpus::tests::live::description_mode_creates_synthetic_description_message
analysis::corpus::tests::live::explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus
analysis::corpus::tests::live::live_corpus_refs_use_local_item_ids
analysis::corpus::tests::live::load_corpus_messages_filters_telegram_to_telegram_message
analysis::corpus::tests::live::load_corpus_messages_filters_youtube_transcript_only_to_transcripts
analysis::corpus::tests::live::load_corpus_messages_includes_youtube_comment_only_in_comments_mode
analysis::corpus::tests::live::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref
analysis::corpus::tests::live::load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content
analysis::corpus::tests::live::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
analysis::corpus::tests::live::preflight_ref_format_matches_corpus_loader_ref_format
analysis::corpus::tests::live::source_group_opt_in_includes_only_members_with_migrated_rows
analysis::corpus::tests::live::youtube_description_missing_typed_metadata_skips_without_decoding_source_blob
analysis::corpus::tests::live::youtube_description_rows_use_typed_metadata_with_corrupt_source_blob
analysis::corpus::tests::live::youtube_transcript_segment_evidence_uses_typed_source_context
analysis::corpus::tests::preflight::preflight_count_matches_loader_for_youtube_corpus_modes
analysis::corpus::tests::preflight::preflight_counts_eligible_text_messages_for_sources
analysis::corpus::tests::preflight::preflight_ignores_media_only_items_without_text_content
analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows
analysis::corpus::tests::source_resolution::resolve_analysis_sources_loads_single_provider_project
analysis::corpus::tests::source_resolution::resolve_analysis_sources_preserves_no_linked_youtube_error_message
analysis::corpus::tests::source_resolution::resolve_analysis_sources_rejects_mixed_provider_project
analysis::corpus::tests::source_resolution::resolve_run_source_ids_loads_project_sources_without_snapshot
analysis::fixtures::tests::active_runs::fixture_active_state_tracks_seeded_running_run
analysis::fixtures::tests::active_runs::fixture_cancel_waiter_marks_running_run_cancelled
analysis::fixtures::tests::clear::clear_deletes_child_rows_through_fixture_parent_ids
analysis::fixtures::tests::clear::clear_preserves_non_fixture_groups_and_members
analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent
analysis::fixtures::tests::harness::fixture_test_pool_has_required_tables
analysis::fixtures::tests::seed::compressed_fixture_fields_are_readable
analysis::fixtures::tests::seed::seed_creates_fixture_runs_with_statuses_templates_and_snapshots
analysis::fixtures::tests::seed::seed_creates_post_sync_reader_content
analysis::fixtures::tests::seed::seed_creates_safe_account_prompt_profile_sources_and_group
analysis::fixtures::tests::seed::seed_creates_sources_that_pass_identity_repair
analysis::fixtures::tests::seed::seed_creates_valid_typed_youtube_detail_metadata
analysis::fixtures::tests::seed::seed_twice_keeps_one_deterministic_fixture_set
analysis::fixtures::tests::snapshot::capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report
analysis::fixtures::tests::snapshot::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot
analysis::fixtures::tests::snapshot::missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages
analysis::fixtures::tests::snapshot::seeded_snapshot_runs_expose_captured_snapshot_state
analysis::fixtures::tests::summary::summary_serializes_with_camel_case_keys
analysis::groups::tests::prepare_analysis_source_group_input_preserves_baseline_error_precedence
analysis::groups::tests::validate_group_source_type_accepts_matching_provider_membership
analysis::groups::tests::validate_group_source_type_rejects_mixed_provider_membership
analysis::groups::tests::validate_group_source_type_rejects_unknown_group_type
analysis::report::tests::capture::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
analysis::store::tests::read_model::list_analysis_run_summaries_filters_project_runs
analysis::store::tests::read_model::list_analysis_run_summaries_matches_all_query_terms_across_any_field
analysis::store::tests::setup::ensure_sources_exist_returns_typed_not_found_error
analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit
analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects
analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot
analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id
analysis::tests_application::report_profile_resolution_failure_prevents_run_creation
analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads
analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels
analysis_documents::tests::document_metadata_envelopes_match_current_evidence_shape
analysis_documents::tests::rebuild_analysis_documents_excludes_migrated_history_rows
analysis_documents::tests::rebuild_source_materializes_text_units_with_document_order
analysis_documents::tests::rebuild_source_removes_stale_documents_and_is_idempotent
analysis_documents::tests::schema_creates_analysis_documents_constraints_and_indexes
apalis_jobs::tests::apalis_jobs_counts_ignore_their_own_active_filter
apalis_jobs::tests::apalis_jobs_decode_failure_returns_redacted_preview_without_json
apalis_jobs::tests::apalis_jobs_limit_excludes_large_payloads_outside_limited_rows
apalis_jobs::tests::apalis_jobs_list_clamps_limit
apalis_jobs::tests::apalis_jobs_list_does_not_mutate_jobs
apalis_jobs::tests::apalis_jobs_list_filters_by_status_job_type_and_search
apalis_jobs::tests::apalis_jobs_list_returns_empty_when_jobs_table_missing
apalis_jobs::tests::apalis_jobs_list_returns_rfc3339_utc_timestamps
apalis_jobs::tests::apalis_jobs_list_returns_rows_from_jobs_table
apalis_jobs::tests::apalis_jobs_list_sorts_by_latest_activity_timestamp
apalis_jobs::tests::apalis_jobs_non_json_result_and_metadata_are_omitted_in_v1
apalis_jobs::tests::apalis_jobs_payloads_are_redacted_and_truncated
apalis_jobs::tests::apalis_jobs_prune_terminal_deletes_only_old_done_killed_and_terminal_failed_jobs
apalis_jobs::tests::apalis_jobs_prune_terminal_returns_zero_when_jobs_table_missing
apalis_jobs::tests::apalis_jobs_row_shape_is_stable_when_optional_columns_are_absent
apalis_jobs::tests::apalis_jobs_schema_probe_documents_local_jobs_table_shape
archive_read_model::tests::create_schema_adds_state_and_item_tables
archive_read_model::tests::current_ready_state_rejects_old_model_version
archive_read_model::tests::rebuild_source_excludes_migrated_history_rows
archive_read_model::tests::rebuild_source_materializes_archive_fidelity_fields
child_process::tests::create_no_window_matches_win32_process_creation_flags
diagnostics::database::tests::database_diagnostics_groups_only_allow_listed_aggregates
diagnostics::database::tests::migration_status_reports_pending_and_failed_versions
diagnostics::dto::tests::diagnostic_summary_fixture_serializes_without_forbidden_sentinels
diagnostics::redaction::tests::redact_json_value_redacts_sensitive_keys_recursively
diagnostics::redaction::tests::redact_text_removes_secret_and_content_patterns
diagnostics::redaction::tests::sanitized_error_message_bounds_unicode_by_chars
diagnostics::redaction::tests::sanitized_error_message_is_bounded
diagnostics::runtime::tests::failed_runtime_check_uses_coarse_summary_without_os_error_text
diagnostics::runtime::tests::secure_storage_failure_does_not_expose_store_error_text
diagnostics::tests::sanitize_diagnostic_error_bounds_and_redacts_command_errors
diagnostics::tests::serialized_diagnostic_summary_preserves_allowed_data_and_excludes_forbidden_data
external_process::tests::admission_wait_consumes_the_shared_graceful_budget
external_process::tests::cleanup_tasks_start_concurrently_and_isolate_error_and_panic
external_process::tests::concurrent_watchdogs_invoke_exit_once
external_process::tests::exhausted_admission_budget_skips_the_cleanup_factory
external_process::tests::injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback
external_process::tests::permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown
external_process::tests::permits_acquired_before_shutdown_are_waited_for
external_process::tests::repeated_start_does_not_replace_code_or_schedule_again
external_process::tests::start_reports_completed_after_watchdog_claims_exit
external_process::tests::start_returns_started_and_schedules_one_watchdog
external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets
external_process::tests::watchdog_exits_with_the_preserved_code_unless_cleanup_completed
gemini_browser::cdp_chrome::tests::drop_falls_back_to_owned_child_shutdown
gemini_browser::cdp_chrome::tests::explicit_shutdown_kills_and_reaps_the_owned_child_once
gemini_browser::cdp_chrome::tests::shutdown_does_not_claim_or_kill_an_already_exited_child
gemini_browser::cdp_chrome::tests::shutdown_reaps_when_the_child_has_already_exited_during_kill
gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_accepts_json_version_response
gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_reports_unreachable_endpoint
gemini_browser::executor::tests::cancelled_run_marks_the_sidecar_transport_tainted
gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json
gemini_browser::jobs::app_tests::apalis_sqlite_status_probe_documents_actual_status_values
gemini_browser::jobs::app_tests::apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job
gemini_browser::jobs::app_tests::apalis_storage_preserves_existing_sqlx_migration_history_table
gemini_browser::jobs::app_tests::apalis_storage_shares_extractum_db_without_locking_app_pool
gemini_browser::jobs::app_tests::apalis_storage_uses_shared_main_extractum_db_identity
gemini_browser::jobs::app_tests::enqueue_duplicate_run_id_returns_conflict
gemini_browser::jobs::app_tests::enqueue_persists_job_before_worker_startup
gemini_browser::jobs::app_tests::failed_gemini_browser_job_is_not_retried
gemini_browser::jobs::app_tests::gemini_browser_jobs_are_built_with_one_total_attempt
gemini_browser::jobs::app_tests::restart_worker_processes_pending_job_after_runtime_restart
gemini_browser::jobs::app_tests::worker_picks_up_job_quickly_after_idle
gemini_browser::sidecar::tests::stderr_drain_consumes_sidecar_output_concurrently
ingest_provenance::tests::completed_zero_observation_batch_is_complete_without_partial_flags
ingest_provenance::tests::create_takeout_batch_inserts_generic_and_detail_rows_atomically
ingest_provenance::tests::migrated_history_deferred_scope_finalizes_partial_and_records_warning_once
ingest_provenance::tests::migrated_small_group_imported_allows_duplicate_only_success
ingest_provenance::tests::migrated_small_group_scope_can_be_marked_running_and_completed
ingest_provenance::tests::mixed_partial_scope_finalizes_as_partial
ingest_provenance::tests::terminal_update_recalculates_counters_and_sanitizes_error
job_helpers::tests::active_job_guards_track_and_release_scoped_jobs
job_helpers::tests::cancellation_state_cancels_child_tokens
job_helpers::tests::cancellation_state_marks_checks_and_clears_jobs
library_sources::tests::catalog_status_for_input_keeps_failed_job_without_detail_empty
library_sources::tests::list_library_catalog_returns_status_capabilities_and_filter_counts
library_sources::tests::list_library_sources_keeps_sources_with_missing_provider_details
library_sources::tests::list_library_sources_returns_youtube_and_telegram_metadata
llm::profiles::tests::active_profile_resolution_loads_key_from_secret_store
llm::profiles::tests::changing_key_scope_without_replacement_is_rejected
llm::profiles::tests::clear_profile_api_key_deletes_secret
llm::profiles::tests::credential_scope_uses_provider_origin_and_effective_port_but_not_path
llm::profiles::tests::delete_profile_fails_if_secret_store_fails_leaving_db_settings_intact
llm::profiles::tests::delete_profile_removes_settings_and_secret_and_resets_active
llm::profiles::tests::empty_save_preserves_existing_secret
llm::profiles::tests::keyed_legacy_profile_materializes_effective_base_url_while_unkeyed_stays_blank
llm::profiles::tests::legacy_remote_http_profile_is_rejected_before_request_configuration
llm::profiles::tests::materialization_write_failure_fails_closed_during_state_load
llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store
llm::profiles::tests::profile_state_lists_multiple_saved_profiles
llm::profiles::tests::provider_access_resolution_uses_configured_key_with_saved_base_url
llm::profiles::tests::provider_access_resolution_uses_saved_key_with_configured_base_url
llm::profiles::tests::set_active_profile_returns_typed_not_found_error
llm::profiles::tests::validate_profile_id_rejects_invalid_characters
llm::tests::llm_command_errors_and_failed_events_keep_distinct_json_shapes
llm::tests::llm_stream_events_serialize_exact_lifecycle_contract
llm::tests::provider_diagnostics_exclude_profile_ids_and_base_urls
migrations::baseline_reset::tests::backup_failure_prevents_migration_history_rewrite
migrations::baseline_reset::tests::classifies_baseline_history_after_post_baseline_migrations
migrations::baseline_reset::tests::classifies_baseline_history_only_when_checksum_matches
migrations::baseline_reset::tests::classifies_old_history_only_when_versions_one_through_twenty_six_are_successful
migrations::baseline_reset::tests::old_history_cutover_backs_up_then_rewrites_only_migration_history
migrations::baseline_reset::tests::rejects_baseline_history_with_wrong_checksum
migrations::baseline_reset::tests::rejects_failed_migration_history
migrations::baseline_reset::tests::rejects_partial_old_history_without_version_twenty_six
migrations::tests::analysis_telegram_history_scope_migration_adds_nullable_checked_column
migrations::tests::app_migrator_accepts_database_with_preexisting_apalis_migration_history
migrations::tests::build_migrations_includes_apalis_sqlite_versions_for_shared_sqlx_history
migrations::tests::build_migrations_includes_prompt_pack_runtime_provider_version_ten
migrations::tests::build_migrations_starts_at_current_schema_baseline
migrations::tests::concurrent_test_migrations_publish_complete_apalis_schemas
migrations::tests::current_schema_baseline_checksum_matches_frozen_reset_boundary
migrations::tests::current_schema_baseline_migration_is_version_one
migrations::tests::fresh_schema_includes_analysis_documents_table_indexes_and_constraints
migrations::tests::fresh_schema_includes_analysis_snapshot_markers
migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints
migrations::tests::fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints
migrations::tests::fresh_schema_includes_projects_redesign_columns_index_and_defaults
migrations::tests::fresh_schema_includes_source_identity_tables_after_sql_managed_migrations
migrations::tests::post_baseline_migration_upgrades_frozen_baseline_for_migrated_history
migrations::tests::prepare_database_skips_cutover_when_database_file_is_missing
migrations::tests::projects_mvp_migration_is_registered
migrations::tests::projects_mvp_schema_applies_to_memory_pool
migrations::tests::prompt_pack_mvp_migration_creates_library_and_run_tables
migrations::tests::prompt_pack_mvp_migration_declares_required_integrity_constraints
migrations::tests::test_migration_batch_rolls_back_schema_and_history_together
migrations::tests::vendored_apalis_sqlite_migrations_match_pinned_dependency
notebooklm_export::chunker::tests::accounts_for_document_overhead_when_splitting
notebooklm_export::chunker::tests::falls_back_to_month_when_year_exceeds_limits
notebooklm_export::chunker::tests::falls_back_to_topic_id_when_topic_title_slug_is_invalid
notebooklm_export::chunker::tests::filters_short_text_without_other_signal
notebooklm_export::chunker::tests::groups_chunks_by_topic_slug
notebooklm_export::chunker::tests::keeps_yearly_group_when_within_limits
notebooklm_export::chunker::tests::splits_by_word_and_byte_limits
notebooklm_export::filename::tests::accepts_safe_relative_child_paths
notebooklm_export::filename::tests::child_paths_stay_under_base
notebooklm_export::filename::tests::rejects_reserved_components
notebooklm_export::filename::tests::rejects_unsafe_relative_child_paths
notebooklm_export::filename::tests::sanitizes_unsafe_filename_parts
notebooklm_export::glossary::tests::aggregates_participants_by_author
notebooklm_export::links::tests::detects_and_trims_http_urls
notebooklm_export::media::tests::renders_numeric_only_media_metadata
notebooklm_export::media::tests::renders_useful_media_placeholder_parts
notebooklm_export::message_mapping::tests::reply_snippet_decode_failures_are_typed_internal_errors
notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_bounded_periods
notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_notebooklm_messages
notebooklm_export::query::tests::corrupt_archive_reply_target_outside_period_fails_archive_loader
notebooklm_export::query::tests::current_export_archive_loader_sets_scope_markers
notebooklm_export::query::tests::export_fixture_rejects_null_published_at_before_loader_parity
notebooklm_export::query::tests::load_export_messages_adds_local_reply_context_outside_period
notebooklm_export::query::tests::load_export_messages_attaches_general_topic_when_topic_header_is_missing
notebooklm_export::query::tests::load_export_messages_attaches_topic_metadata_for_reply_and_root_messages
notebooklm_export::query::tests::load_export_messages_does_not_root_match_non_numeric_external_ids
notebooklm_export::query::tests::load_export_messages_reads_materialized_topic_memberships
notebooklm_export::query::tests::load_export_source_group_exposes_youtube_group_for_hard_validation
notebooklm_export::query::tests::load_export_source_group_keeps_dirty_member_source_type_for_skip_logic
notebooklm_export::query::tests::load_export_source_group_orders_members_by_title_then_id
notebooklm_export::query::tests::load_export_source_rejects_non_telegram_before_message_loader_selection
notebooklm_export::query::tests::load_export_source_uses_canonical_subtype_not_legacy_kind
notebooklm_export::query::tests::migrated_export_reply_lookup_stays_inside_old_history_domain
notebooklm_export::query::tests::notebooklm_archive_export_excludes_migrated_history_rows_even_if_materialized
notebooklm_export::query::tests::notebooklm_default_export_excludes_migrated_history_rows
notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_all_fallback_reasons
notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_missing_and_old_version
notebooklm_export::query::tests::notebooklm_export_loader_selection_uses_archive_for_ready_current_state
notebooklm_export::query::tests::notebooklm_export_query_file_has_no_export_row_mapping
notebooklm_export::query::tests::notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails
notebooklm_export::query::tests::notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states
notebooklm_export::query::tests::notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection
notebooklm_export::query::tests::opted_in_export_loads_migrated_rows_separately_with_markers
notebooklm_export::renderer::tests::formats_metadata_as_rfc3339
notebooklm_export::renderer::tests::renders_json_compatible_yaml_string_scalars
notebooklm_export::renderer::tests::renders_message_metadata_and_text
notebooklm_export::renderer::tests::renders_migrated_history_scope_metadata
notebooklm_export::renderer::tests::renders_reply_thread_and_reaction_metadata
notebooklm_export::renderer::tests::renders_topic_aware_document_header
notebooklm_export::tests::formats_timestamp_folder_suffix
notebooklm_export::tests::group_member_manifest_records_source_scoped_generated_files
notebooklm_export::tests::keeps_migrated_history_opt_in_in_validated_config
notebooklm_export::tests::load_group_export_inputs_rejects_group_without_telegram_members
notebooklm_export::tests::load_group_export_inputs_rejects_youtube_group_for_hard_validation
notebooklm_export::tests::marker_read_and_write_accept_normal_file
notebooklm_export::tests::marker_read_and_write_reject_existing_symlink_file
notebooklm_export::tests::prefix_chunk_filename_adds_sources_directory_and_prefix
notebooklm_export::tests::reads_legacy_single_source_manifest_after_manifest_expansion
notebooklm_export::tests::remove_generated_files_rejects_invalid_manifest_relative_path
notebooklm_export::tests::remove_generated_files_rejects_symlink_parent_directory
notebooklm_export::tests::removes_generated_files_in_sources_subdirectory
notebooklm_export::tests::render_source_export_filters_messages_and_clears_media_placeholders
notebooklm_export::tests::render_source_export_tracks_empty_migrated_history_warning_and_section_prefix
notebooklm_export::tests::render_source_group_export_errors_when_all_members_empty_after_filters
notebooklm_export::tests::render_source_group_export_keeps_empty_member_skipped_reason_out_of_member_warnings
notebooklm_export::tests::source_member_file_prefix_includes_index_id_and_slug
notebooklm_export::tests::source_member_file_prefix_uses_fallback_slug_for_unsafe_title
notebooklm_export::tests::treats_blank_export_id_as_missing
notebooklm_export::tests::trims_optional_export_id
notebooklm_export::tests::validates_exactly_one_export_scope
notebooklm_export::tests::validates_period_order
notebooklm_export::tests::validates_single_source_scope
notebooklm_export::tests::validates_source_group_scope
notebooklm_export::tests::write_export_file_creates_sources_parent_directory
notebooklm_export::tests::write_export_file_rejects_symlink_parent_directory
notebooklm_export::tests::write_group_export_package_records_group_manifest_and_source_files
process_tree::tests::assigns_a_directly_owned_std_child
process_tree::tests::creates_a_job_object
process_tree::tests::dropping_the_guard_closes_the_job_and_kills_its_children
process_tree::tests::process_tree_guard_can_be_owned_by_async_application_state
process_tree::tests::terminate_failure_remains_reportable_and_retryable
process_tree::tests::terminate_is_idempotent
process_tree::tests::terminates_a_descendant_created_after_assignment
projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources
projects::data_range::tests::project_data_range_includes_telegram_migrated_history_when_requested
projects::data_range::tests::project_data_range_preserves_migrated_history_error_for_unknown_source_type
projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram
projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project
projects::data_range::tests::project_data_range_rejects_mixed_provider_project
projects::data_range::tests::project_data_range_returns_nulls_for_empty_project
projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project
projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds
projects::read_model::tests::list_research_projects_counts_playlist_linked_video_materials
projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout
projects::read_model::tests::list_research_projects_prioritizes_running_and_sorts_active_pinned_updated_first
projects::tests::add_project_sources_is_idempotent_and_lists_ui_ready_rows
projects::tests::create_project_trims_and_rejects_duplicate_names_case_insensitively
projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources
projects::tests::list_project_sources_counts_playlist_linked_video_materials
projects::tests::list_project_sources_includes_catalog_status_last_sync_and_handle
projects::tests::project_scoped_delete_blocks_other_active_and_archived_projects_without_mutation
projects::tests::project_scoped_delete_caps_blocking_projects_and_reports_remaining_count
projects::tests::project_scoped_delete_rejects_invalid_sources_and_missing_links
projects::tests::project_scoped_delete_removes_youtube_video_and_cascaded_materials
projects::tests::project_scoped_delete_schema_source_foreign_keys_are_delete_safe
projects::tests::set_project_archived_toggles_timestamp_and_rejects_missing_project
projects::tests::set_project_pinned_toggles_flag_updates_timestamp_and_rejects_missing_project
prompt_packs::browser_adapter::tests::browser_port_delegates_readiness_submission_and_cancellation_without_narrowing_result
prompt_packs::event_adapter::tests::typed_events_map_to_exact_legacy_ipc_payloads
prompt_packs::runtime_commands::tests::execution_adapter_resolves_api_profile_only_inside_spawned_task
prompt_packs::runtime_commands::tests::execution_adapter_spawns_exactly_once_per_ticket
prompt_packs::runtime_commands::tests::execution_task_reuses_start_pool_without_reacquisition
prompt_packs::source_adapter::tests::load_comment_body_performs_a_fresh_read_with_decompression_fallback
prompt_packs::source_adapter::tests::load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows
prompt_packs::source_adapter::tests::load_source_preserves_caller_order_missing_rows_and_nullables
prompt_packs::source_adapter::tests::load_transcript_segments_orders_segment_index_then_row_id
prompt_packs::source_adapter::tests::load_video_maps_full_nullable_metadata_and_missing_rows
prompt_packs::source_adapter::tests::select_comment_candidates_applies_limit_order_and_decompression_fallback
prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled
prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer
readiness::tests::is_ready_current_requires_ready_status_and_current_version
readiness::tests::mark_failed_returns_failed_state
readiness::tests::mark_stale_only_changes_ready_state
readiness::tests::readiness_status_roundtrips_wire_values
secret_store::tests::in_memory_store_can_fail_each_operation
secret_store::tests::secret_ids_are_stable
secret_store::tests::state_reads_writes_and_deletes_secrets
source_ingest::tests::active_kinds_for_sources_reports_matching_locks_only
source_ingest::tests::lock_allows_different_sources
source_ingest::tests::lock_rejects_concurrent_same_source_operations
source_ingest::tests::lock_releases_when_guard_drops
sources::identity::tests::canonical_external_id_rejects_malformed_values
sources::identity::tests::load_telegram_identity_returns_typed_row
sources::identity::tests::load_telegram_runtime_source_pairs_source_with_typed_identity
sources::identity::tests::peer_kind_matches_telegram_subtype
sources::identity::tests::username_normalization_removes_url_and_at_syntax
sources::identity_repair::tests::apply_repair_creates_typed_identity_and_keeps_source_id
sources::identity_repair::tests::apply_repair_is_idempotent
sources::identity_repair::tests::dry_run_reports_repair_without_writing_typed_rows
sources::identity_repair::tests::duplicate_canonical_identity_reports_conflicting_source_ids
sources::identity_repair::tests::duplicate_typed_peer_identity_reports_conflicting_source_ids
sources::identity_repair::tests::fatal_repair_rolls_back_and_does_not_create_canonical_index
sources::identity_repair::tests::malformed_external_ids_fail_without_writing_typed_rows
sources::identity_repair::tests::missing_account_id_is_fatal
sources::identity_repair::tests::repair_creates_minimal_typed_identity_when_legacy_metadata_is_missing_or_malformed
sources::identity_repair::tests::repair_fails_on_conflicting_typed_projection_drift
sources::identity_repair::tests::repair_fails_when_canonical_identity_is_invalid_even_with_legacy_peer_metadata
sources::identity_repair::tests::repair_ignores_malformed_metadata_when_canonical_identity_is_present
sources::identity_repair::tests::repair_ignores_optional_enrichment_gaps_when_typed_identity_is_valid
sources::identity_repair::tests::repair_reads_post_v19_sources_without_legacy_column
sources::identity_repair::tests::repair_rejects_zero_external_id
sources::identity_repair::tests::repair_skips_malformed_metadata_when_typed_identity_is_valid
sources::identity_repair::tests::repair_treats_typed_projection_mismatch_as_fatal
sources::identity_repair::tests::repair_updates_non_conflicting_typed_projection_drift
sources::identity_repair::tests::repair_uses_canonical_subtype_without_legacy_kind
sources::identity_repair::tests::source_identity_gate_blocks_while_running
sources::identity_repair::tests::source_identity_gate_returns_startup_failure
sources::identity_repair::tests::youtube_sources_are_unaffected_by_source_identity_repair
sources::items::query::tests::archive_reader_matches_items_path_for_source_browsing_rows
sources::items::query::tests::archive_reader_matches_topic_filter_and_around_item_semantics
sources::items::query::tests::default_items_path_excludes_migrated_history_rows
sources::items::query::tests::default_source_browsing_does_not_surface_migrated_rows_after_archive_ready
sources::items::query::tests::load_item_rows_attaches_topic_metadata_and_root_matches
sources::items::query::tests::load_item_rows_can_start_at_selected_item
sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_not_ready
sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_stale
sources::items::query::tests::merged_browsing_uses_full_cursor_tuple_for_equal_timestamps
sources::items::query::tests::scoped_browsing_can_load_only_migrated_rows_with_labels
sources::items::query::tests::scoped_browsing_defaults_to_current_rows
sources::items::query::tests::telegram_load_item_rows_uses_items_path_when_archive_model_is_ready
sources::items::query::tests::topic_filters_are_rejected_for_non_current_history_scope
sources::items::query::tests::uncategorized_filter_returns_empty_when_topic_resolution_is_not_ready
sources::items::tests::forum_topic_filter_deserializes_camel_case_topic_id
sources::items::tests::insert_telegram_source_item_allows_same_message_id_in_different_history_domains
sources::items::tests::insert_telegram_source_item_resolves_topic_membership_only_for_new_item
sources::items::tests::insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload
sources::items::tests::insert_telegram_source_item_writes_payload_and_skips_duplicates
sources::items::tests::list_source_items_enriches_youtube_comment_rows_from_raw_payload
sources::items::tests::list_source_items_keeps_base_youtube_comment_when_raw_payload_is_malformed
sources::items::tests::media_metadata_roundtrip_through_zstd
sources::items::tests::migrated_insert_idempotency_uses_old_chat_native_identity
sources::items::tests::migrated_small_group_insert_skips_current_history_derived_writes
sources::items::tests::scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item
sources::items::tests::single_telegram_insert_maintains_ready_archive_model
sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build
sources::items::tests::telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate
sources::items::tests::telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows
sources::items::tests::telegram_insert_writes_analysis_document_in_same_writer_transaction
sources::items::tests::text_roundtrip_through_zstd
sources::items::tests::upsert_youtube_comment_item_updates_existing_text_and_reaction_count
sources::items::tests::upsert_youtube_transcript_item_updates_existing_text_and_returns_id
sources::items::tests::youtube_comment_upsert_targets_non_telegram_partial_unique_index
sources::items::tests::youtube_comment_upsert_writes_analysis_document_and_updates_content
sources::items::tests::youtube_transcript_upsert_targets_non_telegram_partial_unique_index
sources::legacy_metadata_cleanup::tests::audit_ignores_non_telegram_and_null_metadata_rows
sources::legacy_metadata_cleanup::tests::audit_reports_eligible_legacy_telegram_metadata_without_mutating
sources::legacy_metadata_cleanup::tests::audit_skips_invalid_typed_identity
sources::legacy_metadata_cleanup::tests::audit_skips_missing_typed_identity
sources::legacy_metadata_cleanup::tests::audit_skips_subtype_and_account_mismatches
sources::legacy_metadata_cleanup::tests::audit_skips_unsupported_subtype_and_missing_account
sources::legacy_metadata_cleanup::tests::candidate_skip_reason_rejects_unparseable_typed_identity_values
sources::legacy_metadata_cleanup::tests::clear_is_idempotent_after_eligible_metadata_is_removed
sources::legacy_metadata_cleanup::tests::clear_nulls_only_eligible_legacy_telegram_metadata
sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids
sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_empty_refs_as_validation
sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_private_links
sources::peer_resolution::manual_ref::tests::parse_username_accepts_username_and_t_me_links
sources::peer_resolution::tests::add_source_resolution_strategy_distinguishes_username_and_dialog_flows
sources::peer_resolution::tests::source_metadata_decode_failures_are_internal
sources::peer_resolution::tests::source_metadata_decodes_old_dialog_payloads_into_peer_identity
sources::peer_resolution::tests::source_metadata_decodes_old_username_only_payloads
sources::peer_resolution::tests::source_metadata_decodes_typed_peer_identity_payloads
sources::peer_resolution::tests::source_peer_input_rejects_malformed_external_id_as_validation
sources::peer_resolution::tests::source_peer_input_rejects_unsupported_source_type_as_validation
sources::peer_resolution::tests::source_peer_resolution_failure_explains_small_group_dialog_dependency
sources::peer_resolution::tests::source_peer_resolution_plan_prefers_explicit_strategy_order
sources::peer_resolution::tests::typed_identity_plan_allows_username_resolution_without_access_hash
sources::peer_resolution::tests::typed_identity_plan_keeps_dialog_group_dependent_on_dialog_scan
sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_channel_stored_peer_when_access_hash_exists
sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_supergroup_stored_peer_when_access_hash_exists
sources::peer_resolution::tests::typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists
sources::peer_resolution::tests::typed_identity_plan_skips_unusable_stored_peer_when_access_hash_is_missing
sources::settings::tests::initial_sync_policy_label_formats_messages_and_days
sources::settings::tests::sync_settings_default_when_app_settings_are_missing
sources::settings::tests::sync_settings_roundtrip_through_app_settings
sources::settings::tests::validate_sync_settings_rejects_out_of_range_values
sources::store::tests::avatar_cache_key_skips_non_telegram_metadata
sources::store::tests::delete_source_from_pool_enables_foreign_keys_and_cascades_dependents
sources::store::tests::delete_source_is_blocked_when_source_is_used_by_project
sources::store::tests::delete_source_waits_for_temporary_database_write_lock
sources::store::tests::dialog_picked_channel_writes_dialog_typed_identity_with_access_hash
sources::store::tests::dialog_picked_group_writes_dialog_dependent_typed_identity_without_access_hash
sources::store::tests::dialog_picked_supergroup_writes_dialog_typed_identity_with_access_hash
sources::store::tests::list_sources_exposes_migrated_history_counts_without_old_chat_identity
sources::store::tests::list_sources_exposes_sanitized_migrated_history_status_without_chat_id
sources::store::tests::load_source_returns_not_found_for_missing_source
sources::store::tests::source_record_parts_allow_non_telegram_source
sources::store::tests::source_record_parts_emit_only_source_subtype
sources::store::tests::telegram_identity_allows_same_peer_on_different_accounts
sources::store::tests::telegram_identity_rejects_same_account_peer_conflict_at_typed_boundary
sources::store::tests::telegram_source_upsert_inserts_null_metadata
sources::store::tests::telegram_source_upsert_preserves_existing_legacy_metadata_blob
sources::store::tests::telegram_source_upsert_rolls_back_source_when_typed_identity_fails
sources::store::tests::telegram_source_upsert_writes_required_identity_and_available_optional_fields
sources::store::tests::upsert_youtube_playlist_source_handles_legacy_not_null_telegram_kind
sources::store::tests::upsert_youtube_playlist_source_writes_typed_row_and_null_source_metadata
sources::store::tests::upsert_youtube_video_source_conflict_clears_existing_legacy_blob
sources::store::tests::upsert_youtube_video_source_handles_legacy_not_null_telegram_kind
sources::store::tests::upsert_youtube_video_source_rejects_invalid_canonical_url_without_source_row
sources::store::tests::upsert_youtube_video_source_writes_typed_row_and_null_source_metadata
sources::sync::tests::determine_sync_policy_only_applies_initial_settings_on_first_sync
sources::sync::tests::finalize_sync_preserves_existing_legacy_metadata_blob
sources::sync::tests::finalize_sync_updates_source_state_and_typed_avatar_cache
sources::sync::tests::sync_provider_accepts_telegram_sources
sources::sync::tests::sync_provider_rejects_manual_youtube_video_sources
sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error
sources::test_support::tests::source_fixture_creates_expected_tables
sources::topics::tests::forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists
sources::topics::tests::forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind
sources::topics::tests::list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket
sources::topics::tests::topic_refresh_rebuilds_materialized_memberships
sources::topics::tests::upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted
sources::types::tests::item_kind_constants_match_persisted_wire_values
sources::types::tests::source_type_serializes_supported_provider_values
sources::types::tests::telegram_source_subtype_parses_from_canonical_source_subtype
sources::types::tests::telegram_source_subtype_parses_supported_values
sources::types::tests::telegram_source_subtype_rejects_unknown_values_as_validation
sources::types::tests::telegram_source_subtype_rejects_unsupported_source_subtype
sources::types::tests::telegram_source_subtype_serializes_as_existing_wire_value
sql_helpers::tests::push_i64_bind_list_binds_values_in_order
takeout_import::forum_topics::tests::completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups
takeout_import::forum_topics::tests::takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize
takeout_import::forum_topics::tests::takeout_forum_topic_refresh_success_records_no_warning
takeout_import::migrated_history::tests::capability_available_is_source_level_and_restart_safe
takeout_import::migrated_history::tests::capability_unavailable_keeps_reason_internal_and_clears_chat_hint
takeout_import::migrated_history::tests::migrated_history_errors_are_typed_for_frontend_behavior
takeout_import::migrated_history::tests::migrated_small_group_identity_uses_native_old_chat_scope
takeout_import::migrated_history::tests::validation_accepts_matching_revalidated_chat_id
takeout_import::migrated_history::tests::validation_rejects_missing_or_changed_revalidated_chat_id
takeout_import::recovery::tests::recovery_state_includes_migrated_history_scope_for_historical_batches
takeout_import::recovery::tests::takeout_recovery_ignores_non_takeout_batches
takeout_import::recovery::tests::takeout_recovery_latest_complete_hides_older_failed
takeout_import::recovery::tests::takeout_recovery_latest_failed_wins_over_older_complete
takeout_import::recovery::tests::takeout_recovery_returns_partial_completed_and_hides_complete
takeout_import::recovery::tests::takeout_recovery_running_with_active_job_is_hidden
takeout_import::recovery::tests::takeout_recovery_running_without_active_job_is_interrupted
takeout_import::recovery::tests::takeout_recovery_source_filter_limits_results
takeout_import::recovery::tests::takeout_recovery_warning_codes_are_unique_sorted_and_message_free
takeout_import::state::tests::active_jobs_for_sources_filters_non_terminal_jobs
takeout_import::state::tests::job_state_can_cancel_and_finish_job
takeout_import::state::tests::job_state_cancels_child_tokens
takeout_import::state::tests::job_state_records_history_scope_for_frontend_labels
takeout_import::state::tests::job_state_rejects_duplicate_active_source_jobs
takeout_import::state::tests::takeout_cancellation_smoke_fixture_finishes_cancelled_and_clears
takeout_import::state::tests::takeout_cancellation_smoke_fixture_tracks_running_job
takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact
takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation
takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues
takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize
takeout_import::tests::historical_batch_completion_does_not_advance_source_watermark
takeout_import::tests::locked_start_allows_only_one_batch_for_same_source
takeout_import::tests::locked_start_conflict_creates_no_provenance_rows
takeout_import::tests::migrated_history_detected_warning_is_sanitized
takeout_import::tests::migrated_history_start_records_use_same_source_takeout_lock
takeout_import::tests::migrated_history_start_requires_available_capability
takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once
takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers
takeout_import::tests::takeout_step_cancel_wrapper_allows_completed_future
takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future
takeout_import::tests::takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists
takeout_import::tests::takeout_subtype_load_uses_typed_identity_not_legacy_kind
takeout_import::validation_diagnostics::tests::takeout_validation_batch_summary_is_durable_and_sanitized
takeout_import::validation_diagnostics::tests::takeout_validation_duplicate_after_normal_sync_summarizes_outcomes
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_caps_samples_deterministically
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_compares_batch_to_canonical_without_content
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_matched_observations_for_aggregates
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_missing_observations_by_identity
takeout_import::validation_diagnostics::tests::takeout_validation_snapshot_delta_uses_explicit_snapshots
takeout_import::validation_diagnostics::tests::takeout_validation_source_snapshot_is_aggregate_and_sanitized
takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_excludes_non_latest_recovery_candidates
takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_is_durable_only
telegram::tests::diagnostic_status_counts_do_not_return_account_ids_or_messages
telegram::tests::legacy_api_hash_migrates_to_secret_store_and_blanks_column
telegram::tests::legacy_api_hash_remains_when_secret_write_fails
telegram::tests::missing_secure_api_hash_for_blank_legacy_account_is_auth_error
telegram::tests::runtime_status_maps_to_existing_wire_strings
telegram::tests::telegram_api_id_out_of_range_returns_typed_validation_error
telegram::tests::telegram_status_and_event_payload_contract_is_exact
telegram_session_store::tests::delete_session_from_path_removes_file_and_key
telegram_session_store::tests::encrypted_session_load_fails_when_key_is_missing
telegram_session_store::tests::legacy_plaintext_session_migrates_to_encrypted_file
telegram_session_store::tests::legacy_plaintext_session_remains_when_keyring_write_fails
telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact
topic_memberships::tests::rebuild_matches_retained_hidden_and_deleted_topics
topic_memberships::tests::rebuild_prioritizes_specific_topic_matches_before_general_fallback
topic_memberships::tests::rebuild_replaces_stale_memberships_and_versions
topic_memberships::tests::rebuild_uses_legacy_root_only_without_typed_child
tx::tests::begin_immediate_commit_persists_changes
tx::tests::begin_immediate_read_then_write_survives_concurrent_writer
tx::tests::begin_immediate_rollback_discards_changes
tx::tests::begin_immediate_with_foreign_keys_enforces_cascade
tx::tests::deferred_read_then_write_hits_busy_snapshot_under_concurrent_writer
tx::tests::finish_manual_transaction_commits_success_result
tx::tests::finish_manual_transaction_rolls_back_error_result
tx::tests::sqlite_ignores_foreign_keys_pragma_inside_open_transaction
youtube::captions::tests::caption_download_args_request_json3_and_vtt_without_media
youtube::captions::tests::caption_selection_honors_explicit_override_before_original_language
youtube::captions::tests::caption_selection_prefers_original_then_preferred_then_english_then_any
youtube::captions::tests::json3_parser_allows_missing_duration
youtube::captions::tests::json3_parser_concatenates_segments_and_preserves_timing
youtube::captions::tests::replace_transcript_segments_deletes_previous_rows_and_inserts_current_segments
youtube::captions::tests::replace_transcript_segments_rebuilds_analysis_documents_by_segment_order
youtube::captions::tests::transcript_external_id_includes_language_and_track_kind
youtube::captions::tests::vtt_parser_reads_cues_and_skips_blank_text
youtube::captions::tests::vtt_parser_rejects_invalid_timing
youtube::comments::tests::comment_published_at_accepts_numbers_strings_and_fallback
youtube::comments::tests::comments_fetch_args_include_bounded_extractor_args
youtube::comments::tests::comments_fetch_timeout_is_longer_than_metadata_preview_timeout
youtube::comments::tests::default_comment_limit_is_bounded
youtube::comments::tests::normalize_comments_flattens_replies_and_warns_for_timestamp_fallbacks
youtube::comments::tests::normalize_comments_truncates_raw_comment_array_before_normalization
youtube::cookies::tests::accepts_empty_cookie_values
youtube::cookies::tests::accepts_http_only_cookie_rows
youtube::cookies::tests::rejects_empty_cookie_text
youtube::cookies::tests::rejects_files_without_cookie_rows
youtube::cookies::tests::rejects_invalid_cookie_text_before_saving_secret
youtube::cookies::tests::stores_reads_and_clears_youtube_cookies_through_secret_store
youtube::cookies::tests::validates_netscape_cookie_rows_without_exposing_values
youtube::detail::tests::list_summaries_uses_source_id_order_and_marks_no_captions_unavailable
youtube::detail::tests::playlist_detail_reports_ordered_items_and_summary_counts
youtube::detail::tests::playlist_detail_uses_typed_linked_video_metadata_with_corrupt_source_blob
youtube::detail::tests::source_summary_missing_typed_metadata_uses_generic_title_without_blob_decode
youtube::detail::tests::summaries_use_typed_video_metadata_with_corrupt_source_blob
youtube::detail::tests::video_detail_includes_safe_source_metadata_without_item_raw_payloads
youtube::detail::tests::video_detail_missing_typed_metadata_returns_controlled_error
youtube::detail::tests::video_detail_reports_synced_transcript_comments_and_playlist_memberships
youtube::dto::tests::availability_status_serializes_as_snake_case
youtube::dto::tests::preview_kind_deserializes_snake_case
youtube::dto::tests::video_form_serializes_short_value
youtube::errors::tests::invalid_youtube_url_maps_to_validation_error
youtube::errors::tests::ytdlp_deleted_failures_map_to_not_found_error
youtube::errors::tests::ytdlp_network_failures_map_to_network_error
youtube::errors::tests::ytdlp_private_failures_map_to_auth_error
youtube::jobs::tests::active_jobs_for_sources_filters_non_terminal_direct_and_related_sources
youtube::jobs::tests::catalog_jobs_for_sources_includes_latest_failed_jobs
youtube::jobs::tests::diagnostic_counts_group_source_jobs_without_ids_or_raw_errors
youtube::jobs::tests::job_state_cancels_child_tokens
youtube::jobs::tests::job_state_finishes_cancel_requested_jobs_as_cancelled
youtube::jobs::tests::job_state_list_filters_before_limit_and_sorts_newest_first
youtube::jobs::tests::job_state_rejects_duplicate_active_scope_but_allows_different_job_types
youtube::jobs::tests::jobs_missing_typed_video_metadata_errors_after_failed_refresh
youtube::jobs::tests::jobs_reload_missing_typed_video_metadata_after_refresh_callback
youtube::jobs::tests::retryable_playlist_video_rows_excludes_auth_deleted_and_removed_entries
youtube::jobs::tests::source_job_cancellation_smoke_fixture_finishes_cancelled_and_clears
youtube::jobs::tests::source_job_cancellation_smoke_fixture_tracks_running_job
youtube::jobs::tests::source_job_step_with_process_cancel_allows_completed_future
youtube::jobs::tests::source_job_step_with_process_cancel_interrupts_pending_future
youtube::jobs::tests::source_job_type_uses_comments_specific_type_for_comments_only_video_sync
youtube::jobs::tests::source_job_workflow_file_has_no_tauri_command_adapters
youtube::jobs::tests::source_jobs_no_longer_decode_source_metadata_blobs
youtube::metadata::tests::availability_values_map_to_statuses
youtube::metadata::tests::playlist_fixture_maps_metadata_entries_and_preview_warning
youtube::metadata::tests::playlist_metadata_page_args_use_adjacent_playlist_range
youtube::metadata::tests::video_fixture_maps_metadata_and_preview_fields
youtube::metadata::tests::video_fixture_missing_optional_fields_maps_to_none
youtube::playlist::tests::playlist_item_video_source_upsert_writes_typed_video_metadata_not_source_blob
youtube::playlist::tests::upsert_playlist_items_can_skip_video_source_materialization
youtube::playlist::tests::upsert_playlist_items_marks_missing_rows_removed
youtube::playlist::tests::upsert_playlist_items_reuses_existing_video_source_and_keeps_unavailable_null
youtube::playlist::tests::upsert_playlist_items_without_materialization_reuses_existing_video_source
youtube::preview::tests::preview_from_playlist_json_returns_playlist_preview
youtube::preview::tests::preview_from_video_json_uses_parsed_url_kind
youtube::process_runtime::tests::cancellation_reaches_all_reserved_operations
youtube::process_runtime::tests::cookie_guard_retains_file_until_detached_reaper_finishes
youtube::process_runtime::tests::detached_reaper_keeps_cookie_until_the_stuck_child_releases
youtube::process_runtime::tests::dropped_caller_keeps_child_and_registry_owned_until_shutdown_reaps_it
youtube::process_runtime::tests::external_source_job_cancellation_reaps_its_managed_operation
youtube::process_runtime::tests::finite_pipe_backpressure_requires_concurrent_drain
youtube::process_runtime::tests::injected_launcher_drains_backpressured_output_before_waiting_for_exit
youtube::process_runtime::tests::injected_nonzero_exit_preserves_not_found_classification_and_releases_registry
youtube::process_runtime::tests::injected_timeout_reap_detaches_stuck_child_and_keeps_cookie_until_release
youtube::process_runtime::tests::injected_wait_error_reaps_the_child_before_releasing_registry
youtube::process_runtime::tests::registry_reserves_an_operation_before_spawn
youtube::process_runtime::tests::shutdown_rejects_new_ytdlp_admission_before_spawn
youtube::process_runtime::tests::spawn_failure_rolls_back_the_registry_reservation
youtube::process_runtime::tests::timeout_fallback_detaches_cookie_until_stuck_child_reaps
youtube::runtime::tests::runtime_status_serializes_with_camel_case_keys
youtube::settings::tests::auth_cookies_load_only_when_auth_is_enabled
youtube::settings::tests::invalid_stored_settings_return_validation_error_with_key
youtube::settings::tests::invalid_youtube_settings_do_not_write_partial_values
youtube::settings::tests::saving_cookies_enables_auth_and_clear_disables_it
youtube::settings::tests::validate_youtube_settings_normalizes_preferred_captions_language
youtube::settings::tests::validate_youtube_settings_rejects_out_of_range_values
youtube::settings::tests::youtube_settings_default_when_app_settings_are_missing
youtube::settings::tests::youtube_settings_roundtrip_through_app_settings
youtube::settings::tests::youtube_settings_serializes_with_camel_case_keys
youtube::source_metadata::tests::playlist_metadata_columns_are_versioned_and_secret_safe
youtube::source_metadata::tests::upsert_video_metadata_maintains_description_document
youtube::source_metadata::tests::video_metadata_columns_include_wire_values_arrays_caption_override_and_sanitized_raw
youtube::source_metadata::tests::video_metadata_rejects_wrong_canonical_url_shape
youtube::source_metadata::tests::video_source_metadata_restores_raw_caption_metadata_for_provider_sync
youtube::thumbnail::tests::accepts_only_allowlisted_https_thumbnail_urls
youtube::thumbnail::tests::bounds_thumbnail_responses_to_one_mib
youtube::thumbnail::tests::builds_the_dedicated_thumbnail_client
youtube::thumbnail::tests::recognizes_supported_image_magic_bytes
youtube::transcript_reader::tests::list_youtube_transcript_segments_can_start_at_selected_time
youtube::transcript_reader::tests::list_youtube_transcript_segments_filters_by_search
youtube::transcript_reader::tests::list_youtube_transcript_segments_pages_by_time_and_id
youtube::transcript_reader::tests::search_escapes_existing_backslashes_before_like_wildcards
youtube::url::tests::parses_live_url
youtube::url::tests::parses_playlist_url
youtube::url::tests::parses_short_youtu_be_url
youtube::url::tests::parses_shorts_url
youtube::url::tests::parses_watch_video_url
youtube::url::tests::rejects_empty_input
youtube::url::tests::rejects_invalid_host
youtube::url::tests::watch_url_with_playlist_parameter_parses_selected_video
youtube::ytdlp::tests::authenticated_command_args_include_cookie_file_path_without_cookie_content
youtube::ytdlp::tests::cookie_file_content_adds_netscape_header_when_missing
youtube::ytdlp::tests::cookie_file_content_preserves_existing_netscape_header
youtube::ytdlp::tests::preview_playlist_args_limit_entries_to_first_fifty
youtube::ytdlp::tests::preview_video_args_use_dump_json_without_shell_fragments
```

### 71 extractum-telegram identities

```text
dto::tests::telegram_item_kind_constant_matches_persisted_wire_value
dto::tests::telegram_message_draft_has_single_persistence_shape
dto::tests::telegram_message_identity_validation_rejects_invalid_values
error::tests::channel_private_detection_reads_rpc_name_from_error_message
error::tests::non_forum_topic_refresh_errors_are_detected
live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure
live::messages::tests::fallback_peer_identity_uses_telegram_history_peer_vocabulary
live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload
live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule
live::messages::tests::reply_peer_context_uses_telegram_peer_kinds
live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget
live::peer::tests::dialog_lookup_misses_are_not_found
live::peer::tests::dialog_lookup_not_found_message_explains_numeric_manual_limit
live::peer::tests::peer_ref_from_identity_ignores_small_groups_without_supported_identity
live::peer::tests::peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation
live::peer::tests::peer_ref_from_identity_uses_channel_access_hash
live::peer::tests::peer_ref_from_identity_uses_supergroup_access_hash
live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes
live::peer::tests::typed_identity_builds_channel_peer_ref_when_access_hash_exists
live::peer::tests::typed_identity_rejects_subtype_peer_kind_mismatch
live::peer::tests::validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype
live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor
media::tests::derive_content_kind_tracks_text_and_media_presence
media::tests::derive_document_media_kind_prefers_specific_signals
runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors
runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure
runtime::tests::client_preserves_missing_account_error_without_authorization_check
runtime::tests::failed_sign_in_retains_pending_attempt
runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner
runtime::tests::missing_account_authentication_is_false
runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt
runtime::tests::sign_in_without_code_request_preserves_auth_error
runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt
session::tests::encrypted_session_load_fails_for_wrong_account_id
session::tests::encrypted_session_load_round_trips
session::tests::generated_session_key_returns_write_only_encoded_secret
session::tests::legacy_json_returns_rewrite_decision
session::tests::missing_encrypted_key_preserves_auth_error
session::tests::saving_session_writes_encrypted_envelope_not_plaintext
session::tests::session_encryption_key_rejects_invalid_length
takeout::export_dc::tests::export_dc_attempt_state_detects_first_fallback_transition
takeout::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors
takeout::export_dc::tests::export_dc_id_applies_tdesktop_shift
takeout::export_dc::tests::export_dc_invoke_does_not_fallback_for_rpc_errors
takeout::export_dc::tests::export_dc_invoke_falls_back_to_home_dc_on_local_error
takeout::export_dc::tests::export_dc_invoke_uses_home_dc_directly_after_fallback
takeout::export_dc::tests::takeout_init_request_uses_source_subtype_flags_and_file_limit
takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots
takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping
takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome
takeout::operations::tests::history_page_and_search_return_owned_takeout_messages
takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity
takeout::operations::tests::only_my_messages_fallback_is_limited_to_channels
takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges
takeout::pagination::tests::descending_fallback_keeps_raw_order_and_moves_to_min_message_id
takeout::pagination::tests::messages_not_modified_response_is_rejected_for_takeout_page
takeout::pagination::tests::messages_response_without_slice_is_terminal_page
takeout::pagination::tests::split_selection_falls_back_when_telegram_returns_no_ranges
takeout::pagination::tests::split_selection_uses_all_ranges_for_small_group
takeout::pagination::tests::split_selection_uses_last_range_for_channel_and_supergroup
takeout::pagination::tests::tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback
takeout::pagination::tests::tdesktop_non_advancing_cursor_restarts_descending_fallback
takeout::pagination::tests::tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id
takeout::raw_parse::tests::parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids
takeout::raw_parse::tests::parses_document_media_kind_filename_and_dimensions
takeout::raw_parse::tests::parses_photo_message_metadata
takeout::raw_parse::tests::parses_text_message_with_reply_and_reactions
takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids
takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id
takeout::raw_parse::tests::skips_empty_raw_messages
takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error
```

### 736 logical union reconciliation

```text
account_deletion::tests::active_direct_source_analysis_run_blocks_owned_source_only
account_deletion::tests::active_group_analysis_run_blocks_when_any_member_source_is_owned
account_deletion::tests::active_llm_request_for_completed_owned_run_blocks_but_provider_test_does_not
account_deletion::tests::active_source_job_on_owned_source_blocks_but_unowned_job_does_not
account_deletion::tests::active_takeout_job_on_owned_source_blocks
account_deletion::tests::blocker_collection_keeps_multiple_categories_for_internal_diagnostics
account_deletion::tests::existing_account_with_zero_sources_passes
account_deletion::tests::missing_account_returns_not_found
account_deletion::tests::source_ingest_lock_on_owned_source_blocks_without_deleting_rows
accounts::tests::creating_account_rolls_back_when_secret_write_fails
accounts::tests::creating_account_writes_api_hash_to_secret_store_only
accounts::tests::deleting_account_removes_secret_after_database_row
accounts::tests::deleting_missing_account_returns_not_found
accounts::tests::secret_cleanup_failure_keeps_deleted_database_row_deleted
analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion
analysis::corpus::tests::live::default_analysis_corpus_excludes_migrated_history_documents
analysis::corpus::tests::live::description_mode_creates_synthetic_description_message
analysis::corpus::tests::live::explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus
analysis::corpus::tests::live::live_corpus_refs_use_local_item_ids
analysis::corpus::tests::live::load_corpus_messages_filters_telegram_to_telegram_message
analysis::corpus::tests::live::load_corpus_messages_filters_youtube_transcript_only_to_transcripts
analysis::corpus::tests::live::load_corpus_messages_includes_youtube_comment_only_in_comments_mode
analysis::corpus::tests::live::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref
analysis::corpus::tests::live::load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content
analysis::corpus::tests::live::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
analysis::corpus::tests::live::preflight_ref_format_matches_corpus_loader_ref_format
analysis::corpus::tests::live::source_group_opt_in_includes_only_members_with_migrated_rows
analysis::corpus::tests::live::youtube_description_missing_typed_metadata_skips_without_decoding_source_blob
analysis::corpus::tests::live::youtube_description_rows_use_typed_metadata_with_corrupt_source_blob
analysis::corpus::tests::live::youtube_transcript_segment_evidence_uses_typed_source_context
analysis::corpus::tests::preflight::preflight_count_matches_loader_for_youtube_corpus_modes
analysis::corpus::tests::preflight::preflight_counts_eligible_text_messages_for_sources
analysis::corpus::tests::preflight::preflight_ignores_media_only_items_without_text_content
analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows
analysis::corpus::tests::source_resolution::resolve_analysis_sources_loads_single_provider_project
analysis::corpus::tests::source_resolution::resolve_analysis_sources_preserves_no_linked_youtube_error_message
analysis::corpus::tests::source_resolution::resolve_analysis_sources_rejects_mixed_provider_project
analysis::corpus::tests::source_resolution::resolve_run_source_ids_loads_project_sources_without_snapshot
analysis::fixtures::tests::active_runs::fixture_active_state_tracks_seeded_running_run
analysis::fixtures::tests::active_runs::fixture_cancel_waiter_marks_running_run_cancelled
analysis::fixtures::tests::clear::clear_deletes_child_rows_through_fixture_parent_ids
analysis::fixtures::tests::clear::clear_preserves_non_fixture_groups_and_members
analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent
analysis::fixtures::tests::harness::fixture_test_pool_has_required_tables
analysis::fixtures::tests::seed::compressed_fixture_fields_are_readable
analysis::fixtures::tests::seed::seed_creates_fixture_runs_with_statuses_templates_and_snapshots
analysis::fixtures::tests::seed::seed_creates_post_sync_reader_content
analysis::fixtures::tests::seed::seed_creates_safe_account_prompt_profile_sources_and_group
analysis::fixtures::tests::seed::seed_creates_sources_that_pass_identity_repair
analysis::fixtures::tests::seed::seed_creates_valid_typed_youtube_detail_metadata
analysis::fixtures::tests::seed::seed_twice_keeps_one_deterministic_fixture_set
analysis::fixtures::tests::snapshot::capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report
analysis::fixtures::tests::snapshot::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot
analysis::fixtures::tests::snapshot::missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages
analysis::fixtures::tests::snapshot::seeded_snapshot_runs_expose_captured_snapshot_state
analysis::fixtures::tests::summary::summary_serializes_with_camel_case_keys
analysis::groups::tests::prepare_analysis_source_group_input_preserves_baseline_error_precedence
analysis::groups::tests::validate_group_source_type_accepts_matching_provider_membership
analysis::groups::tests::validate_group_source_type_rejects_mixed_provider_membership
analysis::groups::tests::validate_group_source_type_rejects_unknown_group_type
analysis::report::tests::capture::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
analysis::store::tests::read_model::list_analysis_run_summaries_filters_project_runs
analysis::store::tests::read_model::list_analysis_run_summaries_matches_all_query_terms_across_any_field
analysis::store::tests::setup::ensure_sources_exist_returns_typed_not_found_error
analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit
analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects
analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot
analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id
analysis::tests_application::report_profile_resolution_failure_prevents_run_creation
analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads
analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels
analysis_documents::tests::document_metadata_envelopes_match_current_evidence_shape
analysis_documents::tests::rebuild_analysis_documents_excludes_migrated_history_rows
analysis_documents::tests::rebuild_source_materializes_text_units_with_document_order
analysis_documents::tests::rebuild_source_removes_stale_documents_and_is_idempotent
analysis_documents::tests::schema_creates_analysis_documents_constraints_and_indexes
apalis_jobs::tests::apalis_jobs_counts_ignore_their_own_active_filter
apalis_jobs::tests::apalis_jobs_decode_failure_returns_redacted_preview_without_json
apalis_jobs::tests::apalis_jobs_limit_excludes_large_payloads_outside_limited_rows
apalis_jobs::tests::apalis_jobs_list_clamps_limit
apalis_jobs::tests::apalis_jobs_list_does_not_mutate_jobs
apalis_jobs::tests::apalis_jobs_list_filters_by_status_job_type_and_search
apalis_jobs::tests::apalis_jobs_list_returns_empty_when_jobs_table_missing
apalis_jobs::tests::apalis_jobs_list_returns_rfc3339_utc_timestamps
apalis_jobs::tests::apalis_jobs_list_returns_rows_from_jobs_table
apalis_jobs::tests::apalis_jobs_list_sorts_by_latest_activity_timestamp
apalis_jobs::tests::apalis_jobs_non_json_result_and_metadata_are_omitted_in_v1
apalis_jobs::tests::apalis_jobs_payloads_are_redacted_and_truncated
apalis_jobs::tests::apalis_jobs_prune_terminal_deletes_only_old_done_killed_and_terminal_failed_jobs
apalis_jobs::tests::apalis_jobs_prune_terminal_returns_zero_when_jobs_table_missing
apalis_jobs::tests::apalis_jobs_row_shape_is_stable_when_optional_columns_are_absent
apalis_jobs::tests::apalis_jobs_schema_probe_documents_local_jobs_table_shape
archive_read_model::tests::create_schema_adds_state_and_item_tables
archive_read_model::tests::current_ready_state_rejects_old_model_version
archive_read_model::tests::rebuild_source_excludes_migrated_history_rows
archive_read_model::tests::rebuild_source_materializes_archive_fidelity_fields
child_process::tests::create_no_window_matches_win32_process_creation_flags
diagnostics::database::tests::database_diagnostics_groups_only_allow_listed_aggregates
diagnostics::database::tests::migration_status_reports_pending_and_failed_versions
diagnostics::dto::tests::diagnostic_summary_fixture_serializes_without_forbidden_sentinels
diagnostics::redaction::tests::redact_json_value_redacts_sensitive_keys_recursively
diagnostics::redaction::tests::redact_text_removes_secret_and_content_patterns
diagnostics::redaction::tests::sanitized_error_message_bounds_unicode_by_chars
diagnostics::redaction::tests::sanitized_error_message_is_bounded
diagnostics::runtime::tests::failed_runtime_check_uses_coarse_summary_without_os_error_text
diagnostics::runtime::tests::secure_storage_failure_does_not_expose_store_error_text
diagnostics::tests::sanitize_diagnostic_error_bounds_and_redacts_command_errors
diagnostics::tests::serialized_diagnostic_summary_preserves_allowed_data_and_excludes_forbidden_data
external_process::tests::admission_wait_consumes_the_shared_graceful_budget
external_process::tests::cleanup_tasks_start_concurrently_and_isolate_error_and_panic
external_process::tests::concurrent_watchdogs_invoke_exit_once
external_process::tests::exhausted_admission_budget_skips_the_cleanup_factory
external_process::tests::injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback
external_process::tests::permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown
external_process::tests::permits_acquired_before_shutdown_are_waited_for
external_process::tests::repeated_start_does_not_replace_code_or_schedule_again
external_process::tests::start_reports_completed_after_watchdog_claims_exit
external_process::tests::start_returns_started_and_schedules_one_watchdog
external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets
external_process::tests::watchdog_exits_with_the_preserved_code_unless_cleanup_completed
gemini_browser::cdp_chrome::tests::drop_falls_back_to_owned_child_shutdown
gemini_browser::cdp_chrome::tests::explicit_shutdown_kills_and_reaps_the_owned_child_once
gemini_browser::cdp_chrome::tests::shutdown_does_not_claim_or_kill_an_already_exited_child
gemini_browser::cdp_chrome::tests::shutdown_reaps_when_the_child_has_already_exited_during_kill
gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_accepts_json_version_response
gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_reports_unreachable_endpoint
gemini_browser::executor::tests::cancelled_run_marks_the_sidecar_transport_tainted
gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json
gemini_browser::jobs::app_tests::apalis_sqlite_status_probe_documents_actual_status_values
gemini_browser::jobs::app_tests::apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job
gemini_browser::jobs::app_tests::apalis_storage_preserves_existing_sqlx_migration_history_table
gemini_browser::jobs::app_tests::apalis_storage_shares_extractum_db_without_locking_app_pool
gemini_browser::jobs::app_tests::apalis_storage_uses_shared_main_extractum_db_identity
gemini_browser::jobs::app_tests::enqueue_duplicate_run_id_returns_conflict
gemini_browser::jobs::app_tests::enqueue_persists_job_before_worker_startup
gemini_browser::jobs::app_tests::failed_gemini_browser_job_is_not_retried
gemini_browser::jobs::app_tests::gemini_browser_jobs_are_built_with_one_total_attempt
gemini_browser::jobs::app_tests::restart_worker_processes_pending_job_after_runtime_restart
gemini_browser::jobs::app_tests::worker_picks_up_job_quickly_after_idle
gemini_browser::sidecar::tests::stderr_drain_consumes_sidecar_output_concurrently
ingest_provenance::tests::completed_zero_observation_batch_is_complete_without_partial_flags
ingest_provenance::tests::create_takeout_batch_inserts_generic_and_detail_rows_atomically
ingest_provenance::tests::migrated_history_deferred_scope_finalizes_partial_and_records_warning_once
ingest_provenance::tests::migrated_small_group_imported_allows_duplicate_only_success
ingest_provenance::tests::migrated_small_group_scope_can_be_marked_running_and_completed
ingest_provenance::tests::mixed_partial_scope_finalizes_as_partial
ingest_provenance::tests::terminal_update_recalculates_counters_and_sanitizes_error
job_helpers::tests::active_job_guards_track_and_release_scoped_jobs
job_helpers::tests::cancellation_state_cancels_child_tokens
job_helpers::tests::cancellation_state_marks_checks_and_clears_jobs
library_sources::tests::catalog_status_for_input_keeps_failed_job_without_detail_empty
library_sources::tests::list_library_catalog_returns_status_capabilities_and_filter_counts
library_sources::tests::list_library_sources_keeps_sources_with_missing_provider_details
library_sources::tests::list_library_sources_returns_youtube_and_telegram_metadata
llm::profiles::tests::active_profile_resolution_loads_key_from_secret_store
llm::profiles::tests::changing_key_scope_without_replacement_is_rejected
llm::profiles::tests::clear_profile_api_key_deletes_secret
llm::profiles::tests::credential_scope_uses_provider_origin_and_effective_port_but_not_path
llm::profiles::tests::delete_profile_fails_if_secret_store_fails_leaving_db_settings_intact
llm::profiles::tests::delete_profile_removes_settings_and_secret_and_resets_active
llm::profiles::tests::empty_save_preserves_existing_secret
llm::profiles::tests::keyed_legacy_profile_materializes_effective_base_url_while_unkeyed_stays_blank
llm::profiles::tests::legacy_remote_http_profile_is_rejected_before_request_configuration
llm::profiles::tests::materialization_write_failure_fails_closed_during_state_load
llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store
llm::profiles::tests::profile_state_lists_multiple_saved_profiles
llm::profiles::tests::provider_access_resolution_uses_configured_key_with_saved_base_url
llm::profiles::tests::provider_access_resolution_uses_saved_key_with_configured_base_url
llm::profiles::tests::set_active_profile_returns_typed_not_found_error
llm::profiles::tests::validate_profile_id_rejects_invalid_characters
llm::tests::llm_command_errors_and_failed_events_keep_distinct_json_shapes
llm::tests::llm_stream_events_serialize_exact_lifecycle_contract
llm::tests::provider_diagnostics_exclude_profile_ids_and_base_urls
migrations::baseline_reset::tests::backup_failure_prevents_migration_history_rewrite
migrations::baseline_reset::tests::classifies_baseline_history_after_post_baseline_migrations
migrations::baseline_reset::tests::classifies_baseline_history_only_when_checksum_matches
migrations::baseline_reset::tests::classifies_old_history_only_when_versions_one_through_twenty_six_are_successful
migrations::baseline_reset::tests::old_history_cutover_backs_up_then_rewrites_only_migration_history
migrations::baseline_reset::tests::rejects_baseline_history_with_wrong_checksum
migrations::baseline_reset::tests::rejects_failed_migration_history
migrations::baseline_reset::tests::rejects_partial_old_history_without_version_twenty_six
migrations::tests::analysis_telegram_history_scope_migration_adds_nullable_checked_column
migrations::tests::app_migrator_accepts_database_with_preexisting_apalis_migration_history
migrations::tests::build_migrations_includes_apalis_sqlite_versions_for_shared_sqlx_history
migrations::tests::build_migrations_includes_prompt_pack_runtime_provider_version_ten
migrations::tests::build_migrations_starts_at_current_schema_baseline
migrations::tests::concurrent_test_migrations_publish_complete_apalis_schemas
migrations::tests::current_schema_baseline_checksum_matches_frozen_reset_boundary
migrations::tests::current_schema_baseline_migration_is_version_one
migrations::tests::fresh_schema_includes_analysis_documents_table_indexes_and_constraints
migrations::tests::fresh_schema_includes_analysis_snapshot_markers
migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints
migrations::tests::fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints
migrations::tests::fresh_schema_includes_projects_redesign_columns_index_and_defaults
migrations::tests::fresh_schema_includes_source_identity_tables_after_sql_managed_migrations
migrations::tests::post_baseline_migration_upgrades_frozen_baseline_for_migrated_history
migrations::tests::prepare_database_skips_cutover_when_database_file_is_missing
migrations::tests::projects_mvp_migration_is_registered
migrations::tests::projects_mvp_schema_applies_to_memory_pool
migrations::tests::prompt_pack_mvp_migration_creates_library_and_run_tables
migrations::tests::prompt_pack_mvp_migration_declares_required_integrity_constraints
migrations::tests::test_migration_batch_rolls_back_schema_and_history_together
migrations::tests::vendored_apalis_sqlite_migrations_match_pinned_dependency
notebooklm_export::chunker::tests::accounts_for_document_overhead_when_splitting
notebooklm_export::chunker::tests::falls_back_to_month_when_year_exceeds_limits
notebooklm_export::chunker::tests::falls_back_to_topic_id_when_topic_title_slug_is_invalid
notebooklm_export::chunker::tests::filters_short_text_without_other_signal
notebooklm_export::chunker::tests::groups_chunks_by_topic_slug
notebooklm_export::chunker::tests::keeps_yearly_group_when_within_limits
notebooklm_export::chunker::tests::splits_by_word_and_byte_limits
notebooklm_export::filename::tests::accepts_safe_relative_child_paths
notebooklm_export::filename::tests::child_paths_stay_under_base
notebooklm_export::filename::tests::rejects_reserved_components
notebooklm_export::filename::tests::rejects_unsafe_relative_child_paths
notebooklm_export::filename::tests::sanitizes_unsafe_filename_parts
notebooklm_export::glossary::tests::aggregates_participants_by_author
notebooklm_export::links::tests::detects_and_trims_http_urls
notebooklm_export::media::tests::renders_numeric_only_media_metadata
notebooklm_export::media::tests::renders_useful_media_placeholder_parts
notebooklm_export::message_mapping::tests::reply_snippet_decode_failures_are_typed_internal_errors
notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_bounded_periods
notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_notebooklm_messages
notebooklm_export::query::tests::corrupt_archive_reply_target_outside_period_fails_archive_loader
notebooklm_export::query::tests::current_export_archive_loader_sets_scope_markers
notebooklm_export::query::tests::export_fixture_rejects_null_published_at_before_loader_parity
notebooklm_export::query::tests::load_export_messages_adds_local_reply_context_outside_period
notebooklm_export::query::tests::load_export_messages_attaches_general_topic_when_topic_header_is_missing
notebooklm_export::query::tests::load_export_messages_attaches_topic_metadata_for_reply_and_root_messages
notebooklm_export::query::tests::load_export_messages_does_not_root_match_non_numeric_external_ids
notebooklm_export::query::tests::load_export_messages_reads_materialized_topic_memberships
notebooklm_export::query::tests::load_export_source_group_exposes_youtube_group_for_hard_validation
notebooklm_export::query::tests::load_export_source_group_keeps_dirty_member_source_type_for_skip_logic
notebooklm_export::query::tests::load_export_source_group_orders_members_by_title_then_id
notebooklm_export::query::tests::load_export_source_rejects_non_telegram_before_message_loader_selection
notebooklm_export::query::tests::load_export_source_uses_canonical_subtype_not_legacy_kind
notebooklm_export::query::tests::migrated_export_reply_lookup_stays_inside_old_history_domain
notebooklm_export::query::tests::notebooklm_archive_export_excludes_migrated_history_rows_even_if_materialized
notebooklm_export::query::tests::notebooklm_default_export_excludes_migrated_history_rows
notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_all_fallback_reasons
notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_missing_and_old_version
notebooklm_export::query::tests::notebooklm_export_loader_selection_uses_archive_for_ready_current_state
notebooklm_export::query::tests::notebooklm_export_query_file_has_no_export_row_mapping
notebooklm_export::query::tests::notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails
notebooklm_export::query::tests::notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states
notebooklm_export::query::tests::notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection
notebooklm_export::query::tests::opted_in_export_loads_migrated_rows_separately_with_markers
notebooklm_export::renderer::tests::formats_metadata_as_rfc3339
notebooklm_export::renderer::tests::renders_json_compatible_yaml_string_scalars
notebooklm_export::renderer::tests::renders_message_metadata_and_text
notebooklm_export::renderer::tests::renders_migrated_history_scope_metadata
notebooklm_export::renderer::tests::renders_reply_thread_and_reaction_metadata
notebooklm_export::renderer::tests::renders_topic_aware_document_header
notebooklm_export::tests::formats_timestamp_folder_suffix
notebooklm_export::tests::group_member_manifest_records_source_scoped_generated_files
notebooklm_export::tests::keeps_migrated_history_opt_in_in_validated_config
notebooklm_export::tests::load_group_export_inputs_rejects_group_without_telegram_members
notebooklm_export::tests::load_group_export_inputs_rejects_youtube_group_for_hard_validation
notebooklm_export::tests::marker_read_and_write_accept_normal_file
notebooklm_export::tests::marker_read_and_write_reject_existing_symlink_file
notebooklm_export::tests::prefix_chunk_filename_adds_sources_directory_and_prefix
notebooklm_export::tests::reads_legacy_single_source_manifest_after_manifest_expansion
notebooklm_export::tests::remove_generated_files_rejects_invalid_manifest_relative_path
notebooklm_export::tests::remove_generated_files_rejects_symlink_parent_directory
notebooklm_export::tests::removes_generated_files_in_sources_subdirectory
notebooklm_export::tests::render_source_export_filters_messages_and_clears_media_placeholders
notebooklm_export::tests::render_source_export_tracks_empty_migrated_history_warning_and_section_prefix
notebooklm_export::tests::render_source_group_export_errors_when_all_members_empty_after_filters
notebooklm_export::tests::render_source_group_export_keeps_empty_member_skipped_reason_out_of_member_warnings
notebooklm_export::tests::source_member_file_prefix_includes_index_id_and_slug
notebooklm_export::tests::source_member_file_prefix_uses_fallback_slug_for_unsafe_title
notebooklm_export::tests::treats_blank_export_id_as_missing
notebooklm_export::tests::trims_optional_export_id
notebooklm_export::tests::validates_exactly_one_export_scope
notebooklm_export::tests::validates_period_order
notebooklm_export::tests::validates_single_source_scope
notebooklm_export::tests::validates_source_group_scope
notebooklm_export::tests::write_export_file_creates_sources_parent_directory
notebooklm_export::tests::write_export_file_rejects_symlink_parent_directory
notebooklm_export::tests::write_group_export_package_records_group_manifest_and_source_files
process_tree::tests::assigns_a_directly_owned_std_child
process_tree::tests::creates_a_job_object
process_tree::tests::dropping_the_guard_closes_the_job_and_kills_its_children
process_tree::tests::process_tree_guard_can_be_owned_by_async_application_state
process_tree::tests::terminate_failure_remains_reportable_and_retryable
process_tree::tests::terminate_is_idempotent
process_tree::tests::terminates_a_descendant_created_after_assignment
projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources
projects::data_range::tests::project_data_range_includes_telegram_migrated_history_when_requested
projects::data_range::tests::project_data_range_preserves_migrated_history_error_for_unknown_source_type
projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram
projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project
projects::data_range::tests::project_data_range_rejects_mixed_provider_project
projects::data_range::tests::project_data_range_returns_nulls_for_empty_project
projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project
projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds
projects::read_model::tests::list_research_projects_counts_playlist_linked_video_materials
projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout
projects::read_model::tests::list_research_projects_prioritizes_running_and_sorts_active_pinned_updated_first
projects::tests::add_project_sources_is_idempotent_and_lists_ui_ready_rows
projects::tests::create_project_trims_and_rejects_duplicate_names_case_insensitively
projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources
projects::tests::list_project_sources_counts_playlist_linked_video_materials
projects::tests::list_project_sources_includes_catalog_status_last_sync_and_handle
projects::tests::project_scoped_delete_blocks_other_active_and_archived_projects_without_mutation
projects::tests::project_scoped_delete_caps_blocking_projects_and_reports_remaining_count
projects::tests::project_scoped_delete_rejects_invalid_sources_and_missing_links
projects::tests::project_scoped_delete_removes_youtube_video_and_cascaded_materials
projects::tests::project_scoped_delete_schema_source_foreign_keys_are_delete_safe
projects::tests::set_project_archived_toggles_timestamp_and_rejects_missing_project
projects::tests::set_project_pinned_toggles_flag_updates_timestamp_and_rejects_missing_project
prompt_packs::browser_adapter::tests::browser_port_delegates_readiness_submission_and_cancellation_without_narrowing_result
prompt_packs::event_adapter::tests::typed_events_map_to_exact_legacy_ipc_payloads
prompt_packs::runtime_commands::tests::execution_adapter_resolves_api_profile_only_inside_spawned_task
prompt_packs::runtime_commands::tests::execution_adapter_spawns_exactly_once_per_ticket
prompt_packs::runtime_commands::tests::execution_task_reuses_start_pool_without_reacquisition
prompt_packs::source_adapter::tests::load_comment_body_performs_a_fresh_read_with_decompression_fallback
prompt_packs::source_adapter::tests::load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows
prompt_packs::source_adapter::tests::load_source_preserves_caller_order_missing_rows_and_nullables
prompt_packs::source_adapter::tests::load_transcript_segments_orders_segment_index_then_row_id
prompt_packs::source_adapter::tests::load_video_maps_full_nullable_metadata_and_missing_rows
prompt_packs::source_adapter::tests::select_comment_candidates_applies_limit_order_and_decompression_fallback
prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled
prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer
readiness::tests::is_ready_current_requires_ready_status_and_current_version
readiness::tests::mark_failed_returns_failed_state
readiness::tests::mark_stale_only_changes_ready_state
readiness::tests::readiness_status_roundtrips_wire_values
secret_store::tests::in_memory_store_can_fail_each_operation
secret_store::tests::secret_ids_are_stable
secret_store::tests::state_reads_writes_and_deletes_secrets
source_ingest::tests::active_kinds_for_sources_reports_matching_locks_only
source_ingest::tests::lock_allows_different_sources
source_ingest::tests::lock_rejects_concurrent_same_source_operations
source_ingest::tests::lock_releases_when_guard_drops
sources::identity::tests::canonical_external_id_rejects_malformed_values
sources::identity::tests::load_telegram_identity_returns_typed_row
sources::identity::tests::load_telegram_runtime_source_pairs_source_with_typed_identity
sources::identity::tests::peer_kind_matches_telegram_subtype
sources::identity::tests::username_normalization_removes_url_and_at_syntax
sources::identity_repair::tests::apply_repair_creates_typed_identity_and_keeps_source_id
sources::identity_repair::tests::apply_repair_is_idempotent
sources::identity_repair::tests::dry_run_reports_repair_without_writing_typed_rows
sources::identity_repair::tests::duplicate_canonical_identity_reports_conflicting_source_ids
sources::identity_repair::tests::duplicate_typed_peer_identity_reports_conflicting_source_ids
sources::identity_repair::tests::fatal_repair_rolls_back_and_does_not_create_canonical_index
sources::identity_repair::tests::malformed_external_ids_fail_without_writing_typed_rows
sources::identity_repair::tests::missing_account_id_is_fatal
sources::identity_repair::tests::repair_creates_minimal_typed_identity_when_legacy_metadata_is_missing_or_malformed
sources::identity_repair::tests::repair_fails_on_conflicting_typed_projection_drift
sources::identity_repair::tests::repair_fails_when_canonical_identity_is_invalid_even_with_legacy_peer_metadata
sources::identity_repair::tests::repair_ignores_malformed_metadata_when_canonical_identity_is_present
sources::identity_repair::tests::repair_ignores_optional_enrichment_gaps_when_typed_identity_is_valid
sources::identity_repair::tests::repair_reads_post_v19_sources_without_legacy_column
sources::identity_repair::tests::repair_rejects_zero_external_id
sources::identity_repair::tests::repair_skips_malformed_metadata_when_typed_identity_is_valid
sources::identity_repair::tests::repair_treats_typed_projection_mismatch_as_fatal
sources::identity_repair::tests::repair_updates_non_conflicting_typed_projection_drift
sources::identity_repair::tests::repair_uses_canonical_subtype_without_legacy_kind
sources::identity_repair::tests::source_identity_gate_blocks_while_running
sources::identity_repair::tests::source_identity_gate_returns_startup_failure
sources::identity_repair::tests::youtube_sources_are_unaffected_by_source_identity_repair
sources::items::query::tests::archive_reader_matches_items_path_for_source_browsing_rows
sources::items::query::tests::archive_reader_matches_topic_filter_and_around_item_semantics
sources::items::query::tests::default_items_path_excludes_migrated_history_rows
sources::items::query::tests::default_source_browsing_does_not_surface_migrated_rows_after_archive_ready
sources::items::query::tests::load_item_rows_attaches_topic_metadata_and_root_matches
sources::items::query::tests::load_item_rows_can_start_at_selected_item
sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_not_ready
sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_stale
sources::items::query::tests::merged_browsing_uses_full_cursor_tuple_for_equal_timestamps
sources::items::query::tests::scoped_browsing_can_load_only_migrated_rows_with_labels
sources::items::query::tests::scoped_browsing_defaults_to_current_rows
sources::items::query::tests::telegram_load_item_rows_uses_items_path_when_archive_model_is_ready
sources::items::query::tests::topic_filters_are_rejected_for_non_current_history_scope
sources::items::query::tests::uncategorized_filter_returns_empty_when_topic_resolution_is_not_ready
sources::items::tests::forum_topic_filter_deserializes_camel_case_topic_id
sources::items::tests::insert_telegram_source_item_allows_same_message_id_in_different_history_domains
sources::items::tests::insert_telegram_source_item_resolves_topic_membership_only_for_new_item
sources::items::tests::insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload
sources::items::tests::insert_telegram_source_item_writes_payload_and_skips_duplicates
sources::items::tests::list_source_items_enriches_youtube_comment_rows_from_raw_payload
sources::items::tests::list_source_items_keeps_base_youtube_comment_when_raw_payload_is_malformed
sources::items::tests::media_metadata_roundtrip_through_zstd
sources::items::tests::migrated_insert_idempotency_uses_old_chat_native_identity
sources::items::tests::migrated_small_group_insert_skips_current_history_derived_writes
sources::items::tests::scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item
sources::items::tests::single_telegram_insert_maintains_ready_archive_model
sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build
sources::items::tests::telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate
sources::items::tests::telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows
sources::items::tests::telegram_insert_writes_analysis_document_in_same_writer_transaction
sources::items::tests::text_roundtrip_through_zstd
sources::items::tests::upsert_youtube_comment_item_updates_existing_text_and_reaction_count
sources::items::tests::upsert_youtube_transcript_item_updates_existing_text_and_returns_id
sources::items::tests::youtube_comment_upsert_targets_non_telegram_partial_unique_index
sources::items::tests::youtube_comment_upsert_writes_analysis_document_and_updates_content
sources::items::tests::youtube_transcript_upsert_targets_non_telegram_partial_unique_index
sources::legacy_metadata_cleanup::tests::audit_ignores_non_telegram_and_null_metadata_rows
sources::legacy_metadata_cleanup::tests::audit_reports_eligible_legacy_telegram_metadata_without_mutating
sources::legacy_metadata_cleanup::tests::audit_skips_invalid_typed_identity
sources::legacy_metadata_cleanup::tests::audit_skips_missing_typed_identity
sources::legacy_metadata_cleanup::tests::audit_skips_subtype_and_account_mismatches
sources::legacy_metadata_cleanup::tests::audit_skips_unsupported_subtype_and_missing_account
sources::legacy_metadata_cleanup::tests::candidate_skip_reason_rejects_unparseable_typed_identity_values
sources::legacy_metadata_cleanup::tests::clear_is_idempotent_after_eligible_metadata_is_removed
sources::legacy_metadata_cleanup::tests::clear_nulls_only_eligible_legacy_telegram_metadata
sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids
sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_empty_refs_as_validation
sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_private_links
sources::peer_resolution::manual_ref::tests::parse_username_accepts_username_and_t_me_links
sources::peer_resolution::tests::add_source_resolution_strategy_distinguishes_username_and_dialog_flows
sources::peer_resolution::tests::source_metadata_decode_failures_are_internal
sources::peer_resolution::tests::source_metadata_decodes_old_dialog_payloads_into_peer_identity
sources::peer_resolution::tests::source_metadata_decodes_old_username_only_payloads
sources::peer_resolution::tests::source_metadata_decodes_typed_peer_identity_payloads
sources::peer_resolution::tests::source_peer_input_rejects_malformed_external_id_as_validation
sources::peer_resolution::tests::source_peer_input_rejects_unsupported_source_type_as_validation
sources::peer_resolution::tests::source_peer_resolution_failure_explains_small_group_dialog_dependency
sources::peer_resolution::tests::source_peer_resolution_plan_prefers_explicit_strategy_order
sources::peer_resolution::tests::typed_identity_plan_allows_username_resolution_without_access_hash
sources::peer_resolution::tests::typed_identity_plan_keeps_dialog_group_dependent_on_dialog_scan
sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_channel_stored_peer_when_access_hash_exists
sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_supergroup_stored_peer_when_access_hash_exists
sources::peer_resolution::tests::typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists
sources::peer_resolution::tests::typed_identity_plan_skips_unusable_stored_peer_when_access_hash_is_missing
sources::settings::tests::initial_sync_policy_label_formats_messages_and_days
sources::settings::tests::sync_settings_default_when_app_settings_are_missing
sources::settings::tests::sync_settings_roundtrip_through_app_settings
sources::settings::tests::validate_sync_settings_rejects_out_of_range_values
sources::store::tests::avatar_cache_key_skips_non_telegram_metadata
sources::store::tests::delete_source_from_pool_enables_foreign_keys_and_cascades_dependents
sources::store::tests::delete_source_is_blocked_when_source_is_used_by_project
sources::store::tests::delete_source_waits_for_temporary_database_write_lock
sources::store::tests::dialog_picked_channel_writes_dialog_typed_identity_with_access_hash
sources::store::tests::dialog_picked_group_writes_dialog_dependent_typed_identity_without_access_hash
sources::store::tests::dialog_picked_supergroup_writes_dialog_typed_identity_with_access_hash
sources::store::tests::list_sources_exposes_migrated_history_counts_without_old_chat_identity
sources::store::tests::list_sources_exposes_sanitized_migrated_history_status_without_chat_id
sources::store::tests::load_source_returns_not_found_for_missing_source
sources::store::tests::source_record_parts_allow_non_telegram_source
sources::store::tests::source_record_parts_emit_only_source_subtype
sources::store::tests::telegram_identity_allows_same_peer_on_different_accounts
sources::store::tests::telegram_identity_rejects_same_account_peer_conflict_at_typed_boundary
sources::store::tests::telegram_source_upsert_inserts_null_metadata
sources::store::tests::telegram_source_upsert_preserves_existing_legacy_metadata_blob
sources::store::tests::telegram_source_upsert_rolls_back_source_when_typed_identity_fails
sources::store::tests::telegram_source_upsert_writes_required_identity_and_available_optional_fields
sources::store::tests::upsert_youtube_playlist_source_handles_legacy_not_null_telegram_kind
sources::store::tests::upsert_youtube_playlist_source_writes_typed_row_and_null_source_metadata
sources::store::tests::upsert_youtube_video_source_conflict_clears_existing_legacy_blob
sources::store::tests::upsert_youtube_video_source_handles_legacy_not_null_telegram_kind
sources::store::tests::upsert_youtube_video_source_rejects_invalid_canonical_url_without_source_row
sources::store::tests::upsert_youtube_video_source_writes_typed_row_and_null_source_metadata
sources::sync::tests::determine_sync_policy_only_applies_initial_settings_on_first_sync
sources::sync::tests::finalize_sync_preserves_existing_legacy_metadata_blob
sources::sync::tests::finalize_sync_updates_source_state_and_typed_avatar_cache
sources::sync::tests::sync_provider_accepts_telegram_sources
sources::sync::tests::sync_provider_rejects_manual_youtube_video_sources
sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error
sources::test_support::tests::source_fixture_creates_expected_tables
sources::topics::tests::forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists
sources::topics::tests::forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind
sources::topics::tests::list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket
sources::topics::tests::topic_refresh_rebuilds_materialized_memberships
sources::topics::tests::upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted
sources::types::tests::item_kind_constants_match_persisted_wire_values
sources::types::tests::source_type_serializes_supported_provider_values
sources::types::tests::telegram_source_subtype_parses_from_canonical_source_subtype
sources::types::tests::telegram_source_subtype_parses_supported_values
sources::types::tests::telegram_source_subtype_rejects_unknown_values_as_validation
sources::types::tests::telegram_source_subtype_rejects_unsupported_source_subtype
sources::types::tests::telegram_source_subtype_serializes_as_existing_wire_value
sql_helpers::tests::push_i64_bind_list_binds_values_in_order
takeout_import::forum_topics::tests::completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups
takeout_import::forum_topics::tests::takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize
takeout_import::forum_topics::tests::takeout_forum_topic_refresh_success_records_no_warning
takeout_import::migrated_history::tests::capability_available_is_source_level_and_restart_safe
takeout_import::migrated_history::tests::capability_unavailable_keeps_reason_internal_and_clears_chat_hint
takeout_import::migrated_history::tests::migrated_history_errors_are_typed_for_frontend_behavior
takeout_import::migrated_history::tests::migrated_small_group_identity_uses_native_old_chat_scope
takeout_import::migrated_history::tests::validation_accepts_matching_revalidated_chat_id
takeout_import::migrated_history::tests::validation_rejects_missing_or_changed_revalidated_chat_id
takeout_import::recovery::tests::recovery_state_includes_migrated_history_scope_for_historical_batches
takeout_import::recovery::tests::takeout_recovery_ignores_non_takeout_batches
takeout_import::recovery::tests::takeout_recovery_latest_complete_hides_older_failed
takeout_import::recovery::tests::takeout_recovery_latest_failed_wins_over_older_complete
takeout_import::recovery::tests::takeout_recovery_returns_partial_completed_and_hides_complete
takeout_import::recovery::tests::takeout_recovery_running_with_active_job_is_hidden
takeout_import::recovery::tests::takeout_recovery_running_without_active_job_is_interrupted
takeout_import::recovery::tests::takeout_recovery_source_filter_limits_results
takeout_import::recovery::tests::takeout_recovery_warning_codes_are_unique_sorted_and_message_free
takeout_import::state::tests::active_jobs_for_sources_filters_non_terminal_jobs
takeout_import::state::tests::job_state_can_cancel_and_finish_job
takeout_import::state::tests::job_state_cancels_child_tokens
takeout_import::state::tests::job_state_records_history_scope_for_frontend_labels
takeout_import::state::tests::job_state_rejects_duplicate_active_source_jobs
takeout_import::state::tests::takeout_cancellation_smoke_fixture_finishes_cancelled_and_clears
takeout_import::state::tests::takeout_cancellation_smoke_fixture_tracks_running_job
takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact
takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation
takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues
takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize
takeout_import::tests::historical_batch_completion_does_not_advance_source_watermark
takeout_import::tests::locked_start_allows_only_one_batch_for_same_source
takeout_import::tests::locked_start_conflict_creates_no_provenance_rows
takeout_import::tests::migrated_history_detected_warning_is_sanitized
takeout_import::tests::migrated_history_start_records_use_same_source_takeout_lock
takeout_import::tests::migrated_history_start_requires_available_capability
takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once
takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers
takeout_import::tests::takeout_step_cancel_wrapper_allows_completed_future
takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future
takeout_import::tests::takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists
takeout_import::tests::takeout_subtype_load_uses_typed_identity_not_legacy_kind
takeout_import::validation_diagnostics::tests::takeout_validation_batch_summary_is_durable_and_sanitized
takeout_import::validation_diagnostics::tests::takeout_validation_duplicate_after_normal_sync_summarizes_outcomes
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_caps_samples_deterministically
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_compares_batch_to_canonical_without_content
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_matched_observations_for_aggregates
takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_missing_observations_by_identity
takeout_import::validation_diagnostics::tests::takeout_validation_snapshot_delta_uses_explicit_snapshots
takeout_import::validation_diagnostics::tests::takeout_validation_source_snapshot_is_aggregate_and_sanitized
takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_excludes_non_latest_recovery_candidates
takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_is_durable_only
telegram::tests::diagnostic_status_counts_do_not_return_account_ids_or_messages
telegram::tests::legacy_api_hash_migrates_to_secret_store_and_blanks_column
telegram::tests::legacy_api_hash_remains_when_secret_write_fails
telegram::tests::missing_secure_api_hash_for_blank_legacy_account_is_auth_error
telegram::tests::runtime_status_maps_to_existing_wire_strings
telegram::tests::telegram_api_id_out_of_range_returns_typed_validation_error
telegram::tests::telegram_status_and_event_payload_contract_is_exact
telegram_impl::dto::tests::telegram_item_kind_constant_matches_persisted_wire_value
telegram_impl::dto::tests::telegram_message_draft_has_single_persistence_shape
telegram_impl::dto::tests::telegram_message_identity_validation_rejects_invalid_values
telegram_impl::error::tests::channel_private_detection_reads_rpc_name_from_error_message
telegram_impl::error::tests::non_forum_topic_refresh_errors_are_detected
telegram_impl::live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure
telegram_impl::live::messages::tests::fallback_peer_identity_uses_telegram_history_peer_vocabulary
telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload
telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule
telegram_impl::live::messages::tests::reply_peer_context_uses_telegram_peer_kinds
telegram_impl::live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget
telegram_impl::live::peer::tests::dialog_lookup_misses_are_not_found
telegram_impl::live::peer::tests::dialog_lookup_not_found_message_explains_numeric_manual_limit
telegram_impl::live::peer::tests::peer_ref_from_identity_ignores_small_groups_without_supported_identity
telegram_impl::live::peer::tests::peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation
telegram_impl::live::peer::tests::peer_ref_from_identity_uses_channel_access_hash
telegram_impl::live::peer::tests::peer_ref_from_identity_uses_supergroup_access_hash
telegram_impl::live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes
telegram_impl::live::peer::tests::typed_identity_builds_channel_peer_ref_when_access_hash_exists
telegram_impl::live::peer::tests::typed_identity_rejects_subtype_peer_kind_mismatch
telegram_impl::live::peer::tests::validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype
telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor
telegram_impl::media::tests::derive_content_kind_tracks_text_and_media_presence
telegram_impl::media::tests::derive_document_media_kind_prefers_specific_signals
telegram_impl::runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors
telegram_impl::runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure
telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check
telegram_impl::runtime::tests::failed_sign_in_retains_pending_attempt
telegram_impl::runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner
telegram_impl::runtime::tests::missing_account_authentication_is_false
telegram_impl::runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt
telegram_impl::runtime::tests::sign_in_without_code_request_preserves_auth_error
telegram_impl::runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt
telegram_impl::session::tests::encrypted_session_load_fails_for_wrong_account_id
telegram_impl::session::tests::encrypted_session_load_round_trips
telegram_impl::session::tests::generated_session_key_returns_write_only_encoded_secret
telegram_impl::session::tests::legacy_json_returns_rewrite_decision
telegram_impl::session::tests::missing_encrypted_key_preserves_auth_error
telegram_impl::session::tests::saving_session_writes_encrypted_envelope_not_plaintext
telegram_impl::session::tests::session_encryption_key_rejects_invalid_length
telegram_impl::takeout::export_dc::tests::export_dc_attempt_state_detects_first_fallback_transition
telegram_impl::takeout::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors
telegram_impl::takeout::export_dc::tests::export_dc_id_applies_tdesktop_shift
telegram_impl::takeout::export_dc::tests::export_dc_invoke_does_not_fallback_for_rpc_errors
telegram_impl::takeout::export_dc::tests::export_dc_invoke_falls_back_to_home_dc_on_local_error
telegram_impl::takeout::export_dc::tests::export_dc_invoke_uses_home_dc_directly_after_fallback
telegram_impl::takeout::export_dc::tests::takeout_init_request_uses_source_subtype_flags_and_file_limit
telegram_impl::takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots
telegram_impl::takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping
telegram_impl::takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome
telegram_impl::takeout::operations::tests::history_page_and_search_return_owned_takeout_messages
telegram_impl::takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity
telegram_impl::takeout::operations::tests::only_my_messages_fallback_is_limited_to_channels
telegram_impl::takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges
telegram_impl::takeout::pagination::tests::descending_fallback_keeps_raw_order_and_moves_to_min_message_id
telegram_impl::takeout::pagination::tests::messages_not_modified_response_is_rejected_for_takeout_page
telegram_impl::takeout::pagination::tests::messages_response_without_slice_is_terminal_page
telegram_impl::takeout::pagination::tests::split_selection_falls_back_when_telegram_returns_no_ranges
telegram_impl::takeout::pagination::tests::split_selection_uses_all_ranges_for_small_group
telegram_impl::takeout::pagination::tests::split_selection_uses_last_range_for_channel_and_supergroup
telegram_impl::takeout::pagination::tests::tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback
telegram_impl::takeout::pagination::tests::tdesktop_non_advancing_cursor_restarts_descending_fallback
telegram_impl::takeout::pagination::tests::tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id
telegram_impl::takeout::raw_parse::tests::parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids
telegram_impl::takeout::raw_parse::tests::parses_document_media_kind_filename_and_dimensions
telegram_impl::takeout::raw_parse::tests::parses_photo_message_metadata
telegram_impl::takeout::raw_parse::tests::parses_text_message_with_reply_and_reactions
telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids
telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id
telegram_impl::takeout::raw_parse::tests::skips_empty_raw_messages
telegram_impl::takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error
telegram_session_store::tests::delete_session_from_path_removes_file_and_key
telegram_session_store::tests::encrypted_session_load_fails_when_key_is_missing
telegram_session_store::tests::legacy_plaintext_session_migrates_to_encrypted_file
telegram_session_store::tests::legacy_plaintext_session_remains_when_keyring_write_fails
telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact
topic_memberships::tests::rebuild_matches_retained_hidden_and_deleted_topics
topic_memberships::tests::rebuild_prioritizes_specific_topic_matches_before_general_fallback
topic_memberships::tests::rebuild_replaces_stale_memberships_and_versions
topic_memberships::tests::rebuild_uses_legacy_root_only_without_typed_child
tx::tests::begin_immediate_commit_persists_changes
tx::tests::begin_immediate_read_then_write_survives_concurrent_writer
tx::tests::begin_immediate_rollback_discards_changes
tx::tests::begin_immediate_with_foreign_keys_enforces_cascade
tx::tests::deferred_read_then_write_hits_busy_snapshot_under_concurrent_writer
tx::tests::finish_manual_transaction_commits_success_result
tx::tests::finish_manual_transaction_rolls_back_error_result
tx::tests::sqlite_ignores_foreign_keys_pragma_inside_open_transaction
youtube::captions::tests::caption_download_args_request_json3_and_vtt_without_media
youtube::captions::tests::caption_selection_honors_explicit_override_before_original_language
youtube::captions::tests::caption_selection_prefers_original_then_preferred_then_english_then_any
youtube::captions::tests::json3_parser_allows_missing_duration
youtube::captions::tests::json3_parser_concatenates_segments_and_preserves_timing
youtube::captions::tests::replace_transcript_segments_deletes_previous_rows_and_inserts_current_segments
youtube::captions::tests::replace_transcript_segments_rebuilds_analysis_documents_by_segment_order
youtube::captions::tests::transcript_external_id_includes_language_and_track_kind
youtube::captions::tests::vtt_parser_reads_cues_and_skips_blank_text
youtube::captions::tests::vtt_parser_rejects_invalid_timing
youtube::comments::tests::comment_published_at_accepts_numbers_strings_and_fallback
youtube::comments::tests::comments_fetch_args_include_bounded_extractor_args
youtube::comments::tests::comments_fetch_timeout_is_longer_than_metadata_preview_timeout
youtube::comments::tests::default_comment_limit_is_bounded
youtube::comments::tests::normalize_comments_flattens_replies_and_warns_for_timestamp_fallbacks
youtube::comments::tests::normalize_comments_truncates_raw_comment_array_before_normalization
youtube::cookies::tests::accepts_empty_cookie_values
youtube::cookies::tests::accepts_http_only_cookie_rows
youtube::cookies::tests::rejects_empty_cookie_text
youtube::cookies::tests::rejects_files_without_cookie_rows
youtube::cookies::tests::rejects_invalid_cookie_text_before_saving_secret
youtube::cookies::tests::stores_reads_and_clears_youtube_cookies_through_secret_store
youtube::cookies::tests::validates_netscape_cookie_rows_without_exposing_values
youtube::detail::tests::list_summaries_uses_source_id_order_and_marks_no_captions_unavailable
youtube::detail::tests::playlist_detail_reports_ordered_items_and_summary_counts
youtube::detail::tests::playlist_detail_uses_typed_linked_video_metadata_with_corrupt_source_blob
youtube::detail::tests::source_summary_missing_typed_metadata_uses_generic_title_without_blob_decode
youtube::detail::tests::summaries_use_typed_video_metadata_with_corrupt_source_blob
youtube::detail::tests::video_detail_includes_safe_source_metadata_without_item_raw_payloads
youtube::detail::tests::video_detail_missing_typed_metadata_returns_controlled_error
youtube::detail::tests::video_detail_reports_synced_transcript_comments_and_playlist_memberships
youtube::dto::tests::availability_status_serializes_as_snake_case
youtube::dto::tests::preview_kind_deserializes_snake_case
youtube::dto::tests::video_form_serializes_short_value
youtube::errors::tests::invalid_youtube_url_maps_to_validation_error
youtube::errors::tests::ytdlp_deleted_failures_map_to_not_found_error
youtube::errors::tests::ytdlp_network_failures_map_to_network_error
youtube::errors::tests::ytdlp_private_failures_map_to_auth_error
youtube::jobs::tests::active_jobs_for_sources_filters_non_terminal_direct_and_related_sources
youtube::jobs::tests::catalog_jobs_for_sources_includes_latest_failed_jobs
youtube::jobs::tests::diagnostic_counts_group_source_jobs_without_ids_or_raw_errors
youtube::jobs::tests::job_state_cancels_child_tokens
youtube::jobs::tests::job_state_finishes_cancel_requested_jobs_as_cancelled
youtube::jobs::tests::job_state_list_filters_before_limit_and_sorts_newest_first
youtube::jobs::tests::job_state_rejects_duplicate_active_scope_but_allows_different_job_types
youtube::jobs::tests::jobs_missing_typed_video_metadata_errors_after_failed_refresh
youtube::jobs::tests::jobs_reload_missing_typed_video_metadata_after_refresh_callback
youtube::jobs::tests::retryable_playlist_video_rows_excludes_auth_deleted_and_removed_entries
youtube::jobs::tests::source_job_cancellation_smoke_fixture_finishes_cancelled_and_clears
youtube::jobs::tests::source_job_cancellation_smoke_fixture_tracks_running_job
youtube::jobs::tests::source_job_step_with_process_cancel_allows_completed_future
youtube::jobs::tests::source_job_step_with_process_cancel_interrupts_pending_future
youtube::jobs::tests::source_job_type_uses_comments_specific_type_for_comments_only_video_sync
youtube::jobs::tests::source_job_workflow_file_has_no_tauri_command_adapters
youtube::jobs::tests::source_jobs_no_longer_decode_source_metadata_blobs
youtube::metadata::tests::availability_values_map_to_statuses
youtube::metadata::tests::playlist_fixture_maps_metadata_entries_and_preview_warning
youtube::metadata::tests::playlist_metadata_page_args_use_adjacent_playlist_range
youtube::metadata::tests::video_fixture_maps_metadata_and_preview_fields
youtube::metadata::tests::video_fixture_missing_optional_fields_maps_to_none
youtube::playlist::tests::playlist_item_video_source_upsert_writes_typed_video_metadata_not_source_blob
youtube::playlist::tests::upsert_playlist_items_can_skip_video_source_materialization
youtube::playlist::tests::upsert_playlist_items_marks_missing_rows_removed
youtube::playlist::tests::upsert_playlist_items_reuses_existing_video_source_and_keeps_unavailable_null
youtube::playlist::tests::upsert_playlist_items_without_materialization_reuses_existing_video_source
youtube::preview::tests::preview_from_playlist_json_returns_playlist_preview
youtube::preview::tests::preview_from_video_json_uses_parsed_url_kind
youtube::process_runtime::tests::cancellation_reaches_all_reserved_operations
youtube::process_runtime::tests::cookie_guard_retains_file_until_detached_reaper_finishes
youtube::process_runtime::tests::detached_reaper_keeps_cookie_until_the_stuck_child_releases
youtube::process_runtime::tests::dropped_caller_keeps_child_and_registry_owned_until_shutdown_reaps_it
youtube::process_runtime::tests::external_source_job_cancellation_reaps_its_managed_operation
youtube::process_runtime::tests::finite_pipe_backpressure_requires_concurrent_drain
youtube::process_runtime::tests::injected_launcher_drains_backpressured_output_before_waiting_for_exit
youtube::process_runtime::tests::injected_nonzero_exit_preserves_not_found_classification_and_releases_registry
youtube::process_runtime::tests::injected_timeout_reap_detaches_stuck_child_and_keeps_cookie_until_release
youtube::process_runtime::tests::injected_wait_error_reaps_the_child_before_releasing_registry
youtube::process_runtime::tests::registry_reserves_an_operation_before_spawn
youtube::process_runtime::tests::shutdown_rejects_new_ytdlp_admission_before_spawn
youtube::process_runtime::tests::spawn_failure_rolls_back_the_registry_reservation
youtube::process_runtime::tests::timeout_fallback_detaches_cookie_until_stuck_child_reaps
youtube::runtime::tests::runtime_status_serializes_with_camel_case_keys
youtube::settings::tests::auth_cookies_load_only_when_auth_is_enabled
youtube::settings::tests::invalid_stored_settings_return_validation_error_with_key
youtube::settings::tests::invalid_youtube_settings_do_not_write_partial_values
youtube::settings::tests::saving_cookies_enables_auth_and_clear_disables_it
youtube::settings::tests::validate_youtube_settings_normalizes_preferred_captions_language
youtube::settings::tests::validate_youtube_settings_rejects_out_of_range_values
youtube::settings::tests::youtube_settings_default_when_app_settings_are_missing
youtube::settings::tests::youtube_settings_roundtrip_through_app_settings
youtube::settings::tests::youtube_settings_serializes_with_camel_case_keys
youtube::source_metadata::tests::playlist_metadata_columns_are_versioned_and_secret_safe
youtube::source_metadata::tests::upsert_video_metadata_maintains_description_document
youtube::source_metadata::tests::video_metadata_columns_include_wire_values_arrays_caption_override_and_sanitized_raw
youtube::source_metadata::tests::video_metadata_rejects_wrong_canonical_url_shape
youtube::source_metadata::tests::video_source_metadata_restores_raw_caption_metadata_for_provider_sync
youtube::thumbnail::tests::accepts_only_allowlisted_https_thumbnail_urls
youtube::thumbnail::tests::bounds_thumbnail_responses_to_one_mib
youtube::thumbnail::tests::builds_the_dedicated_thumbnail_client
youtube::thumbnail::tests::recognizes_supported_image_magic_bytes
youtube::transcript_reader::tests::list_youtube_transcript_segments_can_start_at_selected_time
youtube::transcript_reader::tests::list_youtube_transcript_segments_filters_by_search
youtube::transcript_reader::tests::list_youtube_transcript_segments_pages_by_time_and_id
youtube::transcript_reader::tests::search_escapes_existing_backslashes_before_like_wildcards
youtube::url::tests::parses_live_url
youtube::url::tests::parses_playlist_url
youtube::url::tests::parses_short_youtu_be_url
youtube::url::tests::parses_shorts_url
youtube::url::tests::parses_watch_video_url
youtube::url::tests::rejects_empty_input
youtube::url::tests::rejects_invalid_host
youtube::url::tests::watch_url_with_playlist_parameter_parses_selected_video
youtube::ytdlp::tests::authenticated_command_args_include_cookie_file_path_without_cookie_content
youtube::ytdlp::tests::cookie_file_content_adds_netscape_header_when_missing
youtube::ytdlp::tests::cookie_file_content_preserves_existing_netscape_header
youtube::ytdlp::tests::preview_playlist_args_limit_entries_to_first_fifty
youtube::ytdlp::tests::preview_video_args_use_dump_json_without_shell_fragments
```

## Focused Rust and contract verification

The coupled eight-file boundary run passed on the final physical layout while
the Phase 8 documentation was still pending:

```text

> extractum@0.2.0 test
> node scripts/run-vitest.mjs run src/lib/telegram-crate-boundary-contract.test.ts src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/rust-workspace-core-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/llm-crate-boundary-contract.test.ts src/lib/prompt-pack-crate-boundary-contract.test.ts src/lib/analysis-crate-boundary-contract.test.ts src/lib/analysis-application-contract.test.ts


 RUN  v4.1.5 G:/Develop/Extractum


 Test Files  8 passed (8)
      Tests  261 passed (261)
   Start at  18:06:30
   Duration  55.07s (transform 5.53s, setup 9.50s, import 3.13s, tests 69.37s, environment 4ms)
```

## Full uncommitted verification

#### check-rustfmt.txt

```text

> extractum@0.2.0 check:rustfmt
> cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
```

#### workspace-check.txt

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At line:6 char:94
+ ... ce check' { cargo check --color never --manifest-path src-tauri/Cargo ...
+                 ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-prompt-packs` (lib) generated 1 warning
warning: `extractum-telegram` (lib) generated 9 warnings
warning: unused imports: `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_GROUP`, and `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:12:43
   |
12 |     SourceSyncTarget, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                           ^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TELEGRAM_KIND_CHANNEL`
  --> src\sources\mod.rs:58:52
   |
58 |     NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                                    ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `GeminiBrowserArtifactMode`
 --> src\gemini_browser\jobs.rs:9:41
  |
9 |     DeliveredJobInput, DeliveryOutcome, GeminiBrowserArtifactMode, GeminiBrowserJob,
  |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, `GeminiBrowserRunStatus`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |     ^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: function `preflight_youtube_summary_in_pool` is never used
   --> crates\extractum-prompt-packs\src\youtube_summary\mod.rs:156:21
    |
156 | pub(crate) async fn preflight_youtube_summary_in_pool(
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> crates\extractum-prompt-packs\src\youtube_summary\test_support.rs:625:21
    |
625 | pub(crate) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib) generated 10 warnings (run `cargo fix --lib -p extractum` to apply 5 suggestions)
warning: `extractum` (lib test) generated 14 warnings (5 duplicates) (run `cargo fix --lib -p extractum --tests` to app
ly 4 suggestions)
warning: `extractum-prompt-packs` (lib test) generated 2 warnings
warning: `extractum-telegram` (lib test) generated 4 warnings (4 duplicates)
warning: methods `has_waiter_for_test` and `worker_execution_timeout` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:182:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
182 |     pub(crate) fn has_waiter_for_test(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^^^^^^^^
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib test) generated 3 warnings (2 duplicates)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.01s
```

#### workspace-test.txt

```text
cargo : warning: function `sidecar_unavailable_result` is never used
At line:13 char:88
+ ... est gate' { cargo test --color never --manifest-path src-tauri/Cargo. ...
+                 ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `extractum-prompt-packs` (lib) generated 1 warning
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
warning: unused imports: `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_GROUP`, and `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:12:43
   |
12 |     SourceSyncTarget, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                           ^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TELEGRAM_KIND_CHANNEL`
  --> src\sources\mod.rs:58:52
   |
58 |     NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                                    ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `GeminiBrowserArtifactMode`
 --> src\gemini_browser\jobs.rs:9:41
  |
9 |     DeliveredJobInput, DeliveryOutcome, GeminiBrowserArtifactMode, GeminiBrowserJob,
  |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, `GeminiBrowserRunStatus`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |     ^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: function `preflight_youtube_summary_in_pool` is never used
   --> crates\extractum-prompt-packs\src\youtube_summary\mod.rs:156:21
    |
156 | pub(crate) async fn preflight_youtube_summary_in_pool(
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> crates\extractum-prompt-packs\src\youtube_summary\test_support.rs:625:21
    |
625 | pub(crate) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib) generated 10 warnings (run `cargo fix --lib -p extractum` to apply 5 suggestions)
warning: `extractum` (lib test) generated 14 warnings (5 duplicates) (run `cargo fix --lib -p extractum --tests` to app
ly 4 suggestions)
warning: `extractum-prompt-packs` (lib test) generated 2 warnings
warning: `extractum-telegram` (lib test) generated 4 warnings (4 duplicates)
warning: methods `has_waiter_for_test` and `worker_execution_timeout` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:182:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
182 |     pub(crate) fn has_waiter_for_test(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^^^^^^^^
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib test) generated 3 warnings (2 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.67s
     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_lib-51295d1508d3c6a6.exe)

running 665 tests
test account_deletion::tests::active_source_job_on_owned_source_blocks_but_unowned_job_does_not ... ok
test account_deletion::tests::active_direct_source_analysis_run_blocks_owned_source_only ... ok
test account_deletion::tests::active_group_analysis_run_blocks_when_any_member_source_is_owned ... ok
test account_deletion::tests::active_llm_request_for_completed_owned_run_blocks_but_provider_test_does_not ... ok
test account_deletion::tests::blocker_collection_keeps_multiple_categories_for_internal_diagnostics ... ok
test account_deletion::tests::active_takeout_job_on_owned_source_blocks ... ok
test account_deletion::tests::existing_account_with_zero_sources_passes ... ok
test account_deletion::tests::missing_account_returns_not_found ... ok
test accounts::tests::creating_account_rolls_back_when_secret_write_fails ... ok
test accounts::tests::creating_account_writes_api_hash_to_secret_store_only ... ok
test accounts::tests::deleting_missing_account_returns_not_found ... ok
test account_deletion::tests::source_ingest_lock_on_owned_source_blocks_without_deleting_rows ... ok
test accounts::tests::deleting_account_removes_secret_after_database_row ... ok
test accounts::tests::secret_cleanup_failure_keeps_deleted_database_row_deleted ... ok
test analysis::corpus::tests::live::default_analysis_corpus_excludes_migrated_history_documents ... ok
test analysis::corpus::tests::live::explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus ... ok
test analysis::corpus::tests::live::description_mode_creates_synthetic_description_message ... ok
test analysis::corpus::tests::live::live_corpus_refs_use_local_item_ids ... ok
test analysis::corpus::tests::live::load_corpus_messages_filters_youtube_transcript_only_to_transcripts ... ok
test analysis::corpus::tests::live::load_corpus_messages_filters_telegram_to_telegram_message ... ok
test analysis::corpus::tests::live::load_corpus_messages_includes_youtube_comment_only_in_comments_mode ... ok
test analysis::corpus::tests::live::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref ... ok
test analysis::corpus::tests::live::load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content ... ok
test analysis::corpus::tests::live::preflight_ref_format_matches_corpus_loader_ref_format ... ok
test analysis::corpus::tests::live::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight ... ok
test analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion ... ok
test analysis::corpus::tests::live::source_group_opt_in_includes_only_members_with_migrated_rows ... ok
test analysis::corpus::tests::live::youtube_description_missing_typed_metadata_skips_without_decoding_source_blob ... ok
test analysis::corpus::tests::live::youtube_description_rows_use_typed_metadata_with_corrupt_source_blob ... ok
test analysis::corpus::tests::live::youtube_transcript_segment_evidence_uses_typed_source_context ... ok
test analysis::corpus::tests::preflight::preflight_ignores_media_only_items_without_text_content ... ok
test analysis::corpus::tests::preflight::preflight_count_matches_loader_for_youtube_corpus_modes ... ok
test analysis::corpus::tests::preflight::preflight_counts_eligible_text_messages_for_sources ... ok
test analysis::corpus::tests::source_resolution::resolve_analysis_sources_loads_single_provider_project ... ok
test analysis::corpus::tests::source_resolution::resolve_analysis_sources_preserves_no_linked_youtube_error_message ... ok
test analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows ... ok
test analysis::corpus::tests::source_resolution::resolve_analysis_sources_rejects_mixed_provider_project ... ok
test analysis::corpus::tests::source_resolution::resolve_run_source_ids_loads_project_sources_without_snapshot ... ok
test analysis::fixtures::tests::active_runs::fixture_cancel_waiter_marks_running_run_cancelled ... ok
test analysis::fixtures::tests::active_runs::fixture_active_state_tracks_seeded_running_run ... ok
test analysis::fixtures::tests::clear::clear_deletes_child_rows_through_fixture_parent_ids ... ok
test analysis::fixtures::tests::clear::clear_preserves_non_fixture_groups_and_members ... ok
test analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent ... ok
test analysis::fixtures::tests::harness::fixture_test_pool_has_required_tables ... ok
test analysis::fixtures::tests::seed::compressed_fixture_fields_are_readable ... ok
test analysis::fixtures::tests::seed::seed_creates_fixture_runs_with_statuses_templates_and_snapshots ... ok
test analysis::fixtures::tests::seed::seed_creates_post_sync_reader_content ... ok
test analysis::fixtures::tests::seed::seed_creates_safe_account_prompt_profile_sources_and_group ... ok
test analysis::fixtures::tests::seed::seed_creates_sources_that_pass_identity_repair ... ok
test analysis::fixtures::tests::seed::seed_creates_valid_typed_youtube_detail_metadata ... ok
test analysis::fixtures::tests::snapshot::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot ... ok
test analysis::fixtures::tests::snapshot::capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report ... ok
test analysis::fixtures::tests::summary::summary_serializes_with_camel_case_keys ... ok
test analysis::groups::tests::prepare_analysis_source_group_input_preserves_baseline_error_precedence ... ok
test analysis::fixtures::tests::seed::seed_twice_keeps_one_deterministic_fixture_set ... ok
test analysis::groups::tests::validate_group_source_type_rejects_mixed_provider_membership ... ok
test analysis::groups::tests::validate_group_source_type_rejects_unknown_group_type ... ok
test analysis::groups::tests::validate_group_source_type_accepts_matching_provider_membership ... ok
test analysis::store::tests::read_model::list_analysis_run_summaries_filters_project_runs ... ok
test analysis::report::tests::capture::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases ... ok
test analysis::store::tests::setup::ensure_sources_exist_returns_typed_not_found_error ... ok
test analysis::store::tests::read_model::list_analysis_run_summaries_matches_all_query_terms_across_any_field ... ok
test analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects ... ok
test analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit ... ok
test analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id ... ok
test analysis::fixtures::tests::snapshot::missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages ... ok
test analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot ... ok
test analysis::tests_application::report_profile_resolution_failure_prevents_run_creation ... ok
test analysis::fixtures::tests::snapshot::seeded_snapshot_runs_expose_captured_snapshot_state ... ok
test analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads ... ok
test analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels ... ok
test analysis_documents::tests::document_metadata_envelopes_match_current_evidence_shape ... ok
test analysis_documents::tests::rebuild_analysis_documents_excludes_migrated_history_rows ... ok
test analysis_documents::tests::rebuild_source_materializes_text_units_with_document_order ... ok
test analysis_documents::tests::rebuild_source_removes_stale_documents_and_is_idempotent ... ok
test analysis_documents::tests::schema_creates_analysis_documents_constraints_and_indexes ... ok
test apalis_jobs::tests::apalis_jobs_counts_ignore_their_own_active_filter ... ok
test apalis_jobs::tests::apalis_jobs_decode_failure_returns_redacted_preview_without_json ... ok
test apalis_jobs::tests::apalis_jobs_limit_excludes_large_payloads_outside_limited_rows ... ok
test apalis_jobs::tests::apalis_jobs_list_returns_empty_when_jobs_table_missing ... ok
test apalis_jobs::tests::apalis_jobs_list_clamps_limit ... ok
test apalis_jobs::tests::apalis_jobs_list_does_not_mutate_jobs ... ok
test apalis_jobs::tests::apalis_jobs_list_filters_by_status_job_type_and_search ... ok
test apalis_jobs::tests::apalis_jobs_list_returns_rows_from_jobs_table ... ok
test apalis_jobs::tests::apalis_jobs_list_returns_rfc3339_utc_timestamps ... ok
test apalis_jobs::tests::apalis_jobs_list_sorts_by_latest_activity_timestamp ... ok
test apalis_jobs::tests::apalis_jobs_prune_terminal_returns_zero_when_jobs_table_missing ... ok
test apalis_jobs::tests::apalis_jobs_payloads_are_redacted_and_truncated ... ok
test apalis_jobs::tests::apalis_jobs_row_shape_is_stable_when_optional_columns_are_absent ... ok
test apalis_jobs::tests::apalis_jobs_non_json_result_and_metadata_are_omitted_in_v1 ... ok
test archive_read_model::tests::current_ready_state_rejects_old_model_version ... ok
test archive_read_model::tests::create_schema_adds_state_and_item_tables ... ok
test apalis_jobs::tests::apalis_jobs_schema_probe_documents_local_jobs_table_shape ... ok
test child_process::tests::create_no_window_matches_win32_process_creation_flags ... ok
test apalis_jobs::tests::apalis_jobs_prune_terminal_deletes_only_old_done_killed_and_terminal_failed_jobs ... ok
test archive_read_model::tests::rebuild_source_excludes_migrated_history_rows ... ok
test diagnostics::dto::tests::diagnostic_summary_fixture_serializes_without_forbidden_sentinels ... ok
test diagnostics::redaction::tests::redact_json_value_redacts_sensitive_keys_recursively ... ok
test diagnostics::redaction::tests::redact_text_removes_secret_and_content_patterns ... ok
test archive_read_model::tests::rebuild_source_materializes_archive_fidelity_fields ... ok
test diagnostics::database::tests::migration_status_reports_pending_and_failed_versions ... ok
test diagnostics::redaction::tests::sanitized_error_message_bounds_unicode_by_chars ... ok
test diagnostics::runtime::tests::failed_runtime_check_uses_coarse_summary_without_os_error_text ... ok
test diagnostics::runtime::tests::secure_storage_failure_does_not_expose_store_error_text ... ok
test diagnostics::redaction::tests::sanitized_error_message_is_bounded ... ok
test diagnostics::tests::serialized_diagnostic_summary_preserves_allowed_data_and_excludes_forbidden_data ... ok
test external_process::tests::admission_wait_consumes_the_shared_graceful_budget ... ok
test external_process::tests::cleanup_tasks_start_concurrently_and_isolate_error_and_panic ... ok
test external_process::tests::concurrent_watchdogs_invoke_exit_once ... ok
test external_process::tests::exhausted_admission_budget_skips_the_cleanup_factory ... ok
test external_process::tests::injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback ... ok
test external_process::tests::permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown ... ok
test external_process::tests::repeated_start_does_not_replace_code_or_schedule_again ... ok
test external_process::tests::start_reports_completed_after_watchdog_claims_exit ... ok
test external_process::tests::start_returns_started_and_schedules_one_watchdog ... ok
test external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets ... ok
test external_process::tests::watchdog_exits_with_the_preserved_code_unless_cleanup_completed ... ok
test gemini_browser::cdp_chrome::tests::drop_falls_back_to_owned_child_shutdown ... ok
test gemini_browser::cdp_chrome::tests::explicit_shutdown_kills_and_reaps_the_owned_child_once ... ok
test gemini_browser::cdp_chrome::tests::shutdown_does_not_claim_or_kill_an_already_exited_child ... ok
test gemini_browser::cdp_chrome::tests::shutdown_reaps_when_the_child_has_already_exited_during_kill ... ok
test diagnostics::tests::sanitize_diagnostic_error_bounds_and_redacts_command_errors ... ok
test gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_accepts_json_version_response ... ok
test external_process::tests::permits_acquired_before_shutdown_are_waited_for ... ok
test gemini_browser::executor::tests::cancelled_run_marks_the_sidecar_transport_tainted ... ok
test gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json ... ok
test diagnostics::database::tests::database_diagnostics_groups_only_allow_listed_aggregates ... ok
test gemini_browser::jobs::app_tests::apalis_sqlite_status_probe_documents_actual_status_values ... ok
test gemini_browser::jobs::app_tests::apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job ... ok
test gemini_browser::jobs::app_tests::apalis_storage_uses_shared_main_extractum_db_identity ... ok
test gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_reports_unreachable_endpoint ... ok
test gemini_browser::jobs::app_tests::apalis_storage_preserves_existing_sqlx_migration_history_table ... ok
test gemini_browser::jobs::app_tests::apalis_storage_shares_extractum_db_without_locking_app_pool ... ok
test gemini_browser::jobs::app_tests::enqueue_persists_job_before_worker_startup ... ok
test gemini_browser::jobs::app_tests::gemini_browser_jobs_are_built_with_one_total_attempt ... ok
test gemini_browser::jobs::app_tests::enqueue_duplicate_run_id_returns_conflict ... ok
test gemini_browser::sidecar::tests::stderr_drain_consumes_sidecar_output_concurrently ... ok
test ingest_provenance::tests::completed_zero_observation_batch_is_complete_without_partial_flags ... ok
test ingest_provenance::tests::create_takeout_batch_inserts_generic_and_detail_rows_atomically ... ok
test ingest_provenance::tests::migrated_history_deferred_scope_finalizes_partial_and_records_warning_once ... ok
test ingest_provenance::tests::migrated_small_group_imported_allows_duplicate_only_success ... ok
test gemini_browser::jobs::app_tests::failed_gemini_browser_job_is_not_retried ... ok
test ingest_provenance::tests::migrated_small_group_scope_can_be_marked_running_and_completed ... ok
test ingest_provenance::tests::mixed_partial_scope_finalizes_as_partial ... ok
test job_helpers::tests::active_job_guards_track_and_release_scoped_jobs ... ok
test ingest_provenance::tests::terminal_update_recalculates_counters_and_sanitizes_error ... ok
test job_helpers::tests::cancellation_state_cancels_child_tokens ... ok
test job_helpers::tests::cancellation_state_marks_checks_and_clears_jobs ... ok
test library_sources::tests::catalog_status_for_input_keeps_failed_job_without_detail_empty ... ok
test library_sources::tests::list_library_sources_keeps_sources_with_missing_provider_details ... ok
test gemini_browser::jobs::app_tests::restart_worker_processes_pending_job_after_runtime_restart ... ok
test library_sources::tests::list_library_catalog_returns_status_capabilities_and_filter_counts ... ok
test llm::profiles::tests::changing_key_scope_without_replacement_is_rejected ... ok
test library_sources::tests::list_library_sources_returns_youtube_and_telegram_metadata ... ok
test llm::profiles::tests::credential_scope_uses_provider_origin_and_effective_port_but_not_path ... ok
test llm::profiles::tests::active_profile_resolution_loads_key_from_secret_store ... ok
test llm::profiles::tests::clear_profile_api_key_deletes_secret ... ok
test llm::profiles::tests::delete_profile_fails_if_secret_store_fails_leaving_db_settings_intact ... ok
test llm::profiles::tests::keyed_legacy_profile_materializes_effective_base_url_while_unkeyed_stays_blank ... ok
test llm::profiles::tests::empty_save_preserves_existing_secret ... ok
test llm::profiles::tests::delete_profile_removes_settings_and_secret_and_resets_active ... ok
test llm::profiles::tests::materialization_write_failure_fails_closed_during_state_load ... ok
test llm::profiles::tests::legacy_remote_http_profile_is_rejected_before_request_configuration ... ok
test llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store ... ok
test llm::profiles::tests::profile_state_lists_multiple_saved_profiles ... ok
test llm::profiles::tests::provider_access_resolution_uses_configured_key_with_saved_base_url ... ok
test llm::profiles::tests::validate_profile_id_rejects_invalid_characters ... ok
test llm::profiles::tests::set_active_profile_returns_typed_not_found_error ... ok
test llm::tests::llm_command_errors_and_failed_events_keep_distinct_json_shapes ... ok
test llm::tests::llm_stream_events_serialize_exact_lifecycle_contract ... ok
test llm::profiles::tests::provider_access_resolution_uses_saved_key_with_configured_base_url ... ok
test gemini_browser::jobs::app_tests::worker_picks_up_job_quickly_after_idle ... ok
test migrations::baseline_reset::tests::classifies_baseline_history_after_post_baseline_migrations ... ok
test migrations::baseline_reset::tests::classifies_baseline_history_only_when_checksum_matches ... ok
test llm::tests::provider_diagnostics_exclude_profile_ids_and_base_urls ... ok
test migrations::baseline_reset::tests::classifies_old_history_only_when_versions_one_through_twenty_six_are_successful ... ok
test migrations::baseline_reset::tests::rejects_baseline_history_with_wrong_checksum ... ok
test migrations::baseline_reset::tests::rejects_failed_migration_history ... ok
test migrations::baseline_reset::tests::rejects_partial_old_history_without_version_twenty_six ... ok
test migrations::baseline_reset::tests::backup_failure_prevents_migration_history_rewrite ... ok
test migrations::tests::build_migrations_includes_apalis_sqlite_versions_for_shared_sqlx_history ... ok
test migrations::tests::build_migrations_includes_prompt_pack_runtime_provider_version_ten ... ok
test migrations::tests::build_migrations_starts_at_current_schema_baseline ... ok
test migrations::tests::analysis_telegram_history_scope_migration_adds_nullable_checked_column ... ok
test migrations::tests::app_migrator_accepts_database_with_preexisting_apalis_migration_history ... ok
test migrations::tests::current_schema_baseline_migration_is_version_one ... ok
test migrations::tests::current_schema_baseline_checksum_matches_frozen_reset_boundary ... ok
test migrations::tests::concurrent_test_migrations_publish_complete_apalis_schemas ... ok
test migrations::baseline_reset::tests::old_history_cutover_backs_up_then_rewrites_only_migration_history ... ok
test migrations::tests::fresh_schema_includes_analysis_snapshot_markers ... ok
test migrations::tests::fresh_schema_includes_analysis_documents_table_indexes_and_constraints ... ok
test migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints ... ok
test migrations::tests::fresh_schema_includes_projects_redesign_columns_index_and_defaults ... ok
test migrations::tests::fresh_schema_includes_source_identity_tables_after_sql_managed_migrations ... ok
test migrations::tests::prepare_database_skips_cutover_when_database_file_is_missing ... ok
test migrations::tests::projects_mvp_migration_is_registered ... ok
test migrations::tests::post_baseline_migration_upgrades_frozen_baseline_for_migrated_history ... ok
test migrations::tests::fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints ... ok
test migrations::tests::test_migration_batch_rolls_back_schema_and_history_together ... ok
test migrations::tests::vendored_apalis_sqlite_migrations_match_pinned_dependency ... ok
test notebooklm_export::chunker::tests::accounts_for_document_overhead_when_splitting ... ok
test notebooklm_export::chunker::tests::falls_back_to_month_when_year_exceeds_limits ... ok
test notebooklm_export::chunker::tests::falls_back_to_topic_id_when_topic_title_slug_is_invalid ... ok
test notebooklm_export::chunker::tests::filters_short_text_without_other_signal ... ok
test notebooklm_export::chunker::tests::groups_chunks_by_topic_slug ... ok
test notebooklm_export::chunker::tests::keeps_yearly_group_when_within_limits ... ok
test notebooklm_export::chunker::tests::splits_by_word_and_byte_limits ... ok
test notebooklm_export::filename::tests::accepts_safe_relative_child_paths ... ok
test notebooklm_export::filename::tests::child_paths_stay_under_base ... ok
test notebooklm_export::filename::tests::rejects_reserved_components ... ok
test notebooklm_export::filename::tests::rejects_unsafe_relative_child_paths ... ok
test notebooklm_export::filename::tests::sanitizes_unsafe_filename_parts ... ok
test migrations::tests::projects_mvp_schema_applies_to_memory_pool ... ok
test notebooklm_export::glossary::tests::aggregates_participants_by_author ... ok
test notebooklm_export::links::tests::detects_and_trims_http_urls ... ok
test notebooklm_export::media::tests::renders_numeric_only_media_metadata ... ok
test notebooklm_export::media::tests::renders_useful_media_placeholder_parts ... ok
test migrations::tests::prompt_pack_mvp_migration_declares_required_integrity_constraints ... ok
test notebooklm_export::message_mapping::tests::reply_snippet_decode_failures_are_typed_internal_errors ... ok
test migrations::tests::prompt_pack_mvp_migration_creates_library_and_run_tables ... ok
test notebooklm_export::query::tests::corrupt_archive_reply_target_outside_period_fails_archive_loader ... ok
test notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_notebooklm_messages ... ok
test notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_bounded_periods ... ok
test notebooklm_export::query::tests::current_export_archive_loader_sets_scope_markers ... ok
test notebooklm_export::query::tests::export_fixture_rejects_null_published_at_before_loader_parity ... ok
test notebooklm_export::query::tests::load_export_messages_attaches_general_topic_when_topic_header_is_missing ... ok
test notebooklm_export::query::tests::load_export_messages_adds_local_reply_context_outside_period ... ok
test notebooklm_export::query::tests::load_export_messages_attaches_topic_metadata_for_reply_and_root_messages ... ok
test notebooklm_export::query::tests::load_export_messages_reads_materialized_topic_memberships ... ok
test notebooklm_export::query::tests::load_export_source_group_exposes_youtube_group_for_hard_validation ... ok
test notebooklm_export::query::tests::load_export_messages_does_not_root_match_non_numeric_external_ids ... ok
test notebooklm_export::query::tests::load_export_source_group_keeps_dirty_member_source_type_for_skip_logic ... ok
test notebooklm_export::query::tests::load_export_source_rejects_non_telegram_before_message_loader_selection ... ok
test notebooklm_export::query::tests::load_export_source_uses_canonical_subtype_not_legacy_kind ... ok
test notebooklm_export::query::tests::load_export_source_group_orders_members_by_title_then_id ... ok
test notebooklm_export::query::tests::migrated_export_reply_lookup_stays_inside_old_history_domain ... ok
test notebooklm_export::query::tests::notebooklm_archive_export_excludes_migrated_history_rows_even_if_materialized ... ok
test notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_missing_and_old_version ... ok
test notebooklm_export::query::tests::notebooklm_default_export_excludes_migrated_history_rows ... ok
test notebooklm_export::query::tests::notebooklm_export_query_file_has_no_export_row_mapping ... ok
test notebooklm_export::query::tests::notebooklm_export_loader_selection_uses_archive_for_ready_current_state ... ok
test notebooklm_export::query::tests::notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails ... ok
test notebooklm_export::query::tests::notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection ... ok
test notebooklm_export::renderer::tests::formats_metadata_as_rfc3339 ... ok
test notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_all_fallback_reasons ... ok
test notebooklm_export::renderer::tests::renders_json_compatible_yaml_string_scalars ... ok
test notebooklm_export::renderer::tests::renders_message_metadata_and_text ... ok
test notebooklm_export::renderer::tests::renders_migrated_history_scope_metadata ... ok
test notebooklm_export::renderer::tests::renders_reply_thread_and_reaction_metadata ... ok
test notebooklm_export::renderer::tests::renders_topic_aware_document_header ... ok
test notebooklm_export::tests::formats_timestamp_folder_suffix ... ok
test notebooklm_export::tests::group_member_manifest_records_source_scoped_generated_files ... ok
test notebooklm_export::tests::keeps_migrated_history_opt_in_in_validated_config ... ok
test notebooklm_export::tests::load_group_export_inputs_rejects_youtube_group_for_hard_validation ... ok
test notebooklm_export::tests::marker_read_and_write_accept_normal_file ... ok
test notebooklm_export::query::tests::opted_in_export_loads_migrated_rows_separately_with_markers ... ok
test notebooklm_export::tests::load_group_export_inputs_rejects_group_without_telegram_members ... ok
test notebooklm_export::tests::prefix_chunk_filename_adds_sources_directory_and_prefix ... ok
test notebooklm_export::tests::marker_read_and_write_reject_existing_symlink_file ... ok
test notebooklm_export::tests::remove_generated_files_rejects_symlink_parent_directory ... ok
test notebooklm_export::tests::reads_legacy_single_source_manifest_after_manifest_expansion ... ok
test notebooklm_export::tests::render_source_export_filters_messages_and_clears_media_placeholders ... ok
test notebooklm_export::tests::render_source_export_tracks_empty_migrated_history_warning_and_section_prefix ... ok
test notebooklm_export::tests::render_source_group_export_errors_when_all_members_empty_after_filters ... ok
test notebooklm_export::tests::remove_generated_files_rejects_invalid_manifest_relative_path ... ok
test notebooklm_export::tests::source_member_file_prefix_includes_index_id_and_slug ... ok
test notebooklm_export::tests::render_source_group_export_keeps_empty_member_skipped_reason_out_of_member_warnings ... ok
test notebooklm_export::tests::source_member_file_prefix_uses_fallback_slug_for_unsafe_title ... ok
test notebooklm_export::tests::treats_blank_export_id_as_missing ... ok
test notebooklm_export::tests::trims_optional_export_id ... ok
test notebooklm_export::tests::validates_exactly_one_export_scope ... ok
test notebooklm_export::tests::validates_period_order ... ok
test notebooklm_export::tests::validates_single_source_scope ... ok
test notebooklm_export::tests::validates_source_group_scope ... ok
test notebooklm_export::tests::removes_generated_files_in_sources_subdirectory ... ok
test notebooklm_export::tests::write_export_file_rejects_symlink_parent_directory ... ok
test notebooklm_export::tests::write_export_file_creates_sources_parent_directory ... ok
test process_tree::tests::creates_a_job_object ... ok
test process_tree::tests::assigns_a_directly_owned_std_child ... ok
test notebooklm_export::query::tests::notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states ... ok
test process_tree::tests::process_tree_guard_can_be_owned_by_async_application_state ... ok
test process_tree::tests::dropping_the_guard_closes_the_job_and_kills_its_children ... ok
test process_tree::tests::terminate_failure_remains_reportable_and_retryable ... ok
test notebooklm_export::tests::write_group_export_package_records_group_manifest_and_source_files ... ok
test process_tree::tests::terminate_is_idempotent ... ok
test projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources ... ok
test projects::data_range::tests::project_data_range_includes_telegram_migrated_history_when_requested ... ok
test projects::data_range::tests::project_data_range_preserves_migrated_history_error_for_unknown_source_type ... ok
test projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project ... ok
test projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram ... ok
test projects::data_range::tests::project_data_range_rejects_mixed_provider_project ... ok
test projects::data_range::tests::project_data_range_returns_nulls_for_empty_project ... ok
test projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project ... ok
test projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds ... ok
test projects::read_model::tests::list_research_projects_counts_playlist_linked_video_materials ... ok
test projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout ... ok
test projects::read_model::tests::list_research_projects_prioritizes_running_and_sorts_active_pinned_updated_first ... ok
test projects::tests::add_project_sources_is_idempotent_and_lists_ui_ready_rows ... ok
test projects::tests::create_project_trims_and_rejects_duplicate_names_case_insensitively ... ok
test projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources ... ok
test projects::tests::list_project_sources_includes_catalog_status_last_sync_and_handle ... ok
test projects::tests::list_project_sources_counts_playlist_linked_video_materials ... ok
test projects::tests::project_scoped_delete_blocks_other_active_and_archived_projects_without_mutation ... ok
test projects::tests::project_scoped_delete_rejects_invalid_sources_and_missing_links ... ok
test projects::tests::project_scoped_delete_caps_blocking_projects_and_reports_remaining_count ... ok
test projects::tests::project_scoped_delete_removes_youtube_video_and_cascaded_materials ... ok
test projects::tests::set_project_archived_toggles_timestamp_and_rejects_missing_project ... ok
test prompt_packs::browser_adapter::tests::browser_port_delegates_readiness_submission_and_cancellation_without_narrowing_result ... ok
test prompt_packs::event_adapter::tests::typed_events_map_to_exact_legacy_ipc_payloads ... ok
test projects::tests::project_scoped_delete_schema_source_foreign_keys_are_delete_safe ... ok
test prompt_packs::runtime_commands::tests::execution_adapter_resolves_api_profile_only_inside_spawned_task ... ok
test prompt_packs::runtime_commands::tests::execution_adapter_spawns_exactly_once_per_ticket ... ok
test prompt_packs::runtime_commands::tests::execution_task_reuses_start_pool_without_reacquisition ... ok
test projects::tests::set_project_pinned_toggles_flag_updates_timestamp_and_rejects_missing_project ... ok
test prompt_packs::source_adapter::tests::load_comment_body_performs_a_fresh_read_with_decompression_fallback ... ok
test prompt_packs::source_adapter::tests::load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows ... ok
test prompt_packs::source_adapter::tests::load_source_preserves_caller_order_missing_rows_and_nullables ... ok
test prompt_packs::source_adapter::tests::load_video_maps_full_nullable_metadata_and_missing_rows ... ok
test prompt_packs::source_adapter::tests::load_transcript_segments_orders_segment_index_then_row_id ... ok
test prompt_packs::source_adapter::tests::select_comment_candidates_applies_limit_order_and_decompression_fallback ... ok
test readiness::tests::is_ready_current_requires_ready_status_and_current_version ... ok
test readiness::tests::mark_failed_returns_failed_state ... ok
test readiness::tests::mark_stale_only_changes_ready_state ... ok
test readiness::tests::readiness_status_roundtrips_wire_values ... ok
test prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer ... ok
test prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled ... ok
test secret_store::tests::in_memory_store_can_fail_each_operation ... ok
test secret_store::tests::secret_ids_are_stable ... ok
test source_ingest::tests::active_kinds_for_sources_reports_matching_locks_only ... ok
test source_ingest::tests::lock_allows_different_sources ... ok
test secret_store::tests::state_reads_writes_and_deletes_secrets ... ok
test source_ingest::tests::lock_rejects_concurrent_same_source_operations ... ok
test source_ingest::tests::lock_releases_when_guard_drops ... ok
test sources::identity::tests::canonical_external_id_rejects_malformed_values ... ok
test sources::identity::tests::peer_kind_matches_telegram_subtype ... ok
test sources::identity::tests::username_normalization_removes_url_and_at_syntax ... ok
test sources::identity::tests::load_telegram_runtime_source_pairs_source_with_typed_identity ... ok
test sources::identity::tests::load_telegram_identity_returns_typed_row ... ok
test sources::identity_repair::tests::apply_repair_creates_typed_identity_and_keeps_source_id ... ok
test sources::identity_repair::tests::dry_run_reports_repair_without_writing_typed_rows ... ok
test sources::identity_repair::tests::duplicate_canonical_identity_reports_conflicting_source_ids ... ok
test sources::identity_repair::tests::apply_repair_is_idempotent ... ok
test sources::identity_repair::tests::duplicate_typed_peer_identity_reports_conflicting_source_ids ... ok
test sources::identity_repair::tests::missing_account_id_is_fatal ... ok
test sources::identity_repair::tests::fatal_repair_rolls_back_and_does_not_create_canonical_index ... ok
test sources::identity_repair::tests::repair_fails_on_conflicting_typed_projection_drift ... ok
test sources::identity_repair::tests::repair_fails_when_canonical_identity_is_invalid_even_with_legacy_peer_metadata ... ok
test sources::identity_repair::tests::repair_creates_minimal_typed_identity_when_legacy_metadata_is_missing_or_malformed ... ok
test sources::identity_repair::tests::malformed_external_ids_fail_without_writing_typed_rows ... ok
test sources::identity_repair::tests::repair_ignores_malformed_metadata_when_canonical_identity_is_present ... ok
test sources::identity_repair::tests::repair_ignores_optional_enrichment_gaps_when_typed_identity_is_valid ... ok
test sources::identity_repair::tests::repair_reads_post_v19_sources_without_legacy_column ... ok
test sources::identity_repair::tests::repair_rejects_zero_external_id ... ok
test sources::identity_repair::tests::repair_treats_typed_projection_mismatch_as_fatal ... ok
test sources::identity_repair::tests::repair_skips_malformed_metadata_when_typed_identity_is_valid ... ok
test sources::identity_repair::tests::source_identity_gate_blocks_while_running ... ok
test sources::identity_repair::tests::source_identity_gate_returns_startup_failure ... ok
test sources::identity_repair::tests::repair_uses_canonical_subtype_without_legacy_kind ... ok
test sources::identity_repair::tests::repair_updates_non_conflicting_typed_projection_drift ... ok
test sources::items::query::tests::archive_reader_matches_items_path_for_source_browsing_rows ... ok
test sources::items::query::tests::archive_reader_matches_topic_filter_and_around_item_semantics ... ok
test sources::items::query::tests::default_items_path_excludes_migrated_history_rows ... ok
test sources::items::query::tests::default_source_browsing_does_not_surface_migrated_rows_after_archive_ready ... ok
test sources::items::query::tests::load_item_rows_can_start_at_selected_item ... ok
test sources::items::query::tests::load_item_rows_attaches_topic_metadata_and_root_matches ... ok
test sources::identity_repair::tests::youtube_sources_are_unaffected_by_source_identity_repair ... ok
test sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_not_ready ... ok
test sources::items::query::tests::merged_browsing_uses_full_cursor_tuple_for_equal_timestamps ... ok
test sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_stale ... ok
test sources::items::query::tests::scoped_browsing_can_load_only_migrated_rows_with_labels ... ok
test sources::items::query::tests::scoped_browsing_defaults_to_current_rows ... ok
test sources::items::query::tests::topic_filters_are_rejected_for_non_current_history_scope ... ok
test sources::items::tests::forum_topic_filter_deserializes_camel_case_topic_id ... ok
test sources::items::query::tests::telegram_load_item_rows_uses_items_path_when_archive_model_is_ready ... ok
test sources::items::query::tests::uncategorized_filter_returns_empty_when_topic_resolution_is_not_ready ... ok
test sources::items::tests::insert_telegram_source_item_allows_same_message_id_in_different_history_domains ... ok
test sources::items::tests::insert_telegram_source_item_resolves_topic_membership_only_for_new_item ... ok
test sources::items::tests::insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload ... ok
test sources::items::tests::insert_telegram_source_item_writes_payload_and_skips_duplicates ... ok
test sources::items::tests::media_metadata_roundtrip_through_zstd ... ok
test sources::items::tests::list_source_items_enriches_youtube_comment_rows_from_raw_payload ... ok
test sources::items::tests::list_source_items_keeps_base_youtube_comment_when_raw_payload_is_malformed ... ok
test sources::items::tests::migrated_insert_idempotency_uses_old_chat_native_identity ... ok
test sources::items::tests::migrated_small_group_insert_skips_current_history_derived_writes ... ok
test sources::items::tests::scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item ... ok
test sources::items::tests::single_telegram_insert_maintains_ready_archive_model ... ok
test sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build ... ok
test sources::items::tests::telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate ... ok
test sources::items::tests::text_roundtrip_through_zstd ... ok
test sources::items::tests::telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows ... ok
test sources::items::tests::telegram_insert_writes_analysis_document_in_same_writer_transaction ... ok
test sources::items::tests::upsert_youtube_comment_item_updates_existing_text_and_reaction_count ... ok
test sources::items::tests::upsert_youtube_transcript_item_updates_existing_text_and_returns_id ... ok
test sources::items::tests::youtube_comment_upsert_targets_non_telegram_partial_unique_index ... ok
test sources::items::tests::youtube_comment_upsert_writes_analysis_document_and_updates_content ... ok
test sources::items::tests::youtube_transcript_upsert_targets_non_telegram_partial_unique_index ... ok
test sources::legacy_metadata_cleanup::tests::audit_ignores_non_telegram_and_null_metadata_rows ... ok
test sources::legacy_metadata_cleanup::tests::audit_reports_eligible_legacy_telegram_metadata_without_mutating ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_invalid_typed_identity ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_missing_typed_identity ... ok
test sources::legacy_metadata_cleanup::tests::candidate_skip_reason_rejects_unparseable_typed_identity_values ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_subtype_and_account_mismatches ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_unsupported_subtype_and_missing_account ... ok
test sources::legacy_metadata_cleanup::tests::clear_is_idempotent_after_eligible_metadata_is_removed ... ok
test sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids ... ok
test sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_empty_refs_as_validation ... ok
test sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_private_links ... ok
test sources::peer_resolution::manual_ref::tests::parse_username_accepts_username_and_t_me_links ... ok
test sources::peer_resolution::tests::add_source_resolution_strategy_distinguishes_username_and_dialog_flows ... ok
test sources::peer_resolution::tests::source_metadata_decode_failures_are_internal ... ok
test sources::peer_resolution::tests::source_metadata_decodes_old_dialog_payloads_into_peer_identity ... ok
test sources::peer_resolution::tests::source_metadata_decodes_old_username_only_payloads ... ok
test sources::peer_resolution::tests::source_metadata_decodes_typed_peer_identity_payloads ... ok
test sources::peer_resolution::tests::source_peer_input_rejects_malformed_external_id_as_validation ... ok
test sources::peer_resolution::tests::source_peer_input_rejects_unsupported_source_type_as_validation ... ok
test sources::peer_resolution::tests::source_peer_resolution_failure_explains_small_group_dialog_dependency ... ok
test sources::peer_resolution::tests::source_peer_resolution_plan_prefers_explicit_strategy_order ... ok
test sources::legacy_metadata_cleanup::tests::clear_nulls_only_eligible_legacy_telegram_metadata ... ok
test sources::peer_resolution::tests::typed_identity_plan_allows_username_resolution_without_access_hash ... ok
test sources::peer_resolution::tests::typed_identity_plan_keeps_dialog_group_dependent_on_dialog_scan ... ok
test sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_channel_stored_peer_when_access_hash_exists ... ok
test sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_supergroup_stored_peer_when_access_hash_exists ... ok
test sources::peer_resolution::tests::typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists ... ok
test sources::peer_resolution::tests::typed_identity_plan_skips_unusable_stored_peer_when_access_hash_is_missing ... ok
test sources::settings::tests::initial_sync_policy_label_formats_messages_and_days ... ok
test sources::settings::tests::validate_sync_settings_rejects_out_of_range_values ... ok
test sources::settings::tests::sync_settings_default_when_app_settings_are_missing ... ok
test sources::store::tests::avatar_cache_key_skips_non_telegram_metadata ... ok
test sources::settings::tests::sync_settings_roundtrip_through_app_settings ... ok
test sources::store::tests::delete_source_is_blocked_when_source_is_used_by_project ... ok
test sources::store::tests::dialog_picked_channel_writes_dialog_typed_identity_with_access_hash ... ok
test sources::store::tests::dialog_picked_group_writes_dialog_dependent_typed_identity_without_access_hash ... ok
test sources::store::tests::dialog_picked_supergroup_writes_dialog_typed_identity_with_access_hash ... ok
test sources::store::tests::delete_source_from_pool_enables_foreign_keys_and_cascades_dependents ... ok
test sources::store::tests::list_sources_exposes_migrated_history_counts_without_old_chat_identity ... ok
test sources::store::tests::load_source_returns_not_found_for_missing_source ... ok
test sources::store::tests::source_record_parts_allow_non_telegram_source ... ok
test sources::store::tests::source_record_parts_emit_only_source_subtype ... ok
test sources::store::tests::list_sources_exposes_sanitized_migrated_history_status_without_chat_id ... ok
test sources::store::tests::telegram_identity_allows_same_peer_on_different_accounts ... ok
test sources::store::tests::telegram_identity_rejects_same_account_peer_conflict_at_typed_boundary ... ok
test sources::store::tests::telegram_source_upsert_inserts_null_metadata ... ok
test sources::store::tests::telegram_source_upsert_preserves_existing_legacy_metadata_blob ... ok
test sources::store::tests::telegram_source_upsert_writes_required_identity_and_available_optional_fields ... ok
test sources::store::tests::telegram_source_upsert_rolls_back_source_when_typed_identity_fails ... ok
test sources::store::tests::delete_source_waits_for_temporary_database_write_lock ... ok
test sources::store::tests::upsert_youtube_playlist_source_writes_typed_row_and_null_source_metadata ... ok
test sources::store::tests::upsert_youtube_playlist_source_handles_legacy_not_null_telegram_kind ... ok
test sources::store::tests::upsert_youtube_video_source_conflict_clears_existing_legacy_blob ... ok
test sources::store::tests::upsert_youtube_video_source_handles_legacy_not_null_telegram_kind ... ok
test sources::store::tests::upsert_youtube_video_source_rejects_invalid_canonical_url_without_source_row ... ok
test sources::store::tests::upsert_youtube_video_source_writes_typed_row_and_null_source_metadata ... ok
test sources::sync::tests::determine_sync_policy_only_applies_initial_settings_on_first_sync ... ok
test sources::sync::tests::sync_provider_accepts_telegram_sources ... ok
test sources::sync::tests::finalize_sync_preserves_existing_legacy_metadata_blob ... ok
test sources::sync::tests::sync_provider_rejects_manual_youtube_video_sources ... ok
test sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error ... ok
test sources::sync::tests::finalize_sync_updates_source_state_and_typed_avatar_cache ... ok
test sources::topics::tests::forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists ... ok
test sources::test_support::tests::source_fixture_creates_expected_tables ... ok
test sources::topics::tests::forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind ... ok
test sources::topics::tests::list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket ... ok
test sources::types::tests::item_kind_constants_match_persisted_wire_values ... ok
test sources::types::tests::source_type_serializes_supported_provider_values ... ok
test sources::types::tests::telegram_source_subtype_parses_from_canonical_source_subtype ... ok
test sources::types::tests::telegram_source_subtype_parses_supported_values ... ok
test sources::topics::tests::topic_refresh_rebuilds_materialized_memberships ... ok
test sources::types::tests::telegram_source_subtype_rejects_unknown_values_as_validation ... ok
test sources::types::tests::telegram_source_subtype_rejects_unsupported_source_subtype ... ok
test sources::types::tests::telegram_source_subtype_serializes_as_existing_wire_value ... ok
test takeout_import::forum_topics::tests::completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups ... ok
test sources::topics::tests::upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted ... ok
test sql_helpers::tests::push_i64_bind_list_binds_values_in_order ... ok
test takeout_import::migrated_history::tests::capability_available_is_source_level_and_restart_safe ... ok
test takeout_import::forum_topics::tests::takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize ... ok
test takeout_import::migrated_history::tests::migrated_history_errors_are_typed_for_frontend_behavior ... ok
test takeout_import::migrated_history::tests::migrated_small_group_identity_uses_native_old_chat_scope ... ok
test takeout_import::migrated_history::tests::validation_accepts_matching_revalidated_chat_id ... ok
test takeout_import::migrated_history::tests::validation_rejects_missing_or_changed_revalidated_chat_id ... ok
test takeout_import::forum_topics::tests::takeout_forum_topic_refresh_success_records_no_warning ... ok
test takeout_import::migrated_history::tests::capability_unavailable_keeps_reason_internal_and_clears_chat_hint ... ok
test takeout_import::recovery::tests::takeout_recovery_ignores_non_takeout_batches ... ok
test takeout_import::recovery::tests::recovery_state_includes_migrated_history_scope_for_historical_batches ... ok
test takeout_import::recovery::tests::takeout_recovery_latest_complete_hides_older_failed ... ok
test takeout_import::recovery::tests::takeout_recovery_latest_failed_wins_over_older_complete ... ok
test takeout_import::recovery::tests::takeout_recovery_returns_partial_completed_and_hides_complete ... ok
test takeout_import::recovery::tests::takeout_recovery_running_with_active_job_is_hidden ... ok
test takeout_import::recovery::tests::takeout_recovery_running_without_active_job_is_interrupted ... ok
test takeout_import::state::tests::active_jobs_for_sources_filters_non_terminal_jobs ... ok
test takeout_import::state::tests::job_state_can_cancel_and_finish_job ... ok
test takeout_import::state::tests::job_state_cancels_child_tokens ... ok
test takeout_import::recovery::tests::takeout_recovery_source_filter_limits_results ... ok
test takeout_import::state::tests::job_state_rejects_duplicate_active_source_jobs ... ok
test takeout_import::state::tests::job_state_records_history_scope_for_frontend_labels ... ok
test takeout_import::state::tests::takeout_cancellation_smoke_fixture_finishes_cancelled_and_clears ... ok
test takeout_import::state::tests::takeout_cancellation_smoke_fixture_tracks_running_job ... ok
test takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact ... ok
test takeout_import::recovery::tests::takeout_recovery_warning_codes_are_unique_sorted_and_message_free ... ok
test takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues ... ok
test takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation ... ok
test takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize ... ok
test takeout_import::tests::locked_start_conflict_creates_no_provenance_rows ... ok
test takeout_import::tests::migrated_history_detected_warning_is_sanitized ... ok
test takeout_import::tests::locked_start_allows_only_one_batch_for_same_source ... ok
test takeout_import::tests::historical_batch_completion_does_not_advance_source_watermark ... ok
test takeout_import::tests::migrated_history_start_requires_available_capability ... ok
test takeout_import::tests::migrated_history_start_records_use_same_source_takeout_lock ... ok
test takeout_import::tests::takeout_step_cancel_wrapper_allows_completed_future ... ok
test takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future ... ok
test takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once ... ok
test takeout_import::tests::takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists ... ok
test takeout_import::tests::takeout_subtype_load_uses_typed_identity_not_legacy_kind ... ok
test takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_batch_summary_is_durable_and_sanitized ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_duplicate_after_normal_sync_summarizes_outcomes ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_caps_samples_deterministically ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_compares_batch_to_canonical_without_content ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_snapshot_delta_uses_explicit_snapshots ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_matched_observations_for_aggregates ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_missing_observations_by_identity ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_source_snapshot_is_aggregate_and_sanitized ... ok
test telegram::tests::diagnostic_status_counts_do_not_return_account_ids_or_messages ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_excludes_non_latest_recovery_candidates ... ok
test telegram::tests::legacy_api_hash_migrates_to_secret_store_and_blanks_column ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_is_durable_only ... ok
test telegram::tests::legacy_api_hash_remains_when_secret_write_fails ... ok
test telegram::tests::runtime_status_maps_to_existing_wire_strings ... ok
test telegram::tests::telegram_api_id_out_of_range_returns_typed_validation_error ... ok
test telegram::tests::telegram_status_and_event_payload_contract_is_exact ... ok
test telegram_session_store::tests::delete_session_from_path_removes_file_and_key ... ok
test telegram::tests::missing_secure_api_hash_for_blank_legacy_account_is_auth_error ... ok
test telegram_session_store::tests::legacy_plaintext_session_remains_when_keyring_write_fails ... ok
test telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact ... ok
test telegram_session_store::tests::encrypted_session_load_fails_when_key_is_missing ... ok
test telegram_session_store::tests::legacy_plaintext_session_migrates_to_encrypted_file ... ok
test topic_memberships::tests::rebuild_matches_retained_hidden_and_deleted_topics ... ok
test topic_memberships::tests::rebuild_prioritizes_specific_topic_matches_before_general_fallback ... ok
test topic_memberships::tests::rebuild_replaces_stale_memberships_and_versions ... ok
test tx::tests::begin_immediate_commit_persists_changes ... ok
test tx::tests::begin_immediate_rollback_discards_changes ... ok
test topic_memberships::tests::rebuild_uses_legacy_root_only_without_typed_child ... ok
test tx::tests::begin_immediate_with_foreign_keys_enforces_cascade ... ok
test tx::tests::finish_manual_transaction_commits_success_result ... ok
test tx::tests::finish_manual_transaction_rolls_back_error_result ... ok
test tx::tests::sqlite_ignores_foreign_keys_pragma_inside_open_transaction ... ok
test youtube::captions::tests::caption_download_args_request_json3_and_vtt_without_media ... ok
test youtube::captions::tests::caption_selection_honors_explicit_override_before_original_language ... ok
test youtube::captions::tests::caption_selection_prefers_original_then_preferred_then_english_then_any ... ok
test youtube::captions::tests::json3_parser_allows_missing_duration ... ok
test youtube::captions::tests::json3_parser_concatenates_segments_and_preserves_timing ... ok
test tx::tests::begin_immediate_read_then_write_survives_concurrent_writer ... ok
test tx::tests::deferred_read_then_write_hits_busy_snapshot_under_concurrent_writer ... ok
test youtube::captions::tests::transcript_external_id_includes_language_and_track_kind ... ok
test youtube::captions::tests::vtt_parser_reads_cues_and_skips_blank_text ... ok
test youtube::captions::tests::vtt_parser_rejects_invalid_timing ... ok
test youtube::comments::tests::comment_published_at_accepts_numbers_strings_and_fallback ... ok
test youtube::comments::tests::comments_fetch_args_include_bounded_extractor_args ... ok
test youtube::captions::tests::replace_transcript_segments_deletes_previous_rows_and_inserts_current_segments ... ok
test youtube::comments::tests::comments_fetch_timeout_is_longer_than_metadata_preview_timeout ... ok
test youtube::comments::tests::default_comment_limit_is_bounded ... ok
test youtube::comments::tests::normalize_comments_flattens_replies_and_warns_for_timestamp_fallbacks ... ok
test youtube::comments::tests::normalize_comments_truncates_raw_comment_array_before_normalization ... ok
test youtube::cookies::tests::accepts_empty_cookie_values ... ok
test youtube::cookies::tests::accepts_http_only_cookie_rows ... ok
test youtube::cookies::tests::rejects_empty_cookie_text ... ok
test youtube::cookies::tests::rejects_files_without_cookie_rows ... ok
test youtube::cookies::tests::rejects_invalid_cookie_text_before_saving_secret ... ok
test youtube::cookies::tests::validates_netscape_cookie_rows_without_exposing_values ... ok
test youtube::cookies::tests::stores_reads_and_clears_youtube_cookies_through_secret_store ... ok
test youtube::captions::tests::replace_transcript_segments_rebuilds_analysis_documents_by_segment_order ... ok
test youtube::detail::tests::list_summaries_uses_source_id_order_and_marks_no_captions_unavailable ... ok
test youtube::detail::tests::playlist_detail_reports_ordered_items_and_summary_counts ... ok
test youtube::detail::tests::playlist_detail_uses_typed_linked_video_metadata_with_corrupt_source_blob ... ok
test youtube::detail::tests::source_summary_missing_typed_metadata_uses_generic_title_without_blob_decode ... ok
test youtube::detail::tests::summaries_use_typed_video_metadata_with_corrupt_source_blob ... ok
test youtube::detail::tests::video_detail_includes_safe_source_metadata_without_item_raw_payloads ... ok
test youtube::dto::tests::availability_status_serializes_as_snake_case ... ok
test youtube::dto::tests::preview_kind_deserializes_snake_case ... ok
test youtube::dto::tests::video_form_serializes_short_value ... ok
test youtube::errors::tests::invalid_youtube_url_maps_to_validation_error ... ok
test youtube::detail::tests::video_detail_missing_typed_metadata_returns_controlled_error ... ok
test youtube::errors::tests::ytdlp_deleted_failures_map_to_not_found_error ... ok
test youtube::errors::tests::ytdlp_network_failures_map_to_network_error ... ok
test youtube::errors::tests::ytdlp_private_failures_map_to_auth_error ... ok
test youtube::jobs::tests::catalog_jobs_for_sources_includes_latest_failed_jobs ... ok
test youtube::jobs::tests::active_jobs_for_sources_filters_non_terminal_direct_and_related_sources ... ok
test youtube::jobs::tests::diagnostic_counts_group_source_jobs_without_ids_or_raw_errors ... ok
test youtube::jobs::tests::job_state_cancels_child_tokens ... ok
test youtube::jobs::tests::job_state_finishes_cancel_requested_jobs_as_cancelled ... ok
test youtube::jobs::tests::job_state_list_filters_before_limit_and_sorts_newest_first ... ok
test youtube::jobs::tests::job_state_rejects_duplicate_active_scope_but_allows_different_job_types ... ok
test youtube::detail::tests::video_detail_reports_synced_transcript_comments_and_playlist_memberships ... ok
test youtube::jobs::tests::jobs_missing_typed_video_metadata_errors_after_failed_refresh ... ok
test youtube::jobs::tests::source_job_cancellation_smoke_fixture_finishes_cancelled_and_clears ... ok
test youtube::jobs::tests::source_job_cancellation_smoke_fixture_tracks_running_job ... ok
test youtube::jobs::tests::jobs_reload_missing_typed_video_metadata_after_refresh_callback ... ok
test youtube::jobs::tests::source_job_step_with_process_cancel_allows_completed_future ... ok
test youtube::jobs::tests::source_job_step_with_process_cancel_interrupts_pending_future ... ok
test youtube::jobs::tests::source_job_workflow_file_has_no_tauri_command_adapters ... ok
test youtube::jobs::tests::source_job_type_uses_comments_specific_type_for_comments_only_video_sync ... ok
test youtube::metadata::tests::availability_values_map_to_statuses ... ok
test youtube::jobs::tests::source_jobs_no_longer_decode_source_metadata_blobs ... ok
test youtube::metadata::tests::playlist_metadata_page_args_use_adjacent_playlist_range ... ok
test youtube::metadata::tests::video_fixture_maps_metadata_and_preview_fields ... ok
test youtube::metadata::tests::video_fixture_missing_optional_fields_maps_to_none ... ok
test youtube::jobs::tests::retryable_playlist_video_rows_excludes_auth_deleted_and_removed_entries ... ok
test youtube::metadata::tests::playlist_fixture_maps_metadata_entries_and_preview_warning ... ok
test youtube::playlist::tests::upsert_playlist_items_can_skip_video_source_materialization ... ok
test youtube::playlist::tests::playlist_item_video_source_upsert_writes_typed_video_metadata_not_source_blob ... ok
test youtube::playlist::tests::upsert_playlist_items_marks_missing_rows_removed ... ok
test youtube::preview::tests::preview_from_playlist_json_returns_playlist_preview ... ok
test youtube::preview::tests::preview_from_video_json_uses_parsed_url_kind ... ok
test youtube::process_runtime::tests::cancellation_reaches_all_reserved_operations ... ok
test youtube::process_runtime::tests::cookie_guard_retains_file_until_detached_reaper_finishes ... ok
test youtube::process_runtime::tests::detached_reaper_keeps_cookie_until_the_stuck_child_releases ... ok
test youtube::playlist::tests::upsert_playlist_items_reuses_existing_video_source_and_keeps_unavailable_null ... ok
test youtube::playlist::tests::upsert_playlist_items_without_materialization_reuses_existing_video_source ... ok
test youtube::process_runtime::tests::finite_pipe_backpressure_requires_concurrent_drain ... ok
test youtube::process_runtime::tests::dropped_caller_keeps_child_and_registry_owned_until_shutdown_reaps_it ... ok
test youtube::process_runtime::tests::external_source_job_cancellation_reaps_its_managed_operation ... ok
test youtube::process_runtime::tests::injected_launcher_drains_backpressured_output_before_waiting_for_exit ... ok
test youtube::process_runtime::tests::injected_wait_error_reaps_the_child_before_releasing_registry ... ok
test youtube::process_runtime::tests::registry_reserves_an_operation_before_spawn ... ok
test youtube::process_runtime::tests::injected_nonzero_exit_preserves_not_found_classification_and_releases_registry ... ok
test youtube::process_runtime::tests::spawn_failure_rolls_back_the_registry_reservation ... ok
test youtube::process_runtime::tests::shutdown_rejects_new_ytdlp_admission_before_spawn ... ok
test youtube::runtime::tests::runtime_status_serializes_with_camel_case_keys ... ok
test youtube::process_runtime::tests::timeout_fallback_detaches_cookie_until_stuck_child_reaps ... ok
test youtube::settings::tests::invalid_stored_settings_return_validation_error_with_key ... ok
test youtube::settings::tests::invalid_youtube_settings_do_not_write_partial_values ... ok
test youtube::settings::tests::auth_cookies_load_only_when_auth_is_enabled ... ok
test youtube::settings::tests::validate_youtube_settings_normalizes_preferred_captions_language ... ok
test youtube::settings::tests::validate_youtube_settings_rejects_out_of_range_values ... ok
test youtube::settings::tests::youtube_settings_default_when_app_settings_are_missing ... ok
test youtube::settings::tests::saving_cookies_enables_auth_and_clear_disables_it ... ok
test youtube::settings::tests::youtube_settings_serializes_with_camel_case_keys ... ok
test youtube::source_metadata::tests::playlist_metadata_columns_are_versioned_and_secret_safe ... ok
test youtube::settings::tests::youtube_settings_roundtrip_through_app_settings ... ok
test youtube::source_metadata::tests::video_metadata_columns_include_wire_values_arrays_caption_override_and_sanitized_raw ... ok
test youtube::source_metadata::tests::video_metadata_rejects_wrong_canonical_url_shape ... ok
test youtube::source_metadata::tests::video_source_metadata_restores_raw_caption_metadata_for_provider_sync ... ok
test youtube::thumbnail::tests::accepts_only_allowlisted_https_thumbnail_urls ... ok
test youtube::thumbnail::tests::bounds_thumbnail_responses_to_one_mib ... ok
test youtube::thumbnail::tests::builds_the_dedicated_thumbnail_client ... ok
test youtube::thumbnail::tests::recognizes_supported_image_magic_bytes ... ok
test youtube::source_metadata::tests::upsert_video_metadata_maintains_description_document ... ok
test youtube::transcript_reader::tests::list_youtube_transcript_segments_can_start_at_selected_time ... ok
test youtube::transcript_reader::tests::list_youtube_transcript_segments_filters_by_search ... ok
test youtube::transcript_reader::tests::list_youtube_transcript_segments_pages_by_time_and_id ... ok
test youtube::url::tests::parses_live_url ... ok
test youtube::url::tests::parses_playlist_url ... ok
test youtube::url::tests::parses_short_youtu_be_url ... ok
test youtube::url::tests::parses_shorts_url ... ok
test youtube::url::tests::parses_watch_video_url ... ok
test youtube::url::tests::rejects_empty_input ... ok
test youtube::transcript_reader::tests::search_escapes_existing_backslashes_before_like_wildcards ... ok
test youtube::url::tests::rejects_invalid_host ... ok
test youtube::ytdlp::tests::authenticated_command_args_include_cookie_file_path_without_cookie_content ... ok
test youtube::url::tests::watch_url_with_playlist_parameter_parses_selected_video ... ok
test youtube::ytdlp::tests::cookie_file_content_adds_netscape_header_when_missing ... ok
test youtube::ytdlp::tests::cookie_file_content_preserves_existing_netscape_header ... ok
test youtube::ytdlp::tests::preview_playlist_args_limit_entries_to_first_fifty ... ok
test youtube::ytdlp::tests::preview_video_args_use_dump_json_without_shell_fragments ... ok
test process_tree::tests::terminates_a_descendant_created_after_assignment ... ok
test youtube::process_runtime::tests::injected_timeout_reap_detaches_stuck_child_and_keeps_cookie_until_release ... ok

test result: ok. 665 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.22s

     Running unittests src\main.rs (src-tauri\target\debug\deps\extractum-4b630ead96220393.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_analysis-369ed65a53dcc581.exe)

running 112 tests
test chat::tests::analysis_chat_request_metadata_uses_run_owner ... ok
test chat::tests::build_chat_request_uses_provider_neutral_source_document_wording ... ok
test chat::tests::chat_context_labels_migrated_history_scope_from_metadata ... ok
test chat::tests::completed_chat_context_accepts_saved_snapshot_messages ... ok
test chat::tests::completed_chat_context_requires_saved_snapshot_messages ... ok
test chat::tests::empty_chat_context_uses_source_document_wording ... ok
test corpus::tests::live::youtube_corpus_mode_parses_wire_values_and_defaults ... ok
test corpus::tests::preflight::default_preflight_limits_are_conservative ... ok
test corpus::tests::preflight::estimated_chunk_count_matches_chunk_boundary_behavior ... ok
test corpus::tests::preflight::estimated_message_chars_match_report_chunk_accounting ... ok
test corpus::tests::preflight::model_limit_preflight_allows_unknown_or_fitting_limits ... ok
test chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message ... ok
test corpus::tests::preflight::model_limit_preflight_reports_oversized_chunks ... ok
test corpus::tests::preflight::preflight_limit_error_allows_runs_within_limits ... ok
test corpus::tests::preflight::preflight_limit_error_reports_all_scale_dimensions ... ok
test chat::tests::chat_execution_persists_turns_before_completed_event ... ok
test corpus::tests::snapshot::list_run_snapshot_messages_page_does_not_fall_back_to_live_source ... ok
test corpus::tests::snapshot::captured_marker_with_missing_rows_returns_corrupt_snapshot_error ... ok
test corpus::tests::snapshot::list_run_snapshot_messages_page_reads_saved_snapshot_only ... ok
test corpus::tests::snapshot::list_run_snapshot_messages_page_returns_typed_internal_for_corrupt_snapshot_content ... ok
test corpus::tests::snapshot::run_message_cursor_uses_ref_and_published_at ... ok
test corpus::tests::snapshot::list_run_snapshot_messages_page_starts_at_around_ref ... ok
test corpus::tests::snapshot::load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows ... ok
test corpus::tests::snapshot::load_run_corpus_messages_uses_snapshot_when_available ... ok
test corpus::tests::source_resolution::resolve_run_source_ids_prefers_snapshot_over_live_group_membership ... ok
test corpus::tests::snapshot::run_snapshot_roundtrips_frozen_corpus ... ok
test report::tests::architecture::analysis_report_workflow_file_has_no_tauri_command_adapters ... ok
test corpus::tests::snapshot::trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot ... ok
test corpus::tests::snapshot::source_group_membership_drift_after_capture_does_not_change_saved_run_corpus ... ok
test report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads ... ok
test report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure ... ok
test report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure ... ok
test report::tests::lifecycle::interrupted_cleanup_preserves_captured_snapshot_state_marker ... ok
test report::tests::lifecycle::request_analysis_run_cancel_completed_run_keeps_conflict_message ... ok
test report::tests::lifecycle::request_analysis_run_cancel_missing_run_keeps_not_found_message ... ok
test report::tests::phases::analysis_step_cancel_wrapper_allows_completed_future ... ok
test report::tests::phases::analysis_step_cancel_wrapper_interrupts_pending_future ... ok
test report::tests::phases::finish_map_phase_preserves_chunk_order_by_original_index ... ok
test report::tests::phases::finish_map_phase_propagates_map_error_without_starting_reduce ... ok
test report::tests::phases::finish_map_phase_rejects_missing_chunk_before_reduce ... ok
test report::tests::lifecycle::request_analysis_run_cancel_running_but_inactive_keeps_conflict_message ... ok
test report::tests::preflight::validate_report_preflight_allows_runs_within_limits ... ok
test report::tests::preflight::validate_report_preflight_rejects_empty_corpus ... ok
test report::tests::preflight::validate_report_preflight_rejects_oversized_runs ... ok
test report::tests::requests::build_map_request_keeps_run_scoped_request_and_profile ... ok
test report::tests::requests::build_reduce_request_keeps_run_scoped_request_and_profile ... ok
test report::tests::requests::extracts_json_inside_markdown_fence ... ok
test report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails ... ok
test report::tests::requests::extracts_json_with_text_before_and_after ... ok
test report::tests::requests::parse_chunk_summary_ignores_non_json_prefix_with_braces ... ok
test report::tests::requests::parse_chunk_summary_rejects_malformed_payload ... ok
test report::tests::scope::chunk_target_chars_are_derived_from_model_input_limit_with_fallback ... ok
test report::tests::scope::migrated_history_opt_in_rejects_non_telegram_analysis ... ok
test report::tests::scope::report_run_input_carries_resolved_profile_snapshot ... ok
test report::tests::scope::report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape ... ok
test report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities ... ok
test report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label ... ok
test report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes ... ok
test report::tests::scope::telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match ... ok
test state::tests::analysis_state_cancels_report_run_child_tokens ... ok
test store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes ... ok
test report::tests::runtime::terminal_cleanup_always_removes_active_report_state ... ok
test report::tests::runtime::report_execution_publishes_typed_events_in_existing_order ... ok
test store::tests::read_model::failed_terminal_run_without_capture_marker_is_capture_failed ... ok
test store::tests::read_model::completed_run_without_capture_marker_is_capture_failed ... ok
test store::tests::read_model::list_analysis_run_summaries_applies_query_before_limit ... ok
test store::tests::read_model::list_analysis_run_summaries_combines_scope_and_field_filters ... ok
test store::tests::read_model::list_analysis_run_summaries_rejects_both_scope_ids ... ok
test store::tests::read_model::list_analysis_run_summaries_filters_source_groups_and_template_names ... ok
test store::tests::read_model::list_analysis_run_summaries_escapes_literal_like_characters ... ok
test store::tests::read_model::list_analysis_run_summaries_filters_status_and_dates ... ok
test store::tests::read_model::map_run_detail_exposes_youtube_corpus_mode ... ok
test store::tests::read_model::map_run_summary_exposes_capture_failed_snapshot_state ... ok
test store::tests::read_model::map_run_summary_exposes_captured_snapshot_state ... ok
test store::tests::read_model::map_run_summary_exposes_frozen_scope_label ... ok
test store::tests::read_model::map_run_summary_exposes_null_snapshot_state_for_active_runs_before_capture ... ok
test store::tests::read_model::resolve_run_scope_label_prefers_frozen_value ... ok
test store::tests::read_model::map_run_summary_exposes_youtube_corpus_mode ... ok
test store::tests::runs::cancellation_after_capture_does_not_write_snapshot_error ... ok
test store::tests::runs::delete_saved_run_removes_run_and_saved_children ... ok
test store::tests::runs::duplicate_lookup_keeps_project_and_source_group_scopes_separate ... ok
test store::tests::runs::delete_saved_run_returns_typed_not_found_error ... ok
test store::tests::runs::duplicate_lookup_matches_telegram_history_scope ... ok
test store::tests::runs::insert_analysis_run_persists_youtube_corpus_mode ... ok
test store::tests::runs::provider_failure_status_update_does_not_write_snapshot_error ... ok
test store::tests::snapshot::capture_run_snapshot_rejects_missing_required_fields_without_marker ... ok
test store::tests::setup::fetch_prompt_template_returns_typed_not_found_error ... ok
test store::tests::snapshot::sanitize_provider_error_redacts_provider_payloads ... ok
test store::tests::snapshot::sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens ... ok
test store::tests::snapshot::capture_run_snapshot_marks_captured_after_reload_and_replaces_rows ... ok
test store::tests::snapshot::mark_run_capture_failed_sets_snapshot_error ... ok
test tests::chat_role_validation_returns_typed_error ... ok
test test_schema::tests::canonical_fixture_applies_analysis_consumed_schema ... ok
test tests::chat_turn_validation_returns_typed_error ... ok
test tests::source_group_input_is_trimmed_and_deduplicated ... ok
test tests::source_group_input_validation_returns_typed_error ... ok
test tests::template_kind_validation_returns_typed_error ... ok
test test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys ... ok
test trace::tests::analysis_trace_ref_serializes_youtube_fields_as_null_for_telegram_refs ... ok
test trace::tests::build_trace_refs_falls_back_to_base_item_refs ... ok
test tests::builtin_template_is_seeded_once ... ok
test trace::tests::build_trace_refs_handles_multibyte_excerpt ... ok
test trace::tests::build_trace_refs_marks_youtube_description_refs_as_synthetic ... ok
test trace::tests::build_trace_refs_resolves_exact_youtube_timestamp_refs ... ok
test trace::tests::clip_excerpt_truncates_on_char_boundary ... ok
test trace::tests::decode_trace_data_returns_typed_internal_for_invalid_json ... ok
test trace::tests::decode_trace_data_returns_typed_internal_for_invalid_zstd ... ok
test trace::tests::legacy_trace_bytes_decode_after_core_compression_handoff ... ok
test trace::tests::normalize_ref_accepts_item_refs ... ok
test trace::tests::trace_ref_json_is_byte_compatible_for_telegram_and_youtube ... ok
test tests::trace_data_roundtrips_through_zstd ... ok
test tests::completed_run_without_snapshot_marker_is_capture_failed ... ok

test result: ok. 112 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.58s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_core-afb8508148d4b372.exe)

running 22 tests
test compression::tests::json_bytes_roundtrip_through_zstd ... ok
test error::tests::classify_message_treats_dialog_lookup_misses_as_not_found ... ok
test compression::tests::text_roundtrip_through_zstd ... ok
test compression::tests::decompress_text_rejects_invalid_utf8 ... ok
test error::tests::classify_message_treats_source_kind_mismatches_as_validation ... ok
test error::tests::database_error_mapper_matches_database_helper ... ok
test error::tests::database_helper_maps_to_internal ... ok
test error::tests::classify_message_treats_resolution_failures_as_not_found ... ok
test error::tests::internal_error_mapper_stringifies_errors ... ok
test error::tests::llm_network_helper_maps_to_network ... ok
test error::tests::telegram_network_helper_maps_to_network ... ok
test media_metadata::tests::absent_media_metadata_decodes_to_default ... ok
test media_metadata::tests::media_label_covers_known_and_fallback_kinds ... ok
test media_metadata::tests::media_metadata_decode_failures_are_typed_internal_errors ... ok
test time::tests::now_rfc3339_utc_returns_current_utc_timestamp ... ok
test time::tests::ymd_to_unix_midnight_parses_compact_youtube_dates ... ok
test time::tests::ymd_to_unix_midnight_parses_iso_dates ... ok
test media_metadata::tests::media_metadata_roundtrip_through_zstd ... ok
test time::tests::now_secs_returns_unix_timestamp_seconds ... ok
test time::tests::ymd_to_unix_midnight_rejects_malformed_dates ... ok
test time::tests::ymd_to_unix_midnight_rejects_non_canonical_iso_dates ... ok
test time::tests::ymd_to_unix_midnight_rejects_nonexistent_calendar_dates ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_gemini_browser-3b4520f56cf26a13.exe)

running 77 tests
test cdp::tests::launch_spec_rejects_remote_cdp_endpoint ... ok
test cdp::tests::launch_spec_uses_endpoint_port_and_dedicated_profile ... ok
test execution::tests::cancel_missing_run_returns_without_run_log_side_effects ... ok
test execution::tests::cancel_gemini_browser_job_cancels_queued_run_and_waiter ... ok
test execution::tests::cancel_queued_run_updates_terminal_snapshot ... ok
test execution::tests::active_cancellation_stops_executor_once_and_ignores_late_success ... ok
test execution::tests::cancel_gemini_browser_job_requests_stop_for_active_run ... ok
test execution::tests::restart_worker_entry_acknowledges_missing_run_log_without_sidecar ... ok
test execution::tests::restart_worker_entry_skips_terminal_cancelled_run_log ... ok
test execution::tests::execution_timeout_stops_executor_with_typed_timeout_reason ... ok
test execution::tests::worker_handler_marks_run_running_and_terminal ... ok
test protocol::tests::decode_sidecar_line_accepts_ack_for_matching_id ... ok
test execution::tests::worker_handler_converts_executor_error_to_terminal_failed_result ... ok
test protocol::tests::decode_sidecar_line_for_request_skips_stale_response_ids ... ok
test protocol::tests::decode_sidecar_line_rejects_mismatched_ids ... ok
test protocol::tests::jsonl_transport_round_trips_a_duplex_request ... ok
test protocol::tests::resume_response_classifies_legacy_ack_for_retry ... ok
test protocol::tests::take_complete_jsonl_lines_handles_partial_and_multiple_chunks ... ok
test reconciliation::tests::degraded_apalis_queue_inspection_leaves_queued_run_log_records_for_worker_entry ... ok
test reconciliation::tests::restart_reconciliation_degraded_leaves_queued_run_log_records ... ok
test reconciliation::tests::restart_reconciliation_matrix_handles_supported_apalis_states ... ok
test execution::tests::worker_timeout_marks_run_failed_and_processes_next_job ... ok
test run_log::tests::get_run_core_returns_exact_run_from_log ... ok
test run_log::tests::create_queued_run_prunes_expired_runs_before_writing_new_run ... ok
test run_log::tests::read_run_returns_exact_run_by_id ... ok
test run_log::tests::read_run_returns_validation_error_for_missing_run ... ok
test execution::tests::worker_timeout_clears_active_and_cancelled_state ... ok
test run_log::tests::list_runs_deletes_run_directories_outside_retention_window ... ok
test runtime::tests::complete_waiter_ignores_dropped_receiver ... ok
test runtime::tests::gemini_browser_job_serializes_queue_payload ... ok
test run_log::tests::recorded_run_dir_prunes_expired_run_before_opening_artifacts ... ok
test run_log::tests::run_log_persists_queued_running_and_terminal_result ... ok
test runtime::tests::register_waiter_rejects_duplicate_run_id ... ok
test runtime::tests::runtime_tracks_and_clears_cancelled_run_ids ... ok
test runtime::tests::wait_for_result_removes_waiter_when_worker_channel_closes ... ok
test runtime::tests::waiter_receives_terminal_worker_result ... ok
test runtime::tests::worker_run_failure_marks_runtime_failed ... ok
test runtime::tests::worker_startup_failure_marks_runtime_failed ... ok
test runtime::tests::worker_status_allows_enqueue_after_ready ... ok
test runtime::tests::worker_status_blocks_enqueue_when_startup_failed ... ok
test sidecar_launch::tests::bundled_sidecar_path_is_beside_the_packaged_executable ... ok
test sidecar_launch::tests::resolve_launch_mode_allows_explicit_dev_sidecar_override_in_release ... ok
test sidecar_launch::tests::resolve_launch_mode_falls_back_to_bundled_when_debug_dev_script_is_absent ... ok
test sidecar_launch::tests::resolve_launch_mode_keeps_dev_node_fallback_for_debug_repo_runs ... ok
test sidecar_launch::tests::resolve_launch_mode_prefers_bundled_when_forced ... ok
test sidecar_launch::tests::resolve_launch_mode_uses_bundled_by_default_for_release_even_when_repo_dist_exists ... ok
test state::tests::set_status_snapshot_if_current_does_not_overwrite_newer_snapshot ... ok
test runtime::tests::worker_status_times_out_while_starting ... ok
test runtime::tests::wait_for_result_removes_waiter_on_timeout ... ok
test state::tests::startup_reconciliation_gate_retries_after_failure ... ok
test state::tests::startup_reconciliation_gate_runs_once_after_success ... ok
test state::tests::state_tracks_active_run_and_cancellation ... ok
test state::tests::status_snapshot_initializes_to_not_started_from_profile_dir ... ok
test state::tests::update_status_snapshot_mutates_cached_status ... ok
test status::tests::provider_status_live_probe_does_not_mutate_cached_snapshot ... ok
test status::tests::provider_status_read_core_waits_for_startup_reconciliation_before_live_status ... ok
test status::tests::provider_status_snapshot_from_reconciled_runs_does_not_keep_stale_running_snapshot ... ok
test status::tests::provider_status_snapshot_from_reconciled_runs_ignores_stale_queued_rows ... ok
test status::tests::provider_status_snapshot_from_reconciled_runs_preserves_live_active_run ... ok
test status::tests::provider_status_snapshot_read_core_skips_stale_write_back_when_snapshot_changed ... ok
test status::tests::provider_status_snapshot_read_core_writes_reconciled_snapshot_back ... ok
test status::tests::status_snapshot_core_returns_cached_status_without_polling_live_sidecar ... ok
test submission::tests::failed_run_log_transition_returns_app_error_without_side_effects ... ok
test submission::tests::send_single_prompt_handoff_writes_run_log_before_enqueue ... ok
test run_log::tests::recorded_run_dir_requires_result_artifact_flag_and_returns_computed_dir ... ok
test status::tests::provider_status_uses_cached_snapshot_when_sidecar_is_busy ... ok
test submission::tests::send_single_prompt_rejects_duplicate_waiter_before_enqueue ... ok
test submission::tests::send_single_prompt_rejects_duplicate_non_terminal_run_id_before_enqueue ... ok
test submission::tests::send_single_prompt_rejects_invalid_artifact_mode_before_side_effects ... ok
test types::tests::manual_action_serializes_start_chrome_cdp ... ok
test types::tests::resume_command_serializes_browser_profile_dir ... ok
test types::tests::run_result_serializes_optional_debug_summary ... ok
test types::tests::sidecar_command_serializes_browser_config ... ok
test types::tests::sidecar_command_serializes_with_snake_case_tag ... ok
test types::tests::success_statuses_include_ready_and_ok ... ok
test submission::tests::send_single_prompt_marks_run_failed_when_enqueue_fails ... ok
test submission::tests::send_single_prompt_rejects_duplicate_terminal_run_id_before_enqueue ... ok

test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_llm-f6781d6f95dee3e0.exe)

running 37 tests
test gemini::tests::gemini_model_listing_requires_typed_auth_error ... ok
test gemini::tests::gemini_model_mapping_uses_short_model_id ... ok
test gemini::tests::gemini_request_mapping_keeps_existing_messages_without_output_limit ... ok
test gemini::tests::gemini_request_mapping_keeps_system_history_and_roles ... ok
test gemini::tests::gemini_request_rejects_unsupported_roles_with_typed_validation_error ... ok
test gemini::tests::gemini_server_error_message_includes_transient_recovery_hint ... ok
test gemini::tests::gemini_stream_chunk_text_and_usage_are_parsed ... ok
test openai_compat::tests::openai_compat_model_listing_requires_typed_auth_error ... ok
test openai_compat::tests::openai_compat_model_mapping_reads_omniroute_limits_and_capabilities ... ok
test openai_compat::tests::openai_compat_model_mapping_uses_model_id ... ok
test openai_compat::tests::openai_compat_request_keeps_standard_roles ... ok
test openai_compat::tests::openai_compat_request_rejects_unsupported_roles_with_typed_validation_error ... ok
test openai_compat::tests::openai_compat_retry_status_policy_is_bounded_to_transient_failures ... ok
test openai_compat::tests::openai_compat_stream_chunk_mapping_reads_delta_and_usage ... ok
test provider::tests::model_input_token_limit_lookup_matches_provider_model_ids_and_names ... ok
test provider::tests::model_output_token_limit_lookup_matches_provider_model_ids_and_names ... ok
test provider::tests::normalize_base_url_returns_typed_validation_error ... ok
test provider::tests::normalize_base_url_allows_https_and_loopback_http_only ... ok
test provider::tests::provider_parse_accepts_openai_compatible_aliases ... ok
test provider::tests::provider_parse_returns_typed_validation_error ... ok
test runner::tests::resolve_effective_model_returns_typed_validation_error ... ok
test runner::tests::run_llm_collect_returns_typed_validation_error ... ok
test runner::tests::validate_request_returns_typed_validation_error ... ok
test scheduler::tests::active_owner_run_ids_reports_running_and_queued_owned_requests ... ok
test scheduler::tests::cancelling_owned_run_requests_aborts_running_work ... ok
test scheduler::tests::failed_requests_preserve_typed_error_kind ... ok
test scheduler::tests::failed_requests_release_capacity_for_next_queued_request ... ok
test scheduler::tests::interactive_requests_jump_ahead_of_background_queue ... ok
test scheduler::tests::llm_request_diagnostic_keys_are_stable_snake_case ... ok
test scheduler::tests::queue_positions_are_recomputed_after_cancelling_a_queued_request ... ok
test scheduler::tests::queued_requests_can_be_cancelled_before_start ... ok
test scheduler::tests::request_snapshots_report_running_and_queued_requests ... ok
test scheduler::tests::requests_with_different_profiles_run_without_blocking_each_other ... ok
test streaming::tests::sse_data_decode_failures_are_typed_internal_errors ... ok
test streaming::tests::sse_data_is_parsed_from_stream_chunks ... ok
test types::tests::resolved_profile_construction_preserves_execution_access_and_public_metadata ... ok
test openai_compat::tests::openai_compat_stream_retries_transient_http_before_streaming ... ok

test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_prompt_packs-77cc860c50d25ed2.exe)

running 249 tests
test completion_transport::tests::api_stage_uses_background_scheduler_prompt_pack_metadata_and_typed_cancellation ... ok
test completion_transport::tests::browser_model_context_has_no_api_fields ... ok
test completion_transport::tests::browser_provenance_is_persisted_before_completion_validation ... ok
test dto::tests::crate_boundary_constructors_and_accessors_preserve_serialized_shapes ... ok
test dto::tests::preflight_request_defaults_to_api_runtime_provider ... ok
test dto::tests::prompt_pack_errors_serialize_exact_json_contract ... ok
test dto::tests::start_outcomes_serialize_exact_ipc_contract ... ok
test dto::tests::start_request_accepts_gemini_browser_runtime_provider ... ok
test dto::tests::prompt_pack_run_events_serialize_exact_ipc_contract ... ok
test gemini_browser_stage::tests::ok_browser_result_maps_to_completion_text ... ok
test gemini_browser_stage::tests::ready_result_is_not_prompt_completion ... ok
test gemini_browser_stage::tests::timeout_latest_ok_result_is_not_prompt_completion ... ok
test completion_transport::tests::api_model_context_retains_profile_and_override ... ok
test library::tests::get_prompt_pack_library_returns_active_youtube_summary_pack ... ok
test projections::tests::low_level_result_persistence_rolls_back_when_projection_insert_fails ... ok
test projections::tests::persist_final_result_projects_youtube_synthesis_items ... ok
test projections::tests::persist_final_result_does_not_overwrite_cancelled_run_status ... ok
test projections::tests::persist_final_result_uses_current_time_for_run_completion ... ok
test projections::tests::persist_final_result_sets_terminal_status_after_projection_rows_exist ... ok
test result_builder::tests::build_canonical_result_assigns_backend_owned_ids ... ok
test result_builder::tests::build_canonical_result_includes_synthesis_output ... ok
test result_builder::tests::build_canonical_result_keeps_partial_result_flag_when_synthesis_is_skipped ... ok
test result_builder::tests::build_canonical_result_marks_multi_video_synthesis_skipped_insufficient_successes ... ok
test result_builder::tests::build_canonical_result_marks_multi_video_synthesis_failed ... ok
test result_builder::tests::build_canonical_result_marks_single_video_synthesis_not_applicable ... ok
test result_builder::tests::build_canonical_result_uses_current_created_at ... ok
test result_builder::tests::build_canonical_result_preserves_synthesis_common_claim_text ... ok
test result_builder::tests::build_canonical_result_rejects_incomplete_intermediate_graph ... ok
test run_control::tests::apply_event_updates_state_before_synchronous_sink_observes_it ... ok
test runtime::tests::browser_cancellation_completes_before_terminal_persistence_and_event_follow_up ... ok
test runtime::tests::browser_prompt_formatter_preserves_role_order_and_content ... ok
test runtime::tests::browser_prompt_formatter_rejects_unsupported_roles ... ok
test runtime::tests::browser_run_id_accepts_optional_gem_discriminator ... ok
test runtime::tests::browser_run_identity_includes_repair_attempt_when_present ... ok
test runtime::tests::browser_runtime_start_gate_allows_ready_status ... ok
test runtime::tests::browser_runtime_start_gate_maps_unready_status_to_preflight_failure ... ok
test runtime::tests::browser_stage_result_maps_to_prompt_pack_completion_without_tokens ... ok
test result_builder::tests::build_canonical_result_uses_intermediate_graph_claims_and_evidence ... ok
test result_builder::tests::gem_analysis_final_output_builds_canonical_single_video_result ... ok
test runtime::tests::detailed_report_control_preset_uses_larger_transcript_analysis_output_budget ... ok
test runtime::tests::cancelled_browser_stage_does_not_persist_success_provenance ... ok
test runtime::tests::gem_analysis_part_llm_request_preserves_part_and_frozen_input ... ok
test runtime::tests::cleanup_interrupted_prompt_pack_runs_marks_stale_active_rows_interrupted ... ok
test runtime::tests::gem_analysis_part_repair_llm_request_preserves_attempt_and_repair_context ... ok
test runtime::tests::gem_input_budget_uses_lower_known_model_limit ... ok
test runtime::tests::delete_prompt_pack_run_rejects_active_runs ... ok
test runtime::tests::list_prompt_pack_runs_returns_recent_runs_for_project ... ok
test runtime::tests::list_prompt_pack_run_stages_returns_browser_provenance ... ok
test runtime::tests::load_run_runtime_config_reads_api_and_browser_rows ... ok
test runtime::tests::now_string_uses_current_utc_time ... ok
test runtime::tests::load_run_runtime_config_rejects_malformed_browser_config ... ok
test runtime::tests::load_run_runtime_config_rejects_unsupported_provider ... ok
test runtime::tests::prompt_pack_browser_stage_cancelled_before_enqueue_is_tolerated ... ok
test runtime::tests::persist_browser_stage_provenance_records_result_identity ... ok
test runtime::tests::prepare_execution_borrows_the_same_ticket_for_terminal_failure ... ok
test runtime::tests::prompt_pack_cancellation_smoke_fixture_tracks_active_run ... ok
test runtime::tests::prompt_pack_run_cancellation_allows_completed_stage_future ... ok
test runtime::tests::prompt_pack_cancellation_smoke_fixture_clear_cancels_tokens_and_deletes_rows ... ok
test runtime::tests::prompt_pack_run_cancellation_interrupts_stage_future ... ok
test runtime::tests::prompt_pack_run_state_cancels_child_tokens ... ok
test runtime::tests::prompt_pack_run_state_tracks_active_and_cancel_requested_runs ... ok
test runtime::tests::prompt_pack_browser_stage_cancelled_while_queued_cancels_browser_job ... ok
test runtime::tests::prompt_pack_browser_stage_cancelled_while_active_stops_sidecar ... ok
test runtime::tests::start_service_rejects_empty_id_before_browser_or_source_ports ... ok
test runtime::tests::start_source_applies_queued_state_and_event_before_spawned_profile_resolution ... ok
test runtime::tests::synthesis_llm_request_describes_allowed_refs_and_forbids_direct_intermediate_refs ... ok
test runtime::tests::synthesis_output_budget_comes_from_stage_runtime_config ... ok
test runtime::tests::start_service_returns_ticket_for_untracked_existing_queued_run ... ok
test runtime::tests::terminal_event_removes_run_from_active_state ... ok
test runtime::tests::start_service_returns_existing_before_browser_or_source_ports ... ok
test runtime::tests::transcript_analysis_llm_request_describes_candidate_indexes_and_forbids_backend_refs ... ok
test runtime::tests::transcript_analysis_llm_request_embeds_frozen_stage_input ... ok
test runtime::tests::transcript_analysis_llm_request_uses_detailed_report_prompt_for_control_preset ... ok
test runtime::tests::transcript_analysis_output_budget_comes_from_stage_runtime_config ... ok
test runtime::tests::start_service_issues_ticket_after_queued_event_and_new_tracking ... ok
test runtime::tests::transcript_analysis_output_budget_is_clamped_to_model_limit ... ok
test runtime::tests::transcript_analysis_stage_max_prompt_token_budget_reads_runtime_config ... ok
test seed::tests::bundled_assets_hashes_and_source_path_match_canonical_bytes ... ok
test runtime::tests::terminal_failure_cleans_state_and_emits_when_persistence_fails ... ok
test runtime::tests::update_prompt_pack_run_updates_user_label_only ... ok
test seed::tests::seed_youtube_summary_pack_is_idempotent ... ok
test seed::tests::seed_youtube_summary_pack_preserves_unknown_newer_bundled_version ... ok
test seed::tests::seed_youtube_summary_pack_rejects_user_collision ... ok
test seed::tests::seed_youtube_summary_pack_rejects_bundled_hash_conflict ... ok
test seed::tests::seed_youtube_summary_pack_writes_required_schema_assets ... ok
test stage_output_normalization::tests::synthesis_runtime_normalization_defaults_readable_arrays ... ok
test stage_io::tests::build_transcript_analysis_stage_input_uses_frozen_registries ... ok
test stage_io::tests::insert_stage_artifact_uses_current_time ... ok
test stage_io::tests::transcript_analysis_stage_input_serializes_contract_keys_as_snake_case ... ok
test store::tests::prompt_pack_runs_allow_null_client_request_id_for_pre_existing_rows ... ok
test validation::tests::extract_json_payload_accepts_fenced_json_object ... ok
test validation::tests::extract_json_payload_accepts_leading_and_trailing_prose ... ok
test validation::tests::extract_json_payload_rejects_malformed_braces ... ok
test validation::tests::extract_json_payload_rejects_multiple_json_objects ... ok
test store::tests::prompt_pack_runs_client_request_id_is_unique_when_present ... ok
test validation::tests::invalid_synthesis_output_surfaces_quarantine_write_failure ... ok
test test_schema::tests::canonical_fixture_applies_declared_consumed_schema ... ok
test validation::tests::synthesis_output_accepts_provider_string_items_for_readable_arrays ... ok
test validation::tests::synthesis_output_rejects_direct_segment_key_point_or_quote_refs_inside_synthesis_candidate ... ok
test validation::tests::synthesis_output_rejects_non_array_or_non_string_ref_values ... ok
test validation::tests::synthesis_output_rejects_unknown_claim_ref ... ok
test validation::tests::synthesis_output_validator_accepts_valid_output ... ok
test validation::tests::synthesis_output_validator_rejects_backend_owned_ids ... ok
test validation::tests::synthesis_output_validator_rejects_missing_summary_text ... ok
test validation::tests::synthesis_output_validator_rejects_non_array_fields ... ok
test validation::tests::synthesis_output_validator_rejects_provider_authored_claim_ref ... ok
test validation::tests::synthesis_output_validator_rejects_structural_schema_errors ... ok
test validation::tests::synthesis_output_validator_rejects_unknown_source_ref ... ok
test validation::tests::synthesis_output_validator_rejects_wrong_schema_version ... ok
test validation::tests::synthesis_output_validator_rejects_wrong_stage ... ok
test test_schema::tests::canonical_fixture_preserves_consumed_indexes_and_foreign_keys ... ok
test validation::tests::synthesis_output_validator_rejects_wrong_stage_io_version ... ok
test validation::tests::transcript_analysis_output_rejects_llm_assigned_final_ids ... ok
test validation::tests::transcript_analysis_output_rejects_structural_schema_errors ... ok
test validation::tests::transcript_analysis_output_rejects_unknown_material_ref ... ok
test youtube_summary::entities_tests::blank_key_point_is_skipped_with_graph_warning ... ok
test youtube_summary::entities_tests::build_source_graph_assigns_backend_refs_and_allowed_refs ... ok
test youtube_summary::entities_tests::evidence_index_pointing_to_skipped_quote_candidate_is_dropped_with_warning ... ok
test validation::tests::invalid_synthesis_output_is_written_to_quarantine_artifacts ... ok
test youtube_summary::entities_tests::evidence_quote_candidate_index_to_missing_quote_is_dropped_with_warning ... ok
test youtube_summary::entities_tests::graph_constants_match_contract ... ok
test youtube_summary::entities_tests::invalid_material_ref_is_rejected ... ok
test youtube_summary::entities_tests::key_point_index_pointing_to_skipped_segment_candidate_is_dropped_with_warning ... ok
test youtube_summary::entities_tests::malformed_candidate_container_is_rejected ... ok
test youtube_summary::entities_tests::provider_output_must_not_supply_backend_refs_or_ids ... ok
test youtube_summary::entities_tests::textless_segment_is_kept_as_structural_navigation ... ok
test validation::tests::invalid_synthesis_output_with_unknown_source_ref_is_quarantined ... ok
test validation::tests::synthesis_quarantine_artifact_uses_current_time ... ok
test youtube_summary::entities_tests::graph_builder_uses_persisted_prompt_input_material_registry ... ok
test youtube_summary::execution_tests::execute_multi_video_run_stops_after_transcript_when_cancelled_before_synthesis ... ok
test youtube_summary::execution_tests::execute_multi_video_run_with_one_provider_failure_finishes_partial ... ok
test youtube_summary::execution_tests::execute_queued_run_repairs_malformed_synthesis_json ... ok
test youtube_summary::execution_tests::execute_queued_run_with_stage_executor_finishes_complete ... ok
test youtube_summary::execution_tests::execute_queued_run_repairs_malformed_transcript_json ... ok
test youtube_summary::execution_tests::execution_graph_build_failure_after_failed_repair_marks_transcript_failed_once ... ok
test youtube_summary::execution_tests::gem_analysis_does_not_start_next_part_after_cancellation_checkpoint ... ok
test youtube_summary::execution_tests::gem_analysis_executes_passport_comments_and_deep_recap_in_order ... ok
test youtube_summary::execution_tests::gem_analysis_repairs_invalid_required_part_once ... ok
test youtube_summary::execution_tests::gem_analysis_input_budget_blocks_before_first_provider_call ... ok
test youtube_summary::execution_tests::gem_analysis_optional_comments_failure_persists_report_with_failure_note ... ok
test youtube_summary::execution_tests::gem_analysis_required_part_failure_fails_stage ... ok
test youtube_summary::execution_tests::youtube_summary_multi_video_partial_transcripts_skip_synthesis_and_mark_partial ... ok
test youtube_summary::execution_tests::youtube_summary_invalid_final_result_records_result_level_findings ... ok
test youtube_summary::execution_tests::gem_analysis_skips_comments_when_trimmed_comment_text_is_empty ... ok
test youtube_summary::execution_tests::youtube_summary_run_marks_partial_when_synthesis_fails ... ok
test youtube_summary::execution_tests::youtube_summary_run_executes_synthesis_after_transcript_stages ... ok
test youtube_summary::facade_tests::now_string_uses_current_utc_time ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::assemble_gem_markdown_nests_part_markdown_under_backend_headings ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::assemble_gem_transcript_output_contains_empty_candidate_arrays ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_analysis_part_types_cover_comments_and_stage_variants ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_materials_input_budget_rejects_over_cap ... ok
test youtube_summary::execution_tests::youtube_summary_single_video_run_skips_synthesis ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_materials_part_prompt_inputs_are_isolated ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::parse_part_output_accepts_json_fence_with_internal_markdown_code_block ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::parse_part_output_accepts_matching_non_empty_markdown ... ok
test youtube_summary::execution_tests::youtube_summary_run_marks_partial_when_synthesis_output_is_invalid ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::parse_part_output_rejects_empty_markdown ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::parse_part_output_rejects_wrong_part ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_materials_load_formats_timestamped_transcript_from_metadata ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_materials_load_skips_empty_comment_rows ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_normalizes_provider_string_readable_items ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_rejects_invalid_output_without_success_artifacts ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_requires_complete_intermediate_graph ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_rejects_unknown_claim_ref_with_quarantine ... ok
test youtube_summary::outputs_tests::execute_transcript_analysis_stage_persists_default_warning_candidates ... ok
test youtube_summary::outputs_tests::execute_transcript_analysis_stage_persists_intermediate_entities_artifact ... ok
test youtube_summary::outputs_tests::malformed_intermediate_candidates_are_quarantined_without_graph_artifact ... ok
test youtube_summary::outputs_tests::execute_transcript_analysis_stage_persists_raw_and_parsed_artifacts ... ok
test youtube_summary::outputs_tests::repair_graph_build_failure_does_not_write_repaired_parsed_output ... ok
test youtube_summary::outputs_tests::repaired_synthesis_stage_rejects_unknown_claim_ref_with_quarantine ... ok
test youtube_summary::outputs_tests::repaired_transcript_analysis_persists_intermediate_entities_for_repair_attempt ... ok
test youtube_summary::outputs_tests::repaired_transcript_success_artifacts_roll_back_when_parsed_insert_fails ... ok
test youtube_summary::outputs_tests::transcript_stage_metrics_can_include_gem_analysis_extension ... ok
test youtube_summary::outputs_tests::transcript_success_artifacts_roll_back_when_parsed_insert_fails ... ok
test youtube_summary::preflight_tests::browser_runtime_preflight_does_not_apply_api_input_limit ... ok
test youtube_summary::preflight_tests::api_runtime_preflight_uses_fixed_32000_input_limit ... ok
test youtube_summary::preflight_tests::preflight_explicit_video_without_transcript_is_blocking_failure ... ok
test youtube_summary::result_validation::tests::blank_video_id_returns_error ... ok
test youtube_summary::result_validation::tests::canonical_result_schema_allows_runtime_string_limitations ... ok
test youtube_summary::result_validation::tests::canonical_result_schema_shape_error_returns_finding ... ok
test youtube_summary::preflight_tests::preflight_gem_analysis_allows_exactly_one_included_video ... ok
test youtube_summary::result_validation::tests::complete_narrative_only_result_allows_empty_videos ... ok
test youtube_summary::preflight_tests::preflight_gem_analysis_blocks_multiple_included_videos ... ok
test youtube_summary::result_validation::tests::complete_standard_result_with_empty_videos_returns_error ... ok
test youtube_summary::result_validation::tests::duplicate_claim_id_returns_error ... ok
test youtube_summary::result_validation::tests::duplicate_evidence_id_returns_error ... ok
test youtube_summary::result_validation::tests::duplicate_synthesis_item_id_across_item_kinds_returns_error ... ok
test youtube_summary::result_validation::tests::duplicate_source_ref_id_returns_error ... ok
test youtube_summary::result_validation::tests::duplicate_video_id_returns_error ... ok
test youtube_summary::result_validation::tests::evidence_with_unknown_claim_id_returns_error ... ok
test youtube_summary::result_validation::tests::known_quality_flag_emits_advisory_finding_without_error ... ok
test youtube_summary::result_validation::tests::missing_required_top_level_array_returns_error ... ok
test youtube_summary::result_validation::tests::missing_video_source_refs_is_allowed ... ok
test youtube_summary::result_validation::tests::nested_synthesis_unknown_claim_ref_returns_error_when_top_level_union_empty ... ok
test youtube_summary::preflight_tests::preflight_playlist_video_without_transcript_is_skipped ... ok
test youtube_summary::result_validation::tests::nested_synthesis_unknown_video_ref_returns_error ... ok
test youtube_summary::result_validation::tests::run_id_mismatch_returns_error ... ok
test youtube_summary::result_validation::tests::single_video_with_synthesis_object_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_duplicate_top_level_claim_ref_returns_error_at_field_path ... ok
test youtube_summary::result_validation::tests::synthesis_extra_top_level_claim_ref_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_extra_top_level_evidence_ref_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_extra_top_level_source_ref_not_in_nested_items_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_missing_nested_claim_ref_in_top_level_union_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_missing_nested_evidence_ref_in_top_level_union_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_null_skips_derived_traversal_validation ... ok
test youtube_summary::result_validation::tests::synthesis_missing_source_ref_derived_from_video_ref_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_object_missing_required_array_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_order_difference_in_top_level_union_is_allowed ... ok
test youtube_summary::result_validation::tests::synthesis_top_level_unknown_claim_ref_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_unknown_video_ref_does_not_cascade_to_source_union_error ... ok
test youtube_summary::result_validation::tests::unknown_quality_flag_is_ignored_by_mvp_validator ... ok
test youtube_summary::result_validation::tests::validate_youtube_summary_canonical_result_valid_minimal_has_no_errors ... ok
test youtube_summary::result_validation::tests::validation_error_keeps_stage_level_findings ... ok
test youtube_summary::result_validation::tests::validation_error_writes_findings_marks_run_failed_and_skips_result ... ok
test youtube_summary::result_validation::tests::validation_error_removes_stale_persisted_result_and_projections ... ok
test youtube_summary::result_validation::tests::validation_persistence_replaces_previous_result_level_findings_on_success ... ok
test youtube_summary::result_validation::tests::video_claim_refs_malformed_shape_returns_error ... ok
test youtube_summary::result_validation::tests::video_claim_refs_unknown_ref_returns_error ... ok
test youtube_summary::result_validation::tests::video_evidence_refs_unknown_ref_returns_error ... ok
test youtube_summary::result_validation::tests::video_evidence_refs_with_non_string_item_returns_error ... ok
test youtube_summary::result_validation::tests::video_source_refs_malformed_shape_returns_error ... ok
test youtube_summary::result_validation::tests::video_source_refs_missing_self_ref_returns_error ... ok
test youtube_summary::result_validation::tests::video_source_refs_unknown_ref_returns_error ... ok
test youtube_summary::result_validation::tests::video_source_refs_with_non_string_item_returns_error ... ok
test youtube_summary::result_validation::tests::video_with_unknown_source_ref_returns_error ... ok
test youtube_summary::snapshots_tests::comment_material_ref_policy_preserves_order_and_token_cap ... ok
test youtube_summary::snapshots_tests::comment_snapshot_source_reads_candidates_for_estimates_then_selected_bodies_again ... ok
test youtube_summary::result_validation::tests::validation_wrapper_rolls_back_result_findings_when_persistence_fails_after_validation ... ok
test youtube_summary::result_validation::tests::validation_persistence_writes_warning_findings_and_persists_result ... ok
test youtube_summary::snapshots_tests::empty_client_request_id_returns_before_any_database_or_source_read ... ok
test youtube_summary::snapshots_tests::duplicate_client_request_id_preserves_existing_runtime_provider ... ok
test youtube_summary::snapshots_tests::duplicate_start_ignores_runtime_blocking_failure ... ok
test youtube_summary::snapshots_tests::gem_analysis_freezes_comments_even_when_include_comments_is_false ... ok
test youtube_summary::snapshots_tests::snapshot_start_source_preserves_repeated_preflight_and_post_insert_fresh_reads ... ok
test youtube_summary::snapshots_tests::runnable_start_uses_complete_fresh_source_read_sequence ... ok
test youtube_summary::snapshots_tests::selected_comment_body_is_reloaded_after_candidate_estimation ... ok
test youtube_summary::snapshots_tests::start_persists_gemini_browser_runtime_and_config_snapshot ... ok
test youtube_summary::snapshots_tests::start_freezes_one_canonical_video_snapshot_with_multiple_origins ... ok
test youtube_summary::snapshots_tests::start_returns_existing_run_for_duplicate_client_request_id ... ok
test youtube_summary::snapshots_tests::start_with_recomputed_blocking_preflight_returns_response_without_run ... ok
test youtube_summary::snapshots_tests::start_with_runtime_blocking_failure_returns_preflight_without_run ... ok
test youtube_summary::snapshots_tests::transcript_material_policy_uses_owned_segment_reader_values ... ok
test youtube_summary::snapshots_tests::transcript_snapshot_text_is_rendered_from_structured_segments ... ok
test youtube_summary::synthesis_input_tests::build_synthesis_stage_input_merges_intermediate_graphs_and_allowed_refs ... ok
test youtube_summary::synthesis_input_tests::build_synthesis_stage_input_collects_successful_transcript_outputs ... ok
test youtube_summary::synthesis_input_tests::build_synthesis_stage_input_orders_graph_by_source_snapshot_id ... ok
test youtube_summary::synthesis_input_tests::build_synthesis_stage_input_uses_latest_parsed_output_wrappers ... ok
test youtube_summary::synthesis_input_tests::load_merged_intermediate_entities_rejects_duplicate_refs_across_sources ... ok

test result: ok. 249 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.36s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_telegram-31ef7f0b457f8e79.exe)

running 71 tests
test dto::tests::telegram_item_kind_constant_matches_persisted_wire_value ... ok
test dto::tests::telegram_message_draft_has_single_persistence_shape ... ok
test dto::tests::telegram_message_identity_validation_rejects_invalid_values ... ok
test error::tests::channel_private_detection_reads_rpc_name_from_error_message ... ok
test error::tests::non_forum_topic_refresh_errors_are_detected ... ok
test live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure ... ok
test live::messages::tests::fallback_peer_identity_uses_telegram_history_peer_vocabulary ... ok
test live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload ... ok
test live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule ... ok
test live::messages::tests::reply_peer_context_uses_telegram_peer_kinds ... ok
test live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget ... ok
test live::peer::tests::dialog_lookup_misses_are_not_found ... ok
test live::peer::tests::dialog_lookup_not_found_message_explains_numeric_manual_limit ... ok
test live::peer::tests::peer_ref_from_identity_ignores_small_groups_without_supported_identity ... ok
test live::peer::tests::peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation ... ok
test live::peer::tests::peer_ref_from_identity_uses_channel_access_hash ... ok
test live::peer::tests::peer_ref_from_identity_uses_supergroup_access_hash ... ok
test live::peer::tests::typed_identity_builds_channel_peer_ref_when_access_hash_exists ... ok
test live::peer::tests::typed_identity_rejects_subtype_peer_kind_mismatch ... ok
test live::peer::tests::validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype ... ok
test live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor ... ok
test media::tests::derive_content_kind_tracks_text_and_media_presence ... ok
test live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes ... ok
test media::tests::derive_document_media_kind_prefers_specific_signals ... ok
test runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors ... ok
test runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure ... ok
test runtime::tests::client_preserves_missing_account_error_without_authorization_check ... ok
test runtime::tests::failed_sign_in_retains_pending_attempt ... ok
test runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner ... ok
test runtime::tests::missing_account_authentication_is_false ... ok
test runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt ... ok
test runtime::tests::sign_in_without_code_request_preserves_auth_error ... ok
test runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt ... ok
test session::tests::encrypted_session_load_fails_for_wrong_account_id ... ok
test session::tests::encrypted_session_load_round_trips ... ok
test session::tests::generated_session_key_returns_write_only_encoded_secret ... ok
test session::tests::legacy_json_returns_rewrite_decision ... ok
test session::tests::missing_encrypted_key_preserves_auth_error ... ok
test session::tests::session_encryption_key_rejects_invalid_length ... ok
test takeout::export_dc::tests::export_dc_attempt_state_detects_first_fallback_transition ... ok
test takeout::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors ... ok
test session::tests::saving_session_writes_encrypted_envelope_not_plaintext ... ok
test takeout::export_dc::tests::export_dc_id_applies_tdesktop_shift ... ok
test takeout::export_dc::tests::export_dc_invoke_falls_back_to_home_dc_on_local_error ... ok
test takeout::export_dc::tests::export_dc_invoke_does_not_fallback_for_rpc_errors ... ok
test takeout::export_dc::tests::export_dc_invoke_uses_home_dc_directly_after_fallback ... ok
test takeout::export_dc::tests::takeout_init_request_uses_source_subtype_flags_and_file_limit ... ok
test takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots ... ok
test takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping ... ok
test takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome ... ok
test takeout::operations::tests::history_page_and_search_return_owned_takeout_messages ... ok
test takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity ... ok
test takeout::operations::tests::only_my_messages_fallback_is_limited_to_channels ... ok
test takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges ... ok
test takeout::pagination::tests::descending_fallback_keeps_raw_order_and_moves_to_min_message_id ... ok
test takeout::pagination::tests::messages_not_modified_response_is_rejected_for_takeout_page ... ok
test takeout::pagination::tests::messages_response_without_slice_is_terminal_page ... ok
test takeout::pagination::tests::split_selection_falls_back_when_telegram_returns_no_ranges ... ok
test takeout::pagination::tests::split_selection_uses_all_ranges_for_small_group ... ok
test takeout::pagination::tests::split_selection_uses_last_range_for_channel_and_supergroup ... ok
test takeout::pagination::tests::tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback ... ok
test takeout::pagination::tests::tdesktop_non_advancing_cursor_restarts_descending_fallback ... ok
test takeout::pagination::tests::tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id ... ok
test takeout::raw_parse::tests::parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids ... ok
test takeout::raw_parse::tests::parses_document_media_kind_filename_and_dimensions ... ok
test takeout::raw_parse::tests::parses_photo_message_metadata ... ok
test takeout::raw_parse::tests::parses_text_message_with_reply_and_reactions ... ok
test takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids ... ok
test takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id ... ok
test takeout::raw_parse::tests::skips_empty_raw_messages ... ok
test takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error ... ok

test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

#### npm-verify.txt

```text

> extractum@0.2.0 verify
> node scripts/verify.mjs


=== npm run test ===

> extractum@0.2.0 test
> node scripts/run-vitest.mjs run


 RUN  v4.1.5 G:/Develop/Extractum


 Test Files  177 passed (177)
      Tests  1639 passed (1639)
   Start at  18:30:41
   Duration  107.91s (transform 25.83s, setup 99.85s, import 48.10s, tests 104.39s, environment 24.45s)


=== npm run check ===

> extractum@0.2.0 check
> svelte-kit sync && svelte-check --tsconfig ./tsconfig.json

Loading svelte-check in workspace: g:\Develop\Extractum
Getting Svelte diagnostics...

[32msvelte-check found 0 errors and 0 warnings
[39m
=== npm run check:rustfmt ===

> extractum@0.2.0 check:rustfmt
> cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check


=== cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets ===
npm.cmd : warning: function `sidecar_unavailable_result` is never used
At line:2 char:124
+ ... e46\npm-verify-debug-2.txt'; & npm.cmd run verify *> $capture; $code= ...
+                                  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: functi...` is never used:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-prompt-packs` (lib) generated 1 warning
warning: `extractum-telegram` (lib) generated 9 warnings
warning: function `preflight_youtube_summary_in_pool` is never used
   --> crates\extractum-prompt-packs\src\youtube_summary\mod.rs:156:21
    |
156 | pub(crate) async fn preflight_youtube_summary_in_pool(
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> crates\extractum-prompt-packs\src\youtube_summary\test_support.rs:625:21
    |
625 | pub(crate) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: methods `has_waiter_for_test` and `worker_execution_timeout` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:182:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
182 |     pub(crate) fn has_waiter_for_test(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^^^^^^^^
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum-prompt-packs` (lib test) generated 2 warnings
warning: `extractum-gemini-browser` (lib test) generated 3 warnings (2 duplicates)
warning: unused imports: `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_GROUP`, and `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:12:43
   |
12 |     SourceSyncTarget, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                           ^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TELEGRAM_KIND_CHANNEL`
  --> src\sources\mod.rs:58:52
   |
58 |     NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                                    ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `GeminiBrowserArtifactMode`
 --> src\gemini_browser\jobs.rs:9:41
  |
9 |     DeliveredJobInput, DeliveryOutcome, GeminiBrowserArtifactMode, GeminiBrowserJob,
  |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, `GeminiBrowserRunStatus`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |     ^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: `extractum` (lib) generated 10 warnings (run `cargo fix --lib -p extractum` to apply 5 suggestions)
warning: `extractum` (lib test) generated 14 warnings (5 duplicates) (run `cargo fix --lib -p extractum --tests` to app
ly 4 suggestions)
warning: `extractum-telegram` (lib test) generated 4 warnings (4 duplicates)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.24s

=== cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets ===
warning: function `sidecar_unavailable_result` is never used
  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-prompt-packs` (lib) generated 1 warning
warning: `extractum-telegram` (lib) generated 9 warnings
warning: function `preflight_youtube_summary_in_pool` is never used
   --> crates\extractum-prompt-packs\src\youtube_summary\mod.rs:156:21
    |
156 | pub(crate) async fn preflight_youtube_summary_in_pool(
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> crates\extractum-prompt-packs\src\youtube_summary\test_support.rs:625:21
    |
625 | pub(crate) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum-prompt-packs` (lib test) generated 2 warnings
warning: unused imports: `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_GROUP`, and `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:12:43
   |
12 |     SourceSyncTarget, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                           ^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TELEGRAM_KIND_CHANNEL`
  --> src\sources\mod.rs:58:52
   |
58 |     NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                                    ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `GeminiBrowserArtifactMode`
 --> src\gemini_browser\jobs.rs:9:41
  |
9 |     DeliveredJobInput, DeliveryOutcome, GeminiBrowserArtifactMode, GeminiBrowserJob,
  |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, `GeminiBrowserRunStatus`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |     ^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:13:5
   |
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `TaskSink`
   --> src\gemini_browser\jobs.rs:344:28
    |
344 |         BoxDynError, Data, TaskSink, WorkerBuilder, WorkerBuilderExt, WorkerContext,
    |                            ^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |                                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserDebugErrorStage` and `GeminiBrowserRunDebugSummary`
  --> src\gemini_browser\mod.rs:42:5
   |
42 |     GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `test_pool_with_ready_video` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:146:21
    |
146 | pub(super) async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `test_pool_with_comments_out_of_order` is never used
   --> src\prompt_packs\youtube_summary\test_support.rs:153:21
    |
153 | pub(super) async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `insert_playlist` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:46:21
   |
46 | pub(super) async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
   |                     ^^^^^^^^^^^^^^^

warning: function `insert_playlist_item` is never used
  --> src\prompt_packs\youtube_summary\test_support.rs:71:21
   |
71 | pub(super) async fn insert_playlist_item(
   |                     ^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^

warning: methods `has_waiter_for_test` and `worker_execution_timeout` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:182:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
182 |     pub(crate) fn has_waiter_for_test(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^^^^^^^^
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib) generated 10 warnings (run `cargo fix --lib -p extractum` to apply 5 suggestions)
warning: `extractum` (lib test) generated 14 warnings (5 duplicates) (run `cargo fix --lib -p extractum --tests` to app
ly 4 suggestions)
warning: `extractum-telegram` (lib test) generated 4 warnings (4 duplicates)
warning: `extractum-gemini-browser` (lib test) generated 3 warnings (2 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.99s
     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_lib-51295d1508d3c6a6.exe)

running 665 tests
test account_deletion::tests::active_source_job_on_owned_source_blocks_but_unowned_job_does_not ... ok
test account_deletion::tests::active_llm_request_for_completed_owned_run_blocks_but_provider_test_does_not ... ok
test account_deletion::tests::active_group_analysis_run_blocks_when_any_member_source_is_owned ... ok
test account_deletion::tests::active_direct_source_analysis_run_blocks_owned_source_only ... ok
test account_deletion::tests::missing_account_returns_not_found ... ok
test account_deletion::tests::existing_account_with_zero_sources_passes ... ok
test account_deletion::tests::blocker_collection_keeps_multiple_categories_for_internal_diagnostics ... ok
test account_deletion::tests::active_takeout_job_on_owned_source_blocks ... ok
test accounts::tests::creating_account_writes_api_hash_to_secret_store_only ... ok
test accounts::tests::deleting_account_removes_secret_after_database_row ... ok
test accounts::tests::creating_account_rolls_back_when_secret_write_fails ... ok
test account_deletion::tests::source_ingest_lock_on_owned_source_blocks_without_deleting_rows ... ok
test accounts::tests::secret_cleanup_failure_keeps_deleted_database_row_deleted ... ok
test accounts::tests::deleting_missing_account_returns_not_found ... ok
test analysis::corpus::tests::live::default_analysis_corpus_excludes_migrated_history_documents ... ok
test analysis::corpus::tests::live::description_mode_creates_synthetic_description_message ... ok
test analysis::corpus::tests::live::explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus ... ok
test analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion ... ok
test analysis::corpus::tests::live::load_corpus_messages_filters_youtube_transcript_only_to_transcripts ... ok
test analysis::corpus::tests::live::live_corpus_refs_use_local_item_ids ... ok
test analysis::corpus::tests::live::load_corpus_messages_filters_telegram_to_telegram_message ... ok
test analysis::corpus::tests::live::load_corpus_messages_includes_youtube_comment_only_in_comments_mode ... ok
test analysis::corpus::tests::live::load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content ... ok
test analysis::corpus::tests::live::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref ... ok
test analysis::corpus::tests::live::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight ... ok
test analysis::corpus::tests::live::preflight_ref_format_matches_corpus_loader_ref_format ... ok
test analysis::corpus::tests::live::source_group_opt_in_includes_only_members_with_migrated_rows ... ok
test analysis::corpus::tests::live::youtube_description_missing_typed_metadata_skips_without_decoding_source_blob ... ok
test analysis::corpus::tests::live::youtube_description_rows_use_typed_metadata_with_corrupt_source_blob ... ok
test analysis::corpus::tests::live::youtube_transcript_segment_evidence_uses_typed_source_context ... ok
test analysis::corpus::tests::preflight::preflight_ignores_media_only_items_without_text_content ... ok
test analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows ... ok
test analysis::corpus::tests::preflight::preflight_count_matches_loader_for_youtube_corpus_modes ... ok
test analysis::corpus::tests::preflight::preflight_counts_eligible_text_messages_for_sources ... ok
test analysis::corpus::tests::source_resolution::resolve_analysis_sources_loads_single_provider_project ... ok
test analysis::corpus::tests::source_resolution::resolve_analysis_sources_rejects_mixed_provider_project ... ok
test analysis::corpus::tests::source_resolution::resolve_analysis_sources_preserves_no_linked_youtube_error_message ... ok
test analysis::corpus::tests::source_resolution::resolve_run_source_ids_loads_project_sources_without_snapshot ... ok
test analysis::fixtures::tests::active_runs::fixture_active_state_tracks_seeded_running_run ... ok
test analysis::fixtures::tests::clear::clear_deletes_child_rows_through_fixture_parent_ids ... ok
test analysis::fixtures::tests::active_runs::fixture_cancel_waiter_marks_running_run_cancelled ... ok
test analysis::fixtures::tests::clear::clear_preserves_non_fixture_groups_and_members ... ok
test analysis::fixtures::tests::harness::fixture_test_pool_has_required_tables ... ok
test analysis::fixtures::tests::clear::clear_removes_only_fixture_rows_and_is_idempotent ... ok
test analysis::fixtures::tests::seed::compressed_fixture_fields_are_readable ... ok
test analysis::fixtures::tests::seed::seed_creates_fixture_runs_with_statuses_templates_and_snapshots ... ok
test analysis::fixtures::tests::seed::seed_creates_post_sync_reader_content ... ok
test analysis::fixtures::tests::seed::seed_creates_safe_account_prompt_profile_sources_and_group ... ok
test analysis::fixtures::tests::seed::seed_creates_sources_that_pass_identity_repair ... ok
test analysis::fixtures::tests::seed::seed_creates_valid_typed_youtube_detail_metadata ... ok
test analysis::fixtures::tests::snapshot::capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report ... ok
test analysis::fixtures::tests::seed::seed_twice_keeps_one_deterministic_fixture_set ... ok
test analysis::fixtures::tests::snapshot::fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot ... ok
test analysis::fixtures::tests::summary::summary_serializes_with_camel_case_keys ... ok
test analysis::groups::tests::prepare_analysis_source_group_input_preserves_baseline_error_precedence ... ok
test analysis::groups::tests::validate_group_source_type_accepts_matching_provider_membership ... ok
test analysis::groups::tests::validate_group_source_type_rejects_mixed_provider_membership ... ok
test analysis::groups::tests::validate_group_source_type_rejects_unknown_group_type ... ok
test analysis::store::tests::read_model::list_analysis_run_summaries_filters_project_runs ... ok
test analysis::report::tests::capture::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases ... ok
test analysis::store::tests::read_model::list_analysis_run_summaries_matches_all_query_terms_across_any_field ... ok
test analysis::store::tests::setup::ensure_sources_exist_returns_typed_not_found_error ... ok
test analysis::tests_application::analysis_wire_values_serialize_to_exact_json_objects ... ok
test analysis::tests_application::analysis_run_search_escapes_percent_underscore_and_backslash_before_limit ... ok
test analysis::tests_application::chat_profile_resolution_failure_is_async_after_request_id ... ok
test analysis::tests_application::chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot ... ok
test analysis::tests_application::report_profile_resolution_failure_prevents_run_creation ... ok
test analysis::tests_application::report_start_preserves_acceptance_order_and_two_corpus_reads ... ok
test analysis::tests_application::run_reads_preserve_deleted_blank_and_snapshot_scope_labels ... ok
test analysis_documents::tests::document_metadata_envelopes_match_current_evidence_shape ... ok
test analysis_documents::tests::rebuild_analysis_documents_excludes_migrated_history_rows ... ok
test analysis_documents::tests::rebuild_source_materializes_text_units_with_document_order ... ok
test analysis::fixtures::tests::snapshot::missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages ... ok
test analysis_documents::tests::schema_creates_analysis_documents_constraints_and_indexes ... ok
test analysis_documents::tests::rebuild_source_removes_stale_documents_and_is_idempotent ... ok
test apalis_jobs::tests::apalis_jobs_counts_ignore_their_own_active_filter ... ok
test apalis_jobs::tests::apalis_jobs_decode_failure_returns_redacted_preview_without_json ... ok
test apalis_jobs::tests::apalis_jobs_limit_excludes_large_payloads_outside_limited_rows ... ok
test apalis_jobs::tests::apalis_jobs_list_clamps_limit ... ok
test apalis_jobs::tests::apalis_jobs_list_returns_empty_when_jobs_table_missing ... ok
test analysis::fixtures::tests::snapshot::seeded_snapshot_runs_expose_captured_snapshot_state ... ok
test apalis_jobs::tests::apalis_jobs_list_does_not_mutate_jobs ... ok
test apalis_jobs::tests::apalis_jobs_list_filters_by_status_job_type_and_search ... ok
test apalis_jobs::tests::apalis_jobs_list_returns_rows_from_jobs_table ... ok
test apalis_jobs::tests::apalis_jobs_list_returns_rfc3339_utc_timestamps ... ok
test apalis_jobs::tests::apalis_jobs_non_json_result_and_metadata_are_omitted_in_v1 ... ok
test apalis_jobs::tests::apalis_jobs_prune_terminal_returns_zero_when_jobs_table_missing ... ok
test apalis_jobs::tests::apalis_jobs_list_sorts_by_latest_activity_timestamp ... ok
test apalis_jobs::tests::apalis_jobs_row_shape_is_stable_when_optional_columns_are_absent ... ok
test apalis_jobs::tests::apalis_jobs_payloads_are_redacted_and_truncated ... ok
test apalis_jobs::tests::apalis_jobs_schema_probe_documents_local_jobs_table_shape ... ok
test archive_read_model::tests::create_schema_adds_state_and_item_tables ... ok
test apalis_jobs::tests::apalis_jobs_prune_terminal_deletes_only_old_done_killed_and_terminal_failed_jobs ... ok
test child_process::tests::create_no_window_matches_win32_process_creation_flags ... ok
test archive_read_model::tests::current_ready_state_rejects_old_model_version ... ok
test archive_read_model::tests::rebuild_source_excludes_migrated_history_rows ... ok
test diagnostics::dto::tests::diagnostic_summary_fixture_serializes_without_forbidden_sentinels ... ok
test diagnostics::database::tests::migration_status_reports_pending_and_failed_versions ... ok
test diagnostics::redaction::tests::redact_json_value_redacts_sensitive_keys_recursively ... ok
test archive_read_model::tests::rebuild_source_materializes_archive_fidelity_fields ... ok
test diagnostics::redaction::tests::redact_text_removes_secret_and_content_patterns ... ok
test diagnostics::runtime::tests::failed_runtime_check_uses_coarse_summary_without_os_error_text ... ok
test diagnostics::redaction::tests::sanitized_error_message_is_bounded ... ok
test diagnostics::redaction::tests::sanitized_error_message_bounds_unicode_by_chars ... ok
test diagnostics::runtime::tests::secure_storage_failure_does_not_expose_store_error_text ... ok
test diagnostics::tests::serialized_diagnostic_summary_preserves_allowed_data_and_excludes_forbidden_data ... ok
test external_process::tests::admission_wait_consumes_the_shared_graceful_budget ... ok
test external_process::tests::cleanup_tasks_start_concurrently_and_isolate_error_and_panic ... ok
test external_process::tests::concurrent_watchdogs_invoke_exit_once ... ok
test external_process::tests::exhausted_admission_budget_skips_the_cleanup_factory ... ok
test external_process::tests::injected_watchdog_scheduler_receives_timing_and_runs_the_gated_callback ... ok
test external_process::tests::permit_drop_between_waiter_registration_and_await_does_not_stall_shutdown ... ok
test external_process::tests::repeated_start_does_not_replace_code_or_schedule_again ... ok
test external_process::tests::start_reports_completed_after_watchdog_claims_exit ... ok
test external_process::tests::start_returns_started_and_schedules_one_watchdog ... ok
test external_process::tests::timing_exposes_the_graceful_and_watchdog_budgets ... ok
test external_process::tests::watchdog_exits_with_the_preserved_code_unless_cleanup_completed ... ok
test gemini_browser::cdp_chrome::tests::drop_falls_back_to_owned_child_shutdown ... ok
test gemini_browser::cdp_chrome::tests::explicit_shutdown_kills_and_reaps_the_owned_child_once ... ok
test gemini_browser::cdp_chrome::tests::shutdown_does_not_claim_or_kill_an_already_exited_child ... ok
test gemini_browser::cdp_chrome::tests::shutdown_reaps_when_the_child_has_already_exited_during_kill ... ok
test gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_accepts_json_version_response ... ok
test diagnostics::tests::sanitize_diagnostic_error_bounds_and_redacts_command_errors ... ok
test gemini_browser::executor::tests::cancelled_run_marks_the_sidecar_transport_tainted ... ok
test gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json ... ok
test external_process::tests::permits_acquired_before_shutdown_are_waited_for ... ok
test diagnostics::database::tests::database_diagnostics_groups_only_allow_listed_aggregates ... ok
test gemini_browser::jobs::app_tests::apalis_sqlite_status_probe_documents_actual_status_values ... ok
test gemini_browser::jobs::app_tests::apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job ... ok
test gemini_browser::jobs::app_tests::apalis_storage_uses_shared_main_extractum_db_identity ... ok
test gemini_browser::jobs::app_tests::apalis_storage_preserves_existing_sqlx_migration_history_table ... ok
test gemini_browser::cdp_chrome::tests::wait_for_cdp_endpoint_reports_unreachable_endpoint ... ok
test gemini_browser::jobs::app_tests::apalis_storage_shares_extractum_db_without_locking_app_pool ... ok
test gemini_browser::jobs::app_tests::gemini_browser_jobs_are_built_with_one_total_attempt ... ok
test gemini_browser::jobs::app_tests::enqueue_duplicate_run_id_returns_conflict ... ok
test gemini_browser::jobs::app_tests::enqueue_persists_job_before_worker_startup ... ok
test gemini_browser::sidecar::tests::stderr_drain_consumes_sidecar_output_concurrently ... ok
test ingest_provenance::tests::completed_zero_observation_batch_is_complete_without_partial_flags ... ok
test ingest_provenance::tests::create_takeout_batch_inserts_generic_and_detail_rows_atomically ... ok
test gemini_browser::jobs::app_tests::failed_gemini_browser_job_is_not_retried ... ok
test ingest_provenance::tests::migrated_history_deferred_scope_finalizes_partial_and_records_warning_once ... ok
test ingest_provenance::tests::migrated_small_group_imported_allows_duplicate_only_success ... ok
test ingest_provenance::tests::migrated_small_group_scope_can_be_marked_running_and_completed ... ok
test gemini_browser::jobs::app_tests::restart_worker_processes_pending_job_after_runtime_restart ... ok
test job_helpers::tests::active_job_guards_track_and_release_scoped_jobs ... ok
test job_helpers::tests::cancellation_state_cancels_child_tokens ... ok
test job_helpers::tests::cancellation_state_marks_checks_and_clears_jobs ... ok
test ingest_provenance::tests::mixed_partial_scope_finalizes_as_partial ... ok
test library_sources::tests::catalog_status_for_input_keeps_failed_job_without_detail_empty ... ok
test ingest_provenance::tests::terminal_update_recalculates_counters_and_sanitizes_error ... ok
test library_sources::tests::list_library_sources_keeps_sources_with_missing_provider_details ... ok
test library_sources::tests::list_library_catalog_returns_status_capabilities_and_filter_counts ... ok
test library_sources::tests::list_library_sources_returns_youtube_and_telegram_metadata ... ok
test llm::profiles::tests::changing_key_scope_without_replacement_is_rejected ... ok
test llm::profiles::tests::active_profile_resolution_loads_key_from_secret_store ... ok
test llm::profiles::tests::credential_scope_uses_provider_origin_and_effective_port_but_not_path ... ok
test llm::profiles::tests::clear_profile_api_key_deletes_secret ... ok
test llm::profiles::tests::delete_profile_fails_if_secret_store_fails_leaving_db_settings_intact ... ok
test llm::profiles::tests::delete_profile_removes_settings_and_secret_and_resets_active ... ok
test llm::profiles::tests::empty_save_preserves_existing_secret ... ok
test llm::profiles::tests::materialization_write_failure_fails_closed_during_state_load ... ok
test llm::profiles::tests::keyed_legacy_profile_materializes_effective_base_url_while_unkeyed_stays_blank ... ok
test llm::profiles::tests::legacy_remote_http_profile_is_rejected_before_request_configuration ... ok
test llm::profiles::tests::profile_settings_roundtrip_stores_api_key_in_secret_store ... ok
test llm::profiles::tests::profile_state_lists_multiple_saved_profiles ... ok
test llm::profiles::tests::provider_access_resolution_uses_configured_key_with_saved_base_url ... ok
test llm::profiles::tests::validate_profile_id_rejects_invalid_characters ... ok
test llm::profiles::tests::set_active_profile_returns_typed_not_found_error ... ok
test llm::tests::llm_command_errors_and_failed_events_keep_distinct_json_shapes ... ok
test llm::tests::llm_stream_events_serialize_exact_lifecycle_contract ... ok
test llm::profiles::tests::provider_access_resolution_uses_saved_key_with_configured_base_url ... ok
test gemini_browser::jobs::app_tests::worker_picks_up_job_quickly_after_idle ... ok
test migrations::baseline_reset::tests::classifies_baseline_history_after_post_baseline_migrations ... ok
test migrations::baseline_reset::tests::classifies_baseline_history_only_when_checksum_matches ... ok
test llm::tests::provider_diagnostics_exclude_profile_ids_and_base_urls ... ok
test migrations::baseline_reset::tests::classifies_old_history_only_when_versions_one_through_twenty_six_are_successful ... ok
test migrations::baseline_reset::tests::rejects_baseline_history_with_wrong_checksum ... ok
test migrations::baseline_reset::tests::rejects_partial_old_history_without_version_twenty_six ... ok
test migrations::baseline_reset::tests::rejects_failed_migration_history ... ok
test migrations::tests::app_migrator_accepts_database_with_preexisting_apalis_migration_history ... ok
test migrations::tests::build_migrations_includes_apalis_sqlite_versions_for_shared_sqlx_history ... ok
test migrations::tests::analysis_telegram_history_scope_migration_adds_nullable_checked_column ... ok
test migrations::tests::build_migrations_includes_prompt_pack_runtime_provider_version_ten ... ok
test migrations::tests::build_migrations_starts_at_current_schema_baseline ... ok
test migrations::tests::current_schema_baseline_checksum_matches_frozen_reset_boundary ... ok
test migrations::baseline_reset::tests::backup_failure_prevents_migration_history_rewrite ... ok
test migrations::tests::current_schema_baseline_migration_is_version_one ... ok
test migrations::tests::fresh_schema_includes_analysis_snapshot_markers ... ok
test migrations::tests::fresh_schema_includes_analysis_documents_table_indexes_and_constraints ... ok
test migrations::baseline_reset::tests::old_history_cutover_backs_up_then_rewrites_only_migration_history ... ok
test migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints ... ok
test migrations::tests::concurrent_test_migrations_publish_complete_apalis_schemas ... ok
test migrations::tests::fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints ... ok
test migrations::tests::prepare_database_skips_cutover_when_database_file_is_missing ... ok
test migrations::tests::projects_mvp_migration_is_registered ... ok
test migrations::tests::post_baseline_migration_upgrades_frozen_baseline_for_migrated_history ... ok
test migrations::tests::fresh_schema_includes_source_identity_tables_after_sql_managed_migrations ... ok
test migrations::tests::fresh_schema_includes_projects_redesign_columns_index_and_defaults ... ok
test migrations::tests::test_migration_batch_rolls_back_schema_and_history_together ... ok
test migrations::tests::vendored_apalis_sqlite_migrations_match_pinned_dependency ... ok
test notebooklm_export::chunker::tests::accounts_for_document_overhead_when_splitting ... ok
test notebooklm_export::chunker::tests::falls_back_to_month_when_year_exceeds_limits ... ok
test notebooklm_export::chunker::tests::falls_back_to_topic_id_when_topic_title_slug_is_invalid ... ok
test notebooklm_export::chunker::tests::filters_short_text_without_other_signal ... ok
test notebooklm_export::chunker::tests::groups_chunks_by_topic_slug ... ok
test notebooklm_export::chunker::tests::keeps_yearly_group_when_within_limits ... ok
test notebooklm_export::chunker::tests::splits_by_word_and_byte_limits ... ok
test notebooklm_export::filename::tests::accepts_safe_relative_child_paths ... ok
test migrations::tests::projects_mvp_schema_applies_to_memory_pool ... ok
test notebooklm_export::filename::tests::child_paths_stay_under_base ... ok
test notebooklm_export::filename::tests::rejects_reserved_components ... ok
test notebooklm_export::filename::tests::rejects_unsafe_relative_child_paths ... ok
test notebooklm_export::filename::tests::sanitizes_unsafe_filename_parts ... ok
test notebooklm_export::glossary::tests::aggregates_participants_by_author ... ok
test notebooklm_export::links::tests::detects_and_trims_http_urls ... ok
test notebooklm_export::media::tests::renders_numeric_only_media_metadata ... ok
test notebooklm_export::media::tests::renders_useful_media_placeholder_parts ... ok
test notebooklm_export::message_mapping::tests::reply_snippet_decode_failures_are_typed_internal_errors ... ok
test migrations::tests::prompt_pack_mvp_migration_creates_library_and_run_tables ... ok
test notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_bounded_periods ... ok
test notebooklm_export::query::tests::archive_export_loader_matches_items_path_for_notebooklm_messages ... ok
test notebooklm_export::query::tests::corrupt_archive_reply_target_outside_period_fails_archive_loader ... ok
test migrations::tests::prompt_pack_mvp_migration_declares_required_integrity_constraints ... ok
test notebooklm_export::query::tests::export_fixture_rejects_null_published_at_before_loader_parity ... ok
test notebooklm_export::query::tests::current_export_archive_loader_sets_scope_markers ... ok
test notebooklm_export::query::tests::load_export_messages_adds_local_reply_context_outside_period ... ok
test notebooklm_export::query::tests::load_export_messages_attaches_general_topic_when_topic_header_is_missing ... ok
test notebooklm_export::query::tests::load_export_messages_does_not_root_match_non_numeric_external_ids ... ok
test notebooklm_export::query::tests::load_export_messages_reads_materialized_topic_memberships ... ok
test notebooklm_export::query::tests::load_export_messages_attaches_topic_metadata_for_reply_and_root_messages ... ok
test notebooklm_export::query::tests::load_export_source_group_exposes_youtube_group_for_hard_validation ... ok
test notebooklm_export::query::tests::load_export_source_group_keeps_dirty_member_source_type_for_skip_logic ... ok
test notebooklm_export::query::tests::load_export_source_rejects_non_telegram_before_message_loader_selection ... ok
test notebooklm_export::query::tests::load_export_source_group_orders_members_by_title_then_id ... ok
test notebooklm_export::query::tests::load_export_source_uses_canonical_subtype_not_legacy_kind ... ok
test notebooklm_export::query::tests::migrated_export_reply_lookup_stays_inside_old_history_domain ... ok
test notebooklm_export::query::tests::notebooklm_default_export_excludes_migrated_history_rows ... ok
test notebooklm_export::query::tests::notebooklm_archive_export_excludes_migrated_history_rows_even_if_materialized ... ok
test notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_missing_and_old_version ... ok
test notebooklm_export::query::tests::notebooklm_export_query_file_has_no_export_row_mapping ... ok
test notebooklm_export::query::tests::notebooklm_export_loader_selection_uses_archive_for_ready_current_state ... ok
test notebooklm_export::query::tests::notebooklm_export_loader_selection_reports_all_fallback_reasons ... ok
test notebooklm_export::query::tests::notebooklm_export_wrapper_does_not_fallback_after_archive_selection_fails ... ok
test notebooklm_export::renderer::tests::formats_metadata_as_rfc3339 ... ok
test notebooklm_export::renderer::tests::renders_json_compatible_yaml_string_scalars ... ok
test notebooklm_export::renderer::tests::renders_message_metadata_and_text ... ok
test notebooklm_export::renderer::tests::renders_migrated_history_scope_metadata ... ok
test notebooklm_export::renderer::tests::renders_reply_thread_and_reaction_metadata ... ok
test notebooklm_export::query::tests::notebooklm_export_wrapper_uses_archive_reply_context_after_ready_selection ... ok
test notebooklm_export::renderer::tests::renders_topic_aware_document_header ... ok
test notebooklm_export::tests::formats_timestamp_folder_suffix ... ok
test notebooklm_export::tests::group_member_manifest_records_source_scoped_generated_files ... ok
test notebooklm_export::tests::keeps_migrated_history_opt_in_in_validated_config ... ok
test notebooklm_export::tests::load_group_export_inputs_rejects_youtube_group_for_hard_validation ... ok
test notebooklm_export::query::tests::opted_in_export_loads_migrated_rows_separately_with_markers ... ok
test notebooklm_export::tests::marker_read_and_write_reject_existing_symlink_file ... ok
test notebooklm_export::tests::marker_read_and_write_accept_normal_file ... ok
test notebooklm_export::query::tests::notebooklm_export_wrapper_matches_items_path_for_missing_stale_and_failed_states ... ok
test notebooklm_export::tests::prefix_chunk_filename_adds_sources_directory_and_prefix ... ok
test notebooklm_export::tests::load_group_export_inputs_rejects_group_without_telegram_members ... ok
test notebooklm_export::tests::reads_legacy_single_source_manifest_after_manifest_expansion ... ok
test notebooklm_export::tests::remove_generated_files_rejects_symlink_parent_directory ... ok
test notebooklm_export::tests::remove_generated_files_rejects_invalid_manifest_relative_path ... ok
test notebooklm_export::tests::render_source_export_filters_messages_and_clears_media_placeholders ... ok
test notebooklm_export::tests::render_source_export_tracks_empty_migrated_history_warning_and_section_prefix ... ok
test notebooklm_export::tests::render_source_group_export_errors_when_all_members_empty_after_filters ... ok
test notebooklm_export::tests::render_source_group_export_keeps_empty_member_skipped_reason_out_of_member_warnings ... ok
test notebooklm_export::tests::source_member_file_prefix_includes_index_id_and_slug ... ok
test notebooklm_export::tests::source_member_file_prefix_uses_fallback_slug_for_unsafe_title ... ok
test notebooklm_export::tests::treats_blank_export_id_as_missing ... ok
test notebooklm_export::tests::trims_optional_export_id ... ok
test notebooklm_export::tests::validates_exactly_one_export_scope ... ok
test notebooklm_export::tests::validates_period_order ... ok
test notebooklm_export::tests::validates_single_source_scope ... ok
test notebooklm_export::tests::validates_source_group_scope ... ok
test notebooklm_export::tests::write_export_file_rejects_symlink_parent_directory ... ok
test notebooklm_export::tests::removes_generated_files_in_sources_subdirectory ... ok
test notebooklm_export::tests::write_group_export_package_records_group_manifest_and_source_files ... ok
test notebooklm_export::tests::write_export_file_creates_sources_parent_directory ... ok
test process_tree::tests::creates_a_job_object ... ok
test process_tree::tests::terminate_failure_remains_reportable_and_retryable ... ok
test process_tree::tests::process_tree_guard_can_be_owned_by_async_application_state ... ok
test process_tree::tests::assigns_a_directly_owned_std_child ... ok
test process_tree::tests::terminate_is_idempotent ... ok
test process_tree::tests::dropping_the_guard_closes_the_job_and_kills_its_children ... ok
test projects::data_range::tests::project_data_range_preserves_migrated_history_error_for_unknown_source_type ... ok
test projects::data_range::tests::project_data_range_includes_telegram_migrated_history_when_requested ... ok
test projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources ... ok
test projects::data_range::tests::project_data_range_rejects_migrated_history_for_non_telegram ... ok
test projects::data_range::tests::project_data_range_rejects_migrated_history_for_unmaterialized_playlist_project ... ok
test projects::data_range::tests::project_data_range_rejects_mixed_provider_project ... ok
test projects::data_range::tests::project_data_range_returns_nulls_for_empty_project ... ok
test projects::data_range::tests::project_data_range_returns_nulls_for_unmaterialized_playlist_project ... ok
test projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds ... ok
test projects::read_model::tests::list_research_projects_counts_playlist_linked_video_materials ... ok
test projects::read_model::tests::list_research_projects_derives_counts_status_and_last_run_without_fanout ... ok
test projects::read_model::tests::list_research_projects_prioritizes_running_and_sorts_active_pinned_updated_first ... ok
test projects::tests::create_project_trims_and_rejects_duplicate_names_case_insensitively ... ok
test projects::tests::add_project_sources_is_idempotent_and_lists_ui_ready_rows ... ok
test projects::tests::delete_project_removes_membership_and_project_runs_but_keeps_sources ... ok
test projects::tests::list_project_sources_includes_catalog_status_last_sync_and_handle ... ok
test projects::tests::list_project_sources_counts_playlist_linked_video_materials ... ok
test projects::tests::project_scoped_delete_blocks_other_active_and_archived_projects_without_mutation ... ok
test projects::tests::project_scoped_delete_rejects_invalid_sources_and_missing_links ... ok
test projects::tests::project_scoped_delete_caps_blocking_projects_and_reports_remaining_count ... ok
test projects::tests::project_scoped_delete_removes_youtube_video_and_cascaded_materials ... ok
test projects::tests::set_project_archived_toggles_timestamp_and_rejects_missing_project ... ok
test prompt_packs::browser_adapter::tests::browser_port_delegates_readiness_submission_and_cancellation_without_narrowing_result ... ok
test projects::tests::project_scoped_delete_schema_source_foreign_keys_are_delete_safe ... ok
test prompt_packs::event_adapter::tests::typed_events_map_to_exact_legacy_ipc_payloads ... ok
test prompt_packs::runtime_commands::tests::execution_adapter_resolves_api_profile_only_inside_spawned_task ... ok
test prompt_packs::runtime_commands::tests::execution_adapter_spawns_exactly_once_per_ticket ... ok
test prompt_packs::runtime_commands::tests::execution_task_reuses_start_pool_without_reacquisition ... ok
test projects::tests::set_project_pinned_toggles_flag_updates_timestamp_and_rejects_missing_project ... ok
test prompt_packs::source_adapter::tests::load_playlist_items_orders_position_then_row_id_and_preserves_unlinked_rows ... ok
test prompt_packs::source_adapter::tests::load_comment_body_performs_a_fresh_read_with_decompression_fallback ... ok
test prompt_packs::source_adapter::tests::load_source_preserves_caller_order_missing_rows_and_nullables ... ok
test prompt_packs::source_adapter::tests::load_video_maps_full_nullable_metadata_and_missing_rows ... ok
test prompt_packs::source_adapter::tests::load_transcript_segments_orders_segment_index_then_row_id ... ok
test prompt_packs::source_adapter::tests::select_comment_candidates_applies_limit_order_and_decompression_fallback ... ok
test readiness::tests::is_ready_current_requires_ready_status_and_current_version ... ok
test readiness::tests::mark_failed_returns_failed_state ... ok
test readiness::tests::mark_stale_only_changes_ready_state ... ok
test readiness::tests::readiness_status_roundtrips_wire_values ... ok
test secret_store::tests::in_memory_store_can_fail_each_operation ... ok
test secret_store::tests::secret_ids_are_stable ... ok
test secret_store::tests::state_reads_writes_and_deletes_secrets ... ok
test source_ingest::tests::active_kinds_for_sources_reports_matching_locks_only ... ok
test source_ingest::tests::lock_allows_different_sources ... ok
test source_ingest::tests::lock_rejects_concurrent_same_source_operations ... ok
test source_ingest::tests::lock_releases_when_guard_drops ... ok
test sources::identity::tests::canonical_external_id_rejects_malformed_values ... ok
test prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled ... ok
test sources::identity::tests::load_telegram_identity_returns_typed_row ... ok
test sources::identity::tests::peer_kind_matches_telegram_subtype ... ok
test sources::identity::tests::username_normalization_removes_url_and_at_syntax ... ok
test prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer ... ok
test sources::identity::tests::load_telegram_runtime_source_pairs_source_with_typed_identity ... ok
test sources::identity_repair::tests::apply_repair_creates_typed_identity_and_keeps_source_id ... ok
test sources::identity_repair::tests::apply_repair_is_idempotent ... ok
test sources::identity_repair::tests::dry_run_reports_repair_without_writing_typed_rows ... ok
test sources::identity_repair::tests::duplicate_canonical_identity_reports_conflicting_source_ids ... ok
test sources::identity_repair::tests::fatal_repair_rolls_back_and_does_not_create_canonical_index ... ok
test sources::identity_repair::tests::duplicate_typed_peer_identity_reports_conflicting_source_ids ... ok
test sources::identity_repair::tests::missing_account_id_is_fatal ... ok
test sources::identity_repair::tests::repair_fails_on_conflicting_typed_projection_drift ... ok
test sources::identity_repair::tests::repair_creates_minimal_typed_identity_when_legacy_metadata_is_missing_or_malformed ... ok
test sources::identity_repair::tests::repair_fails_when_canonical_identity_is_invalid_even_with_legacy_peer_metadata ... ok
test sources::identity_repair::tests::repair_ignores_malformed_metadata_when_canonical_identity_is_present ... ok
test sources::identity_repair::tests::repair_ignores_optional_enrichment_gaps_when_typed_identity_is_valid ... ok
test sources::identity_repair::tests::repair_reads_post_v19_sources_without_legacy_column ... ok
test sources::identity_repair::tests::malformed_external_ids_fail_without_writing_typed_rows ... ok
test sources::identity_repair::tests::repair_treats_typed_projection_mismatch_as_fatal ... ok
test sources::identity_repair::tests::repair_rejects_zero_external_id ... ok
test sources::identity_repair::tests::repair_skips_malformed_metadata_when_typed_identity_is_valid ... ok
test sources::identity_repair::tests::source_identity_gate_blocks_while_running ... ok
test sources::identity_repair::tests::source_identity_gate_returns_startup_failure ... ok
test sources::identity_repair::tests::repair_uses_canonical_subtype_without_legacy_kind ... ok
test sources::identity_repair::tests::repair_updates_non_conflicting_typed_projection_drift ... ok
test sources::items::query::tests::archive_reader_matches_items_path_for_source_browsing_rows ... ok
test sources::identity_repair::tests::youtube_sources_are_unaffected_by_source_identity_repair ... ok
test sources::items::query::tests::archive_reader_matches_topic_filter_and_around_item_semantics ... ok
test sources::items::query::tests::default_items_path_excludes_migrated_history_rows ... ok
test sources::items::query::tests::default_source_browsing_does_not_surface_migrated_rows_after_archive_ready ... ok
test sources::items::query::tests::load_item_rows_attaches_topic_metadata_and_root_matches ... ok
test sources::items::query::tests::load_item_rows_can_start_at_selected_item ... ok
test sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_not_ready ... ok
test sources::items::query::tests::load_item_rows_uses_items_path_when_archive_model_is_stale ... ok
test sources::items::query::tests::merged_browsing_uses_full_cursor_tuple_for_equal_timestamps ... ok
test sources::items::query::tests::scoped_browsing_can_load_only_migrated_rows_with_labels ... ok
test sources::items::query::tests::scoped_browsing_defaults_to_current_rows ... ok
test sources::items::query::tests::topic_filters_are_rejected_for_non_current_history_scope ... ok
test sources::items::tests::forum_topic_filter_deserializes_camel_case_topic_id ... ok
test sources::items::query::tests::telegram_load_item_rows_uses_items_path_when_archive_model_is_ready ... ok
test sources::items::query::tests::uncategorized_filter_returns_empty_when_topic_resolution_is_not_ready ... ok
test sources::items::tests::insert_telegram_source_item_allows_same_message_id_in_different_history_domains ... ok
test sources::items::tests::insert_telegram_source_item_resolves_topic_membership_only_for_new_item ... ok
test sources::items::tests::insert_telegram_source_item_skips_duplicate_native_identity_without_updating_payload ... ok
test sources::items::tests::insert_telegram_source_item_writes_payload_and_skips_duplicates ... ok
test sources::items::tests::media_metadata_roundtrip_through_zstd ... ok
test sources::items::tests::list_source_items_enriches_youtube_comment_rows_from_raw_payload ... ok
test sources::items::tests::list_source_items_keeps_base_youtube_comment_when_raw_payload_is_malformed ... ok
test sources::items::tests::migrated_insert_idempotency_uses_old_chat_native_identity ... ok
test sources::items::tests::scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item ... ok
test sources::items::tests::migrated_small_group_insert_skips_current_history_derived_writes ... ok
test sources::items::tests::single_telegram_insert_maintains_ready_archive_model ... ok
test sources::items::tests::takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build ... ok
test sources::items::tests::telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate ... ok
test sources::items::tests::text_roundtrip_through_zstd ... ok
test sources::items::tests::upsert_youtube_comment_item_updates_existing_text_and_reaction_count ... ok
test sources::items::tests::telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows ... ok
test sources::items::tests::telegram_insert_writes_analysis_document_in_same_writer_transaction ... ok
test sources::items::tests::youtube_comment_upsert_targets_non_telegram_partial_unique_index ... ok
test sources::items::tests::upsert_youtube_transcript_item_updates_existing_text_and_returns_id ... ok
test sources::legacy_metadata_cleanup::tests::audit_ignores_non_telegram_and_null_metadata_rows ... ok
test sources::items::tests::youtube_comment_upsert_writes_analysis_document_and_updates_content ... ok
test sources::items::tests::youtube_transcript_upsert_targets_non_telegram_partial_unique_index ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_invalid_typed_identity ... ok
test sources::legacy_metadata_cleanup::tests::audit_reports_eligible_legacy_telegram_metadata_without_mutating ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_missing_typed_identity ... ok
test sources::legacy_metadata_cleanup::tests::candidate_skip_reason_rejects_unparseable_typed_identity_values ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_unsupported_subtype_and_missing_account ... ok
test sources::legacy_metadata_cleanup::tests::audit_skips_subtype_and_account_mismatches ... ok
test sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids ... ok
test sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_empty_refs_as_validation ... ok
test sources::peer_resolution::manual_ref::tests::parse_supported_manual_telegram_source_ref_rejects_private_links ... ok
test sources::peer_resolution::manual_ref::tests::parse_username_accepts_username_and_t_me_links ... ok
test sources::peer_resolution::tests::add_source_resolution_strategy_distinguishes_username_and_dialog_flows ... ok
test sources::peer_resolution::tests::source_metadata_decode_failures_are_internal ... ok
test sources::peer_resolution::tests::source_metadata_decodes_old_dialog_payloads_into_peer_identity ... ok
test sources::legacy_metadata_cleanup::tests::clear_is_idempotent_after_eligible_metadata_is_removed ... ok
test sources::peer_resolution::tests::source_metadata_decodes_old_username_only_payloads ... ok
test sources::peer_resolution::tests::source_metadata_decodes_typed_peer_identity_payloads ... ok
test sources::peer_resolution::tests::source_peer_input_rejects_malformed_external_id_as_validation ... ok
test sources::peer_resolution::tests::source_peer_input_rejects_unsupported_source_type_as_validation ... ok
test sources::peer_resolution::tests::source_peer_resolution_failure_explains_small_group_dialog_dependency ... ok
test sources::legacy_metadata_cleanup::tests::clear_nulls_only_eligible_legacy_telegram_metadata ... ok
test sources::peer_resolution::tests::source_peer_resolution_plan_prefers_explicit_strategy_order ... ok
test sources::peer_resolution::tests::typed_identity_plan_allows_username_resolution_without_access_hash ... ok
test sources::peer_resolution::tests::typed_identity_plan_keeps_dialog_group_dependent_on_dialog_scan ... ok
test sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_channel_stored_peer_when_access_hash_exists ... ok
test sources::peer_resolution::tests::typed_identity_plan_prefers_dialog_supergroup_stored_peer_when_access_hash_exists ... ok
test sources::peer_resolution::tests::typed_identity_plan_prefers_stored_peer_before_username_when_access_hash_exists ... ok
test sources::peer_resolution::tests::typed_identity_plan_skips_unusable_stored_peer_when_access_hash_is_missing ... ok
test sources::settings::tests::initial_sync_policy_label_formats_messages_and_days ... ok
test sources::settings::tests::validate_sync_settings_rejects_out_of_range_values ... ok
test sources::store::tests::avatar_cache_key_skips_non_telegram_metadata ... ok
test sources::settings::tests::sync_settings_default_when_app_settings_are_missing ... ok
test sources::settings::tests::sync_settings_roundtrip_through_app_settings ... ok
test sources::store::tests::delete_source_is_blocked_when_source_is_used_by_project ... ok
test sources::store::tests::dialog_picked_channel_writes_dialog_typed_identity_with_access_hash ... ok
test sources::store::tests::dialog_picked_group_writes_dialog_dependent_typed_identity_without_access_hash ... ok
test sources::store::tests::dialog_picked_supergroup_writes_dialog_typed_identity_with_access_hash ... ok
test sources::store::tests::list_sources_exposes_migrated_history_counts_without_old_chat_identity ... ok
test sources::store::tests::list_sources_exposes_sanitized_migrated_history_status_without_chat_id ... ok
test sources::store::tests::load_source_returns_not_found_for_missing_source ... ok
test sources::store::tests::source_record_parts_allow_non_telegram_source ... ok
test sources::store::tests::source_record_parts_emit_only_source_subtype ... ok
test sources::store::tests::telegram_identity_allows_same_peer_on_different_accounts ... ok
test sources::store::tests::telegram_identity_rejects_same_account_peer_conflict_at_typed_boundary ... ok
test sources::store::tests::telegram_source_upsert_inserts_null_metadata ... ok
test sources::store::tests::telegram_source_upsert_preserves_existing_legacy_metadata_blob ... ok
test sources::store::tests::telegram_source_upsert_rolls_back_source_when_typed_identity_fails ... ok
test sources::store::tests::telegram_source_upsert_writes_required_identity_and_available_optional_fields ... ok
test sources::store::tests::upsert_youtube_playlist_source_handles_legacy_not_null_telegram_kind ... ok
test sources::store::tests::delete_source_from_pool_enables_foreign_keys_and_cascades_dependents ... ok
test sources::store::tests::upsert_youtube_playlist_source_writes_typed_row_and_null_source_metadata ... ok
test sources::store::tests::upsert_youtube_video_source_handles_legacy_not_null_telegram_kind ... ok
test sources::store::tests::upsert_youtube_video_source_conflict_clears_existing_legacy_blob ... ok
test sources::store::tests::upsert_youtube_video_source_rejects_invalid_canonical_url_without_source_row ... ok
test sources::sync::tests::determine_sync_policy_only_applies_initial_settings_on_first_sync ... ok
test sources::store::tests::upsert_youtube_video_source_writes_typed_row_and_null_source_metadata ... ok
test sources::sync::tests::finalize_sync_preserves_existing_legacy_metadata_blob ... ok
test sources::sync::tests::sync_provider_accepts_telegram_sources ... ok
test sources::sync::tests::sync_provider_rejects_manual_youtube_video_sources ... ok
test sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error ... ok
test sources::sync::tests::finalize_sync_updates_source_state_and_typed_avatar_cache ... ok
test sources::topics::tests::forum_topic_gate_ignores_malformed_source_metadata_when_typed_identity_exists ... ok
test sources::store::tests::delete_source_waits_for_temporary_database_write_lock ... ok
test sources::test_support::tests::source_fixture_creates_expected_tables ... ok
test sources::topics::tests::forum_topic_refresh_gate_uses_typed_identity_not_legacy_kind ... ok
test sources::topics::tests::list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket ... ok
test sources::types::tests::item_kind_constants_match_persisted_wire_values ... ok
test sources::types::tests::source_type_serializes_supported_provider_values ... ok
test sources::topics::tests::topic_refresh_rebuilds_materialized_memberships ... ok
test sources::types::tests::telegram_source_subtype_parses_from_canonical_source_subtype ... ok
test sources::types::tests::telegram_source_subtype_parses_supported_values ... ok
test sources::types::tests::telegram_source_subtype_rejects_unknown_values_as_validation ... ok
test sources::types::tests::telegram_source_subtype_rejects_unsupported_source_subtype ... ok
test sources::types::tests::telegram_source_subtype_serializes_as_existing_wire_value ... ok
test takeout_import::forum_topics::tests::completed_takeout_forum_topic_refresh_policy_only_refreshes_supergroups ... ok
test sql_helpers::tests::push_i64_bind_list_binds_values_in_order ... ok
test sources::topics::tests::upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted ... ok
test takeout_import::forum_topics::tests::takeout_forum_topic_refresh_success_records_no_warning ... ok
test takeout_import::forum_topics::tests::takeout_forum_topic_refresh_failure_records_warning_before_batch_finalize ... ok
test takeout_import::migrated_history::tests::migrated_history_errors_are_typed_for_frontend_behavior ... ok
test takeout_import::migrated_history::tests::capability_available_is_source_level_and_restart_safe ... ok
test takeout_import::migrated_history::tests::migrated_small_group_identity_uses_native_old_chat_scope ... ok
test takeout_import::migrated_history::tests::validation_accepts_matching_revalidated_chat_id ... ok
test takeout_import::migrated_history::tests::validation_rejects_missing_or_changed_revalidated_chat_id ... ok
test takeout_import::recovery::tests::takeout_recovery_ignores_non_takeout_batches ... ok
test takeout_import::migrated_history::tests::capability_unavailable_keeps_reason_internal_and_clears_chat_hint ... ok
test takeout_import::recovery::tests::recovery_state_includes_migrated_history_scope_for_historical_batches ... ok
test takeout_import::recovery::tests::takeout_recovery_latest_complete_hides_older_failed ... ok
test takeout_import::recovery::tests::takeout_recovery_latest_failed_wins_over_older_complete ... ok
test takeout_import::recovery::tests::takeout_recovery_returns_partial_completed_and_hides_complete ... ok
test takeout_import::recovery::tests::takeout_recovery_running_with_active_job_is_hidden ... ok
test takeout_import::recovery::tests::takeout_recovery_running_without_active_job_is_interrupted ... ok
test takeout_import::recovery::tests::takeout_recovery_source_filter_limits_results ... ok
test takeout_import::state::tests::active_jobs_for_sources_filters_non_terminal_jobs ... ok
test takeout_import::state::tests::job_state_can_cancel_and_finish_job ... ok
test takeout_import::state::tests::job_state_cancels_child_tokens ... ok
test takeout_import::state::tests::job_state_records_history_scope_for_frontend_labels ... ok
test takeout_import::state::tests::job_state_rejects_duplicate_active_source_jobs ... ok
test takeout_import::state::tests::takeout_cancellation_smoke_fixture_finishes_cancelled_and_clears ... ok
test takeout_import::state::tests::takeout_cancellation_smoke_fixture_tracks_running_job ... ok
test takeout_import::state::tests::takeout_event_status_and_cancellation_contract_is_exact ... ok
test takeout_import::recovery::tests::takeout_recovery_warning_codes_are_unique_sorted_and_message_free ... ok
test takeout_import::tests::channel_private_count_probe_records_fallback_before_search_continuation ... ok
test takeout_import::tests::channel_private_validation_preflight_records_fallback_and_continues ... ok
test takeout_import::tests::export_dc_fallback_provenance_records_once_before_finalize ... ok
test takeout_import::tests::locked_start_allows_only_one_batch_for_same_source ... ok
test takeout_import::tests::historical_batch_completion_does_not_advance_source_watermark ... ok
test takeout_import::tests::migrated_history_detected_warning_is_sanitized ... ok
test takeout_import::tests::locked_start_conflict_creates_no_provenance_rows ... ok
test takeout_import::tests::migrated_history_start_requires_available_capability ... ok
test takeout_import::tests::migrated_history_start_records_use_same_source_takeout_lock ... ok
test takeout_import::tests::takeout_step_cancel_wrapper_allows_completed_future ... ok
test takeout_import::tests::takeout_step_cancel_wrapper_interrupts_pending_future ... ok
test takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once ... ok
test takeout_import::tests::takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists ... ok
test takeout_import::tests::takeout_subtype_load_uses_typed_identity_not_legacy_kind ... ok
test takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_duplicate_after_normal_sync_summarizes_outcomes ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_batch_summary_is_durable_and_sanitized ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_caps_samples_deterministically ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_compares_batch_to_canonical_without_content ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_snapshot_delta_uses_explicit_snapshots ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_matched_observations_for_aggregates ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_row_fidelity_dedupes_missing_observations_by_identity ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_source_snapshot_is_aggregate_and_sanitized ... ok
test telegram::tests::diagnostic_status_counts_do_not_return_account_ids_or_messages ... ok
test telegram::tests::legacy_api_hash_migrates_to_secret_store_and_blanks_column ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_excludes_non_latest_recovery_candidates ... ok
test telegram::tests::legacy_api_hash_remains_when_secret_write_fails ... ok
test telegram::tests::runtime_status_maps_to_existing_wire_strings ... ok
test telegram::tests::telegram_api_id_out_of_range_returns_typed_validation_error ... ok
test telegram::tests::telegram_status_and_event_payload_contract_is_exact ... ok
test telegram::tests::missing_secure_api_hash_for_blank_legacy_account_is_auth_error ... ok
test takeout_import::validation_diagnostics::tests::takeout_validation_warning_visibility_is_durable_only ... ok
test telegram_session_store::tests::delete_session_from_path_removes_file_and_key ... ok
test telegram_session_store::tests::encrypted_session_load_fails_when_key_is_missing ... ok
test telegram_session_store::tests::legacy_plaintext_session_remains_when_keyring_write_fails ... ok
test telegram_session_store::tests::legacy_plaintext_session_migrates_to_encrypted_file ... ok
test telegram_session_store::tests::session_path_temp_path_and_error_contract_is_exact ... ok
test topic_memberships::tests::rebuild_matches_retained_hidden_and_deleted_topics ... ok
test topic_memberships::tests::rebuild_replaces_stale_memberships_and_versions ... ok
test topic_memberships::tests::rebuild_prioritizes_specific_topic_matches_before_general_fallback ... ok
test tx::tests::begin_immediate_commit_persists_changes ... ok
test topic_memberships::tests::rebuild_uses_legacy_root_only_without_typed_child ... ok
test tx::tests::begin_immediate_rollback_discards_changes ... ok
test tx::tests::begin_immediate_with_foreign_keys_enforces_cascade ... ok
test tx::tests::finish_manual_transaction_commits_success_result ... ok
test tx::tests::finish_manual_transaction_rolls_back_error_result ... ok
test tx::tests::sqlite_ignores_foreign_keys_pragma_inside_open_transaction ... ok
test youtube::captions::tests::caption_download_args_request_json3_and_vtt_without_media ... ok
test youtube::captions::tests::caption_selection_honors_explicit_override_before_original_language ... ok
test youtube::captions::tests::caption_selection_prefers_original_then_preferred_then_english_then_any ... ok
test youtube::captions::tests::json3_parser_allows_missing_duration ... ok
test youtube::captions::tests::json3_parser_concatenates_segments_and_preserves_timing ... ok
test youtube::captions::tests::replace_transcript_segments_deletes_previous_rows_and_inserts_current_segments ... ok
test tx::tests::deferred_read_then_write_hits_busy_snapshot_under_concurrent_writer ... ok
test youtube::captions::tests::transcript_external_id_includes_language_and_track_kind ... ok
test youtube::captions::tests::vtt_parser_reads_cues_and_skips_blank_text ... ok
test youtube::captions::tests::vtt_parser_rejects_invalid_timing ... ok
test youtube::comments::tests::comment_published_at_accepts_numbers_strings_and_fallback ... ok
test youtube::comments::tests::comments_fetch_args_include_bounded_extractor_args ... ok
test youtube::comments::tests::comments_fetch_timeout_is_longer_than_metadata_preview_timeout ... ok
test youtube::comments::tests::default_comment_limit_is_bounded ... ok
test youtube::comments::tests::normalize_comments_flattens_replies_and_warns_for_timestamp_fallbacks ... ok
test youtube::comments::tests::normalize_comments_truncates_raw_comment_array_before_normalization ... ok
test youtube::cookies::tests::accepts_empty_cookie_values ... ok
test youtube::cookies::tests::accepts_http_only_cookie_rows ... ok
test youtube::cookies::tests::rejects_empty_cookie_text ... ok
test youtube::cookies::tests::rejects_files_without_cookie_rows ... ok
test youtube::cookies::tests::rejects_invalid_cookie_text_before_saving_secret ... ok
test youtube::cookies::tests::stores_reads_and_clears_youtube_cookies_through_secret_store ... ok
test youtube::cookies::tests::validates_netscape_cookie_rows_without_exposing_values ... ok
test youtube::captions::tests::replace_transcript_segments_rebuilds_analysis_documents_by_segment_order ... ok
test youtube::detail::tests::list_summaries_uses_source_id_order_and_marks_no_captions_unavailable ... ok
test tx::tests::begin_immediate_read_then_write_survives_concurrent_writer ... ok
test youtube::detail::tests::playlist_detail_reports_ordered_items_and_summary_counts ... ok
test youtube::detail::tests::playlist_detail_uses_typed_linked_video_metadata_with_corrupt_source_blob ... ok
test youtube::detail::tests::source_summary_missing_typed_metadata_uses_generic_title_without_blob_decode ... ok
test youtube::detail::tests::summaries_use_typed_video_metadata_with_corrupt_source_blob ... ok
test youtube::detail::tests::video_detail_includes_safe_source_metadata_without_item_raw_payloads ... ok
test youtube::dto::tests::availability_status_serializes_as_snake_case ... ok
test youtube::dto::tests::preview_kind_deserializes_snake_case ... ok
test youtube::dto::tests::video_form_serializes_short_value ... ok
test youtube::errors::tests::invalid_youtube_url_maps_to_validation_error ... ok
test youtube::errors::tests::ytdlp_deleted_failures_map_to_not_found_error ... ok
test youtube::errors::tests::ytdlp_network_failures_map_to_network_error ... ok
test youtube::detail::tests::video_detail_missing_typed_metadata_returns_controlled_error ... ok
test youtube::errors::tests::ytdlp_private_failures_map_to_auth_error ... ok
test youtube::jobs::tests::active_jobs_for_sources_filters_non_terminal_direct_and_related_sources ... ok
test youtube::jobs::tests::catalog_jobs_for_sources_includes_latest_failed_jobs ... ok
test youtube::jobs::tests::diagnostic_counts_group_source_jobs_without_ids_or_raw_errors ... ok
test youtube::jobs::tests::job_state_cancels_child_tokens ... ok
test youtube::detail::tests::video_detail_reports_synced_transcript_comments_and_playlist_memberships ... ok
test youtube::jobs::tests::job_state_list_filters_before_limit_and_sorts_newest_first ... ok
test youtube::jobs::tests::job_state_finishes_cancel_requested_jobs_as_cancelled ... ok
test youtube::jobs::tests::job_state_rejects_duplicate_active_scope_but_allows_different_job_types ... ok
test youtube::jobs::tests::jobs_missing_typed_video_metadata_errors_after_failed_refresh ... ok
test youtube::jobs::tests::jobs_reload_missing_typed_video_metadata_after_refresh_callback ... ok
test youtube::jobs::tests::source_job_cancellation_smoke_fixture_finishes_cancelled_and_clears ... ok
test youtube::jobs::tests::source_job_cancellation_smoke_fixture_tracks_running_job ... ok
test youtube::jobs::tests::source_job_step_with_process_cancel_allows_completed_future ... ok
test youtube::jobs::tests::source_job_step_with_process_cancel_interrupts_pending_future ... ok
test youtube::jobs::tests::source_job_type_uses_comments_specific_type_for_comments_only_video_sync ... ok
test youtube::jobs::tests::source_job_workflow_file_has_no_tauri_command_adapters ... ok
test youtube::jobs::tests::retryable_playlist_video_rows_excludes_auth_deleted_and_removed_entries ... ok
test youtube::jobs::tests::source_jobs_no_longer_decode_source_metadata_blobs ... ok
test youtube::metadata::tests::availability_values_map_to_statuses ... ok
test youtube::metadata::tests::playlist_metadata_page_args_use_adjacent_playlist_range ... ok
test youtube::metadata::tests::video_fixture_maps_metadata_and_preview_fields ... ok
test youtube::metadata::tests::playlist_fixture_maps_metadata_entries_and_preview_warning ... ok
test youtube::metadata::tests::video_fixture_missing_optional_fields_maps_to_none ... ok
test youtube::playlist::tests::upsert_playlist_items_can_skip_video_source_materialization ... ok
test youtube::playlist::tests::upsert_playlist_items_marks_missing_rows_removed ... ok
test youtube::playlist::tests::playlist_item_video_source_upsert_writes_typed_video_metadata_not_source_blob ... ok
test youtube::preview::tests::preview_from_playlist_json_returns_playlist_preview ... ok
test youtube::preview::tests::preview_from_video_json_uses_parsed_url_kind ... ok
test youtube::process_runtime::tests::cancellation_reaches_all_reserved_operations ... ok
test youtube::process_runtime::tests::cookie_guard_retains_file_until_detached_reaper_finishes ... ok
test youtube::process_runtime::tests::detached_reaper_keeps_cookie_until_the_stuck_child_releases ... ok
test youtube::playlist::tests::upsert_playlist_items_reuses_existing_video_source_and_keeps_unavailable_null ... ok
test youtube::playlist::tests::upsert_playlist_items_without_materialization_reuses_existing_video_source ... ok
test youtube::process_runtime::tests::dropped_caller_keeps_child_and_registry_owned_until_shutdown_reaps_it ... ok
test youtube::process_runtime::tests::external_source_job_cancellation_reaps_its_managed_operation ... ok
test youtube::process_runtime::tests::finite_pipe_backpressure_requires_concurrent_drain ... ok
test youtube::process_runtime::tests::injected_launcher_drains_backpressured_output_before_waiting_for_exit ... ok
test youtube::process_runtime::tests::injected_nonzero_exit_preserves_not_found_classification_and_releases_registry ... ok
test youtube::process_runtime::tests::registry_reserves_an_operation_before_spawn ... ok
test youtube::process_runtime::tests::shutdown_rejects_new_ytdlp_admission_before_spawn ... ok
test youtube::process_runtime::tests::spawn_failure_rolls_back_the_registry_reservation ... ok
test youtube::process_runtime::tests::timeout_fallback_detaches_cookie_until_stuck_child_reaps ... ok
test youtube::runtime::tests::runtime_status_serializes_with_camel_case_keys ... ok
test youtube::process_runtime::tests::injected_wait_error_reaps_the_child_before_releasing_registry ... ok
test youtube::settings::tests::auth_cookies_load_only_when_auth_is_enabled ... ok
test youtube::settings::tests::invalid_stored_settings_return_validation_error_with_key ... ok
test youtube::settings::tests::invalid_youtube_settings_do_not_write_partial_values ... ok
test youtube::settings::tests::validate_youtube_settings_normalizes_preferred_captions_language ... ok
test youtube::settings::tests::saving_cookies_enables_auth_and_clear_disables_it ... ok
test youtube::settings::tests::validate_youtube_settings_rejects_out_of_range_values ... ok
test youtube::settings::tests::youtube_settings_default_when_app_settings_are_missing ... ok
test youtube::settings::tests::youtube_settings_serializes_with_camel_case_keys ... ok
test youtube::source_metadata::tests::playlist_metadata_columns_are_versioned_and_secret_safe ... ok
test youtube::settings::tests::youtube_settings_roundtrip_through_app_settings ... ok
test youtube::source_metadata::tests::video_metadata_columns_include_wire_values_arrays_caption_override_and_sanitized_raw ... ok
test youtube::source_metadata::tests::video_metadata_rejects_wrong_canonical_url_shape ... ok
test youtube::source_metadata::tests::upsert_video_metadata_maintains_description_document ... ok
test youtube::source_metadata::tests::video_source_metadata_restores_raw_caption_metadata_for_provider_sync ... ok
test youtube::thumbnail::tests::accepts_only_allowlisted_https_thumbnail_urls ... ok
test youtube::thumbnail::tests::bounds_thumbnail_responses_to_one_mib ... ok
test youtube::thumbnail::tests::builds_the_dedicated_thumbnail_client ... ok
test youtube::thumbnail::tests::recognizes_supported_image_magic_bytes ... ok
test youtube::transcript_reader::tests::list_youtube_transcript_segments_filters_by_search ... ok
test youtube::transcript_reader::tests::list_youtube_transcript_segments_can_start_at_selected_time ... ok
test youtube::transcript_reader::tests::search_escapes_existing_backslashes_before_like_wildcards ... ok
test youtube::transcript_reader::tests::list_youtube_transcript_segments_pages_by_time_and_id ... ok
test youtube::url::tests::parses_live_url ... ok
test youtube::url::tests::parses_playlist_url ... ok
test youtube::url::tests::parses_short_youtu_be_url ... ok
test youtube::url::tests::parses_shorts_url ... ok
test youtube::url::tests::parses_watch_video_url ... ok
test youtube::url::tests::rejects_empty_input ... ok
test youtube::url::tests::rejects_invalid_host ... ok
test youtube::url::tests::watch_url_with_playlist_parameter_parses_selected_video ... ok
test youtube::ytdlp::tests::authenticated_command_args_include_cookie_file_path_without_cookie_content ... ok
test youtube::ytdlp::tests::cookie_file_content_adds_netscape_header_when_missing ... ok
test youtube::ytdlp::tests::cookie_file_content_preserves_existing_netscape_header ... ok
test youtube::ytdlp::tests::preview_playlist_args_limit_entries_to_first_fifty ... ok
test youtube::ytdlp::tests::preview_video_args_use_dump_json_without_shell_fragments ... ok
test process_tree::tests::terminates_a_descendant_created_after_assignment ... ok
test youtube::process_runtime::tests::injected_timeout_reap_detaches_stuck_child_and_keeps_cookie_until_release ... ok

test result: ok. 665 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.31s

     Running unittests src\main.rs (src-tauri\target\debug\deps\extractum-4b630ead96220393.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_analysis-369ed65a53dcc581.exe)

running 112 tests
test chat::tests::analysis_chat_request_metadata_uses_run_owner ... ok
test chat::tests::build_chat_request_uses_provider_neutral_source_document_wording ... ok
test chat::tests::chat_context_labels_migrated_history_scope_from_metadata ... ok
test chat::tests::completed_chat_context_accepts_saved_snapshot_messages ... ok
test chat::tests::completed_chat_context_requires_saved_snapshot_messages ... ok
test chat::tests::empty_chat_context_uses_source_document_wording ... ok
test corpus::tests::live::youtube_corpus_mode_parses_wire_values_and_defaults ... ok
test corpus::tests::preflight::default_preflight_limits_are_conservative ... ok
test corpus::tests::preflight::estimated_chunk_count_matches_chunk_boundary_behavior ... ok
test corpus::tests::preflight::estimated_message_chars_match_report_chunk_accounting ... ok
test corpus::tests::preflight::model_limit_preflight_allows_unknown_or_fitting_limits ... ok
test corpus::tests::preflight::model_limit_preflight_reports_oversized_chunks ... ok
test chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message ... ok
test corpus::tests::preflight::preflight_limit_error_allows_runs_within_limits ... ok
test corpus::tests::preflight::preflight_limit_error_reports_all_scale_dimensions ... ok
test chat::tests::chat_execution_persists_turns_before_completed_event ... ok
test corpus::tests::snapshot::captured_marker_with_missing_rows_returns_corrupt_snapshot_error ... ok
test corpus::tests::snapshot::list_run_snapshot_messages_page_does_not_fall_back_to_live_source ... ok
test corpus::tests::snapshot::list_run_snapshot_messages_page_reads_saved_snapshot_only ... ok
test corpus::tests::snapshot::list_run_snapshot_messages_page_returns_typed_internal_for_corrupt_snapshot_content ... ok
test corpus::tests::snapshot::run_message_cursor_uses_ref_and_published_at ... ok
test corpus::tests::snapshot::list_run_snapshot_messages_page_starts_at_around_ref ... ok
test corpus::tests::snapshot::load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows ... ok
test corpus::tests::snapshot::load_run_corpus_messages_uses_snapshot_when_available ... ok
test corpus::tests::source_resolution::resolve_run_source_ids_prefers_snapshot_over_live_group_membership ... ok
test report::tests::architecture::analysis_report_workflow_file_has_no_tauri_command_adapters ... ok
test corpus::tests::snapshot::run_snapshot_roundtrips_frozen_corpus ... ok
test corpus::tests::snapshot::trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot ... ok
test corpus::tests::snapshot::source_group_membership_drift_after_capture_does_not_change_saved_run_corpus ... ok
test report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads ... ok
test report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure ... ok
test report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure ... ok
test report::tests::lifecycle::interrupted_cleanup_preserves_captured_snapshot_state_marker ... ok
test report::tests::lifecycle::request_analysis_run_cancel_completed_run_keeps_conflict_message ... ok
test report::tests::phases::analysis_step_cancel_wrapper_allows_completed_future ... ok
test report::tests::phases::analysis_step_cancel_wrapper_interrupts_pending_future ... ok
test report::tests::lifecycle::request_analysis_run_cancel_missing_run_keeps_not_found_message ... ok
test report::tests::phases::finish_map_phase_preserves_chunk_order_by_original_index ... ok
test report::tests::phases::finish_map_phase_propagates_map_error_without_starting_reduce ... ok
test report::tests::preflight::validate_report_preflight_allows_runs_within_limits ... ok
test report::tests::phases::finish_map_phase_rejects_missing_chunk_before_reduce ... ok
test report::tests::preflight::validate_report_preflight_rejects_empty_corpus ... ok
test report::tests::preflight::validate_report_preflight_rejects_oversized_runs ... ok
test report::tests::requests::build_map_request_keeps_run_scoped_request_and_profile ... ok
test report::tests::requests::build_reduce_request_keeps_run_scoped_request_and_profile ... ok
test report::tests::requests::extracts_json_inside_markdown_fence ... ok
test report::tests::requests::extracts_json_with_text_before_and_after ... ok
test report::tests::requests::parse_chunk_summary_ignores_non_json_prefix_with_braces ... ok
test report::tests::requests::parse_chunk_summary_rejects_malformed_payload ... ok
test report::tests::lifecycle::request_analysis_run_cancel_running_but_inactive_keeps_conflict_message ... ok
test report::tests::scope::chunk_target_chars_are_derived_from_model_input_limit_with_fallback ... ok
test report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails ... ok
test report::tests::scope::migrated_history_opt_in_rejects_non_telegram_analysis ... ok
test report::tests::scope::report_run_input_carries_resolved_profile_snapshot ... ok
test report::tests::scope::report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape ... ok
test report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities ... ok
test report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label ... ok
test report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes ... ok
test report::tests::scope::telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match ... ok
test state::tests::analysis_state_cancels_report_run_child_tokens ... ok
test store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes ... ok
test report::tests::runtime::terminal_cleanup_always_removes_active_report_state ... ok
test report::tests::runtime::report_execution_publishes_typed_events_in_existing_order ... ok
test store::tests::read_model::completed_run_without_capture_marker_is_capture_failed ... ok
test store::tests::read_model::failed_terminal_run_without_capture_marker_is_capture_failed ... ok
test store::tests::read_model::list_analysis_run_summaries_applies_query_before_limit ... ok
test store::tests::read_model::list_analysis_run_summaries_combines_scope_and_field_filters ... ok
test store::tests::read_model::list_analysis_run_summaries_rejects_both_scope_ids ... ok
test store::tests::read_model::list_analysis_run_summaries_escapes_literal_like_characters ... ok
test store::tests::read_model::list_analysis_run_summaries_filters_source_groups_and_template_names ... ok
test store::tests::read_model::list_analysis_run_summaries_filters_status_and_dates ... ok
test store::tests::read_model::map_run_detail_exposes_youtube_corpus_mode ... ok
test store::tests::read_model::map_run_summary_exposes_capture_failed_snapshot_state ... ok
test store::tests::read_model::map_run_summary_exposes_captured_snapshot_state ... ok
test store::tests::read_model::map_run_summary_exposes_frozen_scope_label ... ok
test store::tests::read_model::map_run_summary_exposes_null_snapshot_state_for_active_runs_before_capture ... ok
test store::tests::runs::delete_saved_run_removes_run_and_saved_children ... ok
test store::tests::read_model::map_run_summary_exposes_youtube_corpus_mode ... ok
test store::tests::read_model::resolve_run_scope_label_prefers_frozen_value ... ok
test store::tests::runs::duplicate_lookup_keeps_project_and_source_group_scopes_separate ... ok
test store::tests::runs::cancellation_after_capture_does_not_write_snapshot_error ... ok
test store::tests::runs::delete_saved_run_returns_typed_not_found_error ... ok
test store::tests::runs::duplicate_lookup_matches_telegram_history_scope ... ok
test store::tests::runs::insert_analysis_run_persists_youtube_corpus_mode ... ok
test store::tests::runs::provider_failure_status_update_does_not_write_snapshot_error ... ok
test store::tests::setup::fetch_prompt_template_returns_typed_not_found_error ... ok
test store::tests::snapshot::sanitize_provider_error_redacts_provider_payloads ... ok
test store::tests::snapshot::capture_run_snapshot_marks_captured_after_reload_and_replaces_rows ... ok
test store::tests::snapshot::sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens ... ok
test store::tests::snapshot::capture_run_snapshot_rejects_missing_required_fields_without_marker ... ok
test store::tests::snapshot::mark_run_capture_failed_sets_snapshot_error ... ok
test tests::chat_role_validation_returns_typed_error ... ok
test tests::chat_turn_validation_returns_typed_error ... ok
test test_schema::tests::canonical_fixture_applies_analysis_consumed_schema ... ok
test tests::source_group_input_is_trimmed_and_deduplicated ... ok
test tests::source_group_input_validation_returns_typed_error ... ok
test test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys ... ok
test tests::template_kind_validation_returns_typed_error ... ok
test trace::tests::analysis_trace_ref_serializes_youtube_fields_as_null_for_telegram_refs ... ok
test trace::tests::build_trace_refs_falls_back_to_base_item_refs ... ok
test trace::tests::build_trace_refs_handles_multibyte_excerpt ... ok
test trace::tests::build_trace_refs_marks_youtube_description_refs_as_synthetic ... ok
test tests::builtin_template_is_seeded_once ... ok
test trace::tests::clip_excerpt_truncates_on_char_boundary ... ok
test trace::tests::build_trace_refs_resolves_exact_youtube_timestamp_refs ... ok
test trace::tests::decode_trace_data_returns_typed_internal_for_invalid_json ... ok
test trace::tests::decode_trace_data_returns_typed_internal_for_invalid_zstd ... ok
test trace::tests::legacy_trace_bytes_decode_after_core_compression_handoff ... ok
test trace::tests::normalize_ref_accepts_item_refs ... ok
test trace::tests::trace_ref_json_is_byte_compatible_for_telegram_and_youtube ... ok
test tests::completed_run_without_snapshot_marker_is_capture_failed ... ok
test tests::trace_data_roundtrips_through_zstd ... ok

test result: ok. 112 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.19s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_core-afb8508148d4b372.exe)

running 22 tests
test compression::tests::decompress_text_rejects_invalid_utf8 ... ok
test compression::tests::json_bytes_roundtrip_through_zstd ... ok
test compression::tests::text_roundtrip_through_zstd ... ok
test error::tests::classify_message_treats_dialog_lookup_misses_as_not_found ... ok
test error::tests::classify_message_treats_resolution_failures_as_not_found ... ok
test error::tests::classify_message_treats_source_kind_mismatches_as_validation ... ok
test error::tests::database_error_mapper_matches_database_helper ... ok
test error::tests::database_helper_maps_to_internal ... ok
test error::tests::llm_network_helper_maps_to_network ... ok
test error::tests::internal_error_mapper_stringifies_errors ... ok
test error::tests::telegram_network_helper_maps_to_network ... ok
test media_metadata::tests::absent_media_metadata_decodes_to_default ... ok
test media_metadata::tests::media_label_covers_known_and_fallback_kinds ... ok
test media_metadata::tests::media_metadata_decode_failures_are_typed_internal_errors ... ok
test time::tests::now_rfc3339_utc_returns_current_utc_timestamp ... ok
test media_metadata::tests::media_metadata_roundtrip_through_zstd ... ok
test time::tests::now_secs_returns_unix_timestamp_seconds ... ok
test time::tests::ymd_to_unix_midnight_parses_compact_youtube_dates ... ok
test time::tests::ymd_to_unix_midnight_parses_iso_dates ... ok
test time::tests::ymd_to_unix_midnight_rejects_malformed_dates ... ok
test time::tests::ymd_to_unix_midnight_rejects_non_canonical_iso_dates ... ok
test time::tests::ymd_to_unix_midnight_rejects_nonexistent_calendar_dates ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_gemini_browser-3b4520f56cf26a13.exe)

running 77 tests
test cdp::tests::launch_spec_rejects_remote_cdp_endpoint ... ok
test cdp::tests::launch_spec_uses_endpoint_port_and_dedicated_profile ... ok
test execution::tests::active_cancellation_stops_executor_once_and_ignores_late_success ... ok
test execution::tests::cancel_missing_run_returns_without_run_log_side_effects ... ok
test execution::tests::cancel_gemini_browser_job_cancels_queued_run_and_waiter ... ok
test execution::tests::cancel_queued_run_updates_terminal_snapshot ... ok
test execution::tests::cancel_gemini_browser_job_requests_stop_for_active_run ... ok
test execution::tests::restart_worker_entry_acknowledges_missing_run_log_without_sidecar ... ok
test execution::tests::execution_timeout_stops_executor_with_typed_timeout_reason ... ok
test execution::tests::restart_worker_entry_skips_terminal_cancelled_run_log ... ok
test execution::tests::worker_handler_converts_executor_error_to_terminal_failed_result ... ok
test execution::tests::worker_handler_marks_run_running_and_terminal ... ok
test protocol::tests::decode_sidecar_line_accepts_ack_for_matching_id ... ok
test protocol::tests::decode_sidecar_line_for_request_skips_stale_response_ids ... ok
test protocol::tests::decode_sidecar_line_rejects_mismatched_ids ... ok
test protocol::tests::jsonl_transport_round_trips_a_duplex_request ... ok
test protocol::tests::resume_response_classifies_legacy_ack_for_retry ... ok
test protocol::tests::take_complete_jsonl_lines_handles_partial_and_multiple_chunks ... ok
test reconciliation::tests::degraded_apalis_queue_inspection_leaves_queued_run_log_records_for_worker_entry ... ok
test reconciliation::tests::restart_reconciliation_degraded_leaves_queued_run_log_records ... ok
test reconciliation::tests::restart_reconciliation_matrix_handles_supported_apalis_states ... ok
test run_log::tests::get_run_core_returns_exact_run_from_log ... ok
test run_log::tests::create_queued_run_prunes_expired_runs_before_writing_new_run ... ok
test execution::tests::worker_timeout_clears_active_and_cancelled_state ... ok
test run_log::tests::read_run_returns_exact_run_by_id ... ok
test run_log::tests::read_run_returns_validation_error_for_missing_run ... ok
test run_log::tests::list_runs_deletes_run_directories_outside_retention_window ... ok
test execution::tests::worker_timeout_marks_run_failed_and_processes_next_job ... ok
test runtime::tests::complete_waiter_ignores_dropped_receiver ... ok
test runtime::tests::gemini_browser_job_serializes_queue_payload ... ok
test runtime::tests::register_waiter_rejects_duplicate_run_id ... ok
test runtime::tests::runtime_tracks_and_clears_cancelled_run_ids ... ok
test run_log::tests::recorded_run_dir_prunes_expired_run_before_opening_artifacts ... ok
test runtime::tests::wait_for_result_removes_waiter_when_worker_channel_closes ... ok
test runtime::tests::waiter_receives_terminal_worker_result ... ok
test runtime::tests::worker_run_failure_marks_runtime_failed ... ok
test runtime::tests::worker_startup_failure_marks_runtime_failed ... ok
test runtime::tests::worker_status_allows_enqueue_after_ready ... ok
test runtime::tests::worker_status_blocks_enqueue_when_startup_failed ... ok
test run_log::tests::run_log_persists_queued_running_and_terminal_result ... ok
test runtime::tests::worker_status_times_out_while_starting ... ok
test runtime::tests::wait_for_result_removes_waiter_on_timeout ... ok
test sidecar_launch::tests::bundled_sidecar_path_is_beside_the_packaged_executable ... ok
test sidecar_launch::tests::resolve_launch_mode_allows_explicit_dev_sidecar_override_in_release ... ok
test sidecar_launch::tests::resolve_launch_mode_falls_back_to_bundled_when_debug_dev_script_is_absent ... ok
test sidecar_launch::tests::resolve_launch_mode_keeps_dev_node_fallback_for_debug_repo_runs ... ok
test run_log::tests::recorded_run_dir_requires_result_artifact_flag_and_returns_computed_dir ... ok
test sidecar_launch::tests::resolve_launch_mode_prefers_bundled_when_forced ... ok
test sidecar_launch::tests::resolve_launch_mode_uses_bundled_by_default_for_release_even_when_repo_dist_exists ... ok
test state::tests::set_status_snapshot_if_current_does_not_overwrite_newer_snapshot ... ok
test state::tests::startup_reconciliation_gate_retries_after_failure ... ok
test state::tests::startup_reconciliation_gate_runs_once_after_success ... ok
test state::tests::state_tracks_active_run_and_cancellation ... ok
test state::tests::status_snapshot_initializes_to_not_started_from_profile_dir ... ok
test state::tests::update_status_snapshot_mutates_cached_status ... ok
test status::tests::provider_status_live_probe_does_not_mutate_cached_snapshot ... ok
test status::tests::provider_status_read_core_waits_for_startup_reconciliation_before_live_status ... ok
test status::tests::provider_status_snapshot_from_reconciled_runs_does_not_keep_stale_running_snapshot ... ok
test status::tests::provider_status_snapshot_from_reconciled_runs_ignores_stale_queued_rows ... ok
test status::tests::provider_status_snapshot_from_reconciled_runs_preserves_live_active_run ... ok
test status::tests::provider_status_snapshot_read_core_skips_stale_write_back_when_snapshot_changed ... ok
test status::tests::provider_status_snapshot_read_core_writes_reconciled_snapshot_back ... ok
test status::tests::status_snapshot_core_returns_cached_status_without_polling_live_sidecar ... ok
test submission::tests::failed_run_log_transition_returns_app_error_without_side_effects ... ok
test submission::tests::send_single_prompt_handoff_writes_run_log_before_enqueue ... ok
test submission::tests::send_single_prompt_rejects_duplicate_non_terminal_run_id_before_enqueue ... ok
test submission::tests::send_single_prompt_marks_run_failed_when_enqueue_fails ... ok
test status::tests::provider_status_uses_cached_snapshot_when_sidecar_is_busy ... ok
test submission::tests::send_single_prompt_rejects_duplicate_waiter_before_enqueue ... ok
test submission::tests::send_single_prompt_rejects_invalid_artifact_mode_before_side_effects ... ok
test types::tests::manual_action_serializes_start_chrome_cdp ... ok
test types::tests::resume_command_serializes_browser_profile_dir ... ok
test submission::tests::send_single_prompt_rejects_duplicate_terminal_run_id_before_enqueue ... ok
test types::tests::run_result_serializes_optional_debug_summary ... ok
test types::tests::sidecar_command_serializes_browser_config ... ok
test types::tests::sidecar_command_serializes_with_snake_case_tag ... ok
test types::tests::success_statuses_include_ready_and_ok ... ok

test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_llm-f6781d6f95dee3e0.exe)

running 37 tests
test gemini::tests::gemini_model_listing_requires_typed_auth_error ... ok
test gemini::tests::gemini_model_mapping_uses_short_model_id ... ok
test gemini::tests::gemini_request_mapping_keeps_existing_messages_without_output_limit ... ok
test gemini::tests::gemini_request_mapping_keeps_system_history_and_roles ... ok
test gemini::tests::gemini_request_rejects_unsupported_roles_with_typed_validation_error ... ok
test gemini::tests::gemini_server_error_message_includes_transient_recovery_hint ... ok
test gemini::tests::gemini_stream_chunk_text_and_usage_are_parsed ... ok
test openai_compat::tests::openai_compat_model_listing_requires_typed_auth_error ... ok
test openai_compat::tests::openai_compat_model_mapping_reads_omniroute_limits_and_capabilities ... ok
test openai_compat::tests::openai_compat_model_mapping_uses_model_id ... ok
test openai_compat::tests::openai_compat_request_keeps_standard_roles ... ok
test openai_compat::tests::openai_compat_request_rejects_unsupported_roles_with_typed_validation_error ... ok
test openai_compat::tests::openai_compat_retry_status_policy_is_bounded_to_transient_failures ... ok
test openai_compat::tests::openai_compat_stream_chunk_mapping_reads_delta_and_usage ... ok
test provider::tests::model_input_token_limit_lookup_matches_provider_model_ids_and_names ... ok
test provider::tests::model_output_token_limit_lookup_matches_provider_model_ids_and_names ... ok
test provider::tests::normalize_base_url_allows_https_and_loopback_http_only ... ok
test provider::tests::normalize_base_url_returns_typed_validation_error ... ok
test provider::tests::provider_parse_accepts_openai_compatible_aliases ... ok
test provider::tests::provider_parse_returns_typed_validation_error ... ok
test runner::tests::resolve_effective_model_returns_typed_validation_error ... ok
test runner::tests::run_llm_collect_returns_typed_validation_error ... ok
test runner::tests::validate_request_returns_typed_validation_error ... ok
test scheduler::tests::active_owner_run_ids_reports_running_and_queued_owned_requests ... ok
test scheduler::tests::cancelling_owned_run_requests_aborts_running_work ... ok
test scheduler::tests::failed_requests_preserve_typed_error_kind ... ok
test scheduler::tests::failed_requests_release_capacity_for_next_queued_request ... ok
test scheduler::tests::interactive_requests_jump_ahead_of_background_queue ... ok
test scheduler::tests::llm_request_diagnostic_keys_are_stable_snake_case ... ok
test scheduler::tests::queue_positions_are_recomputed_after_cancelling_a_queued_request ... ok
test scheduler::tests::queued_requests_can_be_cancelled_before_start ... ok
test scheduler::tests::request_snapshots_report_running_and_queued_requests ... ok
test scheduler::tests::requests_with_different_profiles_run_without_blocking_each_other ... ok
test streaming::tests::sse_data_decode_failures_are_typed_internal_errors ... ok
test streaming::tests::sse_data_is_parsed_from_stream_chunks ... ok
test types::tests::resolved_profile_construction_preserves_execution_access_and_public_metadata ... ok
test openai_compat::tests::openai_compat_stream_retries_transient_http_before_streaming ... ok

test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_prompt_packs-77cc860c50d25ed2.exe)

running 249 tests
test completion_transport::tests::browser_model_context_has_no_api_fields ... ok
test completion_transport::tests::browser_provenance_is_persisted_before_completion_validation ... ok
test completion_transport::tests::api_stage_uses_background_scheduler_prompt_pack_metadata_and_typed_cancellation ... ok
test dto::tests::prompt_pack_errors_serialize_exact_json_contract ... ok
test dto::tests::preflight_request_defaults_to_api_runtime_provider ... ok
test dto::tests::crate_boundary_constructors_and_accessors_preserve_serialized_shapes ... ok
test dto::tests::start_outcomes_serialize_exact_ipc_contract ... ok
test dto::tests::start_request_accepts_gemini_browser_runtime_provider ... ok
test completion_transport::tests::api_model_context_retains_profile_and_override ... ok
test dto::tests::prompt_pack_run_events_serialize_exact_ipc_contract ... ok
test gemini_browser_stage::tests::ready_result_is_not_prompt_completion ... ok
test gemini_browser_stage::tests::timeout_latest_ok_result_is_not_prompt_completion ... ok
test gemini_browser_stage::tests::ok_browser_result_maps_to_completion_text ... ok
test projections::tests::persist_final_result_does_not_overwrite_cancelled_run_status ... ok
test library::tests::get_prompt_pack_library_returns_active_youtube_summary_pack ... ok
test projections::tests::low_level_result_persistence_rolls_back_when_projection_insert_fails ... ok
test projections::tests::persist_final_result_projects_youtube_synthesis_items ... ok
test projections::tests::persist_final_result_sets_terminal_status_after_projection_rows_exist ... ok
test projections::tests::persist_final_result_uses_current_time_for_run_completion ... ok
test result_builder::tests::build_canonical_result_assigns_backend_owned_ids ... ok
test result_builder::tests::build_canonical_result_includes_synthesis_output ... ok
test result_builder::tests::build_canonical_result_marks_multi_video_synthesis_failed ... ok
test result_builder::tests::build_canonical_result_keeps_partial_result_flag_when_synthesis_is_skipped ... ok
test result_builder::tests::build_canonical_result_marks_multi_video_synthesis_skipped_insufficient_successes ... ok
test result_builder::tests::build_canonical_result_marks_single_video_synthesis_not_applicable ... ok
test result_builder::tests::build_canonical_result_preserves_synthesis_common_claim_text ... ok
test result_builder::tests::build_canonical_result_rejects_incomplete_intermediate_graph ... ok
test result_builder::tests::build_canonical_result_uses_current_created_at ... ok
test run_control::tests::apply_event_updates_state_before_synchronous_sink_observes_it ... ok
test runtime::tests::browser_cancellation_completes_before_terminal_persistence_and_event_follow_up ... ok
test runtime::tests::browser_prompt_formatter_preserves_role_order_and_content ... ok
test runtime::tests::browser_prompt_formatter_rejects_unsupported_roles ... ok
test runtime::tests::browser_run_id_accepts_optional_gem_discriminator ... ok
test runtime::tests::browser_run_identity_includes_repair_attempt_when_present ... ok
test runtime::tests::browser_runtime_start_gate_allows_ready_status ... ok
test runtime::tests::browser_runtime_start_gate_maps_unready_status_to_preflight_failure ... ok
test runtime::tests::browser_stage_result_maps_to_prompt_pack_completion_without_tokens ... ok
test result_builder::tests::build_canonical_result_uses_intermediate_graph_claims_and_evidence ... ok
test runtime::tests::cancelled_browser_stage_does_not_persist_success_provenance ... ok
test runtime::tests::detailed_report_control_preset_uses_larger_transcript_analysis_output_budget ... ok
test result_builder::tests::gem_analysis_final_output_builds_canonical_single_video_result ... ok
test runtime::tests::cleanup_interrupted_prompt_pack_runs_marks_stale_active_rows_interrupted ... ok
test runtime::tests::gem_analysis_part_llm_request_preserves_part_and_frozen_input ... ok
test runtime::tests::gem_analysis_part_repair_llm_request_preserves_attempt_and_repair_context ... ok
test runtime::tests::gem_input_budget_uses_lower_known_model_limit ... ok
test runtime::tests::delete_prompt_pack_run_rejects_active_runs ... ok
test runtime::tests::list_prompt_pack_runs_returns_recent_runs_for_project ... ok
test runtime::tests::list_prompt_pack_run_stages_returns_browser_provenance ... ok
test runtime::tests::now_string_uses_current_utc_time ... ok
test runtime::tests::load_run_runtime_config_reads_api_and_browser_rows ... ok
test runtime::tests::load_run_runtime_config_rejects_malformed_browser_config ... ok
test runtime::tests::load_run_runtime_config_rejects_unsupported_provider ... ok
test runtime::tests::persist_browser_stage_provenance_records_result_identity ... ok
test runtime::tests::prompt_pack_browser_stage_cancelled_before_enqueue_is_tolerated ... ok
test runtime::tests::prepare_execution_borrows_the_same_ticket_for_terminal_failure ... ok
test runtime::tests::prompt_pack_browser_stage_cancelled_while_active_stops_sidecar ... ok
test runtime::tests::prompt_pack_run_cancellation_allows_completed_stage_future ... ok
test runtime::tests::prompt_pack_browser_stage_cancelled_while_queued_cancels_browser_job ... ok
test runtime::tests::prompt_pack_run_cancellation_interrupts_stage_future ... ok
test runtime::tests::prompt_pack_run_state_cancels_child_tokens ... ok
test runtime::tests::prompt_pack_run_state_tracks_active_and_cancel_requested_runs ... ok
test runtime::tests::prompt_pack_cancellation_smoke_fixture_tracks_active_run ... ok
test runtime::tests::prompt_pack_cancellation_smoke_fixture_clear_cancels_tokens_and_deletes_rows ... ok
test runtime::tests::start_service_issues_ticket_after_queued_event_and_new_tracking ... ok
test runtime::tests::start_source_applies_queued_state_and_event_before_spawned_profile_resolution ... ok
test runtime::tests::start_service_rejects_empty_id_before_browser_or_source_ports ... ok
test runtime::tests::synthesis_llm_request_describes_allowed_refs_and_forbids_direct_intermediate_refs ... ok
test runtime::tests::synthesis_output_budget_comes_from_stage_runtime_config ... ok
test runtime::tests::terminal_event_removes_run_from_active_state ... ok
test runtime::tests::transcript_analysis_llm_request_describes_candidate_indexes_and_forbids_backend_refs ... ok
test runtime::tests::transcript_analysis_llm_request_embeds_frozen_stage_input ... ok
test runtime::tests::start_service_returns_existing_before_browser_or_source_ports ... ok
test runtime::tests::transcript_analysis_llm_request_uses_detailed_report_prompt_for_control_preset ... ok
test runtime::tests::transcript_analysis_output_budget_comes_from_stage_runtime_config ... ok
test runtime::tests::start_service_returns_ticket_for_untracked_existing_queued_run ... ok
test runtime::tests::transcript_analysis_output_budget_is_clamped_to_model_limit ... ok
test runtime::tests::transcript_analysis_stage_max_prompt_token_budget_reads_runtime_config ... ok
test runtime::tests::terminal_failure_cleans_state_and_emits_when_persistence_fails ... ok
test seed::tests::seed_youtube_summary_pack_is_idempotent ... ok
test runtime::tests::update_prompt_pack_run_updates_user_label_only ... ok
test seed::tests::seed_youtube_summary_pack_preserves_unknown_newer_bundled_version ... ok
test seed::tests::bundled_assets_hashes_and_source_path_match_canonical_bytes ... ok
test seed::tests::seed_youtube_summary_pack_rejects_bundled_hash_conflict ... ok
test seed::tests::seed_youtube_summary_pack_rejects_user_collision ... ok
test seed::tests::seed_youtube_summary_pack_writes_required_schema_assets ... ok
test stage_output_normalization::tests::synthesis_runtime_normalization_defaults_readable_arrays ... ok
test stage_io::tests::build_transcript_analysis_stage_input_uses_frozen_registries ... ok
test stage_io::tests::insert_stage_artifact_uses_current_time ... ok
test stage_io::tests::transcript_analysis_stage_input_serializes_contract_keys_as_snake_case ... ok
test store::tests::prompt_pack_runs_allow_null_client_request_id_for_pre_existing_rows ... ok
test validation::tests::extract_json_payload_accepts_fenced_json_object ... ok
test validation::tests::extract_json_payload_accepts_leading_and_trailing_prose ... ok
test validation::tests::extract_json_payload_rejects_malformed_braces ... ok
test validation::tests::extract_json_payload_rejects_multiple_json_objects ... ok
test store::tests::prompt_pack_runs_client_request_id_is_unique_when_present ... ok
test validation::tests::invalid_synthesis_output_surfaces_quarantine_write_failure ... ok
test test_schema::tests::canonical_fixture_applies_declared_consumed_schema ... ok
test validation::tests::synthesis_output_accepts_provider_string_items_for_readable_arrays ... ok
test validation::tests::synthesis_output_rejects_direct_segment_key_point_or_quote_refs_inside_synthesis_candidate ... ok
test validation::tests::synthesis_output_rejects_non_array_or_non_string_ref_values ... ok
test validation::tests::synthesis_output_rejects_unknown_claim_ref ... ok
test validation::tests::synthesis_output_validator_accepts_valid_output ... ok
test test_schema::tests::canonical_fixture_preserves_consumed_indexes_and_foreign_keys ... ok
test validation::tests::synthesis_output_validator_rejects_backend_owned_ids ... ok
test validation::tests::synthesis_output_validator_rejects_missing_summary_text ... ok
test validation::tests::synthesis_output_validator_rejects_non_array_fields ... ok
test validation::tests::synthesis_output_validator_rejects_provider_authored_claim_ref ... ok
test validation::tests::synthesis_output_validator_rejects_unknown_source_ref ... ok
test validation::tests::synthesis_output_validator_rejects_structural_schema_errors ... ok
test validation::tests::synthesis_output_validator_rejects_wrong_schema_version ... ok
test validation::tests::synthesis_output_validator_rejects_wrong_stage ... ok
test validation::tests::synthesis_output_validator_rejects_wrong_stage_io_version ... ok
test validation::tests::transcript_analysis_output_rejects_llm_assigned_final_ids ... ok
test validation::tests::transcript_analysis_output_rejects_structural_schema_errors ... ok
test validation::tests::transcript_analysis_output_rejects_unknown_material_ref ... ok
test youtube_summary::entities_tests::blank_key_point_is_skipped_with_graph_warning ... ok
test youtube_summary::entities_tests::build_source_graph_assigns_backend_refs_and_allowed_refs ... ok
test youtube_summary::entities_tests::evidence_index_pointing_to_skipped_quote_candidate_is_dropped_with_warning ... ok
test youtube_summary::entities_tests::evidence_quote_candidate_index_to_missing_quote_is_dropped_with_warning ... ok
test validation::tests::invalid_synthesis_output_is_written_to_quarantine_artifacts ... ok
test validation::tests::invalid_synthesis_output_with_unknown_source_ref_is_quarantined ... ok
test youtube_summary::entities_tests::graph_constants_match_contract ... ok
test youtube_summary::entities_tests::invalid_material_ref_is_rejected ... ok
test youtube_summary::entities_tests::key_point_index_pointing_to_skipped_segment_candidate_is_dropped_with_warning ... ok
test youtube_summary::entities_tests::malformed_candidate_container_is_rejected ... ok
test youtube_summary::entities_tests::provider_output_must_not_supply_backend_refs_or_ids ... ok
test youtube_summary::entities_tests::textless_segment_is_kept_as_structural_navigation ... ok
test validation::tests::synthesis_quarantine_artifact_uses_current_time ... ok
test youtube_summary::entities_tests::graph_builder_uses_persisted_prompt_input_material_registry ... ok
test youtube_summary::execution_tests::execute_multi_video_run_stops_after_transcript_when_cancelled_before_synthesis ... ok
test youtube_summary::execution_tests::execute_multi_video_run_with_one_provider_failure_finishes_partial ... ok
test youtube_summary::execution_tests::execute_queued_run_repairs_malformed_synthesis_json ... ok
test youtube_summary::execution_tests::execute_queued_run_repairs_malformed_transcript_json ... ok
test youtube_summary::execution_tests::execution_graph_build_failure_after_failed_repair_marks_transcript_failed_once ... ok
test youtube_summary::execution_tests::execute_queued_run_with_stage_executor_finishes_complete ... ok
test youtube_summary::execution_tests::gem_analysis_does_not_start_next_part_after_cancellation_checkpoint ... ok
test youtube_summary::execution_tests::gem_analysis_executes_passport_comments_and_deep_recap_in_order ... ok
test youtube_summary::execution_tests::gem_analysis_input_budget_blocks_before_first_provider_call ... ok
test youtube_summary::execution_tests::gem_analysis_optional_comments_failure_persists_report_with_failure_note ... ok
test youtube_summary::execution_tests::gem_analysis_repairs_invalid_required_part_once ... ok
test youtube_summary::execution_tests::gem_analysis_required_part_failure_fails_stage ... ok
test youtube_summary::execution_tests::youtube_summary_invalid_final_result_records_result_level_findings ... ok
test youtube_summary::execution_tests::gem_analysis_skips_comments_when_trimmed_comment_text_is_empty ... ok
test youtube_summary::execution_tests::youtube_summary_multi_video_partial_transcripts_skip_synthesis_and_mark_partial ... ok
test youtube_summary::execution_tests::youtube_summary_run_executes_synthesis_after_transcript_stages ... ok
test youtube_summary::execution_tests::youtube_summary_run_marks_partial_when_synthesis_fails ... ok
test youtube_summary::facade_tests::now_string_uses_current_utc_time ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::assemble_gem_markdown_nests_part_markdown_under_backend_headings ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::assemble_gem_transcript_output_contains_empty_candidate_arrays ... ok
test youtube_summary::execution_tests::youtube_summary_run_marks_partial_when_synthesis_output_is_invalid ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_analysis_part_types_cover_comments_and_stage_variants ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_materials_input_budget_rejects_over_cap ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_materials_part_prompt_inputs_are_isolated ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::parse_part_output_accepts_json_fence_with_internal_markdown_code_block ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::parse_part_output_accepts_matching_non_empty_markdown ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::parse_part_output_rejects_empty_markdown ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::parse_part_output_rejects_wrong_part ... ok
test youtube_summary::execution_tests::youtube_summary_single_video_run_skips_synthesis ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_materials_load_formats_timestamped_transcript_from_metadata ... ok
test youtube_summary::gem_analysis::gem_analysis_part_tests::gem_materials_load_skips_empty_comment_rows ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_normalizes_provider_string_readable_items ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_rejects_invalid_output_without_success_artifacts ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_requires_complete_intermediate_graph ... ok
test youtube_summary::outputs_tests::execute_synthesis_stage_rejects_unknown_claim_ref_with_quarantine ... ok
test youtube_summary::outputs_tests::execute_transcript_analysis_stage_persists_default_warning_candidates ... ok
test youtube_summary::outputs_tests::execute_transcript_analysis_stage_persists_intermediate_entities_artifact ... ok
test youtube_summary::outputs_tests::execute_transcript_analysis_stage_persists_raw_and_parsed_artifacts ... ok
test youtube_summary::outputs_tests::malformed_intermediate_candidates_are_quarantined_without_graph_artifact ... ok
test youtube_summary::outputs_tests::repair_graph_build_failure_does_not_write_repaired_parsed_output ... ok
test youtube_summary::outputs_tests::repaired_transcript_success_artifacts_roll_back_when_parsed_insert_fails ... ok
test youtube_summary::outputs_tests::transcript_stage_metrics_can_include_gem_analysis_extension ... ok
test youtube_summary::outputs_tests::repaired_transcript_analysis_persists_intermediate_entities_for_repair_attempt ... ok
test youtube_summary::outputs_tests::repaired_synthesis_stage_rejects_unknown_claim_ref_with_quarantine ... ok
test youtube_summary::preflight_tests::api_runtime_preflight_uses_fixed_32000_input_limit ... ok
test youtube_summary::outputs_tests::transcript_success_artifacts_roll_back_when_parsed_insert_fails ... ok
test youtube_summary::preflight_tests::preflight_explicit_video_without_transcript_is_blocking_failure ... ok
test youtube_summary::preflight_tests::browser_runtime_preflight_does_not_apply_api_input_limit ... ok
test youtube_summary::result_validation::tests::blank_video_id_returns_error ... ok
test youtube_summary::result_validation::tests::canonical_result_schema_allows_runtime_string_limitations ... ok
test youtube_summary::result_validation::tests::canonical_result_schema_shape_error_returns_finding ... ok
test youtube_summary::result_validation::tests::complete_narrative_only_result_allows_empty_videos ... ok
test youtube_summary::result_validation::tests::complete_standard_result_with_empty_videos_returns_error ... ok
test youtube_summary::result_validation::tests::duplicate_claim_id_returns_error ... ok
test youtube_summary::result_validation::tests::duplicate_evidence_id_returns_error ... ok
test youtube_summary::result_validation::tests::duplicate_source_ref_id_returns_error ... ok
test youtube_summary::result_validation::tests::duplicate_synthesis_item_id_across_item_kinds_returns_error ... ok
test youtube_summary::preflight_tests::preflight_gem_analysis_allows_exactly_one_included_video ... ok
test youtube_summary::result_validation::tests::duplicate_video_id_returns_error ... ok
test youtube_summary::result_validation::tests::evidence_with_unknown_claim_id_returns_error ... ok
test youtube_summary::result_validation::tests::known_quality_flag_emits_advisory_finding_without_error ... ok
test youtube_summary::preflight_tests::preflight_gem_analysis_blocks_multiple_included_videos ... ok
test youtube_summary::result_validation::tests::missing_required_top_level_array_returns_error ... ok
test youtube_summary::result_validation::tests::missing_video_source_refs_is_allowed ... ok
test youtube_summary::result_validation::tests::nested_synthesis_unknown_claim_ref_returns_error_when_top_level_union_empty ... ok
test youtube_summary::result_validation::tests::nested_synthesis_unknown_video_ref_returns_error ... ok
test youtube_summary::result_validation::tests::run_id_mismatch_returns_error ... ok
test youtube_summary::result_validation::tests::single_video_with_synthesis_object_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_duplicate_top_level_claim_ref_returns_error_at_field_path ... ok
test youtube_summary::result_validation::tests::synthesis_extra_top_level_claim_ref_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_extra_top_level_evidence_ref_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_extra_top_level_source_ref_not_in_nested_items_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_missing_nested_claim_ref_in_top_level_union_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_missing_nested_evidence_ref_in_top_level_union_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_missing_source_ref_derived_from_video_ref_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_null_skips_derived_traversal_validation ... ok
test youtube_summary::result_validation::tests::synthesis_object_missing_required_array_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_order_difference_in_top_level_union_is_allowed ... ok
test youtube_summary::result_validation::tests::synthesis_top_level_unknown_claim_ref_returns_error ... ok
test youtube_summary::result_validation::tests::synthesis_unknown_video_ref_does_not_cascade_to_source_union_error ... ok
test youtube_summary::result_validation::tests::unknown_quality_flag_is_ignored_by_mvp_validator ... ok
test youtube_summary::result_validation::tests::validate_youtube_summary_canonical_result_valid_minimal_has_no_errors ... ok
test youtube_summary::preflight_tests::preflight_playlist_video_without_transcript_is_skipped ... ok
test youtube_summary::result_validation::tests::validation_error_writes_findings_marks_run_failed_and_skips_result ... ok
test youtube_summary::result_validation::tests::validation_error_keeps_stage_level_findings ... ok
test youtube_summary::result_validation::tests::validation_persistence_replaces_previous_result_level_findings_on_success ... ok
test youtube_summary::result_validation::tests::validation_error_removes_stale_persisted_result_and_projections ... ok
test youtube_summary::result_validation::tests::video_claim_refs_malformed_shape_returns_error ... ok
test youtube_summary::result_validation::tests::video_claim_refs_unknown_ref_returns_error ... ok
test youtube_summary::result_validation::tests::video_evidence_refs_unknown_ref_returns_error ... ok
test youtube_summary::result_validation::tests::video_evidence_refs_with_non_string_item_returns_error ... ok
test youtube_summary::result_validation::tests::video_source_refs_malformed_shape_returns_error ... ok
test youtube_summary::result_validation::tests::video_source_refs_missing_self_ref_returns_error ... ok
test youtube_summary::result_validation::tests::video_source_refs_unknown_ref_returns_error ... ok
test youtube_summary::result_validation::tests::video_with_unknown_source_ref_returns_error ... ok
test youtube_summary::result_validation::tests::video_source_refs_with_non_string_item_returns_error ... ok
test youtube_summary::snapshots_tests::comment_material_ref_policy_preserves_order_and_token_cap ... ok
test youtube_summary::snapshots_tests::comment_snapshot_source_reads_candidates_for_estimates_then_selected_bodies_again ... ok
test youtube_summary::result_validation::tests::validation_persistence_writes_warning_findings_and_persists_result ... ok
test youtube_summary::result_validation::tests::validation_wrapper_rolls_back_result_findings_when_persistence_fails_after_validation ... ok
test youtube_summary::snapshots_tests::empty_client_request_id_returns_before_any_database_or_source_read ... ok
test youtube_summary::snapshots_tests::duplicate_start_ignores_runtime_blocking_failure ... ok
test youtube_summary::snapshots_tests::duplicate_client_request_id_preserves_existing_runtime_provider ... ok
test youtube_summary::snapshots_tests::snapshot_start_source_preserves_repeated_preflight_and_post_insert_fresh_reads ... ok
test youtube_summary::snapshots_tests::gem_analysis_freezes_comments_even_when_include_comments_is_false ... ok
test youtube_summary::snapshots_tests::selected_comment_body_is_reloaded_after_candidate_estimation ... ok
test youtube_summary::snapshots_tests::runnable_start_uses_complete_fresh_source_read_sequence ... ok
test youtube_summary::snapshots_tests::start_freezes_one_canonical_video_snapshot_with_multiple_origins ... ok
test youtube_summary::snapshots_tests::start_persists_gemini_browser_runtime_and_config_snapshot ... ok
test youtube_summary::snapshots_tests::start_returns_existing_run_for_duplicate_client_request_id ... ok
test youtube_summary::snapshots_tests::start_with_recomputed_blocking_preflight_returns_response_without_run ... ok
test youtube_summary::snapshots_tests::start_with_runtime_blocking_failure_returns_preflight_without_run ... ok
test youtube_summary::snapshots_tests::transcript_material_policy_uses_owned_segment_reader_values ... ok
test youtube_summary::snapshots_tests::transcript_snapshot_text_is_rendered_from_structured_segments ... ok
test youtube_summary::synthesis_input_tests::build_synthesis_stage_input_merges_intermediate_graphs_and_allowed_refs ... ok
test youtube_summary::synthesis_input_tests::build_synthesis_stage_input_collects_successful_transcript_outputs ... ok
test youtube_summary::synthesis_input_tests::build_synthesis_stage_input_orders_graph_by_source_snapshot_id ... ok
test youtube_summary::synthesis_input_tests::build_synthesis_stage_input_uses_latest_parsed_output_wrappers ... ok
test youtube_summary::synthesis_input_tests::load_merged_intermediate_entities_rejects_duplicate_refs_across_sources ... ok

test result: ok. 249 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.63s

     Running unittests src\lib.rs (src-tauri\target\debug\deps\extractum_telegram-31ef7f0b457f8e79.exe)

running 71 tests
test dto::tests::telegram_item_kind_constant_matches_persisted_wire_value ... ok
test dto::tests::telegram_message_draft_has_single_persistence_shape ... ok
test dto::tests::telegram_message_identity_validation_rejects_invalid_values ... ok
test error::tests::channel_private_detection_reads_rpc_name_from_error_message ... ok
test error::tests::non_forum_topic_refresh_errors_are_detected ... ok
test live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure ... ok
test live::messages::tests::fallback_peer_identity_uses_telegram_history_peer_vocabulary ... ok
test live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload ... ok
test live::messages::tests::reply_peer_context_uses_telegram_peer_kinds ... ok
test live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget ... ok
test live::peer::tests::dialog_lookup_misses_are_not_found ... ok
test live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule ... ok
test live::peer::tests::dialog_lookup_not_found_message_explains_numeric_manual_limit ... ok
test live::peer::tests::peer_ref_from_identity_ignores_small_groups_without_supported_identity ... ok
test live::peer::tests::peer_ref_from_identity_uses_channel_access_hash ... ok
test live::peer::tests::peer_ref_from_identity_rejects_unsupported_telegram_kind_as_validation ... ok
test live::peer::tests::peer_ref_from_identity_uses_supergroup_access_hash ... ok
test live::peer::tests::typed_identity_builds_channel_peer_ref_when_access_hash_exists ... ok
test live::peer::tests::typed_identity_rejects_subtype_peer_kind_mismatch ... ok
test live::peer::tests::validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype ... ok
test media::tests::derive_content_kind_tracks_text_and_media_presence ... ok
test live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor ... ok
test media::tests::derive_document_media_kind_prefers_specific_signals ... ok
test runtime::tests::authorized_client_preserves_missing_and_unauthenticated_errors ... ok
test runtime::tests::client_preserves_missing_account_error_without_authorization_check ... ok
test runtime::tests::clear_account_waits_for_inflight_request_then_aborts_runner_and_ignores_sign_out_failure ... ok
test runtime::tests::failed_sign_in_retains_pending_attempt ... ok
test runtime::tests::initialization_maps_authorization_and_last_insert_wins_without_aborting_replaced_runner ... ok
test runtime::tests::missing_account_authentication_is_false ... ok
test runtime::tests::request_login_code_serializes_queued_requests_and_later_success_replaces_attempt ... ok
test runtime::tests::sign_in_without_code_request_preserves_auth_error ... ok
test runtime::tests::successful_sign_in_serializes_clear_then_returns_session_and_clears_attempt ... ok
test session::tests::encrypted_session_load_fails_for_wrong_account_id ... ok
test session::tests::encrypted_session_load_round_trips ... ok
test session::tests::generated_session_key_returns_write_only_encoded_secret ... ok
test session::tests::legacy_json_returns_rewrite_decision ... ok
test session::tests::missing_encrypted_key_preserves_auth_error ... ok
test session::tests::session_encryption_key_rejects_invalid_length ... ok
test session::tests::saving_session_writes_encrypted_envelope_not_plaintext ... ok
test takeout::export_dc::tests::export_dc_attempt_state_detects_first_fallback_transition ... ok
test takeout::export_dc::tests::export_dc_fallback_is_only_for_local_transport_errors ... ok
test takeout::export_dc::tests::export_dc_id_applies_tdesktop_shift ... ok
test takeout::export_dc::tests::export_dc_invoke_does_not_fallback_for_rpc_errors ... ok
test takeout::export_dc::tests::export_dc_invoke_falls_back_to_home_dc_on_local_error ... ok
test takeout::export_dc::tests::takeout_init_request_uses_source_subtype_flags_and_file_limit ... ok
test takeout::export_dc::tests::export_dc_invoke_uses_home_dc_directly_after_fallback ... ok
test takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots ... ok
test takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping ... ok
test takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome ... ok
test takeout::operations::tests::history_page_and_search_return_owned_takeout_messages ... ok
test takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity ... ok
test takeout::operations::tests::only_my_messages_fallback_is_limited_to_channels ... ok
test takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges ... ok
test takeout::pagination::tests::descending_fallback_keeps_raw_order_and_moves_to_min_message_id ... ok
test takeout::pagination::tests::messages_not_modified_response_is_rejected_for_takeout_page ... ok
test takeout::pagination::tests::messages_response_without_slice_is_terminal_page ... ok
test takeout::pagination::tests::split_selection_falls_back_when_telegram_returns_no_ranges ... ok
test takeout::pagination::tests::split_selection_uses_all_ranges_for_small_group ... ok
test takeout::pagination::tests::split_selection_uses_last_range_for_channel_and_supergroup ... ok
test takeout::pagination::tests::tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback ... ok
test takeout::pagination::tests::tdesktop_non_advancing_cursor_restarts_descending_fallback ... ok
test takeout::pagination::tests::tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id ... ok
test takeout::raw_parse::tests::parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids ... ok
test takeout::raw_parse::tests::parses_document_media_kind_filename_and_dimensions ... ok
test takeout::raw_parse::tests::parses_photo_message_metadata ... ok
test takeout::raw_parse::tests::parses_text_message_with_reply_and_reactions ... ok
test takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids ... ok
test takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id ... ok
test takeout::raw_parse::tests::skips_empty_raw_messages ... ok
test takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error ... ok
test live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes ... ok

test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


=== git diff HEAD --check ===

All verification checks passed.
```

The single admitted ordinary workspace-check timing was:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.01s
workspace_check_duration_ms=2010
```

## Release build

- exact command: `npm.cmd run tauri -- build --no-bundle --target x86_64-pc-windows-msvc`
- canonical target directory: `G:\Develop\Extractum\src-tauri\target`
- command exit: `0`

```text
host_target=x86_64-pc-windows-msvc
release_path=G:\Develop\Extractum\src-tauri\target\x86_64-pc-windows-msvc\release\extractum.exe
release_sha256=7f756e9f152087085af925173b8043c53ab58ae4bbb5666def514c8cffdde435
```

```text

> extractum@0.2.0 tauri
> node scripts/tauri.mjs build --no-bundle --target x86_64-pc-windows-msvc

npm.cmd :         Info Looking up installed tauri packages to check mismatched versions...
At line:15 char:65
+ ... se build' { npm.cmd run tauri -- build --no-bundle --target $hostTarg ...
+                 ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (        Info Lo...hed versions...:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

     Running beforeBuildCommand `npm run build:tauri-prereqs`

> extractum@0.2.0 build:tauri-prereqs
> npm run build && npm run build:gemini-browser-sidecar && npm run check:gemini-browser-sidecar-binary


> extractum@0.2.0 build
> vite build

[36mvite v6.4.2 [32mbuilding SSR bundle for production...[36m[39m
transforming...
[33m"hotkeys", "defaultHotkeys" and "scrollTo" are imported from external module "@svar-ui/grid-store" but never used
in "node_modules/@svar-ui/svelte-grid/src/components/Layout.svelte".[39m
[33m"clickOutside", "delegateClick" and "hotkeys" are imported from external module "@svar-ui/lib-dom" but never used
in "node_modules/@svar-ui/svelte-core/src/components/Popup.svelte", "node_modules/@svar-ui/svelte-core/src/components/S
ideArea.svelte", "node_modules/@svar-ui/svelte-grid/src/components/inlineEditors/Text.svelte", "node_modules/@svar-ui/s
velte-grid/src/components/inlineEditors/Combo.svelte", "node_modules/@svar-ui/svelte-grid/src/components/inlineEditors/
Richselect.svelte", "node_modules/@svar-ui/svelte-grid/src/components/inlineEditors/Datepicker.svelte", "node_modules/@
svar-ui/svelte-core/src/components/helpers/InlineDropdown.svelte", "node_modules/@svar-ui/svelte-grid/src/components/in
lineEditors/MultiSelect.svelte", "node_modules/@svar-ui/svelte-menu/src/components/Menu.svelte", "node_modules/@svar-ui
/svelte-grid/src/components/Layout.svelte", "node_modules/@svar-ui/svelte-core/src/components/calendar/Month.svelte", "
node_modules/@svar-ui/svelte-core/src/components/calendar/Year.svelte", "node_modules/@svar-ui/svelte-core/src/componen
ts/calendar/Duodecade.svelte" and "node_modules/@svar-ui/svelte-core/src/components/Fullscreen.svelte".[39m
[32m?[39m 5034 modules transformed.
rendering chunks...
[36mvite v6.4.2 [32mbuilding for production...[36m[39m
transforming...
[32m?[39m 5067 modules transformed.
rendering chunks...
computing gzip size...
[2m.svelte-kit/output/client/[22m[32m_app/version.json                                               [39m[1m[2m  0.03 kB[22m[1m[22m[2m ¦ gzip:  0.05 kB[22m
[2m.svelte-kit/output/client/[22m[32m.vite/manifest.json                                             [39m[1m[2m 31.74 kB[22m[1m[22m[2m ¦ gzip:  3.79 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/15.DzB4hAmu.css                           [39m[1m[2m  0.09 kB[22m[1m[22m[2m ¦ gzip:  0.10 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/2.DLOI3juA.css                            [39m[1m[2m  0.25 kB[22m[1m[22m[2m ¦ gzip:  0.17 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/9.BAy5DLSm.css                            [39m[1m[2m  0.25 kB[22m[1m[22m[2m ¦ gzip:  0.18 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/SurfaceCard.DCl0LXy-.css                  [39m[1m[2m  0.68 kB[22m[1m[22m[2m ¦ gzip:  0.32 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/Badge.CZXNGjcz.css                        [39m[1m[2m  0.76 kB[22m[1m[22m[2m ¦ gzip:  0.33 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/Input.Ct9BFLjf.css                        [39m[1m[2m  0.77 kB[22m[1m[22m[2m ¦ gzip:  0.38 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/Select.BBKsArsW.css                       [39m[1m[2m  0.81 kB[22m[1m[22m[2m ¦ gzip:  0.32 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/StatusMessage.DZy6crxL.css                [39m[1m[2m  0.90 kB[22m[1m[22m[2m ¦ gzip:  0.29 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/6.DXyH1WCK.css                            [39m[1m[2m  0.96 kB[22m[1m[22m[2m ¦ gzip:  0.40 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/1.B1d4-hTe.css                            [39m[1m[2m  1.00 kB[22m[1m[22m[2m ¦ gzip:  0.39 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/SafeMarkdown.Cvpj_ocl.css                 [39m[1m[2m  1.63 kB[22m[1m[22m[2m ¦ gzip:  0.56 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/Button.bYSP5k0C.css                       [39m[1m[2m  1.84 kB[22m[1m[22m[2m ¦ gzip:  0.56 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/4.yWs2plX-.css                            [39m[1m[2m  1.86 kB[22m[1m[22m[2m ¦ gzip:  0.62 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/10.CYlmeCbU.css                           [39m[1m[2m  2.43 kB[22m[1m[22m[2m ¦ gzip:  0.84 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/CheckboxRow.B8jSaJXI.css                  [39m[1m[2m  2.43 kB[22m[1m[22m[2m ¦ gzip:  0.86 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/project-add-source-workflow.Z35krmog.css  [39m[1m[2m  2.60 kB[22m[1m[22m[2m ¦ gzip:  0.76 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/7.NgxEWgFM.css                            [39m[1m[2m  2.63 kB[22m[1m[22m[2m ¦ gzip:  0.82 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/youtube-settings-panel.ChIMwiHI.css       [39m[1m[2m  2.97 kB[22m[1m[22m[2m ¦ gzip:  0.96 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/8.BZ5duqMk.css                            [39m[1m[2m  3.45 kB[22m[1m[22m[2m ¦ gzip:  0.91 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/YoutubeSummaryRunDialog.BKQ_8mvC.css      [39m[1m[2m  5.97 kB[22m[1m[22m[2m ¦ gzip:  1.31 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/library-sources.BzDCvU-G.css              [39m[1m[2m  6.37 kB[22m[1m[22m[2m ¦ gzip:  1.31 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/14.JO-qtKR4.css                           [39m[1m[2m 11.60 kB[22m[1m[22m[2m ¦ gzip:  2.49 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/13.BKA8TZ67.css                           [39m[1m[2m 15.04 kB[22m[1m[22m[2m ¦ gzip:  2.37 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/research-projects-workflow.zsX_Ujdr.css   [39m[1m[2m 23.74 kB[22m[1m[22m[2m ¦ gzip:  4.25 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/12.ByMqRKdX.css                           [39m[1m[2m 36.96 kB[22m[1m[22m[2m ¦ gzip:  5.41 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/input-group-button.BKK_frsD.css           [39m[1m[2m 65.90 kB[22m[1m[22m[2m ¦ gzip:  8.69 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/5.BhOHb4UZ.css                            [39m[1m[2m 66.85 kB[22m[1m[22m[2m ¦ gzip:  9.97 kB[22m
[2m.svelte-kit/output/client/[22m[35m_app/immutable/assets/0.DirrFaB5.css                            [39m[1m[2m 87.28 kB[22m[1m[22m[2m ¦ gzip: 15.66 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BX1VE5XF.js                               [39m[1m[2m  0.04 kB[22m[1m[22m[2m ¦ gzip:  0.06 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/entry/start.DFrgiRBN.js                          [39m[1m[2m  0.08 kB[22m[1m[22m[2m ¦ gzip:  0.09 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/Bzak7iHL.js                               [39m[1m[2m  0.10 kB[22m[1m[22m[2m ¦ gzip:  0.10 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/C1V3-PH7.js                               [39m[1m[2m  0.17 kB[22m[1m[22m[2m ¦ gzip:  0.15 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DdhMrpgN.js                               [39m[1m[2m  0.19 kB[22m[1m[22m[2m ¦ gzip:  0.17 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CMV_iYC9.js                               [39m[1m[2m  0.21 kB[22m[1m[22m[2m ¦ gzip:  0.15 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/3.D7VJAuwg.js                              [39m[1m[2m  0.31 kB[22m[1m[22m[2m ¦ gzip:  0.20 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/fjHw4SRs.js                               [39m[1m[2m  0.34 kB[22m[1m[22m[2m ¦ gzip:  0.28 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/Clj4Lr8F.js                               [39m[1m[2m  0.35 kB[22m[1m[22m[2m ¦ gzip:  0.27 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BnjsYOj-.js                               [39m[1m[2m  0.36 kB[22m[1m[22m[2m ¦ gzip:  0.29 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BCTGH_pp.js                               [39m[1m[2m  0.37 kB[22m[1m[22m[2m ¦ gzip:  0.26 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/B7Xu1rX-.js                               [39m[1m[2m  0.38 kB[22m[1m[22m[2m ¦ gzip:  0.28 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DA7n0yTB.js                               [39m[1m[2m  0.40 kB[22m[1m[22m[2m ¦ gzip:  0.29 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/NFffdFYI.js                               [39m[1m[2m  0.42 kB[22m[1m[22m[2m ¦ gzip:  0.29 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BOJoq5vV.js                               [39m[1m[2m  0.42 kB[22m[1m[22m[2m ¦ gzip:  0.32 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CCFuZJlx.js                               [39m[1m[2m  0.43 kB[22m[1m[22m[2m ¦ gzip:  0.27 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CMg9Eh83.js                               [39m[1m[2m  0.43 kB[22m[1m[22m[2m ¦ gzip:  0.30 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/2.B1shNl-h.js                              [39m[1m[2m  0.43 kB[22m[1m[22m[2m ¦ gzip:  0.27 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/tPV8nMjB.js                               [39m[1m[2m  0.45 kB[22m[1m[22m[2m ¦ gzip:  0.31 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/D3E3ug_1.js                               [39m[1m[2m  0.46 kB[22m[1m[22m[2m ¦ gzip:  0.31 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CZIdqjya.js                               [39m[1m[2m  0.52 kB[22m[1m[22m[2m ¦ gzip:  0.33 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/tGsZp_m7.js                               [39m[1m[2m  0.52 kB[22m[1m[22m[2m ¦ gzip:  0.36 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/B7u8slcl.js                               [39m[1m[2m  0.61 kB[22m[1m[22m[2m ¦ gzip:  0.40 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/Bd-qjXks.js                               [39m[1m[2m  0.64 kB[22m[1m[22m[2m ¦ gzip:  0.31 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BaV30as5.js                               [39m[1m[2m  0.67 kB[22m[1m[22m[2m ¦ gzip:  0.43 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/C-zMR5mX.js                               [39m[1m[2m  0.78 kB[22m[1m[22m[2m ¦ gzip:  0.32 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/1.0OfZV74k.js                              [39m[1m[2m  0.85 kB[22m[1m[22m[2m ¦ gzip:  0.47 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CdDr6og1.js                               [39m[1m[2m  0.89 kB[22m[1m[22m[2m ¦ gzip:  0.51 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/k0zWSNW0.js                               [39m[1m[2m  0.91 kB[22m[1m[22m[2m ¦ gzip:  0.49 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DLzglN-_.js                               [39m[1m[2m  0.93 kB[22m[1m[22m[2m ¦ gzip:  0.51 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DxGhGZy1.js                               [39m[1m[2m  1.01 kB[22m[1m[22m[2m ¦ gzip:  0.47 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/15.DjoiFDYg.js                             [39m[1m[2m  1.03 kB[22m[1m[22m[2m ¦ gzip:  0.55 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CdeTtspI.js                               [39m[1m[2m  1.16 kB[22m[1m[22m[2m ¦ gzip:  0.62 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/OCniWk3i.js                               [39m[1m[2m  1.23 kB[22m[1m[22m[2m ¦ gzip:  0.61 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/TMqJQcQP.js                               [39m[1m[2m  1.29 kB[22m[1m[22m[2m ¦ gzip:  0.63 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DysCKHnx.js                               [39m[1m[2m  1.51 kB[22m[1m[22m[2m ¦ gzip:  0.71 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/5Q86D7-D.js                               [39m[1m[2m  1.54 kB[22m[1m[22m[2m ¦ gzip:  0.77 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/mc6H4CZk.js                               [39m[1m[2m  1.68 kB[22m[1m[22m[2m ¦ gzip:  0.66 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DlS_PIfD.js                               [39m[1m[2m  1.71 kB[22m[1m[22m[2m ¦ gzip:  0.86 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CjFUU5V7.js                               [39m[1m[2m  2.05 kB[22m[1m[22m[2m ¦ gzip:  1.06 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/Cf0M6oAx.js                               [39m[1m[2m  2.22 kB[22m[1m[22m[2m ¦ gzip:  0.98 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CvohNeKP.js                               [39m[1m[2m  2.58 kB[22m[1m[22m[2m ¦ gzip:  0.90 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DfYv9r0w.js                               [39m[1m[2m  2.74 kB[22m[1m[22m[2m ¦ gzip:  1.14 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DXLGBWNZ.js                               [39m[1m[2m  2.76 kB[22m[1m[22m[2m ¦ gzip:  1.27 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BX0T6-EU.js                               [39m[1m[2m  2.96 kB[22m[1m[22m[2m ¦ gzip:  1.31 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/11.n5M4pjyl.js                             [39m[1m[2m  3.22 kB[22m[1m[22m[2m ¦ gzip:  1.24 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/Dy6R51Dz.js                               [39m[1m[2m  3.30 kB[22m[1m[22m[2m ¦ gzip:  1.62 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/9.qMq9VwX8.js                              [39m[1m[2m  3.39 kB[22m[1m[22m[2m ¦ gzip:  1.36 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CXMo2tIc.js                               [39m[1m[2m  3.55 kB[22m[1m[22m[2m ¦ gzip:  1.43 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DFLIenFg.js                               [39m[1m[2m  3.70 kB[22m[1m[22m[2m ¦ gzip:  1.66 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/Bz-eu0gt.js                               [39m[1m[2m  3.85 kB[22m[1m[22m[2m ¦ gzip:  1.57 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/7CU-wMzb.js                               [39m[1m[2m  4.81 kB[22m[1m[22m[2m ¦ gzip:  1.95 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BVNe8ZKS.js                               [39m[1m[2m  5.44 kB[22m[1m[22m[2m ¦ gzip:  2.12 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DPKT2RvO.js                               [39m[1m[2m  6.38 kB[22m[1m[22m[2m ¦ gzip:  2.19 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BDENU9o0.js                               [39m[1m[2m  7.21 kB[22m[1m[22m[2m ¦ gzip:  2.61 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/4.DpTdivZQ.js                              [39m[1m[2m  8.67 kB[22m[1m[22m[2m ¦ gzip:  3.35 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/6.BWBpZF2a.js                              [39m[1m[2m  9.10 kB[22m[1m[22m[2m ¦ gzip:  3.16 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/C1shrt-n.js                               [39m[1m[2m  9.46 kB[22m[1m[22m[2m ¦ gzip:  4.00 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BIEAeoCY.js                               [39m[1m[2m 10.15 kB[22m[1m[22m[2m ¦ gzip:  3.15 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/8.Co7G-1vF.js                              [39m[1m[2m 11.01 kB[22m[1m[22m[2m ¦ gzip:  4.08 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DGoB8wJx.js                               [39m[1m[2m 11.33 kB[22m[1m[22m[2m ¦ gzip:  4.86 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/entry/app.CuOt8_80.js                            [39m[1m[2m 11.97 kB[22m[1m[22m[2m ¦ gzip:  4.67 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DnWqkQzU.js                               [39m[1m[2m 12.07 kB[22m[1m[22m[2m ¦ gzip:  3.45 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BMKukovr.js                               [39m[1m[2m 12.40 kB[22m[1m[22m[2m ¦ gzip:  4.05 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/pCHcGUkV.js                               [39m[1m[2m 13.84 kB[22m[1m[22m[2m ¦ gzip:  4.97 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CElwDYFf.js                               [39m[1m[2m 14.29 kB[22m[1m[22m[2m ¦ gzip:  6.32 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CrPZxlpW.js                               [39m[1m[2m 15.27 kB[22m[1m[22m[2m ¦ gzip:  5.92 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/0.kH09_pE0.js                              [39m[1m[2m 20.31 kB[22m[1m[22m[2m ¦ gzip:  6.55 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/10.CjjBbv-3.js                             [39m[1m[2m 20.67 kB[22m[1m[22m[2m ¦ gzip:  6.70 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/7.BuNuINlz.js                              [39m[1m[2m 21.37 kB[22m[1m[22m[2m ¦ gzip:  6.66 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CqLjYN-H.js                               [39m[1m[2m 21.46 kB[22m[1m[22m[2m ¦ gzip:  7.54 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DkB9moYP.js                               [39m[1m[2m 26.56 kB[22m[1m[22m[2m ¦ gzip:  8.87 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/Bu2Pdnc9.js                               [39m[1m[2m 27.71 kB[22m[1m[22m[2m ¦ gzip: 10.71 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/am1d5eE7.js                               [39m[1m[2m 28.18 kB[22m[1m[22m[2m ¦ gzip: 10.88 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/13.Uarh4BAR.js                             [39m[1m[2m 33.82 kB[22m[1m[22m[2m ¦ gzip: 10.57 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/DdPi23cF.js                               [39m[1m[2m 61.23 kB[22m[1m[22m[2m ¦ gzip: 17.74 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/BUIFcva6.js                               [39m[1m[2m 61.86 kB[22m[1m[22m[2m ¦ gzip: 17.85 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/14.De2jZdse.js                             [39m[1m[2m 67.91 kB[22m[1m[22m[2m ¦ gzip: 20.71 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/12.C7kCKj6T.js                             [39m[1m[2m141.68 kB[22m[1m[22m[2m ¦ gzip: 40.52 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/chunks/CgvySM1k.js                               [39m[1m[2m166.37 kB[22m[1m[22m[2m ¦ gzip: 55.41 kB[22m
[2m.svelte-kit/output/client/[22m[36m_app/immutable/nodes/5.DIHARCN_.js                              [39m[1m[2m357.12 kB[22m[1m[22m[2m ¦ gzip: 92.82 kB[22m
[32m? built in 25.51s[39m
[2m.svelte-kit/output/server/[22m[32m.vite/manifest.json                                             [39m[1m[2m 21.57 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.DzB4hAmu.css                        [39m[1m[2m  0.09 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_layout.DLOI3juA.css                      [39m[1m[2m  0.25 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.BAy5DLSm.css                        [39m[1m[2m  0.25 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/SurfaceCard.DCl0LXy-.css                  [39m[1m[2m  0.68 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/Badge.CZXNGjcz.css                        [39m[1m[2m  0.76 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/Input.Ct9BFLjf.css                        [39m[1m[2m  0.77 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/Select.BBKsArsW.css                       [39m[1m[2m  0.81 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/StatusMessage.DZy6crxL.css                [39m[1m[2m  0.90 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.DXyH1WCK.css                        [39m[1m[2m  0.96 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_error.B1d4-hTe.css                       [39m[1m[2m  1.00 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/SafeMarkdown.Cvpj_ocl.css                 [39m[1m[2m  1.63 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/Button.bYSP5k0C.css                       [39m[1m[2m  1.84 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.yWs2plX-.css                        [39m[1m[2m  1.86 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/CheckboxRow.DN9qUoN0.css                  [39m[1m[2m  2.38 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.CYlmeCbU.css                        [39m[1m[2m  2.43 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/project-add-source-workflow.Z35krmog.css  [39m[1m[2m  2.60 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.NgxEWgFM.css                        [39m[1m[2m  2.63 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/youtube-settings-panel.ChIMwiHI.css       [39m[1m[2m  2.97 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.BZ5duqMk.css                        [39m[1m[2m  3.45 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.k_kJsBWg.css                        [39m[1m[2m  5.59 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/YoutubeSummaryRunDialog.Dg0BKadk.css      [39m[1m[2m  5.94 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/library-sources.BzDCvU-G.css              [39m[1m[2m  6.37 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.BKA8TZ67.css                        [39m[1m[2m 15.04 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/research-projects-workflow.CAcj8Uz4.css   [39m[1m[2m 23.69 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.BFKheMza.css                        [39m[1m[2m 36.94 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/input-group-button.Ywx0-wlv.css           [39m[1m[2m 62.98 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_page.CWHaRUSo.css                        [39m[1m[2m 66.83 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[35m_app/immutable/assets/_layout.BzJSWiOU.css                      [39m[1m[2m 87.05 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/_layout.ts.js                                     [39m[1m[2m  0.04 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/_page.svelte.js                                   [39m[1m[2m  0.34 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/projects/_layout.svelte.js                        [39m[1m[2m  0.35 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36minternal.js                                                     [39m[1m[2m  0.35 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/chevron-right.js                                         [39m[1m[2m  0.36 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/state.svelte.js                                          [39m[1m[2m  0.45 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/index-server.js                                          [39m[1m[2m  0.46 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/index2.js                                                [39m[1m[2m  0.51 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/panel-right-open.js                                      [39m[1m[2m  0.51 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/Badge.js                                                 [39m[1m[2m  0.52 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/trash-2.js                                               [39m[1m[2m  0.55 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/save.js                                                  [39m[1m[2m  0.58 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/refresh-cw.js                                            [39m[1m[2m  0.58 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/environment.js                                           [39m[1m[2m  0.62 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/_error.svelte.js                                  [39m[1m[2m  0.64 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/client.js                                                [39m[1m[2m  0.66 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/StatusMessage.js                                         [39m[1m[2m  0.73 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/EmptyState.js                                            [39m[1m[2m  0.76 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/folder-kanban.js                                         [39m[1m[2m  0.87 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/modals.js                                                [39m[1m[2m  0.89 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/settings/_page.svelte.js                          [39m[1m[2m  0.95 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/sources/_page.svelte.js                           [39m[1m[2m  1.02 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/utils.js                                                 [39m[1m[2m  1.15 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/Button.js                                                [39m[1m[2m  1.30 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/index.js                                                 [39m[1m[2m  1.42 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/StatusMessage2.js                                        [39m[1m[2m  1.54 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/toasts.js                                                [39m[1m[2m  1.56 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/source-jobs.js                                           [39m[1m[2m  1.61 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/SurfaceCard.js                                           [39m[1m[2m  1.71 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/app-error.js                                             [39m[1m[2m  1.83 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/render-context.js                                        [39m[1m[2m  2.60 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/Icon.js                                                  [39m[1m[2m  2.79 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/projects/list/_page.svelte.js                     [39m[1m[2m  3.25 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/internal.js                                              [39m[1m[2m  3.35 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/TextInput.js                                             [39m[1m[2m  3.39 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/projects/_page.svelte.js                          [39m[1m[2m  3.66 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/events.js                                                [39m[1m[2m  3.97 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/auth/_id_/_page.svelte.js                         [39m[1m[2m  4.47 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/exports.js                                               [39m[1m[2m  5.54 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/dialog-content.js                                        [39m[1m[2m  6.58 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/analysis-source-groups.js                                [39m[1m[2m  8.00 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/CheckboxRow.js                                           [39m[1m[2m  9.29 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/SafeMarkdown.js                                          [39m[1m[2m 10.18 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/sources.js                                               [39m[1m[2m 10.35 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/youtube-summary-workflow.js                              [39m[1m[2m 10.94 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/jobs/_page.svelte.js                              [39m[1m[2m 16.17 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/input-group-button.js                                    [39m[1m[2m 20.17 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/create-id.js                                             [39m[1m[2m 21.02 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mremote-entry.js                                                 [39m[1m[2m 24.22 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/root.js                                                  [39m[1m[2m 26.81 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/_layout.svelte.js                                 [39m[1m[2m 27.36 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/accounts/_page.svelte.js                          [39m[1m[2m 27.72 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/projects/library/_page.svelte.js                  [39m[1m[2m 30.51 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/diagnostics/_page.svelte.js                       [39m[1m[2m 31.95 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/shared.js                                                [39m[1m[2m 33.63 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/project-add-source-workflow.js                           [39m[1m[2m 36.35 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/projects/runs/_page.svelte.js                     [39m[1m[2m 44.58 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/YoutubeSummaryRunDialog.js                               [39m[1m[2m 45.23 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/library-sources.js                                       [39m[1m[2m 60.82 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/dialog-description.js                                    [39m[1m[2m 88.55 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/research-projects-workflow.js                            [39m[1m[2m 92.80 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/DataGrid.js                                              [39m[1m[2m 95.87 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mchunks/renderer.js                                              [39m[1m[2m 96.62 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mindex.js                                                        [39m[1m[2m127.28 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/projects/next/_page.svelte.js                     [39m[1m[2m182.36 kB[22m[1m[22m
[2m.svelte-kit/output/server/[22m[36mentries/pages/analysis/_page.svelte.js                          [39m[1m[33m541.27 kB[39m[22m
[32m? built in 53.83s[39m

Run npm run preview to preview your production build locally.

> Using @sveltejs/adapter-static
  Wrote site to "build"
  ? done

> extractum@0.2.0 build:gemini-browser-sidecar
> node scripts/build-gemini-browser-sidecar.mjs


> extractum@0.2.0 test:gemini-browser-sidecar:build
> tsc -p sidecars/gemini-browser/tsconfig.build.json


  artifacts\gemini-browser-sidecar-package\index.cjs  48.2kb

Done in 33ms
> pkg@5.8.1
(node:6440) [DEP0040] DeprecationWarning: The `punycode` module is deprecated. Please use a userland alternative instea
d.
(Use `node --trace-deprecation ...` to show where the warning was created)
Wrote src-tauri\binaries\gemini-browser-sidecar-x86_64-pc-windows-msvc.exe
(node:25024) [DEP0190] DeprecationWarning: Passing args to a child process with shell option true can lead to security
vulnerabilities, as the arguments are not escaped, only concatenated.
(Use `node --trace-deprecation ...` to show where the warning was created)

> extractum@0.2.0 check:gemini-browser-sidecar-binary
> node scripts/check-gemini-browser-sidecar-binary.mjs

Found src-tauri\binaries\gemini-browser-sidecar-x86_64-pc-windows-msvc.exe
(node:1852) [DEP0190] DeprecationWarning: Passing args to a child process with shell option true can lead to security v
ulnerabilities, as the arguments are not escaped, only concatenated.
(Use `node --trace-deprecation ...` to show where the warning was created)
warning: function `sidecar_unavailable_result` is never used
  --> crates\extractum-gemini-browser\src\protocol.rs:96:15
   |
96 | pub(crate) fn sidecar_unavailable_result(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `worker_execution_timeout` and `is_cancelled` are never used
   --> crates\extractum-gemini-browser\src\runtime.rs:220:19
    |
106 | impl GeminiBrowserJobRuntime {
    | ---------------------------- methods in this implementation
...
220 |     pub(crate) fn worker_execution_timeout(&self) -> Duration {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^
...
283 |     pub(crate) fn is_cancelled(&self, run_id: &str) -> bool {
    |                   ^^^^^^^^^^^^

warning: method `run_id` is never used
  --> crates\extractum-gemini-browser\src\state.rs:28:19
   |
27 | impl ActiveRunControl {
   | --------------------- method in this implementation
28 |     pub(crate) fn run_id(&self) -> &str {
   |                   ^^^^^^

warning: methods `init_status_snapshot`, `set_status_snapshot`, and `request_stop` are never used
   --> crates\extractum-gemini-browser\src\state.rs:42:19
    |
 41 | impl GeminiBrowserDomainState {
    | ----------------------------- methods in this implementation
 42 |     pub(crate) fn init_status_snapshot(&self, browser_profile_dir: String) {
    |                   ^^^^^^^^^^^^^^^^^^^^
...
139 |     pub(crate) fn set_status_snapshot(&self, status: GeminiBrowserProviderStatus) {
    |                   ^^^^^^^^^^^^^^^^^^^
...
190 |     pub(crate) async fn request_stop(&self) -> bool {
    |                         ^^^^^^^^^^^^

warning: `extractum-gemini-browser` (lib) generated 4 warnings
warning: method `track` is never used
  --> crates\extractum-prompt-packs\src\run_control.rs:22:25
   |
17 | impl PromptPackRunState {
   | ----------------------- method in this implementation
...
22 |     pub(crate) async fn track(&self, run_id: i64) -> AppResult<()> {
   |                         ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `extractum-prompt-packs` (lib) generated 1 warning
   Compiling extractum v0.2.0 (G:\Develop\Extractum\src-tauri)
   Compiling extractum-telegram v0.2.0 (G:\Develop\Extractum\src-tauri\crates\extractum-telegram)
warning: function `extract_item_payload` is never used
   --> crates\extractum-telegram\src\media.rs:241:15
    |
241 | pub(super) fn extract_item_payload(
    |               ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `phone` is never read
   --> crates\extractum-telegram\src\runtime.rs:187:5
    |
185 | pub struct TelegramLoginAttempt {
    |            -------------------- field in this struct
186 |     token: TelegramLoginAttemptToken,
187 |     phone: String,
    |     ^^^^^

warning: function `export_dc_invoke_with` is never used
  --> crates\extractum-telegram\src\takeout\export_dc.rs:77:10
   |
77 | async fn export_dc_invoke_with<R, Shifted, Home, ShiftedFuture, HomeFuture>(
   |          ^^^^^^^^^^^^^^^^^^^^^

warning: variant `SelfCheck` is never constructed
  --> crates\extractum-telegram\src\takeout\operations.rs:28:5
   |
27 | enum RawCall {
   |      ------- variant in this enum
28 |     SelfCheck,
   |     ^^^^^^^^^
   |
   = note: `RawCall` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during d
ead code analysis

warning: function `start_takeout_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:419:10
    |
419 | async fn start_takeout_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `migration_probe_with_backend` is never used
   --> crates\extractum-telegram\src\takeout\operations.rs:524:10
    |
524 | async fn migration_probe_with_backend<B: OperationsBackend>(
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `peer_ref_identity` is never used
   --> crates\extractum-telegram\src\takeout\raw_parse.rs:345:15
    |
345 | pub(super) fn peer_ref_identity(peer: PeerRef) -> AppResult<(&'static str, i64)> {
    |               ^^^^^^^^^^^^^^^^^

warning: field `session` is never read
  --> crates\extractum-telegram\src\takeout\transport.rs:41:5
   |
39 | pub struct TakeoutTransport {
   |            ---------------- field in this struct
40 |     client: Client,
41 |     session: Arc<MemorySession>,
   |     ^^^^^^^

warning: methods `session` and `home_dc_id` are never used
  --> crates\extractum-telegram\src\takeout\transport.rs:75:19
   |
45 | impl TakeoutTransport {
   | --------------------- methods in this implementation
...
75 |     pub(super) fn session(&self) -> &Arc<MemorySession> {
   |                   ^^^^^^^
...
79 |     pub(super) fn home_dc_id(&self) -> i32 {
   |                   ^^^^^^^^^^

warning: `extractum-telegram` (lib) generated 9 warnings
warning: unused import: `Manager`
 --> src\takeout_import\state.rs:4:33
  |
4 | use tauri::{AppHandle, Emitter, Manager};
  |                                 ^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused imports: `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_GROUP`, and `TELEGRAM_KIND_SUPERGROUP`
  --> src\sources\peer_resolution.rs:12:43
   |
12 |     SourceSyncTarget, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                           ^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
13 |     TELEGRAM_KIND_SUPERGROUP,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `TELEGRAM_KIND_CHANNEL`
  --> src\sources\mod.rs:58:52
   |
58 |     NOTEBOOKLM_HISTORY_SCOPE_MIGRATED_SMALL_GROUP, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
   |                                                    ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `GeminiBrowserArtifactMode`
 --> src\gemini_browser\jobs.rs:9:41
  |
9 |     DeliveredJobInput, DeliveryOutcome, GeminiBrowserArtifactMode, GeminiBrowserJob,
  |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `create_queued_run`, `finish_run`, and `mark_running`
  --> src\gemini_browser\mod.rs:28:5
   |
28 |     create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
   |     ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^             ^^^^^^^^^^^^

warning: unused imports: `GeminiBrowserAnswerCompletionReason`, `GeminiBrowserArtifactRefs`, `GeminiBrowserProviderMode
`, `GeminiBrowserProviderStatusKind`, `GeminiBrowserRunStatus`, and `GeminiBrowserSidecarEnvelope`
  --> src\gemini_browser\mod.rs:32:5
   |
32 |     GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
33 |     GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
34 |     GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
35 |     GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
   |     ^^^^^^^^^^^^^^^^^^^^^^                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `remote` is never read
   --> src\takeout_import\mod.rs:568:9
    |
567 |     AttemptStopped {
    |     -------------- field in this variant
568 |         remote: AppResult<T>,
    |         ^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: variants `Protocol`, `Browser`, and `Invariant` are never constructed
  --> src\gemini_browser\executor.rs:15:5
   |
13 | pub(crate) enum DomainErrorContext {
   |                 ------------------ variants in this enum
14 |     Persistence,
15 |     Protocol,
   |     ^^^^^^^^
16 |     Transport,
17 |     Browser,
   |     ^^^^^^^
18 |     Invariant,
   |     ^^^^^^^^^
   |
   = note: `DomainErrorContext` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead
 code analysis

warning: enum `ApalisQueueInspectionMode` is never used
  --> src\gemini_browser\jobs.rs:20:17
   |
20 | pub(crate) enum ApalisQueueInspectionMode {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `apalis_queue_inspection_mode` is never used
  --> src\gemini_browser\jobs.rs:26:15
   |
26 | pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `startup_reconciliation_checks_queued_runs_against_apalis` is never used
  --> src\gemini_browser\jobs.rs:30:15
   |
30 | pub(crate) fn startup_reconciliation_checks_queued_runs_against_apalis(
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `extractum` (lib) generated 11 warnings (run `cargo fix --lib -p extractum` to apply 6 suggestions)
    Finished `release` profile [optimized] target(s) in 4m 07s
       Built application at: G:\Develop\Extractum\src-tauri\target\x86_64-pc-windows-msvc\release\extractum.exe
```

## Live MCP smoke

The retained capture contains only sanitized backend/window identity, the
actual Vite URL, and the exact empty account-status result:

```text
backend_identifier=org.ai.extractum
backend_name=extractum
backend_version=0.2.0
tauri_version=2.10.3
window_count=1
window_label=main
window_title=extractum
vite_url=http://localhost:1420/
webview_result=[]
```

## Bounded startup smoke

```text
pid=18656
path=G:\Develop\Extractum\src-tauri\target\x86_64-pc-windows-msvc\release\extractum.exe
survival_seconds=5
owned_pid_terminated=true
extractum_residue=0
```

The exact release PID survived five seconds, resolved to the host-qualified
release executable, was terminated by PID, and left zero `extractum` residue.

## Security and non-goals

Phase 8C introduced no credentialed Telegram mutation, wire/API behavior
change, SQLite migration, secret read/write, or production fixture capability.
No account, phone number, API hash, session, login token, source payload, or
private Telegram data is retained in this evidence. `app-test-support` is a
development-isolation mechanism, not a security boundary.

## Final disposition

The one implementation commit is retained, the extraction boundary and exact
test identity are proven, release/MCP/startup evidence passed, and Phase 8 can
advance to `done: retained` in a separate docs-only commit.
