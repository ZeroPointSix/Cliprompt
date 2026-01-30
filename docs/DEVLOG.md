# Dev Log

## 2026-01-30

- 新增: 发送时追加剪贴板内容开关，开启后发送提示词会自动拼接剪贴板文本。
- 新增: 设置页提供“发送时追加剪贴板内容”开关。
- 重要变更: 新增配置字段 `append_clipboard`，默认关闭，旧配置自动补默认值。
- 修复: Clipboard read 权限补齐，允许读取文本内容。
- 文档: 新增剪贴板追加功能 PRD 与测试用例。
- 版本: 0.1.5（同步 Tauri/Cargo/NPM 版本）。
- Tests: `cargo test`（28 tests）。
- Tests: `npm run check`（0 errors, 0 warnings）。
- Build: `npm run tauri build`（MSI/NSIS 产物）。
- Release: `gh release create v0.1.5`（https://github.com/ZeroPointSix/Cliprompt/releases/tag/v0.1.5）。

## 2026-01-26

- 重构: 将全局快捷键注册迁移到后端，新增 `LauncherGate` 控制 UI 就绪与唤起排队。
- 重构: 后端触发 `launcher-shown` 事件，前端统一处理聚焦与“刚显示”保护。
- 版本: 0.1.4（同步 Tauri/Cargo/NPM 版本）。
- 修复: `svelte-check` 报错的 `LogicalSize` 未导入问题。
- 移除: 前端安全注册工具与对应 Node 测试（热键逻辑改为后端持有）。
- 文档: 新增开机自启动热键回归用例文档；更新 BUG 记录与 ADR。
- Tests: `cargo test`（28 tests）。
- Tests: `npm run check`（0 errors, 0 warnings）。
- Dev: `npm run tauri dev` 启动成功后手动中断；日志显示热键冲突（HotKey already registered）。
- 验证: 需要重启验证自启动场景，当前环境无法执行重启，已记录为待验证。
- 验证: 再次执行 `npm run tauri dev`，仍提示热键冲突（HotKey already registered），因此无法在当前环境验证热键唤起。
- Build: `npm run tauri build`（成功，MSI/NSIS 已生成；Vite 提示 `LogicalSize` unused）。
- Release: `gh release create v0.1.4`（https://github.com/ZeroPointSix/Cliprompt/releases/tag/v0.1.4）。

## 2026-01-22

- Updated hotkey registration to keep the previous hotkey on failure by registering the new hotkey before releasing the old one.
- Surfaced hotkey registration errors inline in the settings panel for conflict visibility.
- Extracted hotkey registration flow into `src/lib/hotkey-registration.js`.
- Added Node tests to cover safe hotkey registration ordering, failure behavior, and unregister errors.
- Logged unregister failures for previous hotkeys to improve troubleshooting.
- Dependencies: `npm install` (added 121 packages).
- Build: `npm run build` (warned about missing `.svelte-kit/tsconfig.json` and unused `LogicalSize` import).
- Tests: `node --test` (pass, 4 tests).

## 2026-01-23

- 新增功能: 快捷键注册改为先注册新快捷键，再释放旧快捷键，避免注册失败后失去快捷键。
- 修复: 快捷键注册失败时在设置页展示错误提示，并保留旧快捷键。
- 重要变更: 快速创建提示词文件逻辑拆分为 domain/usecase/infrastructure，统一文件名校验。
- 修复: 新建空提示词文件在 5 秒内保持隐藏，超时自动显示，避免永久隐藏。
- 重要变更: 版本号更新到 0.1.3（Tauri/Cargo/NPM）。
- 测试/验证: `cargo test`（24 tests），`node --test`（4 tests）。
- 构建: `npm run tauri build`，生成 MSI/NSIS 安装包。
- Build: `npm run build` (warned about unused `LogicalSize` import).
- Tests: `node --test` (pass, 4 tests).
- Dev: `npm run tauri dev` (Vite ready at http://localhost:1420/).

## 2026-01-21

- 新增功能: 增加预览长度配置（10-200，默认 50）并在设置页提供输入框。
- 新增功能: 调整预览长度后立即重算并更新列表中的预览内容。
- 修复: 预览截断改为按字符计数，避免多字节字符截断风险。
- 重要变更: 新增 `config.json` 字段 `preview_chars`，旧配置自动补默认值。
- 修复: 设置页版本号改为读取应用版本，避免硬编码与发布版本不一致。
- 重要变更: 版本号统一更新到 0.1.2（Tauri/Cargo/NPM）。
- 测试/验证: `cargo test`（11 tests）。
- 构建: `npm run tauri build`，生成 MSI/NSIS 安装包。

## 2026-01-20

- Fixed quick-create flow so the system editor is visible by hiding the launcher after opening the new file.
- Added status messaging when the editor fails to open after file creation.
- Added openPath fallback to Notepad and surfaced open errors in the status bar.
- Added `opener:allow-open-path` capability to allow opening files with the system editor.
- Added opener scope to allow opening user-selected prompt files.
- Routed file opening through a backend command with prompts-dir path validation to bypass opener scope limits.
- Fixed open_path type mismatch by converting PathBuf to String before calling the opener API.
- Updated FD/TDD docs to cover `open_prompt_path` and path validation behavior.
- Killed leftover Vite dev process on port 1420 to unblock `tauri dev`.
- Ran `cargo test`; all tests passed (9 unit tests).
- Added MSI upgrade code and disallowed downgrades to enable in-place upgrade installs.
- Ran `npm run tauri build`; generated MSI and NSIS installers.
- Updated PRD to include upgrade/uninstall and data retention requirements.
- Updated TDD with manual upgrade verification steps.
- Ran `npm run tauri dev`; app started successfully and was stopped after verification.
- Ran `npm run check`; svelte-check reported 0 errors and 0 warnings.
- Ran `npm run tauri dev`; app started successfully and was stopped after verification.
- Ran `npm run tauri dev`; app started successfully and was stopped after verification.
- Ran `npm run tauri dev`; app started successfully and was stopped after verification.
- Ran `npm run tauri dev`; app started successfully and was stopped after verification.
- Added metadata-backed tags stored in `.tags.json` with filename/folder fallback.
- Implemented tag normalization rules (Chinese/ASCII alnum, length <= 10).
- Added pending-file handling so new empty `.txt` entries appear after save.
- Added Tauri commands for quick `.txt` creation and batch tag updates.
- Added UI support for Ctrl multi-select, right-click tag add/remove, and tag suggestions.
- Added a quick-create `+` button next to settings in the search bar.
- Added FD and TDD documents for tags + quick-create.
- Updated the PRD with tag management and quick-create requirements.
- Ran `cargo test` in `prompt-launcher/src-tauri`; failed due to missing MSVC `link.exe`.
- Ran `npm run tauri dev`; build failed due to missing MSVC `link.exe` and stopped the dev server.
- Re-ran `cargo test` after toolchain install; all tests passed (9 unit tests).
- Re-ran `cargo test`; all tests passed (9 unit tests).
- Added `docs/DECISIONS.md` to capture design/optimization decisions.
- Ran `cargo test` again; all tests passed (9 unit tests).
- Fixed Svelte a11y warnings by switching result items to buttons and using pointerdown on overlays.
- Ran `npm run dev`; frontend compiled without a11y warnings.
- Ran `cargo test`; all tests passed (9 unit tests).
- Ran `npm run tauri dev`; app started successfully and was stopped after verification.
- Ran `cargo test` again; all tests passed (9 unit tests, no code changes).
- Ran `npm run tauri dev` again; app started successfully and was stopped after verification.
- Ran `cargo test` again; all tests passed (9 unit tests, no code changes).

## 2026-01-15

- Scaffolded Tauri v2 + SvelteKit project in `prompt-launcher`.
- Implemented prompt indexing, file watching, and focus/paste flow in Rust.
- Built the MVP UI with search, preview, folder picker, and hotkey settings.
- Expanded search to include prompt bodies and added quick open for the prompts folder.
- Added `docs/PLAN.md` to track iteration scope.
- Added folder-name tags and seeded example prompts when the folder is empty.
- Added a tray menu (show/hide/quit) with click-to-toggle behavior.
- Added a start-with-Windows toggle backed by registry updates.
- Moved fuzzy search scoring to Rust with a lightweight debounce on input.
- Added UI feedback when auto-start registration fails.
- Added a copy-only quick action in the preview panel.
- Added a copy-title action for quick snippet reuse.
- Added a compact settings drawer with hotkey guidance and status.
- Added favorites toggles with a favorites-only filter.
- Added a keyboard shortcut to favorite the active prompt.
- Added a favorites count badge and active filter state.
- Added a favorites section pinned to the top of the list.
- Improved fuzzy ranking with term matching and word-boundary bias.
- Added a filter mode chip in the header.
- Added a copy-path action for prompt files.
- Added a recent section and persisted recently used prompts.
- Added recent filter, enable toggle, and clear action.
- Added matched snippets in list rows when searching.
- Added a Ctrl+Shift+R shortcut to clear recent history.
- Added a Ctrl+Shift+E shortcut to toggle the recent filter.
- Added a last-used timestamp in the preview panel.
- Added a copy-snippet action in the preview panel.
- Added a Ctrl+Shift+G shortcut to toggle favorites filter.
- Highlighted matched terms in list snippets.
- Made tag chips clickable to toggle #tag filters.
- Highlighted matched terms inside the preview pane.
- Applied the recent filter to search results.
- Added a top-tags bar for quick #tag filtering.
- Added a clear button to remove all #tag filters.
- Added a scope toggle for top-tags (all vs results).
- Persisted the top-tags scope toggle in config.
- Added a shortcut to toggle the top-tags scope.
- Added a reset button to clear the search query and tag filters.
- Auto-switched top-tags scope when Favorites/Recent filters are active.
- Added an auto indicator when top-tags scope is overridden by filters.
- Allowed configuring how many top tags are shown.
- Added a collapsible shortcuts legend under the search bar.
- Added a copy-tags action in the preview pane.
- Added a reset action that clears search, tags, and list filters.
- Auto-expanded the shortcuts legend on first launch.
- Fixed Tauri build issues on Windows (tray feature, watcher typing, Win32 imports).
- Translated UI strings and seed prompts to Chinese.
- Added the initial code review report to docs.
- Replaced the settings drawer with a traditional settings page and moved shortcuts there.
- Simplified the launcher UI to a lightweight search-first layout.
- Prevented tray right-click from toggling the window.
- Set tray menu labels to Chinese and updated the window title on launch.
- Auto-shown the main window on startup in debug builds for verification.
- Merged origin/main into feature1 with unrelated histories (added LICENSE).
- Rebuilt the app with `npm run build` (warning: unused LogicalSize import).
- Ran unit tests with `node --test` (4 tests passed).
- Merged local main into feature1 (already up to date).
- Removed unused LogicalSize import to clean build output.
- Rebuilt the app with `npm run build` (no warnings).
- Ran unit tests with `node --test` (4 tests passed).
- Merged feature1 into master.
- Rebuilt the app on master with `npm run build` (no warnings).
- Ran unit tests on master with `node --test` (4 tests passed).
