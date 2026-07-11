# YouTube thumbnail CSP regression design

## Goal

Restore YouTube preview images without adding remote origins to the Tauri CSP.

## Scope

The Rust backend resolves a stored YouTube thumbnail URL to an in-memory data URL for the frontend to render. The cache is process-local and contains no filesystem or SQLite state.

## Data flow

1. A frontend view requests a preview for a stored thumbnail URL.
2. Rust validates that the URL uses HTTPS and that its host is an allowlisted YouTube thumbnail host.
3. Rust downloads bytes once with redirects disabled, validates image magic bytes and size, then encodes a data URL.
4. Rust returns the data URL. Repeated and concurrent requests share a cached/in-flight result without another download.
5. The frontend uses that returned URL for `<img>`; download or validation errors leave its existing placeholder visible.

## Safety and errors

- The frontend never loads a remote thumbnail URL directly; it only receives `data:` URLs.
- CSP remains without remote origins.
- URL validation rejects non-HTTPS URLs and hosts outside `i.ytimg.com`, `i9.ytimg.com`, `img.youtube.com`, and `yt3.ggpht.com`.
- Download failures and unsupported content do not leak secrets or crash the UI.
- The in-memory cache ends with the app process and is not written to SQLite.

## Testing

- Reject a non-HTTPS URL and a non-allowlisted host.
- Cache hit returns the same local asset without a second download.
- Failed fetch leaves the caller with a typed error for placeholder fallback.
- Frontend maps a successful backend result to image `src` and retains the placeholder on error.
