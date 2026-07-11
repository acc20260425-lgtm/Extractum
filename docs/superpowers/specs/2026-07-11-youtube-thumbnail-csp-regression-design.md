# YouTube thumbnail CSP regression design

## Goal

Restore YouTube preview images without adding remote origins to the Tauri CSP.

## Scope

The Rust backend resolves a stored YouTube thumbnail URL to a local temporary file and returns a Tauri asset URL for the frontend to render. The cache is process-local: it is created for the current app run and cleared at startup, so no thumbnails become persistent user data.

## Data flow

1. A frontend view requests a preview for a stored thumbnail URL.
2. Rust validates that the URL uses HTTPS and that its host is an allowlisted YouTube thumbnail host.
3. Rust downloads the bytes once into the current temporary cache under a SHA-256 URL-derived filename.
4. Rust returns the local asset URL. Repeated requests return the cached asset URL without another download.
5. The frontend uses that returned URL for `<img>`; download or validation errors leave its existing placeholder visible.

## Safety and errors

- The frontend never loads a remote thumbnail URL directly.
- CSP remains without remote origins.
- URL validation rejects non-HTTPS URLs and non-YouTube thumbnail hosts.
- Download failures and unsupported content do not leak secrets or crash the UI.
- The cache is cleared at startup and is not written to SQLite.

## Testing

- Reject a non-HTTPS URL and a non-allowlisted host.
- Cache hit returns the same local asset without a second download.
- Failed fetch leaves the caller with a typed error for placeholder fallback.
- Frontend maps a successful backend result to image `src` and retains the placeholder on error.
