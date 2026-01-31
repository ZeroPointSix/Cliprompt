# TC-quality-stability

## 目标
验证“架构治理与可维护性基线”改造后，契约统一、模块拆分与分层一致性满足预期。

## 前置条件
- 已完成 PRD 中的 P0 改造。
- 已完成 prompts 分层策略选择并记录 ADR。

## 用例 1：契约统一（invoke 集中）
1. 执行 `rg -n "invoke\\(" prompt-launcher/src`。
2. 查看命中位置。

预期结果：
- 仅 `prompt-launcher/src/lib/tauriClient.ts` 存在 `invoke` 调用。
- 业务组件与页面中不出现裸 `invoke("...")`。

## 用例 2：事件名常量化
1. 执行 `rg -n "\"launcher-shown\"|\"prompts-updated\"" prompt-launcher/src`。
2. 查看命中位置。

预期结果：
- 字符串仅出现在常量模块中（如 `prompt-launcher/src/lib/constants.ts`）。
- 业务组件/页面使用常量引用。

## 用例 3：命令迁移到 commands 模块
1. 执行 `rg -n "#\\[tauri::command\\]" prompt-launcher/src-tauri/src/lib.rs`。
2. 执行 `rg -n "#\\[tauri::command\\]" prompt-launcher/src-tauri/src/commands`。

预期结果：
- `lib.rs` 中不再包含 `#[tauri::command]`。
- 命令函数集中在 `prompt-launcher/src-tauri/src/commands/`。

## 用例 4：命令模块接入
1. 执行 `rg -n "mod commands|commands::" prompt-launcher/src-tauri/src/lib.rs`。
2. 检查 `prompt-launcher/src-tauri/src/commands/mod.rs` 是否存在。

预期结果：
- `lib.rs` 引入并使用 `commands` 模块。
- `commands/mod.rs` 作为命令聚合入口存在。

## 用例 5：Prompts 分层策略落地
1. 执行 `rg -n "PromptsService|prompts 分层" docs/DECISIONS.md`。
2. 检查命令层对 prompts 的调用路径是否一致（统一走 service/usecase）。

预期结果：
- `docs/DECISIONS.md` 有明确的 prompts 分层决策记录。
- 命令层只依赖选定的统一入口。

## 用例 6：UI store 拆分落地
1. 执行 `rg -n "configStore|promptsStore" prompt-launcher/src/routes/+page.svelte`。
2. 检查 `prompt-launcher/src/lib/stores/` 下是否存在 `configStore.ts` 与 `promptsStore.ts`。

预期结果：
- `+page.svelte` 通过 store 读取与更新配置/提示词数据。
- `stores/` 模块存在并承担业务数据读写。

## 用例 7：UI 拆分回归
1. 打开主界面。
2. 进行搜索、切换设置、打开/关闭窗口。
3. 检查 `+page.svelte` 是否仅承担布局与事件分发。

预期结果：
- 页面正常渲染，功能无回归。
- 业务逻辑主要迁移到 `stores/` 或 `services/`。

## 用例 8：UI 样式内聚
1. 执行 `rg -n "results-list|settings-container" prompt-launcher/src/routes/+page.svelte`。
2. 执行 `rg -n "results-list" prompt-launcher/src/lib/components/ResultsList.svelte`。
3. 执行 `rg -n "settings-container" prompt-launcher/src/lib/components/SettingsPanel.svelte`。

预期结果：
- `+page.svelte` 不包含结果列表/设置页的样式定义。
- 结果列表与设置页样式分别位于各自组件内。

## 用例 9：promptList 单元测试
1. 在 `prompt-launcher/` 目录执行 `npm run test:unit`。
2. 查看输出结果。

预期结果：
- `promptList.test.js` 全部通过，无失败用例。

## 用例 10：launcherFilters 单元测试
1. 在 `prompt-launcher/` 目录执行 `npm run test:unit`。
2. 查看输出结果。

预期结果：
- `launcherFilters.test.js` 全部通过，无失败用例。
