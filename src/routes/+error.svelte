<script lang="ts">
  import { page } from "$app/state";
</script>

<section class="route-error" aria-label="Route error">
  <span>Workspace error</span>
  <h1>{page.status}</h1>
  <p>{page.error?.message ?? "Something went wrong while opening this workspace."}</p>
  <a href="/">Back to workspace</a>
</section>

<style>
  .route-error {
    display: flex;
    min-height: 320px;
    flex-direction: column;
    align-items: flex-start;
    justify-content: center;
    gap: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--panel);
    padding: 2rem;
    box-shadow: var(--shadow);
  }

  .route-error span {
    color: var(--muted);
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .route-error h1,
  .route-error p {
    margin: 0;
  }

  .route-error h1 {
    font-size: 2rem;
  }

  .route-error p {
    max-width: 62ch;
    color: var(--muted);
    line-height: 1.5;
  }

  .route-error a {
    display: inline-flex;
    align-items: center;
    min-height: 2.25rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0 0.85rem;
    color: var(--text);
    font-weight: 600;
    text-decoration: none;
  }

  .route-error a:hover {
    background: var(--panel-hover);
  }
</style>
