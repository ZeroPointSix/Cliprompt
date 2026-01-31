# TDD 文档：架构治理与可维护性基线

## 1. 目标
- 用测试约束架构边界，避免改动后重新堆回“上帝模块”。
- 保障命令/事件契约统一，避免字符串漂移。
- 统一 prompts 领域分层，降低后续新增功能的心智负担。

## 2. 目标模块结构

### 2.1 后端（Rust/Tauri）
- `prompt-launcher/src-tauri/src/commands/`：命令层（只负责参数解析与调用 service）。
- `prompt-launcher/src-tauri/src/services/`：业务编排层（PromptsService/ConfigService 等）。
- `prompt-launcher/src-tauri/src/infrastructure/`：IO 访问层（暂作为 repository 层）。
- `prompt-launcher/src-tauri/src/constants.rs`：事件名/窗口名/命令名常量。
- `prompt-launcher/src-tauri/src/lib.rs`：仅保留 wiring 与启动流程。

### 2.2 前端（SvelteKit）
- `prompt-launcher/src/lib/tauriClient.ts`：所有 `invoke` 统一入口。
- `prompt-launcher/src/lib/constants.ts`：事件名/窗口名等常量。
- `prompt-launcher/src/lib/stores/`：状态与业务逻辑（如 `configStore.ts`、`promptsStore.ts`）。
- `prompt-launcher/src/lib/promptList.js`：列表/标签建议的纯函数，供页面复用并独立测试。
- `prompt-launcher/src/lib/launcherFilters.js`：查询/标签过滤纯函数，供页面复用并独立测试。
- `prompt-launcher/src/lib/components/`：拆分视图组件（`SettingsPanel.svelte`、`ResultsList.svelte`），样式内聚。
- `prompt-launcher/src/routes/+page.svelte`：仅做布局与事件分发。

## 3. 测试原则
- 关注架构边界与行为一致性，不做性能优化测试。
- 单元测试优先，覆盖 services 与核心纯函数。
- 外部依赖使用轻量替身（内存 repo、临时目录）；尽量不依赖真实 Tauri AppHandle。
- 不测试私有实现细节，只验证公开行为与调用边界。

## 4. 测试用例清单（按模块）

### 4.1 契约与常量统一（前端）
- `tauriClient` 覆盖后端全部命令名（命令更新必须同步）。
- `prompt-launcher/src` 内不允许出现裸 `invoke("...")`（仅 `tauriClient` 允许）。
- `launcher-shown` / `prompts-updated` 等事件名只允许在 `constants.ts` 定义。

### 4.2 命令层（backend commands）
- `lib.rs` 中不应包含 `#[tauri::command]`。
- `commands/` 中每个命令仅做参数校验与 service 调用，无 IO 逻辑。
- 命令对 service 的调用参数正确（用 FakeService 断言）。

### 4.3 PromptsService（核心业务）
- `list_prompts`：返回与 repository 一致的提示词集合。
- `search_prompts`：空查询返回限定数量；带标签的查询只返回匹配项。
- `create_prompt_file`：空名/非法名返回错误，合法名创建成功。
- `update_prompt_tags`：保存后可读回（以内存 repo 或临时目录验证）。

### 4.4 ConfigService（配置写入）
- `set_auto_paste` / `set_append_clipboard` / `set_recent_enabled` 能写入并持久化。
- `set_top_tags_scope` / `set_top_tags_limit` / `set_preview_chars` 更新后读取一致。
- `set_hotkey` 在无错误情况下更新成功，错误路径保留旧值。

### 4.5 Store/纯函数层（前端）
- 过滤结果与排序逻辑保持一致（例如 recent/favorites 的合成逻辑）。
- `buildTopTags`、`buildRecentList` 等纯函数输入输出稳定。
- 视图层只做渲染与事件派发（无业务逻辑分支）。
- `promptList` 单元测试覆盖 recent 排序、tag 统计、tag 提示逻辑。
- `launcherFilters` 单元测试覆盖 tag 过滤、tag 建议、过滤状态判断。

### 4.6 架构回归检查（结构性）
- `rg -n "invoke\\(" prompt-launcher/src` 只命中 `tauriClient.ts`。
- `rg -n "#\\[tauri::command\\]" prompt-launcher/src-tauri/src/lib.rs` 应为空。
- `rg -n "prompts-updated|launcher-shown" prompt-launcher/src` 只命中常量模块。

## 5. 测试执行说明
- 测试执行人：非开发者本人。
- 建议命令：
  - `cargo test`（后端单元测试）
  - `npm run check`（前端静态检查）
  - `npm run test:unit`（前端纯函数单元测试）

## 6. 进入实现阶段
完成测试用例编写后进入实现阶段（GREEN），建议顺序：
1) constants + tauriClient
2) commands 抽离
3) PromptsService/ConfigService 建立与迁移
4) UI stores + 组件拆分
