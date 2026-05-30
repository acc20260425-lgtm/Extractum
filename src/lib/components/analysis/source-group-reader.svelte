<script lang="ts">
  import SourceGroupSourcesView from "$lib/components/analysis/source-group-sources-view.svelte";
  import type { SourceReaderItem } from "$lib/source-reader-model";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    items,
    selectedGroupSourceId,
    loading,
    hasMoreBySource = {},
    hasMoreAll = false,
    loadMoreAllLabel = "Load more source material",
    youtubeDetailsBySource,
    selectedTraceRef = null,
    formatTimestamp,
    onLoadMoreSource,
    onLoadMoreAll = () => {},
  }: {
    items: SourceReaderItem[];
    selectedGroupSourceId: number | null;
    loading: boolean;
    hasMoreBySource?: Record<number, boolean>;
    hasMoreAll?: boolean;
    loadMoreAllLabel?: string;
    youtubeDetailsBySource: Record<number, YoutubeVideoDetail | null>;
    selectedTraceRef?: string | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMoreSource: (sourceId: number) => void | Promise<void>;
    onLoadMoreAll?: () => void | Promise<void>;
  } = $props();
</script>

<SourceGroupSourcesView
  {items}
  {selectedGroupSourceId}
  {loading}
  {hasMoreBySource}
  {hasMoreAll}
  {loadMoreAllLabel}
  {youtubeDetailsBySource}
  {selectedTraceRef}
  {formatTimestamp}
  {onLoadMoreSource}
  {onLoadMoreAll}
/>
