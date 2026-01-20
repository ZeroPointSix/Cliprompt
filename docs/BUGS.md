# Bug 记录

## 2026-01-20 22:25:10

### 现象
- 启动 `npm run tauri dev` 失败，提示 `Port 1420 is already in use`。

### 复现步骤
1. 进入 `prompt-launcher` 目录。
2. 执行 `npm run tauri dev`。
3. Vite 报错端口占用。

### 影响范围
- Tauri dev 无法启动，阻塞验证。

### 初步原因
- 上一次 Vite dev 进程未退出，占用 1420 端口。

### 解决方案
- 结束占用端口的进程后重启。

### 状态
- 已处理：终止 PID 33508。

## 2026-01-20 22:20:12

### 现象
- 启动 `npm run tauri dev` 编译失败。

### 复现步骤
1. 进入 `prompt-launcher` 目录。
2. 执行 `npm run tauri dev`。
3. Rust 编译报错：`open_path` 需要 `Into<String>`，但传入了 `&PathBuf`（E0277）。

### 影响范围
- Tauri 后端无法编译，应用无法启动。

### 初步原因
- `open_prompt_path` 中调用 `app.opener().open_path` 时传入了 `&PathBuf`，类型不匹配。

### 解决方案
- 将 `PathBuf` 转成 `String` 后再调用 `open_path`。

### 状态
- 已修复：`prompt-launcher/src-tauri/src/lib.rs`。

## 2026-01-20 21:50:44

### 现象
- 新建短语后提示“文件已创建，但打开失败”，并提示 `open_path` 权限不足。

### 复现步骤
1. 点击搜索框右侧的 `+`。
2. 输入文件名并确认。
3. 状态栏显示“Not allowed to open path ...”。

### 影响范围
- 无法自动打开系统编辑器，影响快速创建流程。

### 初步原因
- `openPath` 打开默认编辑器失败，错误信息未回显给用户。
- 运行时权限缺失：`opener.open_path not allowed`。
- `opener` scope 不匹配用户自定义路径，导致 “Not allowed to open path ...”。

### 解决方案
- 增加 `openPath` 失败回显（状态栏显示错误详情）。
- 增加打开失败时的 fallback（尝试 `notepad` 与 `notepad.exe`）。
- 为打开失败添加控制台日志便于定位。
- 补充 `opener:allow-open-path` capability 权限。
- 为 `open_path` 增加 allow scope（允许用户自定义目录路径）。
- 改为通过后端命令打开文件，校验路径在提示词目录内，避免前端 opener scope 限制。

### 状态
- 已修复：`prompt-launcher/src/routes/+page.svelte`、`prompt-launcher/src-tauri/capabilities/default.json`、`prompt-launcher/src-tauri/src/lib.rs`。

## 2026-01-20 21:33:14

### 现象
- 新建短语仅弹出输入文件名，不自动打开编辑器。

### 复现步骤
1. 点击搜索框右侧的 `+`。
2. 输入文件名并确认。
3. 未看到系统文本编辑器打开。

### 影响范围
- 新建文件后无法立即编辑，影响创建效率。

### 初步原因
- 主窗口始终置顶，系统编辑器在后台打开导致用户误以为未打开。

### 解决方案
- 新建文件后调用 `openPath` 后隐藏主窗口，确保编辑器前台显示。
- 打开失败时提示“已创建但打开失败”。

### 状态
- 已修复：`prompt-launcher/src/routes/+page.svelte`。

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
