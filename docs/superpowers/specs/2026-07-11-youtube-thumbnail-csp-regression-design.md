# YouTube thumbnail CSP regression design

## Goal

Restore YouTube preview images without adding remote origins to the Tauri CSP.

## Scope

The Rust backend resolves a stored YouTube thumbnail URL to an in-memory data URL for the frontend to render. The cache is process-local and contains no filesystem or SQLite state.

## Data flow

1. A frontend view requests a preview for a stored thumbnail URL.
2. Rust validates that the URL uses HTTPS and that its host is an allowlisted YouTube thumbnail host.
3. Rust downloads bytes once with redirects disabled, validates image magic bytes and size, then encodes a data URL.
4. Rust returns a typed result. Concurrent requests share an in-flight fetch and a six-permit semaphore; successful data URLs are memoized only by a module-level bounded frontend LRU.
5. The frontend uses that returned URL for `<img>`; download or validation errors leave its existing placeholder visible.

## Safety and errors

- The frontend never loads a remote thumbnail URL directly; it only receives `data:` URLs.
- CSP remains without remote origins.
- URL validation rejects non-HTTPS URLs and hosts outside `i.ytimg.com`, `i9.ytimg.com`, `img.youtube.com`, and `yt3.ggpht.com`.
- Validation failures are terminal and may be memoized. Network, timeout, and HTTP-status failures are transient and are not memoized; the next component mount retries, with no timer or backoff.
- Each response is limited to 1 MiB before base64 encoding; JPEG, PNG, and WebP magic bytes are accepted.
- The in-memory cache ends with the app process and is not written to SQLite.

## Testing

- Reject a non-HTTPS URL and a non-allowlisted host.
- Cache hit returns the same local asset without a second download.
- Failed fetch leaves the caller with a typed error for placeholder fallback.
- `YoutubeThumbnail` resolves only after an IntersectionObserver reports it visible, preserving lazy loading for long lists. It accepts `url` plus an already-local `fallbackSrc` and retains that fallback on error.
