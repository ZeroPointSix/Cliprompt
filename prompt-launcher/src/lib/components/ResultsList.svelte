<script lang="ts">
  import type { PromptEntry } from "$lib/types";

  export let tagSuggestions: string[];
  export let onApplyTagSuggestion: (tag: string) => void;
  export let filtered: PromptEntry[];
  export let selectedIndex: number;
  export let selectedIds: Set<string>;
  export let onResultClick: (
    event: MouseEvent,
    prompt: PromptEntry,
    index: number
  ) => void;
  export let onResultContextMenu: (
    event: MouseEvent,
    prompt: PromptEntry,
    index: number
  ) => void;
  export let onResultHover: (index: number) => void;
  export let getRowPreviewHtml: (prompt: PromptEntry) => string;
  export let status: string;
</script>

{#if tagSuggestions.length > 0}
  <div class="tag-suggestions">
    {#each tagSuggestions as tag}
      <button class="tag-suggestion" onclick={() => onApplyTagSuggestion(tag)}>
        #{tag}
      </button>
    {/each}
  </div>
{/if}
<div class="results-list">
  {#if filtered.length === 0}
    <div class="empty-state">
      <span class="empty-icon">🔍</span>
      <span>没有找到匹配的提示词</span>
    </div>
  {:else}
    {#each filtered as prompt, index (prompt.id)}
      <button
        class="result-item"
        class:selected={index === selectedIndex}
        class:multi-selected={selectedIds.has(prompt.id)}
        type="button"
        onclick={(event) => onResultClick(event, prompt, index)}
        oncontextmenu={(event) => onResultContextMenu(event, prompt, index)}
        onmouseenter={() => onResultHover(index)}
      >
        <div class="result-main">
          <div class="result-title">
            <span class="title-text">{prompt.title}</span>
            {#if prompt.tags?.length}
              <span class="tags-inline">
                {#each prompt.tags as tag}
                  <span class="tag-pill">#{tag}</span>
                {/each}
              </span>
            {/if}
          </div>
          <div class="result-preview">{@html getRowPreviewHtml(prompt)}</div>
        </div>
      </button>
    {/each}
  {/if}
</div>

<div class="dropdown-footer">
  {#if status}
    <span class="status-msg">{status}</span>
  {:else}
    <span class="keys-hint">
      <span class="key">↵</span> 粘贴
      <span class="key">Esc</span> 返回
      <span class="key">Tab</span> 选择
    </span>
  {/if}
</div>

<style>
  .tag-suggestions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 16px 0 16px;
    background: var(--bg-color);
    border-left: 1px solid var(--border-color);
    border-right: 1px solid var(--border-color);
  }

  .tag-suggestion {
    border: 1px solid var(--border-color);
    background: #ffffff;
    color: #1f2a37;
    border-radius: 999px;
    padding: 4px 10px;
    font-size: 12px;
    cursor: pointer;
  }

  .tag-suggestion:hover {
    background: #f3f5f9;
  }

  .results-list {
    max-height: 360px;
    overflow-y: auto;
    padding: 8px 0;
  }

  .result-item {
    padding: 10px 16px;
    cursor: pointer;
    border-left: 3px solid transparent;
    display: flex;
    align-items: center;
    background: transparent;
    border: none;
    width: 100%;
    text-align: left;
    font: inherit;
  }

  .result-item.selected {
    background-color: var(--selected-bg);
    border-left-color: var(--accent-color);
  }

  .result-item.multi-selected {
    background: #e9f2ff;
    border-color: #cfe2ff;
  }

  .result-main {
    flex: 1;
    overflow: hidden;
  }

  .result-title {
    font-size: 16px;
    font-weight: 500;
    color: var(--text-color);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .title-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tags-inline {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .tag-pill {
    font-size: 11px;
    background: #eef;
    color: #44a;
    padding: 1px 6px;
    border-radius: 12px;
  }

  .result-preview {
    font-size: 13px;
    color: #666;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  :global(.result-preview mark) {
    background: rgba(255, 230, 0, 0.4);
    color: inherit;
    padding: 0;
  }

  .empty-state {
    padding: 30px;
    text-align: center;
    color: #888;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .empty-icon {
    font-size: 24px;
  }

  .dropdown-footer {
    padding: 8px 16px;
    background: #fafafa;
    border-top: 1px solid var(--border-color);
    font-size: 12px;
    color: #888;
    display: flex;
    justify-content: space-between;
  }

  .keys-hint {
    display: flex;
    gap: 12px;
  }

  .key {
    background: #eee;
    padding: 1px 5px;
    border-radius: 3px;
    font-family: monospace;
    font-size: 11px;
    margin-right: 2px;
  }
</style>
