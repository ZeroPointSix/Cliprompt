<script lang="ts">
  import type { AppConfig } from "$lib/types";

  export let appVersion = "";
  export let config: AppConfig;
  export let hotkeyDraft: string;
  export let hotkeyError: string;
  export let settingsError: string;
  export let onChooseFolder: () => void;
  export let onHotkeyInputKeydown: (event: KeyboardEvent) => void;
  export let onApplyHotkey: () => void;
  export let onToggleAutoPaste: () => void;
  export let onToggleAppendClipboard: () => void;
  export let onToggleAutoStart: () => void;
  export let onPreviewCharsChange: (event: Event) => void;
</script>

<div class="settings-container">
  <div class="settings-header-mini">
    <span>设置</span>
    <span class="version">{appVersion ? `v${appVersion}` : "v--"}</span>
  </div>

  <div class="settings-scroll-area">
    <div class="settings-section">
      <div class="section-title">基础配置</div>
      <div class="setting-item">
        <span class="label">提示词目录</span>
        <div class="controls">
          <div class="path-display" title={config.prompts_dir}>
            {config.prompts_dir || "未设置"}
          </div>
          <button class="btn-sm" onclick={onChooseFolder}>选择</button>
        </div>
      </div>
      <div class="setting-item">
        <span class="label">快捷键</span>
        <div class="controls controls-stack">
          <div class="controls-row">
            <input
              class="input-sm"
              bind:value={hotkeyDraft}
              onkeydown={onHotkeyInputKeydown}
              placeholder="按下组合键..."
            />
            <button class="btn-sm" onclick={onApplyHotkey}>应用</button>
          </div>
          {#if hotkeyError}
            <div class="setting-error">{hotkeyError}</div>
          {/if}
        </div>
      </div>
      {#if hotkeyError}
        <div class="settings-error">{hotkeyError}</div>
      {/if}
    </div>

    <div class="settings-section">
      <div class="section-title">行为选项</div>
      <div class="setting-item">
        <span class="label">自动粘贴</span>
        <label class="toggle-switch">
          <input type="checkbox" checked={config.auto_paste} onchange={onToggleAutoPaste} />
          <span class="slider"></span>
        </label>
      </div>
      <div class="setting-item">
        <span class="label">发送时追加剪贴板内容</span>
        <label class="toggle-switch">
          <input
            type="checkbox"
            checked={config.append_clipboard}
            onchange={onToggleAppendClipboard}
          />
          <span class="slider"></span>
        </label>
      </div>
      <div class="setting-item">
        <span class="label">开机自启</span>
        <label class="toggle-switch">
          <input type="checkbox" checked={config.auto_start} onchange={onToggleAutoStart} />
          <span class="slider"></span>
        </label>
      </div>
      {#if settingsError}
        <div class="settings-error">{settingsError}</div>
      {/if}
      <div class="setting-item">
        <span class="label">预览长度(字)</span>
        <div class="controls">
          <input
            class="input-sm"
            type="number"
            min="10"
            max="200"
            step="1"
            value={config.preview_chars}
            onchange={onPreviewCharsChange}
          />
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .settings-container {
    padding: 16px;
    height: 360px;
    display: flex;
    flex-direction: column;
  }

  .settings-header-mini {
    font-size: 14px;
    font-weight: bold;
    color: #555;
    margin-bottom: 12px;
    display: flex;
    justify-content: space-between;
  }

  .settings-scroll-area {
    flex: 1;
    overflow-y: auto;
  }

  .settings-section {
    margin-bottom: 20px;
  }

  .section-title {
    font-size: 12px;
    color: var(--accent-color);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
    font-weight: 600;
  }

  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    font-size: 13px;
    color: var(--text-color);
  }

  .settings-error {
    font-size: 12px;
    color: #b91c1c;
    margin: 2px 0 8px 0;
  }

  .controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .controls.controls-stack {
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }

  .controls-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .setting-error {
    font-size: 11px;
    color: #b91c1c;
    text-align: right;
    max-width: 220px;
  }

  .path-display {
    max-width: 150px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #666;
    font-size: 12px;
    background: #f5f5f5;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .btn-sm {
    padding: 3px 8px;
    font-size: 12px;
    border: 1px solid #ddd;
    background: white;
    border-radius: 4px;
    cursor: pointer;
  }

  .btn-sm:hover {
    background: #f0f0f0;
  }

  .input-sm {
    padding: 3px 6px;
    font-size: 12px;
    border: 1px solid #ddd;
    border-radius: 4px;
    width: 100px;
  }

  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 32px;
    height: 18px;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: #ccc;
    transition: 0.4s;
    border-radius: 34px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 2px;
    bottom: 2px;
    background-color: white;
    transition: 0.4s;
    border-radius: 50%;
  }

  input:checked + .slider {
    background-color: var(--accent-color);
  }

  input:checked + .slider:before {
    transform: translateX(14px);
  }
</style>
