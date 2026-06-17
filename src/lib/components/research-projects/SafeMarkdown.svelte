<script lang="ts">
  import { parseSafeMarkdown } from "$lib/ui/safe-markdown";
  import type { SafeMarkdownBlock, SafeMarkdownInlinePart } from "$lib/ui/safe-markdown";

  let { source }: { source: string } = $props();

  const blocks = $derived(parseSafeMarkdown(source));
</script>

{#snippet Inline(parts: SafeMarkdownInlinePart[])}
  {#each parts as part, index (`inline-${index}`)}
    {#if part.kind === "strong"}
      <strong>{part.text}</strong>
    {:else if part.kind === "code"}
      <code>{part.text}</code>
    {:else}
      {part.text}
    {/if}
  {/each}
{/snippet}

{#snippet Heading(block: Extract<SafeMarkdownBlock, { kind: "heading" }>)}
  {#if block.level === 2}
    <h2>{@render Inline(block.parts)}</h2>
  {:else if block.level === 3}
    <h3>{@render Inline(block.parts)}</h3>
  {:else}
    <h4>{@render Inline(block.parts)}</h4>
  {/if}
{/snippet}

<div class="safe-markdown">
  {#each blocks as block, blockIndex (`block-${blockIndex}`)}
    {#if block.kind === "heading"}
      {@render Heading(block)}
    {:else if block.kind === "paragraph"}
      <p>{@render Inline(block.parts)}</p>
    {:else if block.kind === "blockquote"}
      <blockquote>{@render Inline(block.parts)}</blockquote>
    {:else if block.kind === "list"}
      {#if block.ordered}
        <ol>
          {#each block.items as item, itemIndex (`ordered-${blockIndex}-${itemIndex}`)}
            <li>{@render Inline(item)}</li>
          {/each}
        </ol>
      {:else}
        <ul>
          {#each block.items as item, itemIndex (`unordered-${blockIndex}-${itemIndex}`)}
            <li>{@render Inline(item)}</li>
          {/each}
        </ul>
      {/if}
    {:else if block.kind === "code"}
      <pre><code>{block.text}</code></pre>
    {:else if block.kind === "table"}
      <div class="table-scroll">
        <table>
          <thead>
            <tr>
              {#each block.headers as header, headerIndex (`header-${blockIndex}-${headerIndex}`)}
                <th>{@render Inline(header)}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each block.rows as row, rowIndex (`row-${blockIndex}-${rowIndex}`)}
              <tr>
                {#each row as cell, cellIndex (`cell-${blockIndex}-${rowIndex}-${cellIndex}`)}
                  <td>{@render Inline(cell)}</td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else if block.kind === "divider"}
      <hr />
    {/if}
  {/each}
</div>

<style>
  .safe-markdown {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 10px;
    color: var(--extractum-text);
    font-size: 13px;
    line-height: 1.5;
  }

  h2,
  h3,
  h4,
  p,
  blockquote,
  ul,
  ol,
  pre {
    margin: 0;
  }

  h2,
  h3,
  h4 {
    color: var(--extractum-text);
    font-weight: 700;
    line-height: 1.25;
    text-transform: none;
    letter-spacing: 0;
  }

  h2 {
    font-size: 16px;
  }

  h3 {
    font-size: 14px;
  }

  h4 {
    font-size: 13px;
  }

  ul,
  ol {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-left: 20px;
  }

  blockquote {
    border-left: 3px solid var(--extractum-border-strong, var(--extractum-border));
    padding-left: 10px;
    color: var(--extractum-muted);
  }

  code {
    border-radius: 4px;
    background: var(--extractum-surface-subtle);
    padding: 1px 4px;
    font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
    font-size: 0.95em;
  }

  pre {
    overflow: auto;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    background: var(--extractum-surface-subtle);
    padding: 10px;
  }

  pre code {
    background: transparent;
    padding: 0;
  }

  .table-scroll {
    overflow-x: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  th,
  td {
    border: 1px solid var(--extractum-border);
    padding: 6px 8px;
    text-align: left;
    vertical-align: top;
  }

  th {
    background: var(--extractum-surface-subtle);
    font-weight: 700;
  }

  hr {
    width: 100%;
    border: 0;
    border-top: 1px solid var(--extractum-border);
  }
</style>
