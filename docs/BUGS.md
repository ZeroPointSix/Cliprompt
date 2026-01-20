# Bug 记录

## 2026-01-20 20:00:01

### 现象
- 执行 `npm run tauri dev` 时，Svelte 编译报错并停止运行。

### 复现步骤
1. 进入 `prompt-launcher` 目录。
2. 执行 `npm run tauri dev`。
3. 报错：`Mixing old (on:click) and new syntaxes for event handling is not allowed`。

### 影响范围
- 前端无法启动，阻塞应用运行验证。

### 初步原因
- `src/routes/+page.svelte` 同时使用 `on:click` 与 `onclick` 两种事件语法。

### 解决方案
- 统一使用 `onclick` 语法。
- 将 `on:click|stopPropagation` 替换为 `onclick` + `event.stopPropagation()`。

## 2026-01-20 20:03:51

### 现象
- 执行 `npm run dev` 时出现 Svelte a11y 警告。

### 复现步骤
1. 进入 `prompt-launcher` 目录。
2. 执行 `npm run dev`。
3. 控制台提示 `click` 事件缺少键盘处理和 ARIA role。

### 影响范围
- 开发阶段出现 a11y 警告，影响前端质量基线。

### 初步原因
- 若干 `div` 绑定了 `onclick` 但缺少 `role/tabindex` 和键盘处理。

### 解决方案
- 为 `result-item` 增加 `onkeydown`（Enter/Space）。
- 为 `context-menu`/`modal`/`modal-backdrop` 增加 `role`、`tabindex` 与键盘处理。

### 状态
- 已修复：`result-item` 改为 `button`，`context-menu`/`modal` 使用 `pointerdown`，`modal-backdrop` 增强可访问性。
